use point_cloud::{ types::Vec3, point_cloud::PointCloud };

#[test]
fn test_centroid_single_point() {
    let pc = PointCloud::from_points(vec![Vec3::new(1.0, 2.0, 3.0)]);
    let centroid = pc.centroid();
    assert_eq!(centroid, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_centroid_multiple_points() {
    let pc = PointCloud::from_points(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0)
        ]
    );
    let centroid = pc.centroid();
    assert_eq!(centroid, Vec3::new(0.5, 0.5, 0.5));
}

#[test]
fn test_centroid_empty() {
    let pc = PointCloud::new();
    let centroid = pc.centroid();
    assert_eq!(centroid, Vec3::zeros());
}

#[test]
fn test_centroid_symmetric_points() {
    let pc = PointCloud::from_points(vec![Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)]);
    let centroid = pc.centroid();
    assert_eq!(centroid, Vec3::new(0.0, 0.0, 0.0));
}

#[test]
fn test_bounding_box_single_point() {
    let pc = PointCloud::from_points(vec![Vec3::new(1.0, 2.0, 3.0)]);
    let bbox = pc.bounding_box().unwrap();
    assert_eq!(bbox.0, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(bbox.1, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_bounding_box_multiple_points() {
    let pc = PointCloud::from_points(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(0.5, -0.5, 2.0)
        ]
    );
    let (min, max) = pc.bounding_box().unwrap();
    assert_eq!(min, Vec3::new(-1.0, -1.0, -1.0));
    assert_eq!(max, Vec3::new(1.0, 1.0, 2.0));
}

#[test]
fn test_bounding_box_empty() {
    let pc = PointCloud::new();
    assert!(pc.bounding_box().is_none());
}

#[test]
fn test_bounding_box_axis_aligned() {
    let pc = PointCloud::from_points(
        vec![Vec3::new(10.0, 5.0, 0.0), Vec3::new(2.0, 8.0, -3.0), Vec3::new(15.0, 1.0, 7.0)]
    );
    let (min, max) = pc.bounding_box().unwrap();
    assert_eq!(min, Vec3::new(2.0, 1.0, -3.0));
    assert_eq!(max, Vec3::new(15.0, 8.0, 7.0));
}

#[test]
fn test_min_pairwise_distance_two_points() {
    let pc = PointCloud::from_points(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 4.0, 0.0) // Distance = 5.0
        ]
    );
    let min_dist = pc.min_pairwise_distance().unwrap();
    assert!((min_dist - 5.0).abs() < 1e-10);
}

#[test]
fn test_min_pairwise_distance_multiple_points() {
    let pc = PointCloud::from_points(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0), // Distance = 1.0 (minimum)
            Vec3::new(0.0, 2.0, 0.0), // Distance = 2.0
            Vec3::new(0.0, 0.0, 5.0) // Distance = 5.0
        ]
    );
    let min_dist = pc.min_pairwise_distance().unwrap();
    assert!((min_dist - 1.0).abs() < 1e-10);
}

#[test]
fn test_min_pairwise_distance_single_point() {
    let pc = PointCloud::from_points(vec![Vec3::new(1.0, 2.0, 3.0)]);
    assert!(pc.min_pairwise_distance().is_none());
}

#[test]
fn test_min_pairwise_distance_empty() {
    let pc = PointCloud::new();
    assert!(pc.min_pairwise_distance().is_none());
}

#[test]
fn test_min_pairwise_distance_3d() {
    let pc = PointCloud::from_points(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0), // Distance = sqrt(3) ≈ 1.732
            Vec3::new(0.5, 0.0, 0.0) // Distance = 0.5 (minimum)
        ]
    );
    let min_dist = pc.min_pairwise_distance().unwrap();
    assert!((min_dist - 0.5).abs() < 1e-10);
}

#[test]
fn test_min_pairwise_distance_identical_points() {
    let pc = PointCloud::from_points(vec![Vec3::new(1.0, 1.0, 1.0), Vec3::new(1.0, 1.0, 1.0)]);
    let min_dist = pc.min_pairwise_distance().unwrap();
    assert!((min_dist - 0.0).abs() < 1e-10);
}
