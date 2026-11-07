//! What a robot needs of registration on a small board: closed forms and ICP
//! over [`Real`], `f32` or `f64`, without the standard library. Allocation
//! happens once, when the [`KdTree`] and the [`Workspace`] are made; the loop
//! of [`Icp::align`] allocates nothing.

#![no_std]

extern crate alloc;

mod error;
mod fit;
mod icp;
mod linear;
mod real;
mod search;

pub use error::Error;
pub use fit::{Kernel, Pair, Similarity, kabsch, similarity};
pub use icp::{Alignment, Convergence, Icp, Metric, StopReason, Workspace};
pub use linear::{Pose, UnitQuaternion, Vector3};
pub use real::Real;
pub use search::{KdTree, Nearest};
