use congru::{Error, TriangleMesh};

use crate::solids::Solids;

const SEGMENTS: u32 = 40;

/// An L-shaped machined bracket, about 20 cm long, in metres, z up, its
/// base on the plane `z = 0`: a plate, an upright, a gusset on one side, a
/// boss off centre and a pin on the upright. No two of its features mirror
/// each other, so one pose, and only one, fits it.
///
/// # Errors
/// Never in practice; the type is the mesh constructor's.
pub fn bracket() -> Result<TriangleMesh, Error> {
    Solids::default()
        .cuboid([-0.10, -0.06, 0.0], [0.10, 0.06, 0.012])
        .cuboid([-0.10, -0.06, 0.0], [-0.088, 0.06, 0.09])
        .prism(
            [[-0.088, 0.012], [-0.03, 0.012], [-0.088, 0.07]],
            0.042,
            0.054,
        )
        .cylinder([0.05, -0.02, 0.012], 0.022, 0.03, SEGMENTS)
        .cylinder([0.075, 0.035, 0.012], 0.009, 0.012, SEGMENTS)
        .build()
}

/// The turntable the part stands on: a wide, thin disc below `z = 0`.
///
/// # Errors
/// As [`bracket`].
pub fn turntable() -> Result<TriangleMesh, Error> {
    Solids::default()
        .cylinder([0.0, 0.0, -0.015], 0.2, 0.015, SEGMENTS * 2)
        .build()
}

const TEETH: u32 = 12;

/// A spur gear, about 11 cm across, its axis along z: twelve teeth make it
/// look the same after a twelfth of a turn. A method cannot tell those
/// poses apart from the shape alone, and should not claim it can.
///
/// # Errors
/// As [`bracket`].
pub fn gear() -> Result<TriangleMesh, Error> {
    let outline: Vec<[f64; 2]> = (0..TEETH * 4)
        .map(|step| {
            let angle = std::f64::consts::TAU * f64::from(step) / f64::from(TEETH * 4);
            let radius = if step % 4 < 2 { 0.055 } else { 0.045 };
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect();
    Solids::default()
        .extrusion(&outline, 0.0, 0.02)
        .cylinder([0.0, 0.0, 0.02], 0.018, 0.015, SEGMENTS)
        .build()
}

/// What sits around the turntable, fixed in the camera's frame: a wall
/// behind, a bench top below, a box and a can beside the part.
///
/// # Errors
/// As [`bracket`].
pub fn surroundings() -> Result<TriangleMesh, Error> {
    Solids::default()
        .cuboid([-0.6, -0.5, 0.75], [0.6, 0.4, 0.78])
        .cuboid([-0.6, 0.13, 0.1], [0.6, 0.16, 0.9])
        .cuboid([-0.28, 0.0, 0.42], [-0.18, 0.13, 0.52])
        .cylinder([0.2, 0.3, 0.45], 0.035, 0.12, SEGMENTS)
        .build()
}
