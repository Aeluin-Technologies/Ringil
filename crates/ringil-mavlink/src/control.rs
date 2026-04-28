use crate::connection::MavlinkController;
use anyhow::Result;
use ringil_instinct::events::{InstinctEvent, ObjectClass};

/// Orchestrates flight behavior by translating vision events into MAVLink commands.
pub struct FlightDirector {
    pub mavlink: MavlinkController,
    pub person_safe_distance: f32,
    pub safe_distance: f32,
    pub max_yaw_rate: f32,
    pub repulse_gain: f32,
    pub p_gain_advance: f32,
    pub p_gain_yaw: f32,
    pub resolution: [u32; 2],
}

impl FlightDirector {
    /// Creates a new [`FlightDirector`] with specific gain and safety parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mavlink: MavlinkController,
        person_safe_distance: f32,
        safe_distance: f32,
        max_yaw_rate: f32,
        repulse_gain: f32,
        p_gain_advance: f32,
        p_gain_yaw: f32,
        resolution: [u32; 2],
    ) -> Self {
        tracing::info!(
            ?person_safe_distance,
            ?safe_distance,
            ?repulse_gain,
            "flight direction control connected to mavlink"
        );
        Self {
            mavlink,
            person_safe_distance,
            safe_distance,
            max_yaw_rate,
            repulse_gain,
            p_gain_advance,
            p_gain_yaw,
            resolution,
        }
    }

    /// Routes incoming perception events to specific flight logic handlers.
    pub async fn process_event(&self, event: InstinctEvent) -> Result<()> {
        let [cam_width, cam_height] = self.resolution;
        let cam_width = cam_width as f32;
        let cam_height = cam_height as f32;

        match event {
            // Repulse from static obstacles.
            InstinctEvent::ObstacleDetected { class, obb, .. }
                if class == ObjectClass::Tree
                    || class == ObjectClass::Building =>
            {
                self.handle_avoidance(&obb, cam_width, cam_height).await?;
            },
            // Follow identified persons.
            InstinctEvent::PersonTracked { obb, .. } => {
                self.handle_tracking(&obb, cam_width, cam_height).await?;
            },
            _ => {},
        }
        Ok(())
    }

    /// Calculates repulsive maneuvers to avoid imminent collisions.
    async fn handle_avoidance(
        &self,
        obb: &ringil_instinct::OrientedBoundingBox,
        cam_width: f32,
        cam_height: f32,
    ) -> Result<()> {
        let center_x = cam_width / 2.0;
        let relative_size = obb.height / cam_height;

        // Trigger avoidance if the object is too close (occupies too much of the frame).
        if relative_size > self.safe_distance / cam_height {
            // Determine turn direction: if object is left of center, turn right, and vice versa.
            let offset_x = obb.cx - center_x;
            let yaw_rate = if offset_x < 0.0 {
                self.max_yaw_rate
            } else {
                -self.max_yaw_rate
            };

            // Back away (negative vx) and yaw away from the obstacle.
            self.mavlink
                .send_velocity_ned(-self.repulse_gain, 0.0, 0.0, yaw_rate)
                .await?;
        }
        Ok(())
    }

    /// Computes P-loop adjustments to maintain a fixed distance and heading to a person.
    async fn handle_tracking(
        &self,
        obb: &ringil_instinct::OrientedBoundingBox,
        cam_width: f32,
        cam_height: f32,
    ) -> Result<()> {
        let center_x = cam_width / 2.0;
        let offset_x = obb.cx - center_x;

        // Horizontal error * gain = yaw correction
        let yaw_rate = offset_x * self.p_gain_yaw;

        // Depth error based on height ratio: (target - actual) * gain = forward velocity
        let target_size = self.person_safe_distance / cam_height;
        let current_size = obb.height / cam_height;
        let vx = (target_size - current_size) * self.p_gain_advance;

        self.mavlink
            .send_velocity_ned(vx, 0.0, 0.0, yaw_rate)
            .await?;
        Ok(())
    }
}
