//! Byterack model handler.
//! Combines Kalman Filtering, IoU Matching, and ByteTrack logic.

use std::collections::HashSet;

use nalgebra::{SMatrix, SVector};

pub type StateVector = SVector<f32, 8>; // [x, y, a, h, vx, vy, va, vh]
pub type MeasurementVector = SVector<f32, 4>; // [x, y, a, h]
pub type CovarianceMatrix = SMatrix<f32, 8, 8>;
pub type MeasurementMatrix = SMatrix<f32, 4, 8>;

/// Standard Kalman Filter for constant velocity motion model.
#[derive(Debug, Clone)]
pub struct KalmanFilter {
    motion_mat: SMatrix<f32, 8, 8>,
    update_mat: MeasurementMatrix,
    std_weight_position: f32,
    std_weight_velocity: f32,
}

impl Default for KalmanFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl KalmanFilter {
    /// Initialize matrices for 8D state and 4D measurement.
    pub fn new() -> Self {
        let mut motion_mat = SMatrix::<f32, 8, 8>::identity();
        for i in 0..4 {
            motion_mat[(i, i + 4)] = 1.0;
        }
        let mut update_mat = MeasurementMatrix::zeros();
        for i in 0..4 {
            update_mat[(i, i)] = 1.0;
        }

        Self {
            motion_mat,
            update_mat,
            std_weight_position: 1.0 / 20.0,
            std_weight_velocity: 1.0 / 160.0,
        }
    }

    /// Creates initial state and covariance from first detection.
    pub fn initiate(
        &self,
        measurement: &MeasurementVector,
    ) -> (StateVector, CovarianceMatrix) {
        let mut mean = StateVector::zeros();
        for i in 0..4 {
            mean[i] = measurement[i];
        }
        let mut covariance = CovarianceMatrix::identity();
        // Scale uncertainty based on object height.
        let std = [
            2.0 * self.std_weight_position * measurement[3],
            2.0 * self.std_weight_position * measurement[3],
            1e-2,
            2.0 * self.std_weight_position * measurement[3],
            10.0 * self.std_weight_velocity * measurement[3],
            10.0 * self.std_weight_velocity * measurement[3],
            1e-5,
            10.0 * self.std_weight_velocity * measurement[3],
        ];
        for i in 0..8 {
            covariance[(i, i)] = std[i].powi(2);
        }
        (mean, covariance)
    }

    /// Predicts next state: x' = Fx and P' = FPF^T + Q.
    pub fn predict(
        &self,
        mean: &StateVector,
        covariance: &CovarianceMatrix,
    ) -> (StateVector, CovarianceMatrix) {
        let std_pos = [
            self.std_weight_position * mean[3],
            self.std_weight_position * mean[3],
            1e-2,
            self.std_weight_position * mean[3],
        ];
        let std_vel = [
            self.std_weight_velocity * mean[3],
            self.std_weight_velocity * mean[3],
            1e-5,
            self.std_weight_velocity * mean[3],
        ];
        let mut motion_cov = CovarianceMatrix::zeros();
        for i in 0..4 {
            motion_cov[(i, i)] = std_pos[i].powi(2);
            motion_cov[(i + 4, i + 4)] = std_vel[i].powi(2);
        }
        let mean = self.motion_mat * mean;
        let mut covariance =
            self.motion_mat * covariance * self.motion_mat.transpose()
                + motion_cov;

        // Enforce numerical symmetry to prevent float32 drift over long sequences.
        covariance = (covariance + covariance.transpose()) * 0.5;

        (mean, covariance)
    }

