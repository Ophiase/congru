//! Methods graded on a proved optimum: few marks on a part, whose correspondences
//! are unknown, the corner where a search can stop on a wrong answer unnoticed.

mod instance;
mod page;

use std::io::Write;
use std::path::Path;

use congru::{
    Certifier, Distance, Icp, Multistart, NearestNeighbors, PointCloud, RigidTransform, Target,
    Vector,
};
use instance::{Instance, instances};

const TRUNCATION: f64 = 0.015;
const REACH: f64 = 0.15;
/// Asked of the proof: a gap of this rms value, in the unit of the data.
const TOLERANCE: f64 = 0.001;
const FINE: [f64; 2] = [0.001, 0.0001];
const COARSE: [f64; 2] = [0.04, 0.004];
const BUDGET: usize = 3_000_000;
const SCOUT_BUDGET: usize = 1_000_000;
const NORMALS: usize = 10;
const GUESS_ANGLE: f64 = 0.35;

pub(super) struct Graded {
    pub(super) method: &'static str,
    /// In rms of the objective, not squared.
    pub(super) gap: f64,
    pub(super) turned: f64,
}

/// One instance, proved. `spread` is how far, in degrees, the poses not excluded
/// reach from the best one, and a lower limit unless `scout_closed`.
pub(super) struct Row {
    pub(super) name: String,
    pub(super) closed: bool,
    pub(super) optimum_turned: f64,
    pub(super) spread: f64,
    pub(super) scout_closed: bool,
    pub(super) seconds: f64,
    pub(super) graded: Vec<Graded>,
}

/// Grades every method on every instance, one line per instance, and writes `page`.
///
/// # Errors
/// A mesh that cannot be sampled, or a page that cannot be written.
pub fn run(page: &Path) -> anyhow::Result<()> {
    let rows = instances()?
        .iter()
        .map(|instance| {
            let row = proved(instance)?;
            writeln!(
                std::io::stdout().lock(),
                "{}: {:.1} s, closed {}",
                row.name,
                row.seconds,
                row.closed
            )?;
            Ok(row)
        })
        .collect::<anyhow::Result<Vec<Row>>>()?;
    std::fs::write(page, page::written(&rows))?;
    Ok(())
}

fn methods(instance: &Instance) -> anyhow::Result<Vec<(&'static str, RigidTransform)>> {
    let target = Target::with_estimated_normals(instance.target.clone(), NORMALS)?;
    let guess = RigidTransform::from_axis_angle(Vector::x() * GUESS_ANGLE, [0.01, 0.0, 0.0])
        .then(&instance.truth);
    let icp = Icp::default();
    Ok(vec![
        (
            "ICP, a guess 20° off",
            icp.align(&instance.source, &target, &guess)?.transform,
        ),
        (
            "ICP, no guess",
            icp.align(&instance.source, &target, &RigidTransform::identity())?
                .transform,
        ),
        (
            "Multistart, 64 starts",
            Multistart::default()
                .align(&instance.source, &target)?
                .transform,
        ),
    ])
}

fn proved(instance: &Instance) -> anyhow::Result<Row> {
    let started = std::time::Instant::now();
    let found = methods(instance)?;
    let seeds: Vec<RigidTransform> = found.iter().map(|(_, pose)| *pose).collect();
    let certifier = Certifier {
        truncation: Distance::new(TRUNCATION)?,
        reach: Distance::new(REACH)?,
        margin: 0.0,
        resolution: COARSE,
        budget: SCOUT_BUDGET,
    };
    let scout = certifier.run(&instance.source, &instance.target, &seeds)?;
    let bound = Certifier {
        margin: margin_for(scout.upper),
        resolution: FINE,
        budget: BUDGET,
        ..certifier
    }
    .run(&instance.source, &instance.target, &[scout.pose])?;
    let index = NearestNeighbors::new(&instance.target)?;
    let graded = found
        .iter()
        .map(|(method, pose)| Graded {
            method,
            gap: cost(&instance.source, &index, pose).sqrt() - bound.lower.max(0.0).sqrt(),
            turned: pose.rotation_error_to(&instance.truth).to_degrees(),
        })
        .collect();
    Ok(Row {
        name: instance.name.clone(),
        closed: bound.closed,
        optimum_turned: bound.pose.rotation_error_to(&instance.truth).to_degrees(),
        spread: scout
            .survivors
            .iter()
            .map(|pose| pose.rotation_error_to(&scout.pose).to_degrees())
            .fold(0.0, f64::max),
        scout_closed: scout.closed,
        seconds: started.elapsed().as_secs_f64(),
        graded,
    })
}

/// The margin that proves an rms within [`TOLERANCE`] of `value`: the objective is
/// the square of the rms, so the same rms gap is wider in it the larger the value.
fn margin_for(value: f64) -> f64 {
    let floor = (value.sqrt() - TOLERANCE).max(0.0);
    floor * floor - value
}

/// The objective the certificate bounds.
fn cost(source: &PointCloud, index: &NearestNeighbors, pose: &RigidTransform) -> f64 {
    let total: f64 = source
        .points()
        .iter()
        .map(|point| {
            index
                .nearest(&pose.apply(point))
                .squared_distance
                .min(TRUNCATION * TRUNCATION)
        })
        .sum();
    total / congru_vision::pixel::coordinate(source.count())
}
