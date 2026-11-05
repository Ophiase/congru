use congru::{Distance, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_vision::pixel::coordinate;
use congru_vision::{DepthImage, back_project, in_view, render, within};

use super::super::conditions::Condition;
use super::super::detect::Detector;
use super::{SCAN_VOXEL, Stage};

/// How far outside the predicted silhouette a return may lie, in pixels, and
/// how far from the predicted depth, in metres.
const VIEW_MARGIN: usize = 0;
const VIEW_GATE: f64 = 0.03;
const CROP_MARGIN: f64 = 0.015;
/// A frame with more returns than this many times the part's pixels holds
/// surroundings: a whole room does, a part and a can crossing do not.
const MOST_CROWDED: f64 = 3.0;

impl Stage {
    /// How many times the frame's returns outnumber the pixels of the part at
    /// `pose`. Hardly depends on the pose: a wrong prediction must not look like
    /// surroundings, or the crop would hide what shows it wrong.
    fn crowding(&self, depth: &DepthImage, pose: &RigidTransform) -> f64 {
        let rendered = render(&self.camera, &[(&self.part, *pose)]).depth;
        let count = |image: &DepthImage| {
            image
                .positions()
                .filter(|(column, row)| image.get(*column, *row).is_finite())
                .count()
        };
        coordinate(count(depth)) / coordinate(count(&rendered).max(1))
    }

    /// As [`Stage::crowding`], at the truth, for measuring.
    pub fn crowding_at_truth(&self, condition: &Condition, frame: usize) -> anyhow::Result<f64> {
        let (depth, truth) = self.sensed(condition, frame)?;
        Ok(self.crowding(&depth, &truth))
    }

    /// The scan to align and the pose it starts from. A ball around the part
    /// keeps the edges that correct a slide along a flat face, so it stays while
    /// it holds little else than the part. Otherwise the crop keeps to the
    /// silhouette, and the pose is first brought along rays, which what stands
    /// around cannot pull.
    pub(super) fn scan_for(
        &self,
        detector: &Detector,
        depth: &DepthImage,
        predicted: &RigidTransform,
    ) -> Option<(RigidTransform, PointCloud)> {
        let (centre, reach) = extent(&self.part);
        let points = back_project(depth, &self.camera);
        let ball = within(&points, &predicted.apply(&centre), reach);
        if self.crowding(depth, predicted) <= MOST_CROWDED {
            let cloud = PointCloud::new(ball).ok()?;
            let thinned = cloud.voxel_downsample(Distance::new(SCAN_VOXEL).ok()?);
            return Some((*predicted, thinned));
        }
        let start = detector.along_rays(depth, &self.camera, predicted);
        Some((start, self.scan_in_view(depth, &start)?))
    }

    /// The returns the part, as predicted, can explain.
    fn scan_in_view(&self, depth: &DepthImage, predicted: &RigidTransform) -> Option<PointCloud> {
        let rendered = render(&self.camera, &[(&self.part, *predicted)]).depth;
        Some(
            PointCloud::new(in_view(
                depth,
                &rendered,
                &self.camera,
                VIEW_MARGIN,
                VIEW_GATE,
            ))
            .ok()?
            .voxel_downsample(Distance::new(SCAN_VOXEL).ok()?),
        )
    }
}

/// The centre of the part's bounding box, and how far from it its points
/// and their scan can lie.
fn extent(part: &TriangleMesh) -> (Point, f64) {
    let points = part.vertices().points();
    let (lowest, highest) = points.iter().fold(
        (points[0].coords, points[0].coords),
        |(lowest, highest), point| (lowest.inf(&point.coords), highest.sup(&point.coords)),
    );
    let centre = Point::from((lowest + highest) / 2.0);
    let reach = points
        .iter()
        .map(|point| (point - centre).norm())
        .fold(0.0, f64::max)
        + CROP_MARGIN;
    (centre, reach)
}
