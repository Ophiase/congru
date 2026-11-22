use congru::{Distance, Icp, Kernel, Metric, PointCloud, RigidTransform, Target};
use congru_synth::{Corruption, Outliers};

use super::toy::{observe, spiral, together};
use congru_figures::{Figure, Ink, View};

const TURN_DEGREES: f64 = 20.0;
const OUTLIERS: usize = 14;
const TUKEY_SCALE: f64 = 0.35;

pub fn figure() -> anyhow::Result<String> {
    let model = spiral()?;
    let observed = scene(&model)?;
    let target = Target::new(model.clone())?;
    let plain = solve(
        &observed,
        &target,
        Kernel::Quadratic,
        &RigidTransform::identity(),
    )?;
    let robust = solve(
        &observed,
        &target,
        Kernel::Tukey(Distance::new(TUKEY_SCALE)?),
        &plain,
    )?;
    let everything = together(&model, &observed);
    let mut figure = Figure::new(View::from_above(&everything));
    for (caption, cloud) in [
        ("observed, with outliers", observed.clone()),
        (
            "least squares: dragged by outliers",
            observed.transformed(&plain),
        ),
        (
            "then Tukey: outliers weigh nothing",
            observed.transformed(&robust),
        ),
    ] {
        let split = cloud.count() - OUTLIERS;
        figure
            .panel(caption)
            .crosses(model.points(), Ink::Model)
            .dots(&cloud.points()[..split], Ink::Observed)
            .dots(&cloud.points()[split..], Ink::Outlier);
    }
    Ok(figure.render()?)
}

fn solve(
    observed: &PointCloud,
    target: &Target,
    kernel: Kernel,
    start: &RigidTransform,
) -> anyhow::Result<RigidTransform> {
    let icp = Icp {
        metric: Metric::PointToPoint,
        kernel,
        ..Icp::default()
    };
    Ok(icp.align(observed, target, start)?.transform)
}

fn scene(model: &PointCloud) -> anyhow::Result<PointCloud> {
    let corruption = Corruption {
        noise: 0.05,
        outliers: Outliers::InBox { count: OUTLIERS },
        ..Corruption::default()
    };
    Ok(observe(model, &corruption, TURN_DEGREES, 11)?)
}
