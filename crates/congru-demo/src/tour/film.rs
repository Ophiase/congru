use congru::{Distance, NearestNeighbors, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_scene::Scene;
use congru_sim::Degradation;
use congru_synth::sequence::Handheld;
use congru_track::{Follower, Settings, Sight};
use congru_vision::{Camera, render};

use crate::camera::detect::region_of_interest;

use super::panels::panels;
use crate::picture::Tile;

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
const TURN_PER_FRAME: f64 = 0.006;
const SEARCH_STARTS: usize = 12;
const SEARCH_ITERATIONS: u32 = 15;
const CREEP_PER_FRAME: f64 = 0.004;
const SURFACE_SAMPLING: f64 = 1.0 / 50.0;
/// A lost target is searched at one frame in this many: a search costs seconds.
const SEARCH_EVERY: usize = 6;

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

/// Searches are cut short, and a frame is trusted with a quarter of its points on
/// the model, for the clutter around it.
pub(super) fn tour_settings(span: f64) -> anyhow::Result<Settings> {
    let mut settings = Settings::for_part(span)?;
    settings.find.starts = SEARCH_STARTS;
    settings.health.min_inliers = 0.25;
    settings.health.max_noise_ratio = 4.0;
    settings.find.icp.convergence.max_iterations = SEARCH_ITERATIONS;
    Ok(settings)
}

/// What the wandering camera sees at `frame`, exact then degraded, and the target's
/// true pose in its frame; the target moves too.
pub(super) fn shot(
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
    fn among_alike_crates_the_box_tells_which() -> anyhow::Result<()> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenes/crate-among-crates.toml");
        let shots = film(&Scene::load(&path)?, 1)?;
        assert!(shots[0].off < 0.05, "{:.3} of its size off", shots[0].off);
        Ok(())
    }

    #[test]
    fn a_turn_by_one_tooth_costs_little_where_a_part_turned_costs_much() -> anyhow::Result<()> {
        let tooth = std::f64::consts::TAU / 12.0;
        assert!(off_by(&gear()?, tooth)? < 0.002);
        assert!(off_by(&bracket()?, tooth)? > 0.005);
        Ok(())
    }
}
