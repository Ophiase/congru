use recalage::{Icp, Metric, Multistart, RigidTransform, Target};
use recalage_synth::Corruption;

use super::toy::{observe, spiral, together};
use crate::draw::{Figure, Ink, View};

const TURN_DEGREES: f64 = 150.0;

pub fn figure() -> anyhow::Result<String> {
    let model = spiral()?;
    let corruption = Corruption {
        noise: 0.08,
        ..Corruption::default()
    };
    let observed = observe(&model, &corruption, TURN_DEGREES, 5)?;
    let target = Target::new(model.clone())?;
    let icp = Icp {
        metric: Metric::PointToPoint,
        ..Icp::default()
    };
    let stuck = icp.align(&observed, &target, &RigidTransform::identity())?;
    let found = Multistart { starts: 64, icp }.align(&observed, &target)?;
    let stuck = observed.transformed(&stuck.transform);
    let found = observed.transformed(&found.transform);
    let everything = together(&model, &observed);
    let mut figure = Figure::new(View::from_above(&everything));
    figure
        .panel("turned by 150°")
        .crosses(model.points(), Ink::Model)
        .dots(observed.points(), Ink::Observed);
    figure
        .panel("ICP alone: a wrong local minimum")
        .crosses(model.points(), Ink::Model)
        .dots(stuck.points(), Ink::Observed);
    figure
        .panel("ICP from 64 rotations: the best one")
        .crosses(model.points(), Ink::Model)
        .dots(found.points(), Ink::Observed);
    Ok(figure.render()?)
}
