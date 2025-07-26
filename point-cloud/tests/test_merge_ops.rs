use point_cloud::{ types::Vec3, point_cloud::{ PointCloud } };

#[test]
fn test_merge_basic() {
    let mut pc1 = PointCloud::from_points(vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)]);

    let pc2 = PointCloud::from_points(vec![Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 2.0, 0.0)]);

    assert_eq!(pc1.len(), 2);
    assert_eq!(pc2.len(), 2);

    pc1.merge(&pc2);

    assert_eq!(pc1.len(), 4);
    assert_eq!(pc1.points[0], Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(pc1.points[1], Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(pc1.points[2], Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(pc1.points[3], Vec3::new(0.0, 2.0, 0.0));
}

#[test]
fn test_merge_empty_clouds() {
    let mut pc1 = PointCloud::new();
    let pc2 = PointCloud::from_points(vec![Vec3::new(1.0, 2.0, 3.0)]);

    pc1.merge(&pc2);
    assert_eq!(pc1.len(), 1);
    assert_eq!(pc1.points[0], Vec3::new(1.0, 2.0, 3.0));

    let pc3 = PointCloud::new();
    pc1.merge(&pc3);
    assert_eq!(pc1.len(), 1); // Should remain unchanged
}

#[test]
fn test_merge_multiple_clouds() {
    let pc1 = PointCloud::from_points(vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0)]);

    let pc2 = PointCloud::from_points(vec![Vec3::new(0.0, 1.0, 0.0)]);

    let pc3 = PointCloud::from_points(
        vec![Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, 3.0)]
    );

    let merged = PointCloud::merge_multiple(&[pc1, pc2, pc3]);

    assert_eq!(merged.len(), 6);
    assert_eq!(merged.points[0], Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(merged.points[1], Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(merged.points[2], Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(merged.points[3], Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(merged.points[4], Vec3::new(0.0, 0.0, 2.0));
    assert_eq!(merged.points[5], Vec3::new(0.0, 0.0, 3.0));
}

#[test]
fn test_merge_multiple_empty() {
    let empty_clouds: Vec<PointCloud> = vec![];
    let merged = PointCloud::merge_multiple(&empty_clouds);
    assert_eq!(merged.len(), 0);
}

#[test]
fn test_deduplicate_basic() {
    let mut pc = PointCloud::from_points(
        vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.01, 0.0, 0.0), // Close to first point
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(1.005, 0.0, 0.0) // Also close to first point
        ]
    );

    assert_eq!(pc.len(), 4);

    pc.deduplicate(0.05); // tolerance of 0.05

    assert_eq!(pc.len(), 2);
    assert_eq!(pc.points[0], Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(pc.points[1], Vec3::new(2.0, 0.0, 0.0));
}

#[test]
fn test_deduplicate_no_duplicates() {
    let mut pc = PointCloud::from_points(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0)
        ]
    );

    let original_len = pc.len();
    pc.deduplicate(0.1);

    assert_eq!(pc.len(), original_len); // Should remain unchanged
}

#[test]
fn test_deduplicate_all_same() {
    let mut pc = PointCloud::from_points(
        vec![
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.001, 1.001, 1.001),
            Vec3::new(0.999, 0.999, 0.999),
            Vec3::new(1.002, 1.002, 1.002)
        ]
    );

    pc.deduplicate(0.01);

    assert_eq!(pc.len(), 1);
    assert_eq!(pc.points[0], Vec3::new(1.0, 1.0, 1.0));
}

#[test]
fn test_deduplicate_empty() {
    let mut pc = PointCloud::new();
    pc.deduplicate(0.1);
    assert_eq!(pc.len(), 0);
}

#[test]
fn test_deduplicate_3d_distance() {
    let mut pc = PointCloud::from_points(
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.05, 0.05, 0.05), // Distance ≈ 0.087
            Vec3::new(0.1, 0.0, 0.0) // Distance = 0.1
        ]
    );

    pc.deduplicate(0.1); // Should keep first and third

    assert_eq!(pc.len(), 2);
    assert_eq!(pc.points[0], Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(pc.points[1], Vec3::new(0.1, 0.0, 0.0));
}

#[test]
fn test_merge_preserves_order() {
    let mut pc1 = PointCloud::from_points(vec![Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)]);

    let pc2 = PointCloud::from_points(vec![Vec3::new(3.0, 3.0, 3.0), Vec3::new(4.0, 4.0, 4.0)]);

    pc1.merge(&pc2);

    // Verify order is preserved
    for i in 1..=4 {
        let expected = Vec3::new(i as f64, i as f64, i as f64);
        assert_eq!(pc1.points[i - 1], expected);
    }
}
