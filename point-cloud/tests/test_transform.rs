use point_cloud::{ types::{ Vec3, Mat3 }, point_cloud::{ PointCloud, PointCloudMetadata } };

#[test]
fn test_transform_identity() {
    let mut pc = PointCloud::from_points(vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)]);

    let identity = Mat3::identity();
    let zero_translation = Vec3::zeros();

    let original_points = pc.points.clone();
    pc.transform(&identity, &zero_translation);

    // Points should remain unchanged
    assert_eq!(pc.points, original_points);
}

#[test]
fn test_transform_translation_only() {
    let mut pc = PointCloud::from_points(vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 0.0, 0.0)]);

    let identity = Mat3::identity();
    let translation = Vec3::new(10.0, 20.0, 30.0);

    pc.transform(&identity, &translation);

    assert_eq!(pc.points[0], Vec3::new(11.0, 22.0, 33.0));
    assert_eq!(pc.points[1], Vec3::new(10.0, 20.0, 30.0));
}

#[test]
fn test_transform_rotation_only() {
    let mut pc = PointCloud::from_points(vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)]);

    // 90-degree rotation around Z-axis
    let rotation = Mat3::new(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let zero_translation = Vec3::zeros();

    pc.transform(&rotation, &zero_translation);

    // (1,0,0) -> (0,1,0), (0,1,0) -> (-1,0,0)
    assert!((pc.points[0] - Vec3::new(0.0, 1.0, 0.0)).norm() < 1e-10);
    assert!((pc.points[1] - Vec3::new(-1.0, 0.0, 0.0)).norm() < 1e-10);
}

#[test]
fn test_transform_rotation_and_translation() {
    let mut pc = PointCloud::from_points(vec![Vec3::new(1.0, 0.0, 0.0)]);

    // 90-degree rotation around Z-axis
    let rotation = Mat3::new(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let translation = Vec3::new(5.0, 5.0, 5.0);

    pc.transform(&rotation, &translation);

    // (1,0,0) -> (0,1,0) + (5,5,5) = (5,6,5)
    assert!((pc.points[0] - Vec3::new(5.0, 6.0, 5.0)).norm() < 1e-10);
}

#[test]
fn test_transform_empty_cloud() {
    let mut pc = PointCloud::new();
    let rotation = Mat3::identity();
    let translation = Vec3::new(1.0, 2.0, 3.0);

    pc.transform(&rotation, &translation);

    assert_eq!(pc.len(), 0);
}

#[test]
fn test_transformed_creates_copy() {
    let pc = PointCloud::from_points(vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)]);

    let rotation = Mat3::identity();
    let translation = Vec3::new(10.0, 10.0, 10.0);

    let transformed = pc.transformed(&rotation, &translation);

    // Original should be unchanged
    assert_eq!(pc.points[0], Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(pc.points[1], Vec3::new(4.0, 5.0, 6.0));

    // Copy should be transformed
    assert_eq!(transformed.points[0], Vec3::new(11.0, 12.0, 13.0));
    assert_eq!(transformed.points[1], Vec3::new(14.0, 15.0, 16.0));
}

#[test]
fn test_transformed_preserves_metadata() {
    let metadata = PointCloudMetadata {
        name: "test_cloud".to_string(),
        timestamp: Some("2025-01-01".to_string()),
        source: Some("test".to_string()),
    };

    let pc = PointCloud::with_metadata(vec![Vec3::new(1.0, 2.0, 3.0)], metadata.clone());

    let rotation = Mat3::identity();
    let translation = Vec3::new(1.0, 1.0, 1.0);

    let transformed = pc.transformed(&rotation, &translation);

    assert!(transformed.metadata.is_some());
    let trans_metadata = transformed.metadata.unwrap();
    assert_eq!(trans_metadata.name, metadata.name);
    assert_eq!(trans_metadata.timestamp, metadata.timestamp);
    assert_eq!(trans_metadata.source, metadata.source);
}

#[test]
fn test_transform_centroid_behavior() {
    let mut pc = PointCloud::from_points(
        vec![Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)]
    );

    let original_centroid = pc.centroid();
    assert_eq!(original_centroid, Vec3::zeros());

    let rotation = Mat3::identity();
    let translation = Vec3::new(5.0, 10.0, 15.0);

    pc.transform(&rotation, &translation);

    let new_centroid = pc.centroid();
    let expected_centroid = original_centroid + translation;

    assert!((new_centroid - expected_centroid).norm() < 1e-10);
}

#[test]
fn test_multiple_transforms() {
    let mut pc = PointCloud::from_points(vec![Vec3::new(1.0, 0.0, 0.0)]);

    // First transformation: translate
    let identity = Mat3::identity();
    let translation1 = Vec3::new(1.0, 1.0, 1.0);
    pc.transform(&identity, &translation1);

    assert_eq!(pc.points[0], Vec3::new(2.0, 1.0, 1.0));

    // Second transformation: rotate 90 degrees around Z
    let rotation = Mat3::new(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let zero_translation = Vec3::zeros();
    pc.transform(&rotation, &zero_translation);

    // (2,1,1) -> (-1,2,1)
    assert!((pc.points[0] - Vec3::new(-1.0, 2.0, 1.0)).norm() < 1e-10);
}

#[test]
fn test_transform_preserves_relative_distances() {
    let mut pc = PointCloud::from_points(
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)]
    );

    // Calculate original distances
    let original_dist_01 = (pc.points[0] - pc.points[1]).norm();
    let original_dist_02 = (pc.points[0] - pc.points[2]).norm();
    let original_dist_12 = (pc.points[1] - pc.points[2]).norm();

    // Apply transformation
    let rotation = Mat3::new(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let translation = Vec3::new(10.0, 20.0, 30.0);
    pc.transform(&rotation, &translation);

    // Calculate new distances
    let new_dist_01 = (pc.points[0] - pc.points[1]).norm();
    let new_dist_02 = (pc.points[0] - pc.points[2]).norm();
    let new_dist_12 = (pc.points[1] - pc.points[2]).norm();

    // Distances should be preserved (rigid transformation)
    assert!((original_dist_01 - new_dist_01).abs() < 1e-10);
    assert!((original_dist_02 - new_dist_02).abs() < 1e-10);
    assert!((original_dist_12 - new_dist_12).abs() < 1e-10);
}
