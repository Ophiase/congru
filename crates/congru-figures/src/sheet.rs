/// Figures already rendered, laid out `columns` wide in one SVG: each keeps
/// its own camera and size, nested where its cell starts.
#[must_use]
pub fn sheet(figures: &[String], columns: usize, cell: [f64; 2]) -> String {
    let columns = columns.max(1);
    let rows = figures.len().div_ceil(columns);
    let width = cell[0] * as_float(columns);
    let height = cell[1] * as_float(rows);
    let mut sheet = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
    );
    sheet.push('\n');
    for (slot, figure) in figures.iter().enumerate() {
        let left = cell[0] * as_float(slot % columns);
        let top = cell[1] * as_float(slot / columns);
        sheet.push_str(&figure.replacen("<svg ", &format!(r#"<svg x="{left}" y="{top}" "#), 1));
    }
    sheet.push_str("</svg>\n");
    sheet
}

#[expect(clippy::cast_precision_loss, reason = "a handful of cells")]
fn as_float(count: usize) -> f64 {
    count as f64
}
