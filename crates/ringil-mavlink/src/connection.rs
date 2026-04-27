use anyhow::Result;
use mavlink::common::MavMessage;
use mavlink::{MavConnection, MavHeader};
use std::sync::Arc;
use tokio::sync::Mutex;

/// An asynchronous controller for MAVLink-compatible vehicles.
///
/// Provides a high-level API to send commands via a thread-safe connection.
pub struct MavlinkController {
    vehicle: Arc<Mutex<Box<dyn MavConnection<MavMessage> + Send + Sync>>>,
}

impl MavlinkController {
    /// Establishes a connection to a vehicle.
    pub fn connect(address: &str) -> Result<Self> {
        let vehicle = mavlink::connect::<MavMessage>(address)?;
        Ok(Self {
            vehicle: Arc::new(Mutex::new(vehicle)),
        })
    }

    /// Sends a velocity vector and yaw rate in the Local NED (North-East-Down) frame.
    ///
    /// # Arguments
    /// * `vx`, `vy`, `vz` - Velocities in m/s.
    /// * `yaw_rate` - Rotational speed in rad/s.
    pub async fn send_velocity_ned(
        &self,
        vx: f32,
        vy: f32,
        vz: f32,
        yaw_rate: f32,
    ) -> Result<()> {
        let type_mask = mavlink::common::PositionTargetTypemask::from_bits(
            0b0000_1111_1100_0111,
        )
        .unwrap_or(mavlink::common::PositionTargetTypemask::empty());

        let msg = MavMessage::SET_POSITION_TARGET_LOCAL_NED(
            mavlink::common::SET_POSITION_TARGET_LOCAL_NED_DATA {
                time_boot_ms: 0,
                target_system: 1,
                target_component: 1,
                coordinate_frame:
                    mavlink::common::MavFrame::MAV_FRAME_LOCAL_NED,
                type_mask,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vx,
                vy,
                vz,
                afx: 0.0,
                afy: 0.0,
                afz: 0.0,
                yaw: 0.0,
                yaw_rate,
            },
        );

        let header = MavHeader::default();
        let conn = self.vehicle.lock().await;
        conn.send(&header, &msg)?;
        Ok(())
    }
}
