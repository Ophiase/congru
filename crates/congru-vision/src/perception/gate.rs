use congru::Point;

use crate::{Camera, DepthImage};

/// The returns a model rendered at a predicted pose can explain: those inside
/// its silhouette, grown by `margin` pixels, and within `gate` of its depth
/// there. What stands around the model, or in front of it, never becomes a
/// point it must explain. A crop of the model's shape rather than a ball.
#[must_use]
pub fn in_view(
    depth: &DepthImage,
    rendered: &DepthImage,
    camera: &Camera,
    margin: usize,
    gate: f64,
) -> Vec<Point> {
    depth
        .positions()
        .filter_map(|(column, row)| {
            let seen = depth.get(column, row);
            let near = nearest_around(rendered, column, row, margin)?;
            (seen.is_finite() && (seen - near).abs() <= gate)
                .then(|| camera.back_project(column, row, seen))
        })
        .collect()
}

/// The returns farther than `margin` pixels from the silhouette of a model
/// rendered at a predicted pose: what stands around it, whatever the error of
/// the prediction.
#[must_use]
pub fn around_view(
    depth: &DepthImage,
    rendered: &DepthImage,
    camera: &Camera,
    margin: usize,
) -> Vec<Point> {
    depth
        .positions()
        .filter_map(|(column, row)| {
            let seen = depth.get(column, row);
            (seen.is_finite() && nearest_around(rendered, column, row, margin).is_none())
                .then(|| camera.back_project(column, row, seen))
        })
        .collect()
}

/// The model's nearest depth within `margin` pixels of a pixel.
fn nearest_around(rendered: &DepthImage, column: usize, row: usize, margin: usize) -> Option<f64> {
    let columns = column.saturating_sub(margin)..=(column + margin).min(rendered.width() - 1);
    columns
        .flat_map(|across| {
            let rows = row.saturating_sub(margin)..=(row + margin).min(rendered.height() - 1);
            rows.map(move |down| rendered.get(across, down))
        })
        .filter(|value| value.is_finite())
        .reduce(f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_returns_on_the_silhouette_near_its_depth_are_kept() {
        let camera = Camera {
            width: 4,
            height: 1,
            focal: 1.0,
        };
        let mut rendered = DepthImage::filled(4, 1, f64::NAN);
        rendered.set(1, 0, 1.0);
        rendered.set(2, 0, 1.0);
        let mut depth = DepthImage::filled(4, 1, 1.0);
        depth.set(2, 0, 3.0);
        let kept = in_view(&depth, &rendered, &camera, 0, 0.1);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0], camera.back_project(1, 0, 1.0));
    }
}
