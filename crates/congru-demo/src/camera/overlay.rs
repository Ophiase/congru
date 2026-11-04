use congru::RigidTransform;
use congru_vision::{DepthImage, Image};

use super::stage::{Stage, Step};
use crate::picture::palette::{EMPTY, Tone, hue};
use crate::picture::{outline, outlined, splat};

/// The three panels of a frame, each under the same outline of the part at
/// the estimated pose: the image, the depth, and the points tracked, coloured as the depth.
pub fn panels(stage: &Stage, step: &Step, estimate: Option<&RigidTransform>) -> [Image<Tone>; 3] {
    let silhouette = estimate.map(|pose| {
        stage
            .shoot(&stage.part, pose, &[])
            .depth
            .map(f64::is_finite)
    });
    let mut panels = [
        outlined(&step.image, stage.camera.focal, None),
        rainbow(&step.depth, &step.image),
        rainbow(&splat(&step.scan, &stage.camera), &step.image),
    ];
    if let Some(silhouette) = &silhouette {
        for panel in &mut panels {
            outline(panel, silhouette);
        }
    }
    panels
}

/// The depth coloured by distance over the range the scene spans, holes
/// left empty.
fn rainbow(depth: &DepthImage, scene: &DepthImage) -> Image<Tone> {
    let (near, far) = scene
        .positions()
        .map(|(column, row)| scene.get(column, row))
        .filter(|value| value.is_finite())
        .fold((f64::INFINITY, 0.0_f64), |(near, far), value| {
            (near.min(value), far.max(value))
        });
    depth.map(|value| {
        if value.is_finite() {
            hue((far - value) / (far - near).max(f64::EPSILON))
        } else {
            EMPTY
        }
    })
}
