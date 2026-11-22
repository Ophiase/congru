//! Rigid registration of point clouds and meshes, measured against a known truth.
//!
//! Three entry points, from the most to the least informed:
//!
//! - [`fit_rigid`]: correspondences are known (fiducials, markers);
//!   closed-form weighted Kabsch–Umeyama.
//! - [`Icp`]: a good initial pose is known; iterative closest point,
//!   point-to-point or point-to-plane, with robust kernels and trimming.
//! - [`Multistart`]: no initial pose; ICP from rotations spread
//!   uniformly over SO(3), the best one kept.
//!
//! ```
//! use congru::{Icp, Point, PointCloud, RigidTransform, Target};
//!
//! let model: Vec<Point> = (0..400)
//!     .map(|index| {
//!         let along = f64::from(index % 20) * 0.05;
//!         let across = f64::from(index / 20) * 0.05;
//!         Point::new(along, across, (3.0 * along).sin() * (2.0 * across).cos())
//!     })
//!     .collect();
//! let target = Target::with_estimated_normals(PointCloud::new(model.clone())?, 8)?;
//! let truth = RigidTransform::from_axis_angle(nalgebra::Vector3::z() * 0.05, [0.02, 0.0, 0.01]);
//! let source = PointCloud::new(model)?.transformed(&truth.inverse());
//!
//! let alignment = Icp::default().align(&source, &target, &RigidTransform::identity())?;
//! assert!(alignment.transform.rotation_error_to(&truth) < 1e-6);
//! # Ok::<(), congru::Error>(())
//! ```

mod aligner;
mod error;
mod fit;
mod geometry;
mod icp;
mod search;

pub use aligner::{
    Adaptive, Aligner, Annealed, Consensus, Estimate, Frame, Guarded, Local, Search,
};
pub use error::Error;
pub use fit::{
    Fit, Kernel, MotionKind, Observability, Prior, SimilarFit, WeakMotion, fit_rigid,
    fit_similarity,
};
pub use geometry::{
    Distance, Fraction, OrientedCloud, Point, PointCloud, RigidTransform, TriangleMesh, Vector,
};
pub use icp::{Alignment, Convergence, Icp, Metric, StopReason, Target};
pub use icp::{Certificate, Certifier, Multistart};
pub use search::{
    Closest, DistanceField, Grid, MeshIndex, NearestNeighbors, Neighbor, Surface, map,
};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExample;
