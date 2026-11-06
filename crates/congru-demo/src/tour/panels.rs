use congru_vision::{Camera, DepthImage, Image, back_project};

use crate::camera::overlay::rainbow;
use crate::picture::palette::{EMPTY, TEXT, Tone};
use crate::picture::{Tile, font, outline, outlined, splat};

/// The bar turns amber past 2 % of the target's size, red past 10 %.
const BAR_PERCENT_PER_DEGREE: f64 = 2.0;

/// The three panels of a frame, as demo B draws them, at half the camera's size:
/// the scene, the depth, the points the box holds, each under the outline of the
/// answer. The bar and the number read the surface error, not the rotation, which
/// would fail a gear turned by one tooth.
pub(super) fn panels(
    camera: &Camera,
    (exact, depth, region): (&DepthImage, &DepthImage, &DepthImage),
    silhouette: Option<&Image<bool>>,
    off: f64,
) -> Vec<Tile> {
    let points = back_project(region, camera);
    let mut drawn = [
        outlined(exact, camera.focal, None),
        rainbow(depth, exact),
        rainbow(&splat(&points, camera), exact),
    ];
    if let Some(silhouette) = silhouette {
        for panel in &mut drawn {
            outline(panel, silhouette);
        }
    }
    let percent = 100.0 * off;
    drawn
        .into_iter()
        .map(|picture| {
            let mut picture = halved(&picture);
            let bottom = picture.height() - 14;
            font::write(&mut picture, 4, bottom, &format!("{percent:.1}%"), TEXT);
            Tile {
                picture,
                error_degrees: percent / BAR_PERCENT_PER_DEGREE,
            }
        })
        .collect()
}

fn halved(picture: &Image<Tone>) -> Image<Tone> {
    let mut half = Image::filled(picture.width() / 2, picture.height() / 2, EMPTY);
    for (column, row) in half.positions() {
        half.set(column, row, picture.get(column * 2, row * 2));
    }
    half
}
