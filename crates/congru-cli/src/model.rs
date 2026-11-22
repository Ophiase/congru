use anyhow::Context;
use congru::{Distance, OrientedCloud, Point, PointCloud, Target, Vector};

use crate::files::Shape;

/// Surface samples along the model's size when no spacing is asked for.
const SAMPLES_PER_SPAN: f64 = 60.0;
const NORMAL_NEIGHBORS: usize = 12;

/// The axis-aligned box around a set of points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub lowest: Point,
    pub highest: Point,
}

impl Bounds {
    #[must_use]
    pub fn of(cloud: &PointCloud) -> Self {
        let first = cloud.points()[0];
        cloud.points().iter().fold(
            Self {
                lowest: first,
                highest: first,
            },
            |bounds, point| Self {
                lowest: bounds.lowest.inf(point),
                highest: bounds.highest.sup(point),
            },
        )
    }

    /// The diagonal, in the unit of the points.
    #[must_use]
    pub fn size(&self) -> f64 {
        (self.highest - self.lowest).norm()
    }
}

/// What the methods align onto: a surface with normals, and its size, from
/// which every scale of the methods is set.
#[derive(Debug, Clone)]
pub struct Model {
    pub surface: OrientedCloud,
    /// The diagonal of the bounding box, in the unit of the points.
    pub span: f64,
}

impl Model {
    /// A mesh is sampled every `spacing`, a sixtieth of its size without
    /// one; a cloud is thinned on a grid of `spacing` when given, and its
    /// normals fitted to its nearest points.
    ///
    /// # Errors
    /// When the shape is flat to a point, or too small to fit normals to.
    pub fn new(shape: &Shape, spacing: Option<Distance>) -> anyhow::Result<Self> {
        let span = Bounds::of(&shape.points).size();
        let surface = match shape.mesh()? {
            Some(mesh) => {
                let spacing = match spacing {
                    Some(spacing) => spacing,
                    None => {
                        Distance::new(span / SAMPLES_PER_SPAN).context("the model has no extent")?
                    }
                };
                mesh.sample_surface(spacing)?
            }
            None => oriented(thinned(&shape.points, spacing))?,
        };
        Ok(Self { surface, span })
    }
}

#[must_use]
pub fn thinned(cloud: &PointCloud, spacing: Option<Distance>) -> PointCloud {
    spacing.map_or_else(|| cloud.clone(), |spacing| cloud.voxel_downsample(spacing))
}

fn oriented(cloud: PointCloud) -> anyhow::Result<OrientedCloud> {
    let neighbors = NORMAL_NEIGHBORS.min(cloud.count());
    let target = Target::with_estimated_normals(cloud, neighbors)
        .context("too few points to fit normals to")?;
    let normals: Vec<Vector> = target
        .normals()
        .context("no normals were fitted")?
        .iter()
        .map(|normal| normal.into_inner())
        .collect();
    Ok(OrientedCloud::new(target.cloud().clone(), &normals)?)
}
