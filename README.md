# Ringil 🗡️
 
[GitHub](https://github.com/RealHinome/Ringil)

> *"Yet with his last and desperate stroke Fingolfin hewed the foot with Ringil
> ..."*

**Ringil**  is a system of autonomous swarms for objects--drones, submarines,
etc. Decisions are events recorded and then determined by Galadril, but each
entity can independently decide to take action to avoid an obstacle.

> [!CAUTION]
> This project is still in its early stages.

## Targeted architecture

```mermaid
flowchart TB
    classDef edge fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef onboard fill:#fff3e0,stroke:#e65100,stroke-width:2px
    classDef central fill:#f3e5f5,stroke:#4a148c,stroke-width:2px,stroke-dasharray: 5 5
    classDef action fill:#ffebee,stroke:#b71c1c,stroke-width:2px

    subgraph Galadril_Environment ["Central Intelligence (The Mirror)"]
        direction TB
        G_ESKG[("Galadril ESKG Cluster")]:::central
        G_Logic["Strategic Foresight / Policy"]:::central
    end

    subgraph Ringil_Entity ["Ringil Autonomous Node"]
        direction TB
        
        subgraph Perception_Layer ["Sensory Input"]
            S_Vision["Computer Vision / LiDAR"]:::edge
            S_Telemetry["Inertial / GPS"]:::edge
        end

        subgraph Edge_Brain ["Local Autonomy (The Instinct)"]
            Local_Avoidance{"Obstacle Avoidance Logic"}:::onboard
            Event_Packer["Event Serializer"]:::onboard
        end

        subgraph Actuators ["Action"]
            M_Propulsion["Motor Controllers"]:::action
        end
    end

    S_Vision & S_Telemetry --> Local_Avoidance
    Local_Avoidance --> M_Propulsion
    
    Local_Avoidance -. Telemetry_and_Decisions .-> Event_Packer
    Event_Packer ==>|State_Sync| G_ESKG
    
    G_Logic -. Strategic_Objectives .-> Local_Avoidance
    G_ESKG <--> G_Logic
```

### Internal engine

At its core, Ringil is driven by continuous interaction between physics, local
intent, and collective behavior.

$$\dot{p}_i = v_i \quad,\quad \dot{v}_i = u_i + f_i^{env}$$

Each node is first and foremost a physical system. Position evolves from
velocity, and velocity evolves from applied control and environmental
disturbances (wind, currents, drag, etc.).  

$$u_i = -\nabla \left( \frac{1}{2}k_a \|p_i - p_{goal}\|^2 + \sum_{j}
    \frac{k_r}{\|p_i - p_j\|^2} \right)$$

This defines how an entity reacts with compromise between:
* moving toward an objective (attraction),
* and avoiding others or obstacles (repulsion).

$$\dot{p}_i = \sum_{j \in \mathcal{N}_i} a_{ij}(p_j - p_i)$$

There is no explicit formation controller here.
Instead, each node continuously adjusts itself relative to its neighbors.

$$x_{t} = x_{t|t-1} + K_t \left(z_t - H x_{t|t-1}\right)$$
