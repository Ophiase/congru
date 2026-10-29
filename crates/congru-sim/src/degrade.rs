use congru_vision::DepthImage;
use rand::rngs::ChaCha8Rng;
use rand::{Rng, RngExt, SeedableRng};
use rand_distr::{Distribution, Normal};

const HOLE_RADIUS: usize = 4;
const EDGE_JUMP: f64 = 0.03;

/// What a real depth sensor does to an exact depth image, each effect with
/// one strength, zero meaning off. Applied in the order of the fields.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Degradation {
    /// Box blur radius in pixels: optics and the averaging of the sensor.
    pub blur: usize,
    /// Standard deviation of the depth noise at one metre, in metres; it
    /// grows with the square of the depth, as for stereo.
    pub noise: f64,
    /// Share of pixels lost in round holes: dark or shiny surfaces.
    pub holes: f64,
    /// Whether pixels across a depth edge mix both sides, floating between.
    pub flying_pixels: bool,
}

impl Degradation {
    #[must_use]
    pub fn apply(&self, exact: &DepthImage, seed: u64) -> DepthImage {
        let mut random = ChaCha8Rng::seed_from_u64(seed);
        let mut depth = blur(exact, self.blur);
        if self.flying_pixels {
            depth = mix_edges(&depth);
        }
        add_noise(&mut depth, self.noise, &mut random);
        punch_holes(&mut depth, self.holes, &mut random);
        depth
    }
}

fn blur(depth: &DepthImage, radius: usize) -> DepthImage {
    if radius == 0 {
        return depth.clone();
    }
    let mut blurred = depth.clone();
    for (column, row) in depth
        .positions()
        .filter(|(column, row)| depth.get(*column, *row).is_finite())
    {
        let neighbours = window(depth, column, row, radius).filter(|value| value.is_finite());
        let (sum, count) =
            neighbours.fold((0.0, 0.0), |(sum, count), value| (sum + value, count + 1.0));
        blurred.set(column, row, sum / count);
    }
    blurred
}

fn mix_edges(depth: &DepthImage) -> DepthImage {
    let mut mixed = depth.clone();
    for (column, row) in depth.positions() {
        let here = depth.get(column, row);
        let farthest = window(depth, column, row, 1)
            .filter(|value| value.is_finite())
            .fold(here, f64::max);
        if here.is_finite() && farthest - here > EDGE_JUMP {
            mixed.set(column, row, f64::midpoint(here, farthest));
        }
    }
    mixed
}

fn add_noise(depth: &mut DepthImage, at_one_metre: f64, random: &mut impl Rng) {
    let Ok(unit) = Normal::new(0.0, at_one_metre) else {
        return;
    };
    for value in depth
        .pixels_mut()
        .iter_mut()
        .filter(|value| value.is_finite())
    {
        *value += unit.sample(random) * *value * *value;
    }
}

fn punch_holes(depth: &mut DepthImage, share: f64, random: &mut impl Rng) {
    let pixels = congru_vision::pixel::coordinate(depth.width() * depth.height());
    let disc = std::f64::consts::PI * congru_vision::pixel::coordinate(HOLE_RADIUS).powi(2);
    for _ in 0..hole_count(share * pixels / disc) {
        let column = random.random_range(0..depth.width());
        let row = random.random_range(0..depth.height());
        for (hole_column, hole_row) in disc_around(depth, column, row) {
            depth.set(hole_column, hole_row, f64::NAN);
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a rounded, non-negative count of holes, saturating past the type"
)]
fn hole_count(expected: f64) -> usize {
    expected.round().max(0.0) as usize
}

fn disc_around(depth: &DepthImage, column: usize, row: usize) -> Vec<(usize, usize)> {
    let reach = HOLE_RADIUS * HOLE_RADIUS;
    let rows = row.saturating_sub(HOLE_RADIUS)..(row + HOLE_RADIUS + 1).min(depth.height());
    rows.flat_map(|other_row| {
        let columns =
            column.saturating_sub(HOLE_RADIUS)..(column + HOLE_RADIUS + 1).min(depth.width());
        columns.map(move |other_column| (other_column, other_row))
    })
    .filter(|(other_column, other_row)| {
        other_column.abs_diff(column).pow(2) + other_row.abs_diff(row).pow(2) <= reach
    })
    .collect()
}

fn window(
    depth: &DepthImage,
    column: usize,
    row: usize,
    radius: usize,
) -> impl Iterator<Item = f64> + '_ {
    let rows = row.saturating_sub(radius)..(row + radius + 1).min(depth.height());
    rows.flat_map(move |other_row| {
        let columns = column.saturating_sub(radius)..(column + radius + 1).min(depth.width());
        columns.map(move |other_column| depth.get(other_column, other_row))
    })
}
