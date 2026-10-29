use congru::Alignment;

/// When a frame's alignment is trusted enough to follow from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    /// Smallest share of the frame's points that must find the model: an
    /// occluder or clutter lowers it a little, a wrong pose a lot.
    pub min_inliers: f64,
    /// Largest residual, in the unit of the points: a few times the noise.
    pub max_rmse: f64,
}

impl Health {
    #[must_use]
    pub fn trusts(&self, alignment: &Alignment) -> bool {
        alignment.inlier_fraction >= self.min_inliers && alignment.rmse <= self.max_rmse
    }
}
