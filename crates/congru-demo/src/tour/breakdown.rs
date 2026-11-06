use std::path::Path;

use congru::RigidTransform;
use congru_scene::Scene;
use congru_vision::{Camera, DepthImage, Finder, back_project, render, within};

use super::film::{CROP, around, camera_of, extent, shoot, tracker_for};
use crate::camera::detect::boxed;
use crate::picture::palette::TEXT;
use crate::picture::{self, Tile, font, outlined, splat};

/// The first frame of `scene`, step by step: what the camera returns, the
/// points, the target found by its shape, the region kept around it, the
/// voxels, and the model tracked onto them.
pub fn breakdown(scene: &Scene, target: &Path) -> anyhow::Result<()> {
    let camera = camera_of(scene);
    let model = &scene.target().mesh;
    let (centre, span) = extent(model);
    let (depth, truth) = shoot(scene, &camera, 0);
    let seen = boxed(&depth, &camera, model, &truth);
    let points = back_project(&depth, &camera);
    let found = Finder::for_span(model, span)?
        .located(&seen, &camera)
        .ok_or_else(|| anyhow::anyhow!("the target was not found in {}", scene.name))?;
    let focus = found.apply(&centre);
    let region = within(&points, &focus, CROP * span);
    let voxels = around(&points, &focus, span)?;
    let pose = tracker_for(model, camera, span)?
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
