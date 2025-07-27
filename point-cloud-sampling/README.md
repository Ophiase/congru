# Point Cloud Sampling

This crate defines a point cloud sampling algorithm that simulates the process of generating a corrupted point cloud from an original one. The sampling process includes statistical sampling from Gaussian distributions, adding uniform noise, and applying rigid transformations.

## Usage

To generate a corrupted point cloud:\
Generate an original point cloud and then apply the pipeline on it.

```rust
let original_point_cloud: PointCloud = ...;
let (corrupted_point_cloud, rotation, translation) = apply_sampling_pipeline(
    &original_point_cloud);
```