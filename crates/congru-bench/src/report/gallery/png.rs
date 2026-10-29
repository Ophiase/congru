use std::path::Path;

use resvg::tiny_skia::{Color, Pixmap, Transform};
use resvg::usvg::{Options, Tree};

/// A warm, pale paper, so the sheets read the same in any viewer.
const PAPER: [u8; 3] = [251, 248, 241];

/// Rasterises `svg` to a PNG at `path`, scaled by `scale`, on paper: a
/// sheet of thousands of dots is far lighter as pixels than as vectors.
///
/// # Errors
/// When the SVG cannot be read or the PNG cannot be written.
pub fn write_png(svg: &str, path: &Path, scale: f32) -> anyhow::Result<()> {
    let mut options = Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = Tree::from_str(svg, &options)?;
    let size = tree
        .size()
        .to_int_size()
        .scale_by(scale)
        .ok_or_else(|| anyhow::anyhow!("empty sheet"))?;
    let mut pixmap =
        Pixmap::new(size.width(), size.height()).ok_or_else(|| anyhow::anyhow!("empty sheet"))?;
    pixmap.fill(Color::from_rgba8(PAPER[0], PAPER[1], PAPER[2], 255));
    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.save_png(path)?;
    Ok(())
}
