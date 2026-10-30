mod ply;
#[cfg(test)]
mod write;
mod xyz;

use std::path::Path;

use anyhow::{Context, bail};
use congru::{PointCloud, TriangleMesh};

/// A file's content before any check: interleaved `x y z` coordinates, and
/// triangles over them when the file is a mesh.
#[derive(Debug, Clone, Default, PartialEq)]
struct Raw {
    coordinates: Vec<f64>,
    triangles: Vec<[u32; 3]>,
}

/// What a point file holds: its points, and the triangles over them when
/// it is a mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub points: PointCloud,
    pub triangles: Box<[[u32; 3]]>,
}

impl Shape {
    /// # Errors
    /// When a triangle refers to a missing vertex.
    pub fn mesh(&self) -> anyhow::Result<Option<TriangleMesh>> {
        if self.triangles.is_empty() {
            return Ok(None);
        }
        Ok(Some(TriangleMesh::new(
            self.points.clone(),
            self.triangles.to_vec(),
        )?))
    }
}

/// Whether `path` names a format [`read`] understands, by its extension.
pub fn is_point_file(path: &Path) -> bool {
    matches!(extension(path).as_deref(), Some("xyz" | "ply"))
}

/// Reads a `.xyz` file (`x y z` per line, further columns ignored) or a
/// `.ply` file, ASCII or binary little-endian.
///
/// # Errors
/// When the file cannot be read, its extension is neither, its content is
/// malformed, or it holds no point.
pub fn read(path: &Path) -> anyhow::Result<Shape> {
    let bytes = std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let raw = match extension(path).as_deref() {
        Some("xyz") => xyz::parse(&bytes),
        Some("ply") => ply::parse(&bytes),
        _ => bail!("{}: expected a .xyz or a .ply file", path.display()),
    }
    .with_context(|| path.display().to_string())?;
    Ok(Shape {
        points: PointCloud::from_xyz(&raw.coordinates)
            .with_context(|| path.display().to_string())?,
        triangles: raw.triangles.into_boxed_slice(),
    })
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(str::to_ascii_lowercase)
}
