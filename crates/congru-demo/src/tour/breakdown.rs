use std::path::Path;

use congru::{Distance, Point, PointCloud, RigidTransform};
use congru_scene::Scene;
use congru_track::{Settings, Tracker};
use congru_vision::pixel::{containing, coordinate};
use congru_vision::{Camera, DepthImage, back_project, render, within, without_support};

use super::film::{
    CROP, HEIGHT, SAMPLE_SHARE, SUPPORT_TOLERANCE, SUPPORT_WIDER, VOXEL_SHARE, WIDTH, detected,
    extent, shoot,
};
use crate::camera::view::CameraView;
use crate::picture::palette::TEXT;
use crate::picture::{self, Tile, font, outlined};

/// One frame of `scene`, step by step: what the camera returns, the points,
/// the support taken away, the region kept, the voxels, and the model
/// aligned onto them.
pub fn breakdown(scene: &Scene, target: &Path) -> anyhow::Result<()> {
    let scale = coordinate(WIDTH) / coordinate(scene.camera.width);
    let camera = Camera {
        width: WIDTH,
        height: HEIGHT,
        focal: scene.camera.focal * scale,
    };
    let model = &scene.target().mesh;
    let (centre, span) = extent(model);
    let (depth, truth) = shoot(scene, &camera, 0);
    let points = back_project(&depth, &camera);
    let ground_free = without_support(&points, SUPPORT_TOLERANCE * span, SUPPORT_WIDER * span);
    let guess = detected(&truth);
    let region = within(&ground_free, &guess.apply(&centre), CROP * span);
    let voxels =
        PointCloud::new(region.clone())?.voxel_downsample(Distance::new(VOXEL_SHARE * span)?);
    let view = CameraView::new(model, camera, Distance::new(span * SAMPLE_SHARE)?)?;
    let mut tracker = Tracker::new("tracker", view, Settings::for_part(span)?);
    let pose = tracker.update_from(&voxels, Some(guess))?.pose;
    let silhouette =
        |pose: &RigidTransform| render(&camera, &[(model, *pose)]).depth.map(f64::is_finite);
    let steps = [
        ("1 depth frame", depth.clone(), None),
        ("2 points", splat(&points, &camera), None),
        ("3 support removed", splat(&ground_free, &camera), None),
        (
            "4 region of interest",
            splat(&region, &camera),
            Some(silhouette(&guess)),
        ),
        ("5 voxels", splat(voxels.points(), &camera), None),
        (
            "6 aligned",
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

/// Points back into a depth image, the nearest per pixel, so every step is
/// drawn the same way.
fn splat(points: &[Point], camera: &Camera) -> DepthImage {
    let mut depth = DepthImage::filled(camera.width, camera.height, f64::NAN);
    for point in points {
        let Some([across, down]) = camera.project(point) else {
            continue;
        };
        if let (Some(column), Some(row)) = (
            containing(across, camera.width),
            containing(down, camera.height),
        ) {
            let current = depth.get(column, row);
            if current.is_nan() || point.z < current {
                depth.set(column, row, point.z);
            }
        }
    }
    depth
}
