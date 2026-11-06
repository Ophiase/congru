use congru_vision::{DepthImage, Image};

use crate::picture::palette::{EMPTY, Tone, hue};

const MARGIN: usize = 4;
const SMALLEST_WIDTH: usize = 16;
/// Flying pixels are noise: the colours span the depths between these shares.
const NEAR_SHARE: usize = 2;
const FAR_SHARE: usize = 98;

/// A window of the image over all the boxes of a film, at the tile's proportions,
/// and the depths they hold. One per film: a box drawn anew at each frame moves
/// inside it, where a zoom on each box would shake with its noise.
pub(super) struct Zoom {
    left: usize,
    top: usize,
    width: usize,
    height: usize,
    near: f64,
    far: f64,
    tile_width: usize,
    tile_height: usize,
}

impl Zoom {
    /// The whole image if no box holds a point.
    pub(super) fn over<'a>(
        regions: impl Iterator<Item = &'a DepthImage>,
        (image_width, image_height): (usize, usize),
    ) -> Self {
        let (tile_width, tile_height) = (image_width / 2, image_height / 2);
        let (bounds, mut depths) = held(regions);
        let [left, top, right, bottom] =
            bounds.unwrap_or([0, 0, image_width - 1, image_height - 1]);
        let wanted = (right - left + 1 + 2 * MARGIN)
            .max((bottom - top + 1 + 2 * MARGIN) * image_width / image_height)
            .max(SMALLEST_WIDTH)
            .min(image_width);
        let tall = wanted * image_height / image_width;
        depths.sort_by(f64::total_cmp);
        let at = |share: usize| {
            depths
                .get(depths.len().saturating_sub(1) * share / 100)
                .copied()
                .unwrap_or(0.0)
        };
        Self {
            left: start(usize::midpoint(left, right), wanted, image_width),
            top: start(usize::midpoint(top, bottom), tall, image_height),
            width: wanted,
            height: tall,
            near: at(NEAR_SHARE),
            far: at(FAR_SHARE),
            tile_width,
            tile_height,
        }
    }

    /// What the box holds, enlarged to the tile and coloured over the box's own
    /// depths, not the scene's, which would leave the object one flat colour.
    pub(super) fn colours(&self, region: &DepthImage) -> Image<Tone> {
        let span = (self.far - self.near).max(f64::EPSILON);
        self.enlarged(&region.map(|value| {
            if value.is_finite() {
                hue((self.far - value) / span)
            } else {
                EMPTY
            }
        }))
    }

    pub(super) fn enlarged<T: Copy>(&self, picture: &Image<T>) -> Image<T> {
        let mut large = Image::filled(self.tile_width, self.tile_height, picture.get(0, 0));
        for (column, row) in large.positions() {
            large.set(
                column,
                row,
                picture.get(
                    self.left + column * self.width / self.tile_width,
                    self.top + row * self.height / self.tile_height,
                ),
            );
        }
        large
    }
}

/// The rectangle around every point the regions hold, as left, top, right and
/// bottom, and their depths.
fn held<'a>(regions: impl Iterator<Item = &'a DepthImage>) -> (Option<[usize; 4]>, Vec<f64>) {
    let mut bounds: Option<[usize; 4]> = None;
    let mut depths = Vec::new();
    for region in regions {
        for (column, row) in region.positions() {
            let value = region.get(column, row);
            if value.is_finite() {
                depths.push(value);
                let [left, top, right, bottom] = bounds.unwrap_or([column, row, column, row]);
                bounds = Some([
                    left.min(column),
                    top.min(row),
                    right.max(column),
                    bottom.max(row),
                ]);
            }
        }
    }
    (bounds, depths)
}

fn start(middle: usize, length: usize, limit: usize) -> usize {
    middle.saturating_sub(length / 2).min(limit - length)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: usize = 320;
    const HEIGHT: usize = 240;

    fn box_at(left: usize, top: usize) -> DepthImage {
        let mut region = DepthImage::filled(WIDTH, HEIGHT, f64::NAN);
        for column in left..left + 40 {
            for row in top..top + 20 {
                region.set(
                    column,
                    row,
                    1.0 + 0.001 * f64::from(u32::try_from(row).unwrap_or(0)),
                );
            }
        }
        region
    }

    #[test]
    fn the_window_holds_every_box_of_the_film_at_the_tile_proportions() {
        let regions = [box_at(100, 100), box_at(120, 90)];
        let zoom = Zoom::over(regions.iter(), (WIDTH, HEIGHT));
        assert!(zoom.left <= 100 && zoom.left + zoom.width >= 160);
        assert!(zoom.top <= 90 && zoom.top + zoom.height >= 120);
        assert_eq!(zoom.width * HEIGHT, zoom.height * WIDTH);
        assert!(zoom.width < WIDTH / 2, "{} is no zoom", zoom.width);
    }

    #[test]
    fn a_box_against_a_corner_stays_inside_the_image() {
        let zoom = Zoom::over([box_at(280, 220)].iter(), (WIDTH, HEIGHT));
        assert!(zoom.left + zoom.width <= WIDTH);
        assert!(zoom.top + zoom.height <= HEIGHT);
    }

    #[test]
    fn an_empty_box_shows_the_whole_image() {
        let empty = DepthImage::filled(WIDTH, HEIGHT, f64::NAN);
        let zoom = Zoom::over([empty].iter(), (WIDTH, HEIGHT));
        assert_eq!((zoom.left, zoom.width), (0, WIDTH));
    }

    #[test]
    fn the_nearest_and_the_farthest_of_the_box_take_different_colours() {
        let region = box_at(100, 100);
        let zoom = Zoom::over([&region].into_iter(), (WIDTH, HEIGHT));
        let tile = zoom.colours(&region);
        let tones: std::collections::HashSet<Tone> = tile
            .positions()
            .map(|(column, row)| tile.get(column, row))
            .filter(|tone| *tone != EMPTY)
            .collect();
        assert!(tones.len() > 5, "{} colours", tones.len());
    }
}
