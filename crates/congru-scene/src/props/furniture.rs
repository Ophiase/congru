use congru::{Error, TriangleMesh};
use congru_sim::Solids;

const SEGMENTS: u32 = 32;
const BOARD: f64 = 0.03;
const LEG: f64 = 0.05;

pub fn table([length, width, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    let (half_length, half_width) = (length / 2.0, width / 2.0);
    let mut solids = Solids::default();
    solids.cuboid(
        [-half_length, -half_width, height - BOARD],
        [half_length, half_width, height],
    );
    for (across, along) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let centre = [across * (half_length - LEG), along * (half_width - LEG)];
        solids.cuboid(
            [centre[0] - LEG / 2.0, centre[1] - LEG / 2.0, 0.0],
            [centre[0] + LEG / 2.0, centre[1] + LEG / 2.0, height - BOARD],
        );
    }
    solids.build()
}

pub fn shelf([length, width, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    let (half_length, half_width) = (length / 2.0, width / 2.0);
    let mut solids = Solids::default();
    for side in [-1.0, 1.0] {
        let inner = side * (half_length - BOARD);
        solids.cuboid(
            [inner.min(side * half_length), -half_width, 0.0],
            [inner.max(side * half_length), half_width, height],
        );
    }
    for level in 0..4 {
        let bottom = f64::from(level) * (height - BOARD) / 3.0;
        solids.cuboid(
            [-half_length, -half_width, bottom],
            [half_length, half_width, bottom + BOARD],
        );
    }
    solids.cuboid(
        [-half_length, half_width - BOARD, 0.0],
        [half_length, half_width, height],
    );
    solids.build()
}

/// An open box: a floor and four walls.
pub fn bin([length, width, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    let (half_length, half_width) = (length / 2.0, width / 2.0);
    Solids::default()
        .cuboid(
            [-half_length, -half_width, 0.0],
            [half_length, half_width, BOARD],
        )
        .cuboid(
            [-half_length, -half_width, 0.0],
            [-half_length + BOARD, half_width, height],
        )
        .cuboid(
            [half_length - BOARD, -half_width, 0.0],
            [half_length, half_width, height],
        )
        .cuboid(
            [-half_length, -half_width, 0.0],
            [half_length, -half_width + BOARD, height],
        )
        .cuboid(
            [-half_length, half_width - BOARD, 0.0],
            [half_length, half_width, height],
        )
        .build()
}

pub fn crate_box([length, width, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    Solids::default()
        .cuboid(
            [-length / 2.0, -width / 2.0, 0.0],
            [length / 2.0, width / 2.0, height],
        )
        .build()
}

/// A cylinder of diameter `length` and its handle, a square loop on one side.
pub fn mug([length, _, height]: [f64; 3]) -> Result<TriangleMesh, Error> {
    let radius = length / 2.0;
    let thickness = radius / 5.0;
    let reach = radius + 2.0 * thickness;
    Solids::default()
        .cylinder([0.0, 0.0, 0.0], radius, height, SEGMENTS)
        .cuboid(
            [radius - thickness, -thickness / 2.0, 0.75 * height],
            [
                reach + thickness,
                thickness / 2.0,
                0.75 * height + thickness,
            ],
        )
        .cuboid(
            [radius - thickness, -thickness / 2.0, 0.2 * height],
            [reach + thickness, thickness / 2.0, 0.2 * height + thickness],
        )
        .cuboid(
            [reach, -thickness / 2.0, 0.2 * height],
            [
                reach + thickness,
                thickness / 2.0,
                0.75 * height + thickness,
            ],
        )
        .build()
}
