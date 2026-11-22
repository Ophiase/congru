use std::fmt::Write;
use std::path::{Path, PathBuf};

use congru_scene::Scene;
use congru_vision::DepthImage;
use nalgebra::Vector3;
use resvg::tiny_skia::{ColorU8, Pixmap, PixmapPaint, Transform};

const SKY: [f64; 3] = [233.0, 240.0, 247.0];
const SURFACE: [f64; 3] = [226.0, 221.0, 210.0];
const TARGET: [f64; 3] = [232.0, 128.0, 110.0];
const AMBIENT: f64 = 0.55;
const COLUMNS: u32 = 3;
const GAP: u32 = 8;
/// Two renders agree on a pixel when their depths differ by less than this.
const SAME_DEPTH: f64 = 0.002;

/// Every scene of `folder`, sorted by file name.
///
/// # Errors
/// When the folder cannot be listed or a scene cannot be built.
pub fn load_scenes(folder: &Path) -> anyhow::Result<Vec<Scene>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(folder)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .collect();
    paths.sort();
    Ok(paths
        .iter()
        .map(|path| Scene::load(path))
        .collect::<Result<_, _>>()?)
}

/// One sheet of every scene as the camera sees it, the target tinted, and
/// the text that names them.
///
/// # Errors
/// When the sheet cannot be written.
pub fn realistic(scenes: &[Scene], sheet: &Path) -> anyhow::Result<String> {
    let pictures: Vec<Pixmap> = scenes.iter().map(picture).collect::<anyhow::Result<_>>()?;
    let width = pictures.iter().map(Pixmap::width).max().unwrap_or(1);
    let height = pictures.iter().map(Pixmap::height).max().unwrap_or(1);
    let rows = u32::try_from(pictures.len())?.div_ceil(COLUMNS);
    let mut canvas = Pixmap::new(COLUMNS * (width + GAP), rows * (height + GAP))
        .ok_or_else(|| anyhow::anyhow!("empty sheet"))?;
    for (slot, picture) in (0_u32..).zip(&pictures) {
        let left = i32::try_from((slot % COLUMNS) * (width + GAP))?;
        let top = i32::try_from((slot / COLUMNS) * (height + GAP))?;
        canvas.draw_pixmap(
            left,
            top,
            picture.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
    canvas.save_png(sheet)?;
    let mut text = String::new();
    writeln!(
        text,
        "| Scene, left to right | What it tests |\n| --- | --- |"
    )?;
    for scene in scenes {
        writeln!(text, "| {} | {} |", scene.name, scene.summary)?;
    }
    Ok(text)
}

fn picture(scene: &Scene) -> anyhow::Result<Pixmap> {
    let depth = scene.render().depth;
    let target = scene.render_target().depth;
    let mut pixmap = Pixmap::new(
        u32::try_from(depth.width())?,
        u32::try_from(depth.height())?,
    )
    .ok_or_else(|| anyhow::anyhow!("empty scene"))?;
    let focal = scene.camera.focal;
    for (index, pixel) in pixmap.pixels_mut().iter_mut().enumerate() {
        let (column, row) = (index % depth.width(), index / depth.width());
        let here = depth.get(column, row);
        let colour = if here.is_finite() {
            let tint = if (target.get(column, row) - here).abs() < SAME_DEPTH {
                TARGET
            } else {
                SURFACE
            };
            tint.map(|channel| channel * shade(&depth, column, row, focal))
        } else {
            SKY
        };
        *pixel = ColorU8::from_rgba(byte(colour[0]), byte(colour[1]), byte(colour[2]), 255)
            .premultiply();
    }
    Ok(pixmap)
}

/// Lit from the camera by the normal the depth implies.
fn shade(depth: &DepthImage, column: usize, row: usize, focal: f64) -> f64 {
    let here = depth.get(column, row);
    let left = depth.get(column.saturating_sub(1), row);
    let above = depth.get(column, row.saturating_sub(1));
    if !(left.is_finite() && above.is_finite()) || column == 0 || row == 0 {
        return 1.0;
    }
    let pixel = here / focal;
    let normal = Vector3::new((here - left) / pixel, (here - above) / pixel, -1.0).normalize();
    AMBIENT + (1.0 - AMBIENT) * -normal.z
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to a byte first"
)]
fn byte(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}
