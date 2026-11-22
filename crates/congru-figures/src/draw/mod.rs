mod mark;
mod style;
mod view;

use std::fmt::Write;

use congru::Point;

pub use mark::{Ink, Mark};
use view::Frame;
pub use view::View;

const PANEL: f64 = 300.0;
const CAPTION_HEIGHT: f64 = 34.0;

/// Panels side by side, one shared camera, colours that follow the
/// reader's light or dark theme.
#[derive(Debug)]
pub struct Figure {
    view: View,
    panels: Vec<Panel>,
}

#[derive(Debug)]
struct Panel {
    caption: String,
    marks: Vec<(f64, Mark)>,
}

impl Figure {
    #[must_use]
    pub fn new(view: View) -> Self {
        Self {
            view,
            panels: Vec::new(),
        }
    }

    pub fn panel(&mut self, caption: &str) -> &mut Self {
        self.panels.push(Panel {
            caption: caption.to_owned(),
            marks: Vec::new(),
        });
        self
    }

    pub fn crosses(&mut self, points: &[Point], ink: Ink) -> &mut Self {
        self.each(points, |at| Mark::Cross { at, ink })
    }

    pub fn dots(&mut self, points: &[Point], ink: Ink) -> &mut Self {
        self.each(points, |at| Mark::Dot { at, ink })
    }

    pub fn links(&mut self, from: &[Point], to: &[Point]) -> &mut Self {
        for (start, end) in from.iter().zip(to) {
            let (start, depth) = self.view.project(start);
            let (end, _) = self.view.project(end);
            self.push(depth + 1.0, Mark::Link { start, end });
        }
        self
    }

    /// A dashed circle of `radius`, a length of the scene, around each point.
    pub fn rings(&mut self, points: &[Point], radius: f64) -> &mut Self {
        let projected = self.view.scale(radius);
        for point in points {
            let (at, depth) = self.view.project(point);
            self.push(
                depth + 0.5,
                Mark::Ring {
                    at,
                    radius: projected,
                },
            );
        }
        self
    }

    pub fn labels(&mut self, points: &[Point], texts: &[&'static str]) -> &mut Self {
        for (point, text) in points.iter().zip(texts) {
            let (at, depth) = self.view.project(point);
            self.push(depth, Mark::Label { at, text });
        }
        self
    }

    fn each(&mut self, points: &[Point], mark: impl Fn([f64; 2]) -> Mark) -> &mut Self {
        for point in points {
            let (at, depth) = self.view.project(point);
            self.push(depth, mark(at));
        }
        self
    }

    fn push(&mut self, depth: f64, mark: Mark) {
        if let Some(panel) = self.panels.last_mut() {
            panel.marks.push((depth, mark));
        }
    }

    /// # Errors
    /// Never in practice; writing to a string cannot fail.
    pub fn render(mut self) -> Result<String, std::fmt::Error> {
        let count = u32::try_from(self.panels.len()).unwrap_or(u32::MAX);
        let mut svg = String::new();
        writeln!(
            svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" font-family="system-ui, sans-serif">"#,
            width = PANEL * f64::from(count),
            height = PANEL + CAPTION_HEIGHT,
        )?;
        svg.push_str(style::STYLE);
        for (slot, panel) in (0..count).zip(&mut self.panels) {
            let left = PANEL * f64::from(slot);
            writeln!(
                svg,
                r#"<text class="caption" x="{x}" y="{y}" text-anchor="middle">{caption}</text>"#,
                x = left + PANEL / 2.0,
                y = PANEL + CAPTION_HEIGHT / 2.0,
                caption = panel.caption,
            )?;
            panel.marks.sort_by(|far, near| near.0.total_cmp(&far.0));
            for (depth, mark) in &panel.marks {
                mark.write(&mut svg, &Frame::new(left, PANEL), self.view.shade(*depth))?;
            }
        }
        svg.push_str("</svg>\n");
        Ok(svg)
    }
}
