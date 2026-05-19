import os

from ament_index_python.packages import get_package_share_directory
from launch import LaunchDescription
from launch.actions import IncludeLaunchDescription
from launch.launch_description_sources import PythonLaunchDescriptionSource
from launch_ros.actions import Node


def generate_launch_description():
    package_name = "ringil_bringup"
    config_dir = os.path.join(get_package_share_directory(package_name), "config")

    planner_yaml = os.path.join(config_dir, "ego_planner.yaml")
    rtabmap_yaml = os.path.join(config_dir, "rtabmap.yaml")
    nvblox_yaml = os.path.join(config_dir, "nvblox.yaml")
    isaac_vslam_yaml = os.path.join(config_dir, "isaac_ros_vslam.yaml")

    return LaunchDescription(
        [
            Node(
                package="isaac_ros_visual_slam",
                executable="isaac_ros_visual_slam",
                name="visual_slam_node",
                parameters=[isaac_vslam_yaml],
                remappings=[("visual_slam/odom", "/visual_odom")],
            ),
            Node(
                package="rtabmap_slam",
                executable="rtabmap",
                name="rtabmap",
                parameters=[rtabmap_yaml],
                arguments=["-d"],  # Delete previous database on start.
            ),
            Node(
                package="nvblox_ros",
                executable="nvblox_node",
                name="nvblox_node",
                parameters=[nvblox_yaml],
            ),
            Node(
                package="ego_planner",
                executable="ego_planner_node",
                name="ego_planner_node",
                parameters=[planner_yaml],
                output="screen",
            ),
        ]
    )
