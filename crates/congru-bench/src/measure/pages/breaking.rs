//! How far a rigid answer holds when the object is not quite the model: a wrong
//! scale, or a part that bends. A curve per trouble, against the known truth.

use std::fmt::Write;
use std::path::Path;

use congru::{
    Distance, Icp, NearestNeighbors, Point, PointCloud, RigidTransform, Target, TriangleMesh,
    Vector, fit_similarity,
};
use congru_sim::{bracket, flexed};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

const SEEDS: u64 = 16;
const TARGET_SPACING: f64 = 0.003;
const SCAN_SPACING: f64 = 0.0047;
const NOISE: f64 = 0.001;
const NORMALS: usize = 10;
const GUESS_ANGLE: f64 = 0.17;
const GATE: f64 = 0.02;
const SCALED_STEPS: usize = 40;
/// The frame of a flexed part at which its swing is widest.
const WIDEST_FRAME: usize = 10;
const SCALE_ERRORS: [f64; 6] = [0.0, 0.005, 0.01, 0.02, 0.05, 0.1];
const FLEXES: [f64; 5] = [0.001, 0.002, 0.004, 0.008, 0.016];

const INTRO: &str = "# When the object is not quite the model\n\n\
Written by `just bench-breaking`: a bracket scanned with 1 mm of noise, from a guess\n\
10° and 1 cm off, median of 16 seeds.\n\n\
- **Rigid**: ICP, the only motion it knows.\n\
- **With a scale**: nearest points and a\n  \
[similarity](https://doi.org/10.1109/34.88573), in turn.\n\
- **Centre off**: how far the object's centre, which a scale leaves in place, lands\n  \
from where it should.\n\n\
| The object | Rigid, ° | Rigid, centre off, mm | With a scale, ° | Centre off, mm | Scale found |\n\
| --- | ---: | ---: | ---: | ---: | ---: |\n";

struct Outcome {
    degrees: f64,
    centre_mm: f64,
    scale: f64,
}

/// Writes `page`, one line per trouble.
///
/// # Errors
/// A mesh that cannot be sampled, or a page that cannot be written.
pub fn run(page: &Path) -> anyhow::Result<()> {
    let model = bracket()?;
    let span = spread(model.vertices());
    let centre = model.vertices().centroid();
    let target = Target::with_estimated_normals(sampled(&model, TARGET_SPACING)?, NORMALS)?;
    let mut text = String::from(INTRO);
    for error in SCALE_ERRORS {
        let larger: Vec<Point> = model
            .vertices()
            .points()
            .iter()
            .map(|point| centre + (point - centre) * (1.0 + error))
            .collect();
        let scaled = TriangleMesh::new(PointCloud::new(larger)?, model.triangles().to_vec())?;
        let name = format!("{:.1} % larger", 100.0 * error);
        text += &line(&name, &scaled, &target)?;
    }
    for flex in FLEXES {
        let bent = flexed(&model, flex, span, WIDEST_FRAME)?;
        let name = format!("bent by {:.0} mm", 1e3 * flex);
        text += &line(&name, &bent, &target)?;
    }
    std::fs::write(page, text)?;
    Ok(())
}

fn sampled(mesh: &TriangleMesh, spacing: f64) -> anyhow::Result<PointCloud> {
    Ok(mesh
        .sample_surface(Distance::new(spacing)?)?
        .cloud()
        .clone())
}

fn spread(cloud: &PointCloud) -> f64 {
    let centre = cloud.centroid();
    2.0 * cloud
        .points()
        .iter()
        .map(|point| (point - centre).norm())
        .fold(0.0, f64::max)
}

fn line(name: &str, seen: &TriangleMesh, target: &Target) -> anyhow::Result<String> {
    let scan = sampled(seen, SCAN_SPACING)?;
    let index = NearestNeighbors::new(target.cloud())?;
    let (mut rigid, mut scaled) = (Vec::new(), Vec::new());
    for seed in 0..SEEDS {
        let mut random = ChaCha8Rng::seed_from_u64(seed);
        let truth = RigidTransform::from_axis_angle(drawn(&mut random, 1.2), [0.02, -0.01, 0.03]);
        let moved = truth.inverse();
        let noisy: Vec<Point> = scan
            .points()
            .iter()
            .map(|point| moved.apply(point) + drawn(&mut random, 2.0 * NOISE))
            .collect();
        let source = PointCloud::new(noisy)?;
        let guess = RigidTransform::from_axis_angle(Vector::x() * GUESS_ANGLE, [0.01, 0.0, 0.0])
            .then(&truth);
        let found = Icp::default().align(&source, target, &guess)?.transform;
        let centre = moved.apply(&seen.vertices().centroid());
        rigid.push(Outcome {
            degrees: found.rotation_error_to(&truth).to_degrees(),
            centre_mm: 1e3 * (found.apply(&centre) - truth.apply(&centre)).norm(),
            scale: 1.0,
        });
        scaled.push(with_scale(
            &source, target, &index, &guess, &truth, &centre,
        )?);
    }
    let (rigid, scaled) = (median(&rigid), median(&scaled));
    let mut row = String::new();
    writeln!(
        row,
        "| {name} | {:.2} | {:.1} | {:.2} | {:.1} | {:.3} |",
        rigid.degrees, rigid.centre_mm, scaled.degrees, scaled.centre_mm, scaled.scale
    )?;
    Ok(row)
}

/// Nearest points and a similarity, in turn. The scan's scale is the unknown here: the
/// answer says what factor brings it onto the model.
fn with_scale(
    source: &PointCloud,
    target: &Target,
    index: &NearestNeighbors,
    guess: &RigidTransform,
    truth: &RigidTransform,
    centre: &Point,
) -> anyhow::Result<Outcome> {
    let (mut pose, mut scale) = (*guess, 1.0);
    let place = |pose: &RigidTransform, scale: f64, point: &Point| -> Point {
        Point::from(scale * (pose.rotation() * point.coords) + pose.translation())
    };
    for _ in 0..SCALED_STEPS {
        let (from, to): (Vec<Point>, Vec<Point>) = source
            .points()
            .iter()
            .filter_map(|point| {
                let found = index.nearest(&place(&pose, scale, point));
                (found.squared_distance < GATE * GATE)
                    .then(|| (*point, target.cloud().points()[found.index]))
            })
            .unzip();
        let fit = fit_similarity(&from, &to)?;
        (pose, scale) = (fit.transform, fit.scale);
    }
    Ok(Outcome {
        degrees: pose.rotation_error_to(truth).to_degrees(),
        centre_mm: 1e3 * (place(&pose, scale, centre) - truth.apply(centre)).norm(),
        scale,
    })
}

fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
    Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
}

fn median(outcomes: &[Outcome]) -> Outcome {
    let middle = |pick: fn(&Outcome) -> f64| {
        let mut values: Vec<f64> = outcomes.iter().map(pick).collect();
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    Outcome {
        degrees: middle(|one| one.degrees),
        centre_mm: middle(|one| one.centre_mm),
        scale: middle(|one| one.scale),
    }
}
