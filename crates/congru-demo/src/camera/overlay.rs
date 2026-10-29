use congru::RigidTransform;
use congru_vision::{DepthImage, Image};

use super::conditions::Clutter;
use super::stage::Stage;
use crate::picture::outlined;
use crate::picture::palette::Tone;

/// What a panel shows: the degraded depth as a relief, the part's outline
/// at the estimated pose over it.
pub fn panel(stage: &Stage, depth: &DepthImage, estimate: Option<&RigidTransform>) -> Image<Tone> {
    let silhouette = estimate.map(|pose| {
        stage
            .shoot(&stage.part, pose, Clutter::Nothing)
            .depth
            .map(f64::is_finite)
    });
    outlined(depth, stage.camera.focal, silhouette.as_ref())
}
