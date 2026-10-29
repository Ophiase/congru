/// Fractal gradient noise: Perlin noise summed over `octaves`, each of half
/// the wavelength and half the amplitude of the one before. Deterministic
/// in its seed; heights lie within `[-amplitude, amplitude]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fractal {
    pub octaves: u32,
    /// Wavelength of the first octave.
    pub wave: f64,
    pub amplitude: f64,
    pub seed: u64,
}

impl Fractal {
    #[must_use]
    pub fn height(&self, across: f64, along: f64) -> f64 {
        let mut total = 0.0;
        let mut weight = 0.0;
        let mut strength = 1.0;
        let mut frequency = 1.0 / self.wave.max(f64::EPSILON);
        for octave in 0..self.octaves.max(1) {
            total += strength
                * perlin(
                    across * frequency,
                    along * frequency,
                    self.seed.wrapping_add(u64::from(octave)),
                );
            weight += strength;
            strength /= 2.0;
            frequency *= 2.0;
        }
        self.amplitude * total / weight
    }
}

fn perlin(across: f64, along: f64, seed: u64) -> f64 {
    let (column, row) = (cell(across), cell(along));
    let (inside_across, inside_along) = (across - across.floor(), along - along.floor());
    let corner = |step_across: i64, step_along: i64| {
        let angle =
            unit(hash(column + step_across, row + step_along, seed)) * std::f64::consts::TAU;
        let offset_across = inside_across - as_float(step_across);
        let offset_along = inside_along - as_float(step_along);
        angle.cos() * offset_across + angle.sin() * offset_along
    };
    let (fade_across, fade_along) = (fade(inside_across), fade(inside_along));
    let lower = lerp(corner(0, 0), corner(1, 0), fade_across);
    let upper = lerp(corner(0, 1), corner(1, 1), fade_across);
    lerp(lower, upper, fade_along) * std::f64::consts::SQRT_2
}

fn fade(share: f64) -> f64 {
    share * share * share * (share * (share * 6.0 - 15.0) + 10.0)
}

fn lerp(from: f64, to: f64, share: f64) -> f64 {
    from + share * (to - from)
}

fn hash(column: i64, row: i64, seed: u64) -> u64 {
    let mut mixed = seed
        ^ column.cast_unsigned().wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ row.cast_unsigned().wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^ (mixed >> 31)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the top 53 bits of a hash, as a share of a turn"
)]
fn unit(bits: u64) -> f64 {
    (bits >> 11) as f64 / (1_u64 << 53) as f64
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "a floored coordinate of a scene, far inside i64"
)]
fn cell(coordinate: f64) -> i64 {
    coordinate.floor() as i64
}

#[expect(clippy::cast_precision_loss, reason = "a step of zero or one")]
const fn as_float(step: i64) -> f64 {
    step as f64
}
