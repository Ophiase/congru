use std::f64::consts::FRAC_PI_2;

use congru::{Error, RigidTransform, TriangleMesh, Vector};
use congru_scene::{Fractal, ground};
use congru_sim::{Solids, surroundings};

const WALL_SAMPLES: u32 = 140;

/// What surrounds the turntable, in the camera's frame (y down, z forward):
/// the box and the can of a workshop, a bench top that is never quite flat,
/// and a wall behind with relief at every scale, octaves of Perlin noise,
/// so the depth around the part is as busy as a real one. A ground is
/// generated in its xy plane, which already faces the camera: the wall
/// needs no turn, the bench a quarter turn about x.
pub fn room() -> Result<TriangleMesh, Error> {
    let wall = ground(
        1.6,
        WALL_SAMPLES,
        &Fractal {
            octaves: 6,
            wave: 0.5,
            amplitude: 0.03,
            seed: 11,
        },
    )?;
    let bench = ground(
        1.6,
        WALL_SAMPLES,
        &Fractal {
            octaves: 5,
            wave: 0.3,
            amplitude: 0.004,
            seed: 12,
        },
    )?;
    let mut solids = Solids::default();
    solids
        .merge(&surroundings()?, &RigidTransform::identity())
        .merge(
            &wall,
            &RigidTransform::from_axis_angle(Vector::zeros(), [0.0, 0.0, 0.74]),
        )
        .merge(
            &bench,
            &RigidTransform::from_axis_angle(Vector::x() * FRAC_PI_2, [0.0, 0.13, 0.45]),
        );
    solids.build()
}
