use congru::{Distance, RigidTransform, TriangleMesh};
use congru_vision::pixel::coordinate;
use congru_vision::{Camera, DepthImage, Finder, render};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

const STEP_SHARE: f64 = 1.0 / 20.0;
/// A simulated box is this much wider than the object, and off by this share.
const BOX_GROWTH: f64 = 0.4;
const BOX_SHIFT: f64 = 0.1;
/// The region of interest of a video: a little larger, a little off.
const ROI_GROWTH: f64 = 0.2;
const ROI_SHIFT: f64 = 0.05;
const ROI_SEED: u64 = 7_000;
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
        let brought = self.finder.refined(depth, camera, pose);
        let seen = self.finder.observed_at(depth, camera, &brought);
        let polished = self.finder.polished(&seen, camera, &brought);
        self.finder
            .settled(depth, camera, &polished, seen.tolerance())
    }

    /// The share of what the model could say at `pose` that the frame says, less
    /// what its rays refute.
    pub fn confirmed(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> f64 {
        self.finder.confirmed_at(depth, camera, pose)
    }

    /// The share of the model's pixels at `pose` whose ray went through it.
    pub fn through_share(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> f64 {
        self.finder.through_share(depth, camera, pose)
    }

    /// The model found again where `forecast` puts it, turned any way.
    pub fn turned_again(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        forecast: &RigidTransform,
    ) -> Option<RigidTransform> {
        self.finder
            .turned_again(depth, camera, forecast)
            .map(|found| found.pose)
    }

    /// The model's pose in the camera's frame, when the frame holds it: the better
    /// of the search's own and that pose brought along rays. The rays usually
    /// settle it, and under heavy noise take it away.
    pub fn find(&self, depth: &DepthImage, camera: &Camera) -> Option<RigidTransform> {
        self.find_in(depth, depth, camera)
    }

    /// As [`Detector::find`], `region` proposing and all of `depth` judging.
    pub fn find_in(
        &self,
        depth: &DepthImage,
        region: &DepthImage,
        camera: &Camera,
    ) -> Option<RigidTransform> {
        let found = self
            .finder
            .find_in(depth, region, camera, f64::NEG_INFINITY)?;
        let brought = self.along_rays(depth, camera, &found.pose);
        [found.pose, brought]
            .into_iter()
            .map(|pose| (self.confirmed(depth, camera, &pose), pose))
            .max_by(|left, right| left.0.total_cmp(&right.0))
            .filter(|(confirmed, _)| *confirmed >= LEAST_AGREEMENT)
            .map(|(_, pose)| pose)
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
    kept_in_box(depth, camera, model, truth, BOX_GROWTH, [BOX_SHIFT; 2])
}

/// The region of interest a video would be given: the true silhouette's box,
/// a little larger, its offset drawn anew at each frame. The one input taken from
/// the truth, standing in for a detector: never the shape, only a rectangle.
pub fn region_of_interest(
    depth: &DepthImage,
    camera: &Camera,
    model: &TriangleMesh,
    truth: &RigidTransform,
    frame: u64,
) -> DepthImage {
    let mut random = ChaCha8Rng::seed_from_u64(frame + ROI_SEED);
    let mut offset = || ROI_SHIFT * (random.random::<f64>() * 2.0 - 1.0);
    let shifts = [offset(), offset()];
    kept_in_box(depth, camera, model, truth, ROI_GROWTH, shifts)
}

fn kept_in_box(
    depth: &DepthImage,
    camera: &Camera,
    model: &TriangleMesh,
    truth: &RigidTransform,
    growth: f64,
    shifts: [f64; 2],
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
    let grown = |low: usize, high: usize, shift: f64| {
        let (low, high) = (coordinate(low), coordinate(high));
        let size = high - low;
        (
            low - growth * size / 2.0 + shift * size,
            high + growth * size / 2.0 + shift * size,
        )
    };
    let (columns, rows) = (grown(left, right, shifts[0]), grown(top, bottom, shifts[1]));
    let mut kept = DepthImage::filled(depth.width(), depth.height(), f64::NAN);
    for (column, row) in depth.positions() {
        let (across, down) = (coordinate(column), coordinate(row));
        if (columns.0..=columns.1).contains(&across) && (rows.0..=rows.1).contains(&down) {
            kept.set(column, row, depth.get(column, row));
        }
    }
    kept
}
