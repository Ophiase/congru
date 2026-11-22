use super::{Aligner, Estimate, Frame, start};
use crate::{Distance, Error, Icp, Kernel, Target};

/// ICP from the guess with a Cauchy kernel whose scale halves at each
/// stage, from `coarsest` down to `finest`: early stages see the whole
/// shape and widen the basin of convergence, late ones ignore what is far
/// and gain the accuracy of a tight kernel (graduated non-convexity).
#[derive(Debug)]
pub struct Annealed {
    pub name: String,
    /// Run at every stage, its kernel and gate replaced.
    pub icp: Icp,
    pub coarsest: Distance,
    pub finest: Distance,
    pub target: Target,
}

/// A pair farther than this many kernel scales is dropped at each stage.
const GATE_SCALES: f64 = 4.0;

impl Aligner for Annealed {
    fn name(&self) -> &str {
        &self.name
    }

    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error> {
        let mut pose = start(frame);
        let mut scale = self.coarsest.get();
        loop {
            let finest = scale <= self.finest.get();
            let stage = Icp {
                kernel: Kernel::Cauchy(Distance::new(scale.max(self.finest.get()))?),
                max_correspondence_distance: Some(Distance::new(
                    GATE_SCALES * scale.max(self.finest.get()),
                )?),
                ..self.icp
            };
            let alignment = stage.align(frame.scan, &self.target, &pose)?;
            if finest {
                return Ok(Estimate::from_alignment(alignment));
            }
            pose = alignment.transform;
            scale /= 2.0;
        }
    }
}
