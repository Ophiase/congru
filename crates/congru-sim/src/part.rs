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
