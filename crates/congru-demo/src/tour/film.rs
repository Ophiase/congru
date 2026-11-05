use congru::{Distance, NearestNeighbors, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_scene::Scene;
use congru_sim::Degradation;
use congru_synth::sequence::Handheld;
use congru_track::{Settings, Tracker};
use congru_vision::{Camera, render, within};

use crate::camera::detect::{Detector, region_of_interest};
use crate::camera::stage::hold::Hold;

use crate::camera::view::CameraView;
use crate::picture::palette::{MUTED, TEXT};
use crate::picture::{Tile, font, outlined};

const WIDTH: usize = 320;
const HEIGHT: usize = 240;
pub(super) const SENSOR: Degradation = Degradation {
    blur: 0,
    noise: 0.005,
    holes: 0.02,
    flying_pixels: true,
    lens: 0.0,
};
const HANDHELD: Handheld = Handheld {
    angle: 0.05,
    shift: 0.0,
    period: 40.0,
};
pub(super) const CROP: f64 = 0.6;
const VOXEL_SHARE: f64 = 1.0 / 60.0;
const TURN_PER_FRAME: f64 = 0.006;
const SEARCH_STARTS: usize = 12;
const SAMPLE_SHARE: f64 = 1.0 / 80.0;
const SEARCH_ITERATIONS: u32 = 15;
const CREEP_PER_FRAME: f64 = 0.004;
const SURFACE_SAMPLING: f64 = 1.0 / 50.0;

/// The scene filmed for `frames` frames by a wandering camera, the target
/// found by its shape and then tracked, one tile per frame. The truth only
/// scores the tiles.
pub fn film(scene: &Scene, frames: usize) -> anyhow::Result<Vec<(Tile, f64)>> {
    let camera = camera_of(scene);
    let target = &scene.target().mesh;
    let (_, span) = extent(target);
    let detector = Detector::new(target, span)?;
    let mut hold = Hold::new(Settings::for_part(span)?.gains);
    let samples = target
        .sample_surface(Distance::new(span * SURFACE_SAMPLING)?)?
        .cloud()
        .clone();
    (0..frames)
        .map(|frame| {
            let (depth, truth) = shoot(scene, &camera, frame);
            let region = region_of_interest(&depth, &camera, target, &truth, frame as u64);
            let measured = hold
                .forecast()
                .or_else(|| detector.find(&region, &camera))
                .map(|start| detector.along_rays(&depth, &camera, &start));
            let pose = hold.judge(&detector, &camera, (&depth, &region), measured, false);
            let off = pose.map_or(Ok(f64::INFINITY), |pose| {
                surface_error(&samples, &pose, &truth).map(|metres| metres / span)
            })?;
            Ok((tile(scene, &camera, &depth, pose.as_ref(), &truth), off))
        })
        .collect()
}

/// The mean distance from the model at `pose` to the model at `truth`, nearest
/// point to nearest point: [ADD-S], blind to what a symmetry leaves unchanged, as
/// a gear turned by one tooth.
///
/// [ADD-S]: https://arxiv.org/abs/1711.00199
fn surface_error(
    samples: &PointCloud,
    pose: &RigidTransform,
    truth: &RigidTransform,
) -> anyhow::Result<f64> {
    let index = NearestNeighbors::new(&samples.transformed(truth))?;
    let total: f64 = samples
        .points()
        .iter()
        .map(|point| index.nearest(&pose.apply(point)).squared_distance.sqrt())
        .sum();
    Ok(total / congru_vision::pixel::coordinate(samples.count()))
}

/// The scene's camera, at the tour's resolution.
pub(super) fn camera_of(scene: &Scene) -> Camera {
    let scale = congru_vision::pixel::coordinate(WIDTH)
        / congru_vision::pixel::coordinate(scene.camera.width);
    Camera {
        width: WIDTH,
        height: HEIGHT,
        focal: scene.camera.focal * scale,
    }
}

/// The tracker of the tour: searches are cut short, and a frame is trusted
/// with a quarter of its points on the model, for the clutter around it.
pub(super) fn tracker_for(
    model: &TriangleMesh,
    camera: Camera,
    span: f64,
) -> anyhow::Result<Tracker<CameraView>> {
    let mut settings = Settings::for_part(span)?;
    settings.find.starts = SEARCH_STARTS;
    settings.health.min_inliers = 0.25;
    settings.health.max_noise_ratio = 4.0;
    settings.find.icp.convergence.max_iterations = SEARCH_ITERATIONS;
    let view = CameraView::new(model, camera, Distance::new(span * SAMPLE_SHARE)?)?;
    Ok(Tracker::new("tracker", view, settings))
}

/// What the wandering camera sees at `frame`, degraded, and the target's
/// true pose in its frame; the target moves too.
pub(super) fn shoot(
    scene: &Scene,
    camera: &Camera,
    frame: usize,
) -> (congru_vision::DepthImage, RigidTransform) {
    let view = scene.view.then(&HANDHELD.pose(frame));
    let target = scene.target();
    let moving = drive(target, frame).then(&target.pose);
    let placed: Vec<(&TriangleMesh, RigidTransform)> = scene
        .objects
        .iter()
        .map(|object| {
            let pose = if std::ptr::eq(object, target) {
                moving
            } else {
                object.pose
            };
            (&object.mesh, pose.then(&view))
        })
        .collect();
    (
        SENSOR.apply(&render(camera, &placed).depth, frame as u64),
        moving.then(&view),
    )
}

/// The target's own motion at `frame`, in its frame: it creeps forward along
/// its length and turns slowly, as a car pulling out or a part on a belt.
fn drive(target: &congru_scene::Placed, frame: usize) -> RigidTransform {
    let (_, span) = extent(&target.mesh);
    let time = congru_vision::pixel::coordinate(frame);
    RigidTransform::from_axis_angle(
        congru::Vector::z() * (TURN_PER_FRAME * time),
        [CREEP_PER_FRAME * span * time, 0.0, 0.0],
    )
}

/// The frame's points near `focus`, thinned: one point per voxel, so the
/// density no longer depends on the distance.
pub(super) fn around(points: &[Point], focus: &Point, span: f64) -> anyhow::Result<PointCloud> {
    Ok(PointCloud::new(within(points, focus, CROP * span))?
        .voxel_downsample(Distance::new(VOXEL_SHARE * span)?))
}

fn tile(
    scene: &Scene,
    camera: &Camera,
    depth: &congru_vision::DepthImage,
    estimate: Option<&RigidTransform>,
    truth: &RigidTransform,
) -> Tile {
    let target = &scene.target().mesh;
    let silhouette =
        estimate.map(|pose| render(camera, &[(target, *pose)]).depth.map(f64::is_finite));
    let error = estimate.map_or(f64::INFINITY, |pose| {
        pose.rotation_error_to(truth).to_degrees()
    });
    let mut picture = outlined(depth, camera.focal, silhouette.as_ref());
    font::write(&mut picture, 8, 8, &scene.name, MUTED);
    font::write(&mut picture, 8, HEIGHT - 22, &format!("{error:.1}°"), TEXT);
    Tile {
        picture,
        error_degrees: error,
    }
}

/// The centre of the target's bounding box, and its diagonal.
pub(super) fn extent(mesh: &TriangleMesh) -> (Point, f64) {
    let points = mesh.vertices().points();
    let (lowest, highest) = points.iter().fold(
        (points[0].coords, points[0].coords),
        |(lowest, highest), point| (lowest.inf(&point.coords), highest.sup(&point.coords)),
    );
    (
        Point::from((lowest + highest) / 2.0),
        (highest - lowest).norm(),
    )
}

#[cfg(test)]
mod tests {
    use congru_sim::{bracket, gear};

    use super::*;

    fn off_by(mesh: &TriangleMesh, angle: f64) -> anyhow::Result<f64> {
        let samples = mesh.sample_surface(Distance::new(0.003)?)?.cloud().clone();
        let turned = RigidTransform::from_axis_angle(congru::Vector::z() * angle, [0.0; 3]);
        surface_error(&samples, &turned, &RigidTransform::identity())
    }

    #[test]
    fn a_turn_by_one_tooth_costs_little_where_a_part_turned_costs_much() -> anyhow::Result<()> {
        let tooth = std::f64::consts::TAU / 12.0;
        assert!(off_by(&gear()?, tooth)? < 0.002);
        assert!(off_by(&bracket()?, tooth)? > 0.005);
        Ok(())
    }
}
