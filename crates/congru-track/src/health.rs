/// When a frame's alignment is trusted enough to follow from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    /// Smallest share of the frame's points that must find the model within
    /// the gate: an occluder or clutter lowers it a little, a wrong pose a
    /// lot.
    pub min_inliers: f64,
    /// Largest noise of a frame, as a multiple of the noise learned so far.
    pub max_noise_ratio: f64,
}

impl Health {
    #[must_use]
    pub fn trusts(&self, inliers: f64, deviation: f64, learned: Option<f64>) -> bool {
        inliers >= self.min_inliers
            && learned.is_none_or(|known| deviation <= self.max_noise_ratio * known)
    }
}
