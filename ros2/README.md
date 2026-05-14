## ROS2 for Ringil 🌊

ROS2 is not the core of Ringil. It is not designed to control everything, but
it handles most of the in-flight autonomy. For more complex tasks, we need to
use custom components. ROS2 is excellent because it brings together a vast
library of academic and industrial research.

| Node | Main Responsibility | Sensor Inputs | Output for the Stack | Why it's used here |
| :--- | :--- | :--- | :--- | :--- |
| **Isaac ROS VSLAM** | Visual-Inertial Odometry (VIO) | Stereo Cameras, IMU | `/visual_odom` | GPU-accelerated (cuVSLAM) for high-frequency, low-latency pose estimation. |
| **RTAB-Map** | SLAM & Sensor Abstraction | LiDAR, PointClouds, VIO | `/rtabmap/mapData`, `/octomap` | Merges heterogeneous sensors into a single 3D map and handles loop closure. |
| **GTSAM** | Pose Graph Optimization | Odom constraints, Loop closures | Optimized Trajectory | Ensures the drone's path is mathematically smoothed and drift-corrected. |
| **nvblox** | 3D Reconstruction | Depth Images, PointClouds | ESDF (Distance Field) | Transforms raw points into a GPU-based distance map for fast obstacle avoidance. |
| **Nav2** | Path Planning & Control | Map, Odom, Costmaps | `cmd_vel` (Velocity) | Calculates global and local paths while avoiding obstacles in real-time. |

```mermaid
flowchart TB
    classDef sensor fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef gpuNode fill:#f1f8e9,stroke:#33691e,stroke-width:2px
    classDef slamNode fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef navNode fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef hardware fill:#ffebee,stroke:#b71c1c,stroke-width:2px

    subgraph Sensors ["Sensory Input Layer"]
        S_LiDAR["LiDAR 2D/3D"]:::sensor
        S_Cam["RGB-D Camera"]:::sensor
        S_IMU["IMU / Inertial"]:::sensor
        S_EvCam["Event Camera"]:::sensor
    end

    subgraph GPU_Perception ["NVIDIA Isaac Acceleration"]
        direction TB
        P_Conv["Event-to-PointCloud"]:::gpuNode
        P_VSLAM["Isaac ROS VSLAM (cuVSLAM)"]:::gpuNode
        P_NVB["nvblox GPU Mapping"]:::gpuNode
    end

    subgraph SLAM_Core ["Global Consistency (RTAB-Map)"]
        direction TB
        C_RTAB["RTAB-Map Node"]:::slamNode
        C_GTSAM[/"GTSAM Optimizer"/]:::slamNode
    end

    subgraph Autonomy_Control ["Navigation & Flight"]
        direction TB
        A_Nav2["Nav2 Stack"]:::navNode
        A_FCU["Flight Controller (PX4/Ardu)"]:::hardware
    end

    %% Connections
    S_LiDAR & P_Conv --> C_RTAB
    S_EvCam --> P_Conv
    
    S_Cam & S_IMU --> P_VSLAM
    
    P_VSLAM ==>|High-Freq Odometry| C_RTAB
    C_RTAB <--> C_GTSAM
    
    C_RTAB -->|Filtered Cloud| P_NVB
    P_NVB ==>|3D Costmap| A_Nav2
    C_RTAB -->|Optimized Pose| A_Nav2
    
    A_Nav2 ==>|cmd_vel / Setpoints| A_FCU
    A_FCU -.->|State Feedback| P_VSLAM
```
