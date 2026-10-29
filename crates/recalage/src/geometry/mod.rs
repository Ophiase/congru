mod cloud;
mod mesh;
mod oriented;
mod transform;
mod units;

pub use cloud::PointCloud;
pub(crate) use cloud::{count_as_float, mean};
pub use mesh::TriangleMesh;
pub use oriented::OrientedCloud;
pub use transform::RigidTransform;
pub use units::{Distance, Fraction};

pub type Point = nalgebra::Point3<f64>;

pub type Vector = nalgebra::Vector3<f64>;
