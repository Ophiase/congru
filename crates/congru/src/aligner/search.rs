use super::{Aligner, Estimate, Frame};
use crate::{Error, Multistart, Target};

/// [`Multistart`] over all rotations; the guess is ignored.
#[derive(Debug)]
pub struct Search {
    pub name: String,
    pub multistart: Multistart,
    pub target: Target,
}

impl Aligner for Search {
    fn name(&self) -> &str {
        &self.name
    }

    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error> {
        Ok(Estimate::from_alignment(
            self.multistart.align(frame.scan, &self.target)?,
        ))
    }
}
