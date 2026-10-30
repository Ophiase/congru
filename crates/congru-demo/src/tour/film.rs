use congru::{Distance, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_scene::Scene;
use congru_sim::Degradation;
use congru_synth::sequence::Handheld;
use congru_track::{Settings, Tracker};
use congru_vision::{Camera, back_project, render, within};

use crate::camera::detect::{Detector, boxed};

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

/// The scene filmed for `frames` frames by a wandering camera, the target
/// found by its shape and then tracked, one tile per frame. The truth only
/// scores the tiles.
pub fn film(scene: &Scene, frames: usize) -> anyhow::Result<Vec<Tile>> {
    let camera = camera_of(scene);
    let target = &scene.target().mesh;
    let (centre, span) = extent(target);
    let mut tracker = tracker_for(target, camera, span)?;
    let detector = Detector::new(target, span)?;
    let mut last: Option<RigidTransform> = None;
    (0..frames)
        .map(|frame| {
            let (depth, truth) = shoot(scene, &camera, frame);
            let points = back_project(&depth, &camera);
            let update = last
                .or_else(|| detector.find(&boxed(&depth, &camera, target, &truth), &camera))
                .and_then(|start| {
                    let region = around(&points, &start.apply(&centre), span).ok()?;
                    tracker.update_from(&region, Some(start)).ok()
                })
                .filter(|update| update.trusted);
            last = update.map(|update| update.pose);
            Ok(tile(scene, &camera, &depth, last.as_ref(), &truth))
        })
        .collect()
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
