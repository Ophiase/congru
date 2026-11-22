use congru::{Distance, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_vision::pixel::coordinate;
use congru_vision::{Camera, DepthImage, Finder, back_project, render, within};

use super::Sight;

/// A scan is thinned to one point per voxel of this share of the object's size.
const VOXEL_SHARE: f64 = 1.0 / 60.0;
/// How far past the object's reach its points may lie, as a share of its size.
const MARGIN_SHARE: f64 = 0.06;
/// A frame with more returns than this many times the object's pixels holds
/// surroundings: a whole room does, a part and a can crossing do not.
const MOST_CROWDED: f64 = 3.0;

/// The points to align one frame on, and the pose they start from.
#[derive(Debug)]
pub(super) struct Scan {
    pub(super) start: RigidTransform,
    pub(super) points: PointCloud,
    pub(super) crowded: bool,
}

/// Where an object's points can lie around its pose.
#[derive(Debug)]
pub(super) struct Scope {
    part: TriangleMesh,
    centre: Point,
    reach: f64,
    voxel: f64,
}

impl Scope {
    pub(super) fn new(part: &TriangleMesh, span: f64) -> Self {
        let points = part.vertices().points();
        let (lowest, highest) = points.iter().fold(
            (points[0].coords, points[0].coords),
            |(lowest, highest), point| (lowest.inf(&point.coords), highest.sup(&point.coords)),
        );
        let centre = Point::from((lowest + highest) / 2.0);
        let reach = points
            .iter()
            .map(|point| (point - centre).norm())
            .fold(0.0, f64::max);
        Self {
            part: part.clone(),
            centre,
            reach: reach + MARGIN_SHARE * span,
            voxel: VOXEL_SHARE * span,
        }
    }

    /// A ball around the object keeps the edges that correct a slide along a flat
    /// face, so it stays while the frame holds little else. Otherwise the scan is
    /// what the region holds, and the pose is first brought along rays, which what
    /// stands around cannot pull. The model never cuts the scan to its shape: a
    /// hypothesis must not choose the data that confirm it.
    pub(super) fn scan(
        &self,
        finder: &Finder,
        camera: &Camera,
        sight: Sight<'_>,
        predicted: &RigidTransform,
    ) -> Option<Scan> {
        let thin = |points: Vec<Point>| {
            let cloud = PointCloud::new(points).ok()?;
            Some(cloud.voxel_downsample(Distance::new(self.voxel).ok()?))
        };
        if crowding(&self.part, camera, sight.depth, predicted) <= MOST_CROWDED {
            let points = back_project(sight.depth, camera);
            let ball = within(&points, &predicted.apply(&self.centre), self.reach);
            return Some(Scan {
                start: *predicted,
                points: thin(ball)?,
                crowded: false,
            });
        }
        Some(Scan {
            start: finder.brought(sight.depth, camera, predicted),
            points: thin(back_project(sight.region, camera))?,
            crowded: true,
        })
    }
}

/// How many times the frame's returns outnumber the pixels of `part` at `pose`.
/// Hardly depends on the pose: a wrong prediction must not look like surroundings,
/// or the crop would hide what shows it wrong.
#[must_use]
pub fn crowding(
    part: &TriangleMesh,
    camera: &Camera,
    depth: &DepthImage,
    pose: &RigidTransform,
) -> f64 {
    let rendered = render(camera, &[(part, *pose)]).depth;
    let count = |image: &DepthImage| {
        image
            .positions()
            .filter(|(column, row)| image.get(*column, *row).is_finite())
            .count()
    };
    coordinate(count(depth)) / coordinate(count(&rendered).max(1))
}
