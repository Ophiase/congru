pub mod types;
pub mod point_cloud;
pub mod basic_ops;
pub mod merge_ops;
pub mod geometry;
pub mod transform;
pub mod linalg;
pub mod random_utils;
pub mod generation;

// Re-export main types and functions for convenience
pub use types::{ Vec3, Mat3, Point3D };
pub use point_cloud::{ PointCloud, PointCloudMetadata };
