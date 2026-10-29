use super::{Aligner, Estimate, Frame, start};
use crate::{Error, Icp, Target};

/// One [`Icp`] run from the frame's guess, the identity without one.
#[derive(Debug)]
pub struct Local {
    pub name: String,
    pub icp: Icp,
    pub target: Target,
}

impl Aligner for Local {
    fn name(&self) -> &str {
        &self.name
    }

    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error> {
        Ok(Estimate::from_alignment(self.icp.align(
            frame.scan,
            &self.target,
            &start(frame),
        )?))
    }
}
