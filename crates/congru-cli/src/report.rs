use std::io::Write;

use congru::{MotionKind, Observability, Point, Vector, WeakMotion};
use serde::Serialize;

/// Which motions the final pairs left free, in the target's frame.
#[derive(Debug, Clone, Serialize)]
pub struct Observed {
    pub conditioning: f64,
    pub weak_motions: Vec<Weak>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Weak {
    Rotation {
        stiffness: f64,
        axis: [f64; 3],
        through: [f64; 3],
    },
    Translation {
        stiffness: f64,
        direction: [f64; 3],
    },
}

impl From<&Observability> for Observed {
    fn from(observability: &Observability) -> Self {
        Self {
            conditioning: observability.conditioning,
            weak_motions: observability.weak_motions.iter().map(weak).collect(),
        }
    }
}

fn weak(motion: &WeakMotion) -> Weak {
    match motion.kind {
        MotionKind::Rotation { axis, through } => Weak::Rotation {
            stiffness: motion.stiffness,
            axis: vector(&axis),
            through: point(&through),
        },
        MotionKind::Translation { direction } => Weak::Translation {
            stiffness: motion.stiffness,
            direction: vector(&direction),
        },
    }
}

#[must_use]
pub fn vector(vector: &Vector) -> [f64; 3] {
    [vector.x, vector.y, vector.z]
}

#[must_use]
pub fn point(point: &Point) -> [f64; 3] {
    vector(&point.coords)
}

/// One JSON object on its own line.
///
/// # Errors
/// When the output cannot be written.
pub fn emit(output: &mut dyn Write, value: &impl Serialize) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
    Ok(())
}
