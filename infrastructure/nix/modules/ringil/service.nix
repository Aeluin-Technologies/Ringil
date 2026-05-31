{
  pkgs,
  lib,
  config,
  ...
}: let
  ringilPkg = pkgs.callPackage ./package.nix {};

  doraCli = pkgs.callPackage ./dora-cli.nix {};

  dataflowYml = pkgs.writeText "ringil-dataflow.yml" ''
    nodes:
      - id: ros2_bridge
        ros2:
          topics:
            - topic: /camera/color/image_raw
              message_type: sensor_msgs/Image
              direction: subscribe
              output: image_raw
            - topic: /visual_odom
              message_type: nav_msgs/Odometry
              direction: subscribe
              output: visual_odom
          qos:
            reliable: false
            keep_last: 1

      - id: ringil_perception
        path: ${ringilPkg}/bin/ringil-perception
        inputs:
          image: ros2_bridge/image_raw
        outputs:
          - obstacles

      - id: ringil_bridge
        path: ${ringilPkg}/bin/ringil-bridge
        inputs:
          visual_odom: ros2_bridge/visual_odom
          obstacles: ringil_perception/obstacles
  '';

  doraStartScript = pkgs.writeScriptBin "ringil-dora-start" ''
    #!${pkgs.stdenv.shell}

    source ${pkgs.rosPackages.lyrical.ros-workspace}/setup.bash
    export RMW_IMPLEMENTATION=rmw_zenoh_cpp
    export ROS_DOMAIN_ID=42

    echo "Starting Dora Daemon..."
    ${doraCli}/bin/dora up

    echo "Starting Ringil Dataflow..."
    exec ${doraCli}/bin/dora start ${dataflowYml} --attach
  '';
in {
  systemd.services.ringil = {
    description = "Ringil Dora-rs Autonomy Stack";
    wantedBy = ["multi-user.target"];
    # Ensure it starts AFTER the ROS2 nodes (Nav2, VSLAM, etc.)
    after = ["network.target" "ringil-ros2.service" "wg-quick-wg-galadril.service"];

    serviceConfig = {
      ExecStart = "${doraStartScript}/bin/ringil-dora-start";
      ExecStopPost = "${doraCli}/bin/dora destroy";
      Restart = "always";
      RestartSec = "1s";

      User = "ringil";
      Group = "ringil";

      LimitRTPRIO = 99;
      CPUSchedulingPolicy = "fifo";
      CPUSchedulingPriority = 80;

      NoNewPrivileges = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      PrivateTmp = true;
      DeviceAllow = [
        "/dev/ttyTHS* rw" # MAVLink UART.
        "/dev/i2c-* rw" # I2C.
        "/dev/video* rw" # Camera V4L2.
      ];
    };
  };
}
