use std::fs::File;
use std::path::Path;

use congru_vision::Image;

use super::palette::{BAD, FAIR, GOOD, PAPER, Tone, colours};

const GAP: usize = 4;
const BAR: usize = 5;
const GOOD_DEGREES: f64 = 1.0;
const FAIR_DEGREES: f64 = 5.0;
const FULL_BAR_DEGREES: f64 = 20.0;
const DELAY_HUNDREDTHS: u16 = 7;

/// A tracked panel and its rotation error, drawn as a bar below it.
pub struct Tile {
    pub picture: Image<Tone>,
    pub error_degrees: f64,
}

/// Tiles laid out `columns` wide, one animation frame per element of
/// `frames`, looping forever.
pub fn write(path: &Path, frames: &[Vec<Tile>], columns: usize) -> anyhow::Result<()> {
    let Some(first) = frames.first().and_then(|tiles| tiles.first()) else {
        return Ok(());
    };
    let (tile_width, tile_height) = (first.picture.width(), first.picture.height() + BAR);
    let rows = frames[0].len().div_ceil(columns);
    let width = columns * (tile_width + GAP) + GAP;
    let height = rows * (tile_height + GAP) + GAP;
    let mut encoder = gif::Encoder::new(
        File::create(path)?,
        u16::try_from(width)?,
        u16::try_from(height)?,
        &colours(),
    )?;
    encoder.set_repeat(gif::Repeat::Infinite)?;
    for tiles in frames {
        let mut canvas = Image::filled(width, height, PAPER);
        for (slot, tile) in tiles.iter().enumerate() {
            let left = GAP + (slot % columns) * (tile_width + GAP);
            let top = GAP + (slot / columns) * (tile_height + GAP);
            paste(&mut canvas, &tile.picture, left, top);
            bar(&mut canvas, tile, left, top + tile.picture.height());
        }
        let mut frame = gif::Frame::from_indexed_pixels(
            u16::try_from(width)?,
            u16::try_from(height)?,
            canvas.pixels().to_vec(),
            None,
        );
        frame.delay = DELAY_HUNDREDTHS;
        encoder.write_frame(&frame)?;
    }
    Ok(())
}

fn paste(canvas: &mut Image<Tone>, picture: &Image<Tone>, left: usize, top: usize) {
    for (column, row) in picture.positions() {
        canvas.set(left + column, top + row, picture.get(column, row));
    }
}

fn bar(canvas: &mut Image<Tone>, tile: &Tile, left: usize, top: usize) {
    let tone = match tile.error_degrees {
        error if error < GOOD_DEGREES => GOOD,
        error if error < FAIR_DEGREES => FAIR,
        _ => BAD,
    };
    let share = if tile.error_degrees.is_finite() {
        tile.error_degrees / FULL_BAR_DEGREES
    } else {
        1.0
    };
    let length = congru_vision::pixel::containing(
        share.clamp(0.02, 1.0) * congru_vision::pixel::coordinate(tile.picture.width() - 1),
        tile.picture.width(),
    )
    .unwrap_or(0);
    for column in left..=left + length {
        for row in top + 1..top + BAR {
            canvas.set(column, row, tone);
        }
    }
}
