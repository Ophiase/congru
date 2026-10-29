use congru::{Distance, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_scene::Scene;
use congru_sim::Degradation;
use congru_synth::sequence::Handheld;
use congru_track::{Settings, Tracker};
use congru_vision::{Camera, back_project, render, within, without_support};

use crate::camera::view::CameraView;
use crate::picture::palette::{MUTED, TEXT};
use crate::picture::{Tile, font, outlined};

pub const FRAMES: usize = 72;
const WIDTH: usize = 320;
const HEIGHT: usize = 240;
const SENSOR: Degradation = Degradation {
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
const CROP: f64 = 0.75;
const SUPPORT_TOLERANCE: f64 = 0.02;
const SUPPORT_WIDER: f64 = 1.5;
const VOXEL_SHARE: f64 = 1.0 / 60.0;

/// The scene filmed for `FRAMES` frames by a wandering camera, the target
/// tracked from a detector's first crop, one tile per frame.
pub fn film(scene: &Scene) -> anyhow::Result<Vec<Tile>> {
    let scale = congru_vision::pixel::coordinate(WIDTH)
        / congru_vision::pixel::coordinate(scene.camera.width);
    let camera = Camera {
        width: WIDTH,
        height: HEIGHT,
        focal: scene.camera.focal * scale,
    };
    let target = &scene.target().mesh;
    let (centre, span) = extent(target);
    let mut tracker = Tracker::new(
        "tracker",
        CameraView::new(target, camera)?,
        Settings::for_part(span)?,
    );
    let mut last: Option<RigidTransform> = None;
    (0..FRAMES)
        .map(|frame| {
            let (depth, truth) = shoot(scene, &camera, frame);
            let focus = last.unwrap_or(truth).apply(&centre);
            let update = tracker
                .update(&perceive(&depth, &camera, &focus, span)?)
                .ok()
                .filter(|update| update.trusted);
            last = update.as_ref().map(|update| update.pose).or(last);
            Ok(tile(scene, &camera, &depth, last.as_ref(), &truth))
        })
        .collect()
}

/// What the wandering camera sees at `frame`, degraded, and the target's
/// true pose in its frame.
fn shoot(
    scene: &Scene,
    camera: &Camera,
    frame: usize,
) -> (congru_vision::DepthImage, RigidTransform) {
    let view = scene.view.then(&HANDHELD.pose(frame));
    let placed: Vec<(&TriangleMesh, RigidTransform)> = scene
        .objects
        .iter()
        .map(|object| (&object.mesh, object.pose.then(&view)))
        .collect();
    (
        SENSOR.apply(&render(camera, &placed).depth, frame as u64),
        scene.target().pose.then(&view),
    )
}

/// The frame's points without the support plane, cropped around `focus`
/// and thinned.
fn perceive(
    depth: &congru_vision::DepthImage,
    camera: &Camera,
    focus: &Point,
    span: f64,
) -> anyhow::Result<PointCloud> {
    let points = without_support(
        &back_project(depth, camera),
        SUPPORT_TOLERANCE * span,
        SUPPORT_WIDER * span,
    );
    Ok(PointCloud::new(within(&points, focus, CROP * span))?
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
fn extent(mesh: &TriangleMesh) -> (Point, f64) {
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
