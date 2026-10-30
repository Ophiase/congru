use anyhow::{Context, bail};

use super::header::{Format, Scalar};

/// Reads the values of a PLY body one at a time, whatever its encoding.
#[derive(Debug)]
pub enum Values<'body> {
    Ascii(std::str::SplitAsciiWhitespace<'body>),
    Binary(&'body [u8]),
}

impl<'body> Values<'body> {
    pub fn new(format: Format, body: &'body [u8]) -> anyhow::Result<Self> {
        Ok(match format {
            Format::Ascii => Self::Ascii(
                std::str::from_utf8(body)
                    .context("ASCII body is not text")?
                    .split_ascii_whitespace(),
            ),
            Format::BinaryLittleEndian => Self::Binary(body),
        })
    }

    /// Every type a PLY property holds fits an `f64` exactly.
    pub fn next(&mut self, kind: Scalar) -> anyhow::Result<f64> {
        match self {
            Self::Ascii(words) => words
                .next()
                .context("body ends early")?
                .parse()
                .context("a value is not a number"),
            Self::Binary(bytes) => {
                let size = width(kind);
                if bytes.len() < size {
                    bail!("body ends early");
                }
                let (value, rest) = bytes.split_at(size);
                *bytes = rest;
                Ok(decode(kind, value))
            }
        }
    }
}

const fn width(kind: Scalar) -> usize {
    match kind {
        Scalar::Int8 | Scalar::Uint8 => 1,
        Scalar::Int16 | Scalar::Uint16 => 2,
        Scalar::Int32 | Scalar::Uint32 | Scalar::Float32 => 4,
        Scalar::Float64 => 8,
    }
}

/// `bytes` holds exactly `width(kind)` bytes, little-endian.
fn decode(kind: Scalar, bytes: &[u8]) -> f64 {
    let mut buffer = [0_u8; 8];
    buffer[..bytes.len()].copy_from_slice(bytes);
    let [first, second, third, fourth, ..] = buffer;
    let four = [first, second, third, fourth];
    match kind {
        Scalar::Int8 => f64::from(i8::from_le_bytes([first])),
        Scalar::Uint8 => f64::from(first),
        Scalar::Int16 => f64::from(i16::from_le_bytes([first, second])),
        Scalar::Uint16 => f64::from(u16::from_le_bytes([first, second])),
        Scalar::Int32 => f64::from(i32::from_le_bytes(four)),
        Scalar::Uint32 => f64::from(u32::from_le_bytes(four)),
        Scalar::Float32 => f64::from(f32::from_le_bytes(four)),
        Scalar::Float64 => f64::from_le_bytes(buffer),
    }
}
