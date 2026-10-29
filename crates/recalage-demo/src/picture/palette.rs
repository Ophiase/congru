//! One palette for the whole animation: a ramp of greys for the depth
//! relief, then the few colours that carry meaning.

/// An index into [`colours`].
pub type Tone = u8;

const GREYS: u8 = 200;

pub const EMPTY: Tone = GREYS;
pub const ESTIMATE: Tone = GREYS + 1;
pub const GOOD: Tone = GREYS + 2;
pub const FAIR: Tone = GREYS + 3;
pub const BAD: Tone = GREYS + 4;
pub const PAPER: Tone = GREYS + 5;

const ACCENTS: [[u8; 3]; 6] = [
    [16, 18, 22],
    [64, 224, 120],
    [64, 224, 120],
    [250, 176, 5],
    [240, 62, 62],
    [26, 28, 34],
];

/// RGB triples, laid out for a GIF colour table.
#[must_use]
pub fn colours() -> Vec<u8> {
    let ramp = (0..GREYS).flat_map(|level| {
        let value =
            u8::try_from(u32::from(level) * 235 / u32::from(GREYS - 1) + 10).unwrap_or(u8::MAX);
        [value, value, value]
    });
    ramp.chain(ACCENTS.into_iter().flatten()).collect()
}

/// The grey for a brightness in `[0, 1]`, clamped.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to the ramp before the cast"
)]
#[must_use]
pub fn grey(brightness: f64) -> Tone {
    (brightness.clamp(0.0, 1.0) * f64::from(GREYS - 1)).round() as Tone
}
