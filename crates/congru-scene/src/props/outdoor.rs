use std::f64::consts::FRAC_PI_2;

use congru::{Error, RigidTransform, TriangleMesh};
use congru_sim::Solids;
use nalgebra::Vector3;

const SEGMENTS: u32 = 24;

/// The side of a saloon as shares of its length and height, around its middle: a
/// bonnet, a sloped windscreen, a roof, a rear window and a boot.
const SIDE: [[f64; 2]; 11] = [
    [-0.5, -0.3],
    [0.5, -0.3],
    [0.5, -0.1],
    [0.47, 0.02],
    [0.2, 0.09],
    [0.03, 0.46],
    [-0.2, 0.5],
    [-0.36, 0.13],
    [-0.47, 0.1],
    [-0.5, 0.0],
    [-0.5, -0.15],
];
/// The cabin is narrower than the body: the sides slope in above the waist.
const WAIST: f64 = 0.1;
const CABIN_WIDTH: f64 = 0.84;

/// A saloon: a shaped body, a narrower cabin, bumpers, mirrors, four wheels.
pub fn car([length, width, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    let half_width = width / 2.0;
    let radius = 0.18 * height;
    let tread = 0.12 * width;
    let wheel = Solids::default()
        .cylinder([0.0, 0.0, -tread / 2.0], radius, tread, SEGMENTS)
        .build()?;
    let outline = SIDE.map(|[along, up]| [along * length, up * height]);
    let lower = outline.map(|[along, up]| [along, up.min(WAIST * height)]);
    let body = Solids::default()
        .extrusion(&lower, -half_width, half_width)
        .build()?;
    let cabin = Solids::default()
        .extrusion(
            &outline,
            -CABIN_WIDTH * half_width,
            CABIN_WIDTH * half_width,
        )
        .build()?;
    let standing =
        RigidTransform::from_axis_angle(Vector3::x() * FRAC_PI_2, [0.0, 0.0, height / 2.0]);
    let mut solids = Solids::default();
    solids
        .merge(&body, &standing)
        .merge(&cabin, &standing)
        .cuboid(
            [0.49 * length, -0.98 * half_width, 0.2 * height],
            [0.52 * length, 0.98 * half_width, 0.32 * height],
        )
        .cuboid(
            [-0.52 * length, -0.98 * half_width, 0.2 * height],
            [-0.49 * length, 0.98 * half_width, 0.32 * height],
        )
        .cuboid(
            [0.1 * length, half_width, 0.6 * height],
            [0.14 * length, 1.1 * half_width, 0.66 * height],
        )
        .cuboid(
            [0.1 * length, -1.1 * half_width, 0.6 * height],
            [0.14 * length, -half_width, 0.66 * height],
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
