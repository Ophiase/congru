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
pub const MODEL: Tone = GREYS + 6;
pub const OBSERVED: Tone = GREYS + 7;
pub const TEXT: Tone = GREYS + 8;
pub const CARD: Tone = GREYS + 9;
pub const MUTED: Tone = GREYS + 10;

/// A light theme on warm paper: a slightly lighter card per tile, coral
/// for what the sensor saw, slate blue for the model, soft green, amber and
/// brick for the state, ink for the text.
const ACCENTS: [[u8; 3]; 11] = [
    [245, 241, 232],
    [63, 155, 92],
    [63, 155, 92],
    [201, 138, 28],
    [196, 74, 60],
    [238, 233, 222],
    [74, 127, 176],
    [224, 104, 84],
    [58, 63, 71],
    [251, 249, 244],
    [138, 143, 152],
];

/// RGB triples, laid out for a GIF colour table.
#[must_use]
pub fn colours() -> Vec<u8> {
    let ramp = (0..GREYS).flat_map(|level| {
        let value =
            u8::try_from(u32::from(level) * 150 / u32::from(GREYS - 1) + 95).unwrap_or(u8::MAX);
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
