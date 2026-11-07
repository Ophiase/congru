//! Frames written to disk with congru's own answers, for the other libraries to
//! run on the same data: `ideas/experiments/comparison` reads them.

use std::fmt::Write;
use std::path::Path;

use congru::{
    Convergence, Distance, DistanceField, Icp, Metric, Point, PointCloud, RigidTransform, Target,
    Vector,
};
use congru_sim::bracket;
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

const SEEDS: u64 = 16;
const TARGET_SPACING: f64 = 0.003;
const SOURCE_SPACING: f64 = 0.0047;
const GATE: f64 = 0.03;
const ITERATIONS: u32 = 50;
const STRAY_REACH: f64 = 0.2;
const HEADER: &str =
    "situation,seed,seconds,guess,truth,congru,early_seconds,early,field_seconds,field\n";
const FIELD_VOXEL: f64 = 0.002;
const FIELD_BAND: f64 = 0.035;
/// The stopping steps of the fastest library's defaults, for a column on equal terms.
const EARLY_ROTATION: f64 = 0.001_745;
const EARLY_TRANSLATION: f64 = 0.001;

/// One way of making a frame hard.
struct Situation {
    name: &'static str,
    noise: f64,
    strays_percent: usize,
    half_seen: bool,
    guess_degrees: f64,
}

const BASE: Situation = Situation {
    name: "a_guess_10°_off",
    noise: 0.001,
    strays_percent: 0,
    half_seen: false,
    guess_degrees: 10.0,
};

const SITUATIONS: [Situation; 6] = [
    BASE,
    Situation {
        name: "a_guess_30°_off",
        guess_degrees: 30.0,
        ..BASE
    },
    Situation {
        name: "5_mm_of_noise",
        noise: 0.005,
        ..BASE
    },
    Situation {
        name: "30_%_strays",
        strays_percent: 30,
        ..BASE
    },
    Situation {
        name: "half_of_it_seen",
        half_seen: true,
        ..BASE
    },
    Situation {
        name: "all_of_them",
        noise: 0.005,
        strays_percent: 30,
        half_seen: true,
        guess_degrees: 20.0,
    },
];

/// Writes `target.csv`, one `source-*.csv` per trial and `trials.csv` into `folder`.
///
/// # Errors
/// A mesh that cannot be sampled, a failed alignment, or a file that cannot be written.
pub fn run(folder: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(folder)?;
    let model = bracket()?;
    let target = Target::from_oriented(model.sample_surface(Distance::new(TARGET_SPACING)?)?)?;
    write_target(&folder.join("target.csv"), &target)?;
    let scan = model
        .sample_surface(Distance::new(SOURCE_SPACING)?)?
        .cloud()
        .clone();
    let icp = aligner(Convergence::default())?;
    let early = early_aligner()?;
    let (voxel, band) = (Distance::new(FIELD_VOXEL)?, Distance::new(FIELD_BAND)?);
    let field = DistanceField::from_mesh(&model, voxel, band)?;
    let mut trials = String::from(HEADER);
    for situation in &SITUATIONS {
        for seed in 0..SEEDS {
            let (source, truth, guess) = frame(&scan, situation, seed)?;
            let file = format!("source-{}-{seed}.csv", situation.name);
            write_points(&folder.join(file), &source)?;
            let (seconds, found) = timed(|| icp.align(&source, &target, &guess))?;
            let (early_seconds, stopped) = timed(|| early.align(&source, &target, &guess))?;
            let (field_seconds, read) = timed(|| early.align_on(&source, &field, &guess))?;
            writeln!(
                trials,
                "{},{seed},{seconds},{},{},{},{early_seconds},{},{field_seconds},{}",
                situation.name,
                row(&guess),
                row(&truth),
                row(&found),
                row(&stopped),
                row(&read)
            )?;
        }
    }
    std::fs::write(folder.join("trials.csv"), trials)?;
    Ok(())
}

fn early_aligner() -> anyhow::Result<Icp> {
    aligner(Convergence {
        rotation: EARLY_ROTATION,
        translation: EARLY_TRANSLATION,
        ..Convergence::default()
    })
}

fn timed(
    align: impl Fn() -> Result<congru::Alignment, congru::Error>,
) -> anyhow::Result<(f64, RigidTransform)> {
    let started = std::time::Instant::now();
    let found = align()?.transform;
    Ok((started.elapsed().as_secs_f64(), found))
}

fn aligner(convergence: Convergence) -> anyhow::Result<Icp> {
    Ok(Icp {
        metric: Metric::PointToPlane,
        max_correspondence_distance: Some(Distance::new(GATE)?),
        convergence: Convergence {
            max_iterations: ITERATIONS,
            ..convergence
        },
        ..Icp::default()
    })
}

fn frame(
    scan: &PointCloud,
    situation: &Situation,
    seed: u64,
) -> anyhow::Result<(PointCloud, RigidTransform, RigidTransform)> {
    let mut random = ChaCha8Rng::seed_from_u64(seed);
    let truth = RigidTransform::from_axis_angle(drawn(&mut random, 1.5), [0.02, -0.01, 0.03]);
    let back = truth.inverse();
    let centre = scan.centroid();
    let mut points: Vec<Point> = scan
        .points()
        .iter()
        .filter(|point| !situation.half_seen || point.x <= centre.x)
        .map(|point| back.apply(point) + drawn(&mut random, 2.0 * situation.noise))
        .collect();
    for _ in 0..points.len() * situation.strays_percent / 100 {
        points.push(back.apply(&(centre + drawn(&mut random, STRAY_REACH))));
    }
    let tilt = drawn(&mut random, 1.0).normalize() * situation.guess_degrees.to_radians();
    let guess = RigidTransform::from_axis_angle(tilt, [0.01, 0.0, 0.0]).then(&truth);
    Ok((PointCloud::new(points)?, truth, guess))
}

fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
    Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
}

fn row(pose: &RigidTransform) -> String {
    pose.to_row_major().map(|entry| entry.to_string()).join(" ")
}

fn write_points(path: &Path, cloud: &PointCloud) -> anyhow::Result<()> {
    let mut text = String::new();
    for point in cloud.points() {
        writeln!(text, "{},{},{}", point.x, point.y, point.z)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

fn write_target(path: &Path, target: &Target) -> anyhow::Result<()> {
    let normals = target
        .normals()
        .ok_or_else(|| anyhow::anyhow!("a sampled surface has normals"))?;
    let mut text = String::new();
    for (point, normal) in target.cloud().points().iter().zip(normals) {
        writeln!(
            text,
            "{},{},{},{},{},{}",
            point.x, point.y, point.z, normal.x, normal.y, normal.z
        )?;
    }
    std::fs::write(path, text)?;
    Ok(())
}
