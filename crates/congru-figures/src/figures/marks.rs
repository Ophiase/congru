use congru::{
    Distance, Fraction, Icp, Kernel, Point, PointCloud, RigidTransform, Target, fit_rigid,
};
use congru_synth::{Corruption, MotionBounds, Scenario, shapes};

use congru_figures::{Figure, Ink, View};

const NOISE: f64 = 0.01;
const SURROUNDINGS: f64 = 0.45;

pub fn figure() -> anyhow::Result<String> {
    let scenario = marks()?;
    let truth = scenario.observed.transformed(&scenario.expected());
    let target = Target::from_mesh(&shapes::lumpy_sphere_mesh(64)?, Distance::new(0.01)?)?;
    let icp = Icp {
        kernel: Kernel::Huber(Distance::new(NOISE)?),
        ..Icp::default()
    };
    let alignment = icp.align(&scenario.observed, &target, &RigidTransform::identity())?;
    let slid = alignment
        .transform
        .rotation_error_to(&scenario.expected())
        .to_degrees();
    let free = alignment
        .observability
        .as_ref()
        .map_or(0, |report| report.weak_motions.len());
    let paired = fit_rigid(scenario.observed.points(), truth.points(), None)?;
    let nearby = surroundings(&truth)?;
    let mut figure = Figure::new(View::facing(&nearby, &truth.centroid().coords));
    for (caption, marks) in [
        (
            "a dozen marks, rough first pose".to_owned(),
            scenario.observed.clone(),
        ),
        (
            format!("ICP: {slid:.0}° off, {free} free motions flagged"),
            scenario.observed.transformed(&alignment.transform),
        ),
        (
            "numbered marks: closed form".to_owned(),
            scenario.observed.transformed(&paired.transform),
        ),
    ] {
        figure
            .panel(&caption)
            .dots(&nearby, Ink::Faint)
            .crosses(truth.points(), Ink::Model)
            .dots(marks.points(), Ink::Observed);
    }
    Ok(figure.render()?)
}

fn marks() -> anyhow::Result<Scenario> {
    let corruption = Corruption {
        noise: NOISE / 2.0,
        visible: Fraction::new(0.03)?,
        ..Corruption::default()
    };
    let rough = MotionBounds {
        max_angle: 0.25,
        max_translation: 0.05,
    };
    Ok(Scenario::generate(
        &shapes::lumpy_sphere(400)?,
        &corruption,
        &rough,
        2,
    )?)
}

fn surroundings(marks: &PointCloud) -> anyhow::Result<Vec<Point>> {
    let centre = marks.centroid();
    Ok(shapes::lumpy_sphere(6000)?
        .points()
        .iter()
        .filter(|point| (*point - centre).norm() < SURROUNDINGS)
        .copied()
        .collect())
}
