use std::f64::consts::FRAC_PI_2;

use congru::{Error, RigidTransform, TriangleMesh};
use congru_sim::Solids;
use nalgebra::Vector3;

const SEGMENTS: u32 = 24;

/// A car: a body, a cabin set back, four wheels along the width.
pub fn car([length, width, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    let (half_length, half_width) = (length / 2.0, width / 2.0);
    let radius = 0.18 * height;
    let tread = 0.12 * width;
    let wheel = Solids::default()
        .cylinder([0.0, 0.0, -tread / 2.0], radius, tread, SEGMENTS)
        .build()?;
    let mut solids = Solids::default();
    solids
        .cuboid(
            [-half_length, -half_width, radius],
            [half_length, half_width, 0.55 * height],
        )
        .cuboid(
            [-0.3 * length, -0.42 * width, 0.55 * height],
            [0.2 * length, 0.42 * width, height],
        );
    for (across, along) in [(-0.32, -1.0), (0.32, -1.0), (-0.32, 1.0), (0.32, 1.0)] {
        let pose = RigidTransform::from_axis_angle(
            Vector3::x() * FRAC_PI_2,
            [across * length, along * (half_width - tread / 2.0), radius],
        );
        solids.merge(&wheel, &pose);
    }
    solids.build()
}

/// A trunk and a stack of two crowns, narrowing upward.
pub fn tree([length, _, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    let crown = length / 2.0;
    Solids::default()
        .cylinder([0.0, 0.0, 0.0], 0.08 * crown, 0.4 * height, SEGMENTS)
        .cylinder([0.0, 0.0, 0.4 * height], crown, 0.35 * height, SEGMENTS)
        .cylinder(
            [0.0, 0.0, 0.75 * height],
            0.6 * crown,
            0.25 * height,
            SEGMENTS,
        )
        .build()
}

/// A block with a smaller block on its roof.
pub fn building([length, width, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    Solids::default()
        .cuboid(
            [-length / 2.0, -width / 2.0, 0.0],
            [length / 2.0, width / 2.0, height],
        )
        .cuboid(
            [-length / 6.0, -width / 6.0, height],
            [length / 6.0, width / 6.0, 1.08 * height],
        )
        .build()
}
