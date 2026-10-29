use congru::Point;
use congru_vision::Image;
use congru_vision::pixel::{containing, coordinate};
use nalgebra::{Rotation3, Vector3};

use crate::picture::palette::{CARD, Tone};

const TURN: f64 = 0.65;
const TILT: f64 = -0.45;
const HALF_SPAN: f64 = 0.2;

/// A fixed camera over the sensor's space, beside and above the sensor, so
/// depth shows: every tile of a run is framed the same way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lens {
    rotation: Rotation3<f64>,
    centre: Vector3<f64>,
    width: usize,
    height: usize,
}

/// Points drawn in one tone, as square dots `size` pixels wide.
pub struct Layer<'points> {
    pub points: &'points [Point],
    pub tone: Tone,
    pub size: usize,
}

impl Lens {
    pub fn around(centre: &Point, width: usize, height: usize) -> Self {
        let rotation = Rotation3::from_euler_angles(TILT, 0.0, 0.0)
            * Rotation3::from_euler_angles(0.0, TURN, 0.0);
        Self {
            rotation,
            centre: rotation * centre.coords,
            width,
            height,
        }
    }

    /// Each layer drawn over the previous ones, its points far to near.
    pub fn draw(&self, layers: &[Layer<'_>]) -> Image<Tone> {
        let mut picture = Image::filled(self.width, self.height, CARD);
        for layer in layers {
            let mut dots: Vec<(f64, usize, usize)> = layer
                .points
                .iter()
                .filter_map(|point| self.place(point))
                .collect();
            dots.sort_by(|near, far| far.0.total_cmp(&near.0));
            for (_, column, row) in dots {
                self.dab(&mut picture, column, row, layer);
            }
        }
        picture
    }

    fn dab(&self, picture: &mut Image<Tone>, column: usize, row: usize, layer: &Layer<'_>) {
        for down in row..(row + layer.size).min(self.height) {
            for across in column..(column + layer.size).min(self.width) {
                picture.set(across, down, layer.tone);
            }
        }
    }

    fn place(&self, point: &Point) -> Option<(f64, usize, usize)> {
        let local = self.rotation * point.coords - self.centre;
        let scale = coordinate(self.height) / 2.0 / HALF_SPAN;
        let column = containing(coordinate(self.width) / 2.0 + local.x * scale, self.width)?;
        let row = containing(coordinate(self.height) / 2.0 + local.y * scale, self.height)?;
        Some((local.z, column, row))
    }
}
