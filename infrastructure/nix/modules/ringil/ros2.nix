{
  config,
  lib,
  pkgs,
  ...
}: {
  environment.variables = {
    ROS_DOMAIN_ID = "42";
    RMW_IMPLEMENTATION = "rmw_zenoh_cpp";
    # ROS_PARAMS_FILE = "/etc/ringil/config/high_speed.yaml";
  };

  environment.systemPackages = with pkgs; [
    rosPackages.jazzy.ros-core
    rosPackages.jazzy.rmw-zenoh-cpp

    rosPackages.jazzy.navigation2
    rosPackages.jazzy.nav2-bringup
    rosPackages.jazzy.behaviortree-cpp

    rosPackages.jazzy.ros2cli
  ];
}
