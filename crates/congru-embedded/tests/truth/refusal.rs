use congru_embedded::{Error, Icp, KdTree, Metric, Pose, Real, Workspace};

fn refuses<Float: Real>() {
    let points = [[Float::ZERO; 3]; 8];
    let tree = KdTree::new(&points).unwrap();
    let identity = Pose::identity();
    let mut enough = Workspace::new(points.len());
    let icp = Icp::<Float>::default();
    assert_eq!(
        icp.align(&points, &tree, &identity, &mut enough),
        Err(Error::MissingNormals)
    );
    let point_to_point = Icp {
        metric: Metric::PointToPoint,
        ..icp
    };
    assert!(matches!(
        point_to_point.align(&points, &tree, &identity, &mut Workspace::new(2)),
        Err(Error::WorkspaceTooSmall { .. })
    ));
    let gated = Icp {
        gate: Some(-Float::ONE),
        ..point_to_point
    };
    assert!(matches!(
        gated.align(&points, &tree, &identity, &mut enough),
        Err(Error::InvalidParameter { .. })
    ));
}

#[test]
fn bad_configurations_are_refused() {
    refuses::<f32>();
    refuses::<f64>();
}
