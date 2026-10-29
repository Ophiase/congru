use congru::{Convergence, Icp, Metric, PointCloud, RigidTransform, Target};
use congru_synth::Corruption;

use super::toy::{nearest, observe, spiral, together};
use congru_figures::{Figure, Ink, View};

const TURN_DEGREES: f64 = 35.0;
const NOISE: f64 = 0.08;

pub fn figure() -> anyhow::Result<String> {
    let model = spiral()?;
    let corruption = Corruption {
        noise: NOISE,
        ..Corruption::default()
    };
    let observed = observe(&model, &corruption, TURN_DEGREES, 3)?;
    let target = Target::new(model.clone())?;
    let first = after(&observed, &target, 1)?;
    let settled = after(&observed, &target, 100)?;
    let everything = together(&model, &observed);
    let mut figure = Figure::new(View::from_above(&everything));
    figure
        .panel("1. the observation, moved")
        .crosses(model.points(), Ink::Model)
        .dots(observed.points(), Ink::Observed);
    for (caption, cloud) in [
        ("2. pair each dot with its nearest cross", &observed),
        ("3. move by the best fit, pair again", &first),
    ] {
        figure
            .panel(caption)
            .crosses(model.points(), Ink::Model)
            .dots(cloud.points(), Ink::Observed)
            .links(cloud.points(), &nearest(cloud.points(), &model)?);
    }
    figure
        .panel("4. repeat until nothing moves")
        .crosses(model.points(), Ink::Model)
        .dots(settled.points(), Ink::Observed);
    Ok(figure.render()?)
}

fn after(observed: &PointCloud, target: &Target, iterations: u32) -> anyhow::Result<PointCloud> {
    let icp = Icp {
        metric: Metric::PointToPoint,
        convergence: Convergence {
            max_iterations: iterations,
            ..Convergence::default()
        },
        ..Icp::default()
    };
    let alignment = icp.align(observed, target, &RigidTransform::identity())?;
    Ok(observed.transformed(&alignment.transform))
}
