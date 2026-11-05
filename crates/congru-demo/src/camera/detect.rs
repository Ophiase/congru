use congru::{Distance, RigidTransform, TriangleMesh};
use congru_vision::pixel::coordinate;
use congru_vision::{Camera, DepthImage, Finder, render};

const STEP_SHARE: f64 = 1.0 / 20.0;
/// A simulated box is this much wider than the object, and off by this share.
const BOX_GROWTH: f64 = 0.4;
const BOX_SHIFT: f64 = 0.1;
/// Below this agreement with the depth, the object is reported missing.
const LEAST_AGREEMENT: f64 = 0.3;

/// Finds the target in a whole frame from its shape alone: no guess, no
/// assumption on what it stands on. What a headset does when it first sees
/// a known object, and again whenever the tracker lets go of it.
#[derive(Debug)]
pub struct Detector {
    finder: Finder,
}

impl Detector {
    pub fn new(model: &TriangleMesh, span: f64) -> anyhow::Result<Self> {
        Ok(Self {
            finder: Finder::new(model, Distance::new(span * STEP_SHARE)?)?,
        })
    }

    /// `pose` brought onto the frame along the camera's rays, which the
    /// surroundings cannot pull.
    pub fn along_rays(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        pose: &RigidTransform,
    ) -> RigidTransform {
        self.finder.refined(depth, camera, pose)
    }

    /// The model's pose in the camera's frame, when the frame holds it.
    pub fn find(&self, depth: &DepthImage, camera: &Camera) -> Option<RigidTransform> {
        self.finder
            .find(depth, camera, LEAST_AGREEMENT)
            .map(|found| found.pose)
    }
}

/// What a 2D detector on the colour image gives: a box around the object,
/// here simulated from the truth, grown and shifted. Only the pixels in it
/// are kept; neither depth nor orientation comes from the truth.
pub fn boxed(
    depth: &DepthImage,
    camera: &Camera,
    model: &TriangleMesh,
    truth: &RigidTransform,
) -> DepthImage {
    let silhouette = render(camera, &[(model, *truth)]).depth;
    let inside: Vec<(usize, usize)> = silhouette
        .positions()
        .filter(|(column, row)| silhouette.get(*column, *row).is_finite())
        .collect();
    let bounds = |pick: fn(&(usize, usize)) -> usize| {
        let values = inside.iter().map(pick);
        (values.clone().min().unwrap_or(0), values.max().unwrap_or(0))
    };
    let (left, right) = bounds(|pixel| pixel.0);
    let (top, bottom) = bounds(|pixel| pixel.1);
    let grown = |low: usize, high: usize| {
        let (low, high) = (coordinate(low), coordinate(high));
        let size = high - low;
        let shift = BOX_SHIFT * size;
        (
            low - BOX_GROWTH * size / 2.0 + shift,
            high + BOX_GROWTH * size / 2.0 + shift,
        )
    };
    let (columns, rows) = (grown(left, right), grown(top, bottom));
    let mut kept = DepthImage::filled(depth.width(), depth.height(), f64::NAN);
    for (column, row) in depth.positions() {
        let (across, down) = (coordinate(column), coordinate(row));
        if (columns.0..=columns.1).contains(&across) && (rows.0..=rows.1).contains(&down) {
            kept.set(column, row, depth.get(column, row));
        }
    }
    kept
}
