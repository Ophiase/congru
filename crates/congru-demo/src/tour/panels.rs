use congru_vision::{DepthImage, Image};

use super::zoom::Zoom;
use crate::camera::overlay::rainbow;
use crate::picture::palette::{TEXT, Tone};
use crate::picture::{Tile, font, outline, outlined};

/// The bar turns amber past 2 % of the target's size, red past 10 %.
const BAR_PERCENT_PER_DEGREE: f64 = 2.0;
const LABEL_INSET: usize = 4;

/// What one frame shows, at the camera's size.
pub(super) struct Seen {
    pub(super) exact: DepthImage,
    pub(super) depth: DepthImage,
    pub(super) region: DepthImage,
    pub(super) silhouette: Option<Image<bool>>,
    pub(super) off: f64,
    pub(super) degrees: f64,
}

/// The three panels of a frame: the scene and the depth at half the camera's size,
/// then what the box holds, enlarged. The outline is drawn at the size shown, so
/// that no pixel of it is lost in the reduction. The bar and the number read the
/// surface error, not the rotation, which would fail a gear turned by one tooth.
pub(super) fn panels(seen: &Seen, zoom: &Zoom, focal: f64) -> Vec<Tile> {
    let percent = 100.0 * seen.off;
    let mut scene = halved(&outlined(&seen.exact, focal, None));
    let mut depth = halved(&rainbow(&seen.depth, &seen.exact));
    let mut held = zoom.colours(&seen.region);
    if let Some(silhouette) = &seen.silhouette {
        let small = halved(silhouette);
        outline(&mut scene, &small);
        outline(&mut depth, &small);
        outline(&mut held, &zoom.enlarged(silhouette));
    }
    let bottom = scene.height() - 18;
    font::write(
        &mut scene,
        LABEL_INSET,
        bottom,
        &format!("{percent:.1}%"),
        TEXT,
    );
    [("SCENE", scene), ("DEPTH", depth), ("BOX", held)]
        .into_iter()
        .map(|(label, mut picture)| {
            font::write(&mut picture, LABEL_INSET, LABEL_INSET, label, TEXT);
            finished(picture, percent)
        })
        .collect()
}

fn finished(picture: Image<Tone>, percent: f64) -> Tile {
    Tile {
        picture,
        error_degrees: percent / BAR_PERCENT_PER_DEGREE,
    }
}

fn halved<T: Copy>(picture: &Image<T>) -> Image<T> {
    let mut half = Image::filled(picture.width() / 2, picture.height() / 2, picture.get(0, 0));
    for (column, row) in half.positions() {
        half.set(column, row, picture.get(column * 2, row * 2));
    }
    half
}