    /// Updates state with new measurement: x = x + Ky and P = (I - KH)P.
    pub fn update(
        &self,
        mean: &StateVector,
        covariance: &CovarianceMatrix,
        measurement: &MeasurementVector,
    ) -> (StateVector, CovarianceMatrix) {
        let projected_mean = self.update_mat * mean;
        let projected_cov =
            self.update_mat * covariance * self.update_mat.transpose();
        let std = [
            self.std_weight_position * mean[3],
            self.std_weight_position * mean[3],
            1e-1,
            self.std_weight_position * mean[3],
        ];
        let mut diag = SMatrix::<f32, 4, 4>::zeros();
        for i in 0..4 {
            diag[(i, i)] = std[i].powi(2);
        }

        let innovation_cov = projected_cov + diag;
        // Handle matrix inversion for Kalman Gain.
        let inv_innovation_cov = innovation_cov
            .try_inverse()
            .unwrap_or_else(SMatrix::<f32, 4, 4>::identity);
        let kalman_gain =
            covariance * self.update_mat.transpose() * inv_innovation_cov;
        let innovation = measurement - projected_mean;

        let new_mean = mean + kalman_gain * innovation;
        let mut new_covariance = covariance
            - kalman_gain * innovation_cov * kalman_gain.transpose();

        // Enforce numerical symmetry to prevent float32 drift over long sequences.
        new_covariance = (new_covariance + new_covariance.transpose()) * 0.5;

        (new_mean, new_covariance)
    }
}

/// Calculate Intersection over Union for two boxes.
fn iou(box1: &[f32; 4], box2: &[f32; 4]) -> f32 {
    let x1 = box1[0].max(box2[0]);
    let y1 = box1[1].max(box2[1]);
    let x2 = (box1[0] + box1[2]).min(box2[0] + box2[2]);
    let y2 = (box1[1] + box1[3]).min(box2[1] + box2[3]);
    let w = (x2 - x1).max(0.0);
    let h = (y2 - y1).max(0.0);
    let inter_area = w * h;
    let union_area = (box1[2] * box1[3]) + (box2[2] * box2[3]) - inter_area;
    if union_area <= 0.0 {
        0.0
    } else {
        inter_area / union_area
    }
}

/// Lifecycle states of a tracklet.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TrackState {
    New,
    Tracked,
    Lost,
    Removed,
}

/// Individual tracklet holding state and Kalman statistics.
#[derive(Debug, Clone)]
pub struct STrack {
    pub tlwh: [f32; 4], // Top-left corner, width, height.
    pub score: f32,
    pub class_id: i64,
    pub track_id: u64,
    pub state: TrackState,
    pub is_activated: bool,
    pub tracklet_len: usize,
    pub start_frame: usize,
    pub frame_id: usize,
    mean: StateVector,
    covariance: CovarianceMatrix,
}

impl STrack {
    /// Create a new [`STrack`].
    pub fn new(tlwh: [f32; 4], score: f32, class_id: i64) -> Self {
        Self {
            tlwh,
            score,
            class_id,
            track_id: 0,
            state: TrackState::New,
            is_activated: false,
            tracklet_len: 0,
            start_frame: 0,
            frame_id: 0,
            mean: StateVector::zeros(),
            covariance: CovarianceMatrix::identity(),
        }
    }

    /// Convert [x, y, w, h] to [center_x, center_y, aspect_ratio, height].
    fn tlwh_to_xyah(tlwh: &[f32; 4]) -> MeasurementVector {
        MeasurementVector::new(
            tlwh[0] + tlwh[2] / 2.0,
            tlwh[1] + tlwh[3] / 2.0,
            tlwh[2] / tlwh[3].max(1e-6),
            tlwh[3],
        )
    }

    /// Convert state vector back to bounding box format.
    fn xyah_to_tlwh(state: &StateVector) -> [f32; 4] {
        let w = state[2] * state[3];
        let h = state[3];
        [state[0] - w / 2.0, state[1] - h / 2.0, w, h]
    }

    /// Advance the track's Kalman filter to the current time.
    pub fn predict(&mut self, kf: &KalmanFilter) {
        if self.state != TrackState::Tracked {
            self.mean[7] = 0.0; // Stop height velocity if lost.
        }
        let (mean, cov) = kf.predict(&self.mean, &self.covariance);
        self.mean = mean;
        self.covariance = cov;
        self.tlwh = Self::xyah_to_tlwh(&self.mean);
    }

    /// Update tracklet with a matched detection.
    pub fn update(
        &mut self,
        new_track: &STrack,
        frame_id: usize,
        kf: &KalmanFilter,
    ) -> STrack {
        self.frame_id = frame_id;
        self.tracklet_len += 1;
        self.state = TrackState::Tracked;
        self.is_activated = true;
        self.score = new_track.score;
        self.tlwh = new_track.tlwh;

        let measurement = Self::tlwh_to_xyah(&self.tlwh);
        let (mean, cov) =
            kf.update(&self.mean, &self.covariance, &measurement);
        self.mean = mean;
        self.covariance = cov;
        self.clone()
    }

