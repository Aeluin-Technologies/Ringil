{
  config,
  lib,
  pkgs,
  ...
}: let
  rosDistro = pkgs.rosPackages.jazzy;

  ringilBringup = pkgs.callPackage ./ringil-bringup.nix {
    buildRosPackage = rosDistro.buildRosPackage;
    ament-python = rosDistro.ament-python;
    launch = rosDistro.launch;
    launch-ros = rosDistro.launch-ros;
  };

  zenohConfigFile = pkgs.writeText "zenoh_config.json" (builtins.toJSON {
    transport = {
      shared_memory = {
        enabled = true;
      };
    };
  });

  autonomyLaunchScript = pkgs.writeScriptBin "ringil-ros2-launch" ''
    #!${pkgs.stdenv.shell}
    source ${rosDistro.ros-workspace}/setup.bash
    cleanup() {
      kill $AGENT_PID || true
    }
    trap cleanup EXIT
    MicroXRCEAgent serial --dev /dev/ttyTHS0 -b 921600 &
    AGENT_PID=$!
    sleep 1
    exec ros2 launch ringil_bringup autonomy.launch.py
  '';
in {
  environment.variables = {
    ROS_DOMAIN_ID = "42";
    RMW_IMPLEMENTATION = "rmw_zenoh_cpp";
    ZENOH_CONFIG_FILE = "${zenohConfigFile}";
    ROS_DISABLE_LOANED_MESSAGES = "0";
  };

  environment.systemPackages = with rosDistro; [
    ros-core
    rmw-zenoh-cpp

    rosDistro.isaac-ros-visual-slam
    rosDistro.isaac-ros-nvblox

    rosDistro.aerostack2
    rosDistro.as2-core
    rosDistro.as2-motion-controller
    rosDistro.as2-state-estimator
    rosDistro.as2-platform-pixhawk

    pkgs.micro-xrce-dds-agent

    ringilBringup
    autonomyLaunchScript
  ];

  systemd.services.ringil-ros2 = {
    description = "Ringil ROS 2 Autonomy Stack";
    wantedBy = ["multi-user.target"];
    after = ["network.target"];

    environment = {
      ROS_DOMAIN_ID = "42";
      RMW_IMPLEMENTATION = "rmw_zenoh_cpp";
      ZENOH_CONFIG_FILE = "${zenohConfigFile}";
      ROS_DISABLE_LOANED_MESSAGES = "0";
      MALLOC_TRIM_THRESHOLD_ = "131072";
    };

    serviceConfig = {
      ExecStart = "${autonomyLaunchScript}/bin/ringil-ros2-launch";
      Restart = "always";
      RestartSec = "1s";
      User = "ringil";
      Group = "ringil";

      LimitRTPRIO = 99;
      CPUSchedulingPolicy = "fifo";
      CPUSchedulingPriority = 80;
      OOMScoreAdjust = "-1000";
    };
  };
}
