use std::path::Path;

use congru::{Distance, Point, PointCloud, RigidTransform};
use congru_scene::Scene;
use congru_track::{CameraView, Tracker};
use congru_vision::{Camera, DepthImage, Finder, back_project, render, within};

use super::film::{camera_of, extent, shot, tour_settings};
use crate::camera::detect::boxed;
use crate::picture::palette::TEXT;
use crate::picture::{self, Tile, font, outlined, splat};

const CROP: f64 = 0.6;
const VOXEL_SHARE: f64 = 1.0 / 60.0;
const SAMPLE_SHARE: f64 = 1.0 / 80.0;

/// The first frame of `scene`, step by step: what the camera returns, the
/// points, the target found by its shape, the region kept around it, the
/// voxels, and the model tracked onto them.
pub fn breakdown(scene: &Scene, target: &Path) -> anyhow::Result<()> {
    let camera = camera_of(scene);
    let model = &scene.target().mesh;
    let (centre, span) = extent(model);
    let (_, depth, truth) = shot(scene, &camera, 0);
    let seen = boxed(&depth, &camera, model, &truth);
    let points = back_project(&depth, &camera);
    let found = Finder::for_span(model, span)?
        .located(&seen, &camera)
        .ok_or_else(|| anyhow::anyhow!("the target was not found in {}", scene.name))?;
    let focus = found.apply(&centre);
    let region = within(&points, &focus, CROP * span);
    let voxels = around(&points, &focus, span)?;
    let view = CameraView::new(model, camera, Distance::new(span * SAMPLE_SHARE)?)?;
    let pose = Tracker::new("tracker", view, tour_settings(span)?)
        .update_from(&voxels, Some(found))?
        .pose;
    let silhouette = |pose: &RigidTransform| outline_of(model, &camera, pose);
    let steps = [
        ("1 depth frame", depth.clone(), None),
        ("2 detector box", seen.clone(), None),
        (
            "3 found by shape",
            splat(&points, &camera),
            Some(silhouette(&found)),
        ),
        ("4 region kept", splat(&region, &camera), None),
        ("5 voxels", splat(voxels.points(), &camera), None),
        (
            "6 tracked",
            splat(voxels.points(), &camera),
            Some(silhouette(&pose)),
        ),
    ];
    let tiles: Vec<Tile> = steps
        .into_iter()
        .map(|(name, image, outline)| tile(name, &image, outline.as_ref(), camera.focal))
        .collect();
    picture::write(target, &[tiles], 3)
}

/// The frame's points near `focus`, thinned: one point per voxel, so the density no
/// longer depends on the distance.
fn around(points: &[Point], focus: &Point, span: f64) -> anyhow::Result<PointCloud> {
    Ok(PointCloud::new(within(points, focus, CROP * span))?
        .voxel_downsample(Distance::new(VOXEL_SHARE * span)?))
}

fn outline_of(
    model: &congru::TriangleMesh,
    camera: &Camera,
    pose: &RigidTransform,
) -> congru_vision::Image<bool> {
    render(camera, &[(model, *pose)]).depth.map(f64::is_finite)
}

fn tile(
    name: &str,
    image: &DepthImage,
    outline: Option<&congru_vision::Image<bool>>,
    focal: f64,
) -> Tile {
    let mut picture = outlined(image, focal, outline);
    font::write(&mut picture, 8, 8, name, TEXT);
    Tile {
        picture,
        error_degrees: 0.0,
    }
}