    /// Activate a new tracklet.
    pub fn activate(
        &mut self,
        kf: &KalmanFilter,
        frame_id: usize,
        track_id: u64,
    ) {
        let measurement = Self::tlwh_to_xyah(&self.tlwh);
        let (mean, cov) = kf.initiate(&measurement);
        self.mean = mean;
        self.covariance = cov;
        self.track_id = track_id;
        self.state = TrackState::Tracked;
        self.is_activated = true;
        self.frame_id = frame_id;
        self.start_frame = frame_id;
        self.tracklet_len = 0;
    }

    pub fn mark_lost(&mut self) {
        self.state = TrackState::Lost;
    }
}

/// ByteTrack core manager.
pub struct ByteTrack {
    pub tracked_stracks: Vec<STrack>,
    pub lost_stracks: Vec<STrack>,
    pub lost_ids: HashSet<u64>,
    frame_id: usize,
    track_buffer: usize,
    track_thresh: f32,
    match_thresh: f32,
    det_thresh: f32,
    kalman_filter: KalmanFilter,
    next_id: u64,
    det_high_cache: Vec<STrack>,
    det_low_cache: Vec<STrack>,
    pool_cache: Vec<STrack>,
    pool_unmatched_indices: Vec<usize>,
}

impl ByteTrack {
    pub fn new(
        track_thresh: f32,
        track_buffer: usize,
        match_thresh: f32,
        det_thresh: f32,
    ) -> Self {
        Self {
            tracked_stracks: Vec::new(),
            lost_stracks: Vec::new(),
            lost_ids: HashSet::new(),
            frame_id: 0,
            track_buffer,
            track_thresh,
            match_thresh,
            det_thresh,
            kalman_filter: KalmanFilter::new(),
            next_id: 1,
            det_high_cache: Vec::with_capacity(32),
            det_low_cache: Vec::with_capacity(32),
            pool_cache: Vec::with_capacity(64),
            pool_unmatched_indices: Vec::with_capacity(64),
        }
    }

