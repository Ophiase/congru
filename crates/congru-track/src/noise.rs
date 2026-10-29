use congru::{PointCloud, RigidTransform, Target};

const MEMORY: f64 = 0.3;
/// The median of the length of a three-dimensional standard normal vector.
const CHI_THREE_MEDIAN: f64 = 1.538;

/// The noise of the frames trusted so far, as a moving average: the
/// sensor's standard deviation as the tracker sees it, never below a floor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LearnedNoise {
    floor: f64,
    deviation: Option<f64>,
}

impl LearnedNoise {
    #[must_use]
    pub const fn new(floor: f64) -> Self {
        Self {
            floor,
            deviation: None,
        }
    }

    #[must_use]
    pub fn deviation(&self) -> Option<f64> {
        self.deviation.map(|deviation| deviation.max(self.floor))
    }

    pub fn learn(&mut self, deviation: f64) {
        self.deviation = Some(
            self.deviation
                .map_or(deviation, |known| known + MEMORY * (deviation - known)),
        );
    }
}

/// The noise of one aligned frame, robust to outliers: the median distance
/// from the moved scan to the target, read as the standard deviation of an
/// isotropic Gaussian. An occluder or clutter barely moves a median.
#[must_use]
pub fn deviation_of(scan: &PointCloud, target: &Target, transform: &RigidTransform) -> f64 {
    let mut distances: Vec<f64> = scan
        .points()
        .iter()
        .map(|point| {
            target
                .index()
                .nearest(&transform.apply(point))
                .squared_distance
                .sqrt()
        })
        .collect();
    let middle = distances.len() / 2;
    distances.select_nth_unstable_by(middle, f64::total_cmp);
    distances[middle] / CHI_THREE_MEDIAN
}
