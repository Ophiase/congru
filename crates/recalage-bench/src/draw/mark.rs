use std::fmt::{self, Write};

use super::view::Frame;

const CROSS_ARM: f64 = 4.5;
const DOT_RADIUS: f64 = 2.6;
const LABEL_OFFSET: f64 = 6.0;

/// The roles a point plays in a figure, each with its colour: the model as
/// red crosses, the observation as blue dots, outliers green, as in the
/// first drawings of the problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Model,
    Observed,
    Outlier,
    Faint,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mark {
    Cross { at: [f64; 2], ink: Ink },
    Dot { at: [f64; 2], ink: Ink },
    Link { start: [f64; 2], end: [f64; 2] },
    Label { at: [f64; 2], text: &'static str },
}

impl Ink {
    const fn class(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Observed => "observed",
            Self::Outlier => "outlier",
            Self::Faint => "faint",
        }
    }
}

impl Mark {
    pub fn write(&self, svg: &mut String, frame: &Frame, opacity: f64) -> fmt::Result {
        match *self {
            Self::Cross { at, ink } => {
                let [across, down] = frame.place(at);
                writeln!(
                    svg,
                    r#"<path class="{class} stroke" opacity="{opacity:.2}" d="M{left:.1} {top:.1}L{right:.1} {bottom:.1}M{left:.1} {bottom:.1}L{right:.1} {top:.1}"/>"#,
                    class = ink.class(),
                    left = across - CROSS_ARM,
                    right = across + CROSS_ARM,
                    top = down - CROSS_ARM,
                    bottom = down + CROSS_ARM,
                )
            }
            Self::Dot { at, ink } => {
                let [across, down] = frame.place(at);
                writeln!(
                    svg,
                    r#"<circle class="{class} fill" opacity="{opacity:.2}" cx="{across:.1}" cy="{down:.1}" r="{DOT_RADIUS}"/>"#,
                    class = ink.class(),
                )
            }
            Self::Label { at, text } => {
                let [across, down] = frame.place(at);
                writeln!(
                    svg,
                    r#"<text class="label" x="{x:.1}" y="{y:.1}">{text}</text>"#,
                    x = across + LABEL_OFFSET,
                    y = down + 2.0 * LABEL_OFFSET,
                )
            }
            Self::Link { start, end } => {
                let ([from_across, from_down], [to_across, to_down]) =
                    (frame.place(start), frame.place(end));
                writeln!(
                    svg,
                    r#"<line class="link" x1="{from_across:.1}" y1="{from_down:.1}" x2="{to_across:.1}" y2="{to_down:.1}"/>"#,
                )
            }
        }
    }
}
