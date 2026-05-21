import os

from ament_index_python.packages import get_package_share_directory
from launch import LaunchDescription
from launch_ros.actions import ComposableNodeContainer, Node
from launch_ros.descriptions import ComposableNode


def generate_launch_description():
    package_name = "ringil_bringup"
    config_dir = os.path.join(get_package_share_directory(package_name), "config")

    planner_yaml = os.path.join(config_dir, "ego_planner.yaml")
    rtabmap_yaml = os.path.join(config_dir, "rtabmap.yaml")
    nvblox_yaml = os.path.join(config_dir, "nvblox.yaml")
    isaac_vslam_yaml = os.path.join(config_dir, "isaac_ros_vslam.yaml")
    as2_state_estimator_yaml = os.path.join(config_dir, "as2_state_estimator.yaml")
    as2_platform_pixhawk_yaml = os.path.join(config_dir, "as2_platform_pixhawk.yaml")
    as2_control_modes_yaml = os.path.join(config_dir, "control_modes.yaml")

    nvidia_container = ComposableNodeContainer(
        name="ringil_nvidia_container",
        namespace="",
        package="rclcpp_components",
        executable="component_container_mt",
        composable_node_descriptions=[
            ComposableNode(
                package="isaac_ros_visual_slam",
                plugin="nvidia::isaac_ros::visual_slam::VisualSlamNode",
                name="visual_slam_node",
                parameters=[isaac_vslam_yaml],
                remappings=[("visual_slam/tracking/odometry", "visual_odom")]
            ),
            ComposableNode(
                package="isaac_ros_nvblox",
                plugin="nvidia::isaac_ros::nvblox::NvbloxNode",
                name="nvblox_node",
                parameters=[nvblox_yaml],
            ),
        ],
        output="screen",
    )

    as2_state_estimator_node = Node(
        package="as2_state_estimator",
        executable="as2_state_estimator_node",
        name="as2_state_estimator_node",
        parameters=[as2_state_estimator_yaml],
    )
    as2_platform_pixhawk_node = Node(
        package="as2_platform_pixhawk",
        executable="as2_platform_pixhawk_node",
        name="as2_platform_pixhawk_node",
        output="screen",
        emulate_tty=True,
        parameters=[
            as2_platform_pixhawk_yaml,
            {"control_modes_file": as2_control_modes_yaml}
        ],
    )

    return LaunchDescription(
        [
            nvidia_container,
            as2_state_estimator_node,
            as2_platform_pixhawk_node,
        ]
    )
