use super::{Aligner, Estimate, Frame, start};
use crate::{Distance, Error, Icp, Kernel, PointCloud, RigidTransform, Target};

/// The median length of a three-dimensional standard normal vector.
const CHI_THREE_MEDIAN: f64 = 1.538;
/// Tukey's constant for 95 % efficiency on Gaussian residuals.
const TUKEY_DEVIATIONS: f64 = 4.685;
const GATE_DEVIATIONS: f64 = 5.0;

/// ICP from the guess in a few stages; before each, the noise is estimated
/// from the median distance of the moved scan to the model, and the Tukey
/// kernel and the gate are set from it. No scale to tune: it works on a
/// clean scan and a noisy one alike, and outliers barely move a median.
#[derive(Debug)]
pub struct Adaptive {
    pub name: String,
    /// Run at every stage, its kernel and gate replaced.
    pub icp: Icp,
    /// The noise assumed before the first stage, when nothing is aligned
    /// yet: a generous share of the model's size.
    pub first_deviation: Distance,
    pub stages: u32,
    pub target: Target,
}

impl Aligner for Adaptive {
    fn name(&self) -> &str {
        &self.name
    }

    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error> {
        let mut pose = start(frame);
        let mut deviation = self.first_deviation.get();
        let mut last = None;
        for _ in 0..self.stages.max(1) {
            let stage = Icp {
                kernel: Kernel::Tukey(Distance::new(TUKEY_DEVIATIONS * deviation)?),
                max_correspondence_distance: Some(Distance::new(GATE_DEVIATIONS * deviation)?),
                ..self.icp
            };
            let alignment = stage.align(frame.scan, &self.target, &pose)?;
            pose = alignment.transform;
            deviation = noise(frame.scan, &self.target, &pose).max(deviation / 4.0);
            last = Some(alignment);
        }
        last.map(Estimate::from_alignment).ok_or(Error::Degenerate)
    }
}

/// The noise of the scan moved by `pose`, read from the median distance to
/// the model as the deviation of an isotropic Gaussian.
fn noise(scan: &PointCloud, target: &Target, pose: &RigidTransform) -> f64 {
    let mut distances: Vec<f64> = scan
        .points()
        .iter()
        .map(|point| {
            target
                .index()
                .nearest(&pose.apply(point))
                .squared_distance
                .sqrt()
        })
        .collect();
    let middle = distances.len() / 2;
    distances.select_nth_unstable_by(middle, f64::total_cmp);
    distances[middle] / CHI_THREE_MEDIAN
}
