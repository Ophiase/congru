use congru_vision::{DepthImage, Image};
use nalgebra::Vector3;

use super::palette::{CARD, EMPTY, ESTIMATE, Tone, grey};

const AMBIENT: f64 = 0.15;

/// The depth as a relief, and the outline of `silhouette` over it, when
/// there is one: what a tracking panel shows.
pub fn outlined(depth: &DepthImage, focal: f64, silhouette: Option<&Image<bool>>) -> Image<Tone> {
    let mut picture = relief(depth, focal);
    if let Some(silhouette) = silhouette {
        outline(&mut picture, silhouette);
    }
    picture
}

/// The outline of `silhouette` over `picture`, two pixels wide across its edge, so
/// a small object keeps its inside; a pale ring lets it read on any depth colour.
pub fn outline(picture: &mut Image<Tone>, silhouette: &Image<bool>) {
    let line = grown(silhouette);
    let ring = grown(&line);
    for (column, row) in silhouette.positions() {
        if line.get(column, row) {
            if !silhouette.get(column, row) || on_outline(silhouette, column, row) {
                picture.set(column, row, ESTIMATE);
            }
        } else if ring.get(column, row) {
            picture.set(column, row, CARD);
        }
    }
}

fn grown(mask: &Image<bool>) -> Image<bool> {
    let mut larger = mask.clone();
    for (column, row) in mask.positions() {
        if neighbours(mask, column, row)
            .iter()
            .any(|(across, down)| mask.get(*across, *down))
        {
            larger.set(column, row, true);
        }
    }
    larger
}

fn neighbours(mask: &Image<bool>, column: usize, row: usize) -> [(usize, usize); 4] {
    [
        (column.saturating_sub(1), row),
        ((column + 1).min(mask.width() - 1), row),
        (column, row.saturating_sub(1)),
        (column, (row + 1).min(mask.height() - 1)),
    ]
}

/// Shaded by the surface normal the depth itself implies, lit from the
/// camera: noise shows as grain, blur as soft edges, holes as black.
fn relief(depth: &DepthImage, focal: f64) -> Image<Tone> {
    let mut picture = Image::filled(depth.width(), depth.height(), EMPTY);
    for (column, row) in depth
        .positions()
        .filter(|(column, row)| *column > 0 && *row > 0)
    {
        let here = depth.get(column, row);
        let (left, above) = (depth.get(column - 1, row), depth.get(column, row - 1));
        if !(here.is_finite() && left.is_finite() && above.is_finite()) {
            continue;
        }
        let pixel = here / focal;
        let normal = Vector3::new((here - left) / pixel, (here - above) / pixel, -1.0).normalize();
        picture.set(column, row, grey(AMBIENT + (1.0 - AMBIENT) * -normal.z));
    }
    picture
}

fn on_outline(silhouette: &Image<bool>, column: usize, row: usize) -> bool {
    if !silhouette.get(column, row) {
        return false;
    }
    neighbours(silhouette, column, row)
        .iter()
        .any(|(across, down)| !silhouette.get(*across, *down))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INSIDE: Tone = 7;

    fn drawn_over_a_square() -> Image<Tone> {
        let mut silhouette = Image::filled(20, 20, false);
        for column in 6..14 {
            for row in 6..14 {
                silhouette.set(column, row, true);
            }
        }
        let mut picture = Image::filled(20, 20, INSIDE);
        outline(&mut picture, &silhouette);
        picture
    }

    #[test]
    fn the_line_crosses_the_edge_and_the_ring_lies_outside_it() {
        let picture = drawn_over_a_square();
        assert_eq!(picture.get(4, 10), CARD);
        assert_eq!(picture.get(5, 10), ESTIMATE);
        assert_eq!(picture.get(6, 10), ESTIMATE);
    }

    #[test]
    fn the_inside_of_the_object_stays_visible() {
        assert_eq!(drawn_over_a_square().get(10, 10), INSIDE);
    }
}
