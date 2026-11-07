use alloc::boxed::Box;

use crate::Real;

pub(super) const NO_MATCH: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Match<Float> {
    pub(super) target: u32,
    pub(super) weight: Float,
}

/// The one buffer of an alignment, sized once and reused by every iteration of
/// every alignment: an index and a weight per source point. The moved points
/// and the normal equations are recomputed on the fly, never stored.
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace<Float> {
    pub(super) matches: Box<[Match<Float>]>,
}

impl<Float: Real> Workspace<Float> {
    /// Enough for sources of up to `source_points` points.
    #[must_use]
    pub fn new(source_points: usize) -> Self {
        let empty = Match {
            target: NO_MATCH,
            weight: Float::ZERO,
        };
        Self {
            matches: alloc::vec![empty; source_points].into_boxed_slice(),
        }
    }

    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        size_of_val(&*self.matches)
    }
}
