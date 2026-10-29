use nalgebra::{Rotation3, Vector3};
use congru::Point;

const MARGIN: f64 = 22.0;
const FAINTEST: f64 = 0.3;

/// An orthographic camera framing a fixed box of the scene, so every panel
/// of a figure shares one scale; a flat scene seen from above keeps full
/// opacity.
pub struct View {
    rotation: Rotation3<f64>,
    centre: Vector3<f64>,
    half_extent: f64,
    flat: bool,
}

pub struct Frame {
    left: f64,
    side: f64,
}

impl View {
    pub fn from_above(points: &[Point]) -> Self {
        Self {
            flat: true,
            ..Self::framing(points, Rotation3::identity())
        }
    }

    /// Framing `points`, looking at them from `direction`.
    pub fn facing(points: &[Point], direction: &Vector3<f64>) -> Self {
        let rotation = Rotation3::rotation_between(direction, &Vector3::z())
            .unwrap_or_else(Rotation3::identity);
        Self::framing(points, rotation)
    }

    fn framing(points: &[Point], rotation: Rotation3<f64>) -> Self {
        let turned: Vec<Vector3<f64>> =
            points.iter().map(|point| rotation * point.coords).collect();
        let lowest = turned
            .iter()
            .fold(Vector3::repeat(f64::INFINITY), |low, point| low.inf(point));
        let highest = turned
            .iter()
            .fold(Vector3::repeat(f64::NEG_INFINITY), |high, point| {
                high.sup(point)
            });
        let span = highest - lowest;
        Self {
            rotation,
            centre: (lowest + highest) / 2.0,
            half_extent: (span.x.max(span.y) / 2.0).max(f64::EPSILON),
            flat: false,
        }
    }

    /// Position in `[-1, 1]²` over the framed box, and depth, larger
    /// farther from the reader.
    pub fn project(&self, point: &Point) -> ([f64; 2], f64) {
        let local = (self.rotation * point.coords - self.centre) / self.half_extent;
        ([local.x, local.y], -local.z)
    }

    pub fn shade(&self, depth: f64) -> f64 {
        if self.flat {
            return 1.0;
        }
        FAINTEST + (1.0 - FAINTEST) * ((1.0 - depth) / 2.0).clamp(0.0, 1.0)
    }
}

impl Frame {
    pub const fn new(left: f64, side: f64) -> Self {
        Self { left, side }
    }

    pub fn place(&self, [across, up]: [f64; 2]) -> [f64; 2] {
        let scale = self.side / 2.0 - MARGIN;
        [
            self.left + self.side / 2.0 + across * scale,
            self.side / 2.0 - up * scale,
        ]
    }
}
