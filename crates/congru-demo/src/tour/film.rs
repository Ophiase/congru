use congru::{Distance, NearestNeighbors, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_scene::Scene;
use congru_sim::Degradation;
use congru_synth::sequence::Handheld;
use congru_track::{CameraView, Follower, Settings, Sight, Tracker};
use congru_vision::{Camera, Image, back_project, render, within};

use crate::camera::detect::region_of_interest;

use crate::camera::overlay::rainbow;
use crate::picture::palette::{EMPTY, TEXT, Tone};
use crate::picture::{Tile, font, outline, outlined, splat};

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
/// A lost target is searched at one frame in this many: a search costs seconds.
const SEARCH_EVERY: usize = 6;
/// The bar turns amber past 2 % of the target's size, red past 10 %.
const BAR_PERCENT_PER_DEGREE: f64 = 2.0;

/// One filmed frame: what is drawn, and how far off the answer is, as a share of
/// the target's size.
pub(super) struct Shot {
    pub(super) tiles: Vec<Tile>,
    pub(super) off: f64,
    pub(super) degrees: f64,
}

/// The scene filmed for `frames` frames by a wandering camera, the target found
/// by its shape inside a noisy box and followed. The truth gives the box, and
/// otherwise only scores.
pub fn film(scene: &Scene, frames: usize) -> anyhow::Result<Vec<Shot>> {
    let camera = camera_of(scene);
    let target = &scene.target().mesh;
    let (_, span) = extent(target);
    let mut follower =
        Follower::new(target, camera, span, tour_settings(span)?)?.searching_every(SEARCH_EVERY);
    let samples = target
        .sample_surface(Distance::new(span * SURFACE_SAMPLING)?)?
        .cloud()
        .clone();
    (0..frames)
        .map(|frame| {
            let (exact, depth, truth) = shot(scene, &camera, frame);
            let region = region_of_interest(&depth, &camera, target, &truth, frame as u64);
            let pose = follower
                .step(Sight {
                    depth: &depth,
                    region: &region,
                })
                .pose;
            let off = pose.map_or(Ok(f64::INFINITY), |pose| {
                surface_error(&samples, &pose, &truth).map(|metres| metres / span)
            })?;
            let silhouette =
                pose.map(|pose| render(&camera, &[(target, pose)]).depth.map(f64::is_finite));
            let tiles = panels(&camera, (&exact, &depth, &region), silhouette.as_ref(), off);
            let degrees = pose.map_or(f64::INFINITY, |pose| {
                pose.rotation_error_to(&truth).to_degrees()
            });
            Ok(Shot {
                tiles,
                off,
                degrees,
            })
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
    let view = CameraView::new(model, camera, Distance::new(span * SAMPLE_SHARE)?)?;
    Ok(Tracker::new("tracker", view, tour_settings(span)?))
}

fn tour_settings(span: f64) -> anyhow::Result<Settings> {
    let mut settings = Settings::for_part(span)?;
    settings.find.starts = SEARCH_STARTS;
    settings.health.min_inliers = 0.25;
    settings.health.max_noise_ratio = 4.0;
    settings.find.icp.convergence.max_iterations = SEARCH_ITERATIONS;
    Ok(settings)
}

/// What the wandering camera sees at `frame`, degraded, and the target's
/// true pose in its frame; the target moves too.
pub(super) fn shoot(
    scene: &Scene,
    camera: &Camera,
    frame: usize,
) -> (congru_vision::DepthImage, RigidTransform) {
    let (_, depth, truth) = shot(scene, camera, frame);
    (depth, truth)
}

/// As [`shoot`], with the exact depth first.
fn shot(
    scene: &Scene,
    camera: &Camera,
    frame: usize,
) -> (
    congru_vision::DepthImage,
    congru_vision::DepthImage,
    RigidTransform,
) {
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
    let exact = render(camera, &placed).depth;
    let sensor = Degradation {
        noise: scene.noise_at_one_metre.unwrap_or(SENSOR.noise),
        ..SENSOR
    };
    let depth = sensor.apply(&exact, frame as u64);
    (exact, depth, moving.then(&view))
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

/// The three panels of a frame, as demo B draws them, at half the camera's size:
/// the scene, the depth, the points the box holds, each under the outline of the
/// answer. The bar and the number read the surface error, not the rotation, which
/// would fail a gear turned by one tooth.
fn panels(
    camera: &Camera,
    (exact, depth, region): (
        &congru_vision::DepthImage,
        &congru_vision::DepthImage,
        &congru_vision::DepthImage,
    ),
    silhouette: Option<&Image<bool>>,
    off: f64,
) -> Vec<Tile> {
    let points = back_project(region, camera);
    let mut drawn = [
        outlined(exact, camera.focal, None),
        rainbow(depth, exact),
        rainbow(&splat(&points, camera), exact),
    ];
    if let Some(silhouette) = silhouette {
        for panel in &mut drawn {
            outline(panel, silhouette);
        }
    }
    let percent = 100.0 * off;
    drawn
        .into_iter()
        .map(|picture| {
            let mut picture = halved(&picture);
            let bottom = picture.height() - 14;
            font::write(&mut picture, 4, bottom, &format!("{percent:.1}%"), TEXT);
            Tile {
                picture,
                error_degrees: percent / BAR_PERCENT_PER_DEGREE,
            }
        })
        .collect()
}

fn halved(picture: &Image<Tone>) -> Image<Tone> {
    let mut half = Image::filled(picture.width() / 2, picture.height() / 2, EMPTY);
    for (column, row) in half.positions() {
        half.set(column, row, picture.get(column * 2, row * 2));
    }
    half
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
