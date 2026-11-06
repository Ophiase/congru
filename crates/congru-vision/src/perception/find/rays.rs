use congru::{Point, RigidTransform, fit_rigid};

use super::Finder;
use super::pairs::Oriented;
use crate::pixel::{containing, coordinate};
use crate::{Camera, DepthImage, ViewScore, render};

/// A wide gate gives the basin, a narrow one the precision: each pass of a
/// refinement divides the first gate by this, down to the last.
pub(super) const GATE_SHRINKS: [f64; 4] = [1.0, 1.5, 2.5, 4.0];
/// Fewer pairs than this and the motion is not trusted.
const LEAST_PAIRS: usize = 12;
/// How many pixels around its own a model point looks for its nearest seen
/// point: enough to slide sideways, too few to reach another surface.
const WINDOW: usize = 3;

/// Below this agreement with the depth, the object is reported missing.
const LEAST_AGREEMENT: f64 = 0.3;
/// The same in a box, which already said where it is: half hidden, a right pose
/// confirms this little.
const LEAST_IN_A_BOX: f64 = 0.1;
/// Less of its outline than this standing out, and nothing marks an object there.
const LEAST_OUTLINE: f64 = 0.25;

impl Finder {
    /// The share of the model's pixels at `pose` whose ray went through it, read
    /// at the tolerance the noise there asks for.
    #[must_use]
    pub fn through_share(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> f64 {
        let rendered = render(camera, &[(&self.model, *pose)]).depth;
        let view = ViewScore::of(depth, &rendered, camera, self.close_to(depth, &rendered));
        let total = view.support + view.violation + view.occluded;
        if total > 0.0 {
            view.violation / total
        } else {
            0.0
        }
    }

    /// Whether enough of the outline of `pose` stands out of the frame: what only an
    /// object explains. A flat face flush on a wall confirms itself and stands out
    /// nowhere.
    #[must_use]
    pub fn stands_out(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> bool {
        let rendered = render(camera, &[(&self.model, *pose)]).depth;
        let view = ViewScore::of(depth, &rendered, camera, self.close_to(depth, &rendered));
        let outline = view.standing + view.flush;
        outline > 0 && coordinate(view.standing) / coordinate(outline) >= LEAST_OUTLINE
    }

    /// `pose` brought onto the frame along the camera's rays, which the surroundings
    /// cannot pull: to a bin of the search, then polished, then settled.
    #[must_use]
    pub fn brought(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        pose: &RigidTransform,
    ) -> RigidTransform {
        let brought = self.refined(depth, camera, pose);
        let seen = self.observed_at(depth, camera, &brought);
        let polished = self.polished(&seen, camera, &brought);
        self.settled(depth, camera, &polished, seen.tolerance())
    }

    /// The model's pose in the camera's frame, when the frame holds it: the better
    /// of the search's own and that pose brought along rays. The rays usually
    /// settle it, and under heavy noise take it away.
    #[must_use]
    pub fn located(&self, depth: &DepthImage, camera: &Camera) -> Option<RigidTransform> {
        self.located_by(depth, depth, camera, LEAST_AGREEMENT)
    }

    /// As [`Finder::located`], `region` proposing and all of `depth` judging.
    #[must_use]
    pub fn located_in(
        &self,
        depth: &DepthImage,
        region: &DepthImage,
        camera: &Camera,
    ) -> Option<RigidTransform> {
        self.located_by(depth, region, camera, LEAST_IN_A_BOX)
    }

    fn located_by(
        &self,
        depth: &DepthImage,
        region: &DepthImage,
        camera: &Camera,
        least: f64,
    ) -> Option<RigidTransform> {
        let found = self.find_in(depth, region, camera, f64::NEG_INFINITY)?;
        let brought = self.brought(depth, camera, &found.pose);
        [found.pose, brought]
            .into_iter()
            .map(|pose| (self.confirmed_at(depth, camera, &pose), pose))
            .max_by(|left, right| left.0.total_cmp(&right.0))
            .filter(|(confirmed, pose)| *confirmed >= least && self.stands_out(depth, camera, pose))
            .map(|(_, pose)| pose)
    }
}

/// `pose` brought onto `depth` along the camera's rays: each model point
/// facing the camera reads only the pixels around its own, so the surfaces
/// around the object cannot pull it. Pairs farther apart than `gate` are
/// left out: a pixel seen in front is a shadow, one seen behind is void.
pub(super) fn along_rays(
    depth: &DepthImage,
    camera: &Camera,
    points: &[Oriented],
    pose: &RigidTransform,
    gate: f64,
    iterations: u32,
) -> RigidTransform {
    let mut pose = *pose;
    for _ in 0..iterations {
        let (model, seen): (Vec<Point>, Vec<Point>) = points
            .iter()
            .filter_map(|oriented| hit(depth, camera, &pose, oriented, gate))
            .unzip();
        if model.len() < LEAST_PAIRS {
            break;
        }
        match fit_rigid(&model, &seen, None) {
            Ok(fit) => pose = pose.then(&fit.transform),
            Err(_) => break,
        }
    }
    pose
}

fn hit(
    depth: &DepthImage,
    camera: &Camera,
    pose: &RigidTransform,
    oriented: &Oriented,
    gate: f64,
) -> Option<(Point, Point)> {
    let placed = pose.apply(&oriented.point);
    let normal = pose.rotation() * oriented.normal;
    if normal.dot(&placed.coords) >= 0.0 {
        return None;
    }
    let [across, down] = camera.project(&placed)?;
    let (column, row) = (
        containing(across, camera.width)?,
        containing(down, camera.height)?,
    );
    let columns = column.saturating_sub(WINDOW)..=(column + WINDOW).min(camera.width - 1);
    let rows = row.saturating_sub(WINDOW)..=(row + WINDOW).min(camera.height - 1);
    rows.flat_map(|row| columns.clone().map(move |column| (column, row)))
        .filter_map(|(column, row)| {
            let value = depth.get(column, row);
            value
                .is_finite()
                .then(|| camera.back_project(column, row, value))
        })
        .map(|seen| ((seen - placed).norm(), seen))
        .filter(|(distance, _)| *distance <= gate)
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, seen)| (placed, seen))
}
