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
const HUES_START: Tone = GREYS + 11;
const HUES: u8 = 40;

/// A light theme on warm paper: a slightly lighter card per tile, coral
/// for what the sensor saw, slate blue for the model, soft green, amber and
/// brick for the state, ink for the text.
const ACCENTS: [[u8; 3]; 11] = [
    [245, 241, 232],
    [10, 140, 70],
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
    let hues = (0..HUES).flat_map(|step| rainbow(f64::from(step) / f64::from(HUES - 1)));
    ramp.chain(ACCENTS.into_iter().flatten())
        .chain(hues)
        .collect()
}

/// Stops of a rainbow from far, blue, to near, red, as depth maps are drawn.
const RAINBOW: [[f64; 3]; 6] = [
    [59.0, 76.0, 192.0],
    [64.0, 160.0, 230.0],
    [80.0, 200.0, 160.0],
    [200.0, 220.0, 90.0],
    [245.0, 160.0, 60.0],
    [200.0, 50.0, 40.0],
];

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the stops are within [0, 255]"
)]
fn rainbow(nearness: f64) -> [u8; 3] {
    let scaled = nearness.clamp(0.0, 1.0) * 5.0;
    let floor = scaled.floor().min(4.0);
    let low = floor as usize;
    let share = scaled - floor;
    let mix = |channel: usize| {
        (RAINBOW[low][channel] * (1.0 - share) + RAINBOW[low + 1][channel] * share).round() as u8
    };
    [mix(0), mix(1), mix(2)]
}

/// The rainbow tone for a nearness in `[0, 1]`, 1 nearest, clamped.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to the hues before the cast"
)]
#[must_use]
pub fn hue(nearness: f64) -> Tone {
    HUES_START + (nearness.clamp(0.0, 1.0) * f64::from(HUES - 1)).round() as Tone
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
