use std::path::Path;

use serde::Serialize;

use crate::files::read;
use crate::model::Bounds;
use crate::report::point;

#[derive(Debug, Serialize)]
pub struct Description {
    file: String,
    points: usize,
    triangles: usize,
    bounds: Extent,
    size: f64,
}

#[derive(Debug, Serialize)]
struct Extent {
    min: [f64; 3],
    max: [f64; 3],
}

/// # Errors
/// When the file cannot be read as points.
pub fn run(file: &Path) -> anyhow::Result<Description> {
    let shape = read(file)?;
    let bounds = Bounds::of(&shape.points);
    Ok(Description {
        file: file.display().to_string(),
        points: shape.points.count(),
        triangles: shape.triangles.len(),
        bounds: Extent {
            min: point(&bounds.lowest),
            max: point(&bounds.highest),
        },
        size: bounds.size(),
    })
}