    /// Process a new frame of detections.
    pub fn update(
        &mut self,
        detections: &[([f32; 4], f32, i64)],
    ) -> Vec<STrack> {
        self.frame_id += 1;
        self.lost_ids.clear();

        // Clear cached tracking layers from previous execution loop.
        self.det_high_cache.clear();
        self.det_low_cache.clear();
        self.pool_cache.clear();
        self.pool_unmatched_indices.clear();

        // Parse borrowed slice inputs directly into pre-allocated memory caches.
        for &(tlwh, score, class_id) in detections {
            let track = STrack::new(tlwh, score, class_id);
            if score >= self.track_thresh {
                self.det_high_cache.push(track);
            } else if score >= self.det_thresh {
                self.det_low_cache.push(track);
            }
        }

        // Predict current positions of tracks.
        for mut t in self.tracked_stracks.drain(..) {
            t.predict(&self.kalman_filter);
            self.pool_cache.push(t);
        }
        for mut t in self.lost_stracks.drain(..) {
            t.predict(&self.kalman_filter);
            self.pool_cache.push(t);
        }

        let (matches_high, unmatch_high, unmatch_trk) = Self::associate(
            &self.pool_cache,
            &self.det_high_cache,
            self.match_thresh,
        );

        let mut new_tracked = Vec::with_capacity(
            self.pool_cache.len() + self.det_high_cache.len(),
        );

        // Track indices that were matched during first stage.
        let mut matched_pool_indices =
            HashSet::with_capacity(matches_high.len());

        for (trk_idx, det_idx) in matches_high {
            matched_pool_indices.insert(trk_idx);
            let updated_track = self.pool_cache[trk_idx].update(
                &self.det_high_cache[det_idx],
                self.frame_id,
                &self.kalman_filter,
            );
            new_tracked.push(updated_track);
        }

        // Second association pass focuses strictly on remaining non-lost
        // active tracks against low-confidence detections.
        for &idx in &unmatch_trk {
            if self.pool_cache[idx].state == TrackState::Tracked {
                self.pool_unmatched_indices.push(idx);
            }
        }

        let secondary_tracks_view: Vec<STrack> = self
            .pool_unmatched_indices
            .iter()
            .map(|&idx| self.pool_cache[idx].clone())
            .collect();

        let (matches_low, _, unmatch_low_trk) =
            Self::associate(&secondary_tracks_view, &self.det_low_cache, 0.5);

        for (view_idx, det_idx) in matches_low {
            let actual_pool_idx = self.pool_unmatched_indices[view_idx];
            matched_pool_indices.insert(actual_pool_idx);

            let updated_track = self.pool_cache[actual_pool_idx].update(
                &self.det_low_cache[det_idx],
                self.frame_id,
                &self.kalman_filter,
            );
            new_tracked.push(updated_track);
        }

        for &view_idx in &unmatch_low_trk {
            let actual_pool_idx = self.pool_unmatched_indices[view_idx];
            let mut t = self.pool_cache[actual_pool_idx].clone();
            if t.state != TrackState::Lost {
                t.mark_lost();
                self.lost_ids.insert(t.track_id);
            }
            self.lost_stracks.push(t);
        }

        for &idx in &unmatch_trk {
            if !matched_pool_indices.contains(&idx)
                && self.pool_cache[idx].state == TrackState::Lost
            {
                self.lost_stracks.push(self.pool_cache[idx].clone());
            }
        }

        for &det_idx in &unmatch_high {
            let mut new_t = self.det_high_cache[det_idx].clone();
            if new_t.score >= self.track_thresh {
                new_t.activate(
                    &self.kalman_filter,
                    self.frame_id,
                    self.next_id,
                );
                self.next_id += 1;
                new_tracked.push(new_t);
            }
        }

        // Evict expired tracklets out of buffer window constraints.
        let current_frame = self.frame_id;
        let max_buffer = self.track_buffer;
        self.lost_stracks
            .retain(|t| current_frame - t.frame_id <= max_buffer);

        self.tracked_stracks = new_tracked;

        self.tracked_stracks
            .iter()
            .filter(|t| t.is_activated)
            .cloned()
            .collect()
    }

    pub fn get_lost_track_ids(&self) -> Vec<u64> {
        self.lost_ids.iter().copied().collect()
    }

    /// Fast Greedy Assignment based on IoU distance.
    fn associate(
        tracks: &[STrack],
        detections: &[STrack],
        threshold: f32,
    ) -> (Vec<(usize, usize)>, Vec<usize>, Vec<usize>) {
        if tracks.is_empty() {
            return (Vec::new(), (0..detections.len()).collect(), Vec::new());
        }
        if detections.is_empty() {
            return (Vec::new(), Vec::new(), (0..tracks.len()).collect());
        }

        // Compute 1-IoU cost matrix.
        let mut costs: Vec<(f32, usize, usize)> =
            Vec::with_capacity(tracks.len() * detections.len());
        for (r, trk) in tracks.iter().enumerate() {
            for (c, det) in detections.iter().enumerate() {
                let cost = 1.0 - iou(&trk.tlwh, &det.tlwh);
                costs.push((cost, r, c));
            }
        }

        costs.sort_unstable_by(|a, b| a.0.to_bits().cmp(&b.0.to_bits()));

        let mut matches =
            Vec::with_capacity(tracks.len().min(detections.len()));
        let mut unmatched_tracks: HashSet<usize> = (0..tracks.len()).collect();
        let mut unmatched_dets: HashSet<usize> =
            (0..detections.len()).collect();

        for (cost, trk_idx, det_idx) in costs {
            if cost > threshold {
                continue;
            }
            if unmatched_tracks.contains(&trk_idx)
                && unmatched_dets.contains(&det_idx)
            {
                matches.push((trk_idx, det_idx));
                unmatched_tracks.remove(&trk_idx);
                unmatched_dets.remove(&det_idx);
            }
        }

        (
            matches,
            unmatched_dets.into_iter().collect(),
            unmatched_tracks.into_iter().collect(),
        )
    }
}
