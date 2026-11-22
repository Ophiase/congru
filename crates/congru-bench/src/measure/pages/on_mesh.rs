//! A model held four ways, each aligned on: what a mesh gives over its samples, and
//! what reading a distance field costs. Against the known truth.

use std::fmt::Write;
use std::path::Path;
use std::time::Instant;

use congru::{
    Convergence, Distance, DistanceField, Icp, MeshIndex, Point, PointCloud, RigidTransform,
    Target, TriangleMesh, Vector,
};
use congru_sim::bracket;
use congru_vision::pixel::coordinate;
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

const SEEDS: u64 = 16;
const SCAN_SPACING: f64 = 0.0047;
const GATE: f64 = 0.03;
const BAND: f64 = 0.035;
const NOISE: f64 = 0.001;
const GUESS_ANGLE: f64 = 0.17;
const COARSE_SPACING: f64 = 0.012;
const FINE_SPACING: f64 = 0.003;
const FIELD_VOXEL: f64 = 0.002;
const INTRO: &str = "# Aligned on what the model is\n\n\
Written by `just bench-on-mesh`, [why](../../ideas/method/representations.md). A\n\
bracket scanned with about 1 mm of noise, from a guess 10° off, 16 seeds, one thread.\n\n\
- **Error**: the mean distance of the scan's points to where they truly belong.\n\
- **Time**: one alignment, with what it needs already built.\n\
- **Built in**: making that once, per model.\n\n\
| Model held as | Error, mm | Time, ms | Built in, ms | Bytes |\n\
| --- | ---: | ---: | ---: | ---: |\n";

/// Measures one way of holding the model, with what it took to build.
struct Row {
    name: String,
    error_mm: f64,
    milliseconds: f64,
    build_ms: f64,
    bytes: usize,
}

/// Writes `page`, one line per way of holding the model.
///
/// # Errors
/// A mesh that cannot be sampled, a failed alignment, or a page that cannot be written.
pub fn run(page: &Path) -> anyhow::Result<()> {
    let mesh = bracket()?;
    let frames = frames(&mesh)?;
    let icp = Icp {
        max_correspondence_distance: Some(Distance::new(GATE)?),
        convergence: Convergence {
            rotation: 1e-5,
            translation: 1e-5,
            ..Convergence::default()
        },
        ..Icp::default()
    };
    let mut text = String::from(INTRO);
    for row in [
        sampled(&mesh, COARSE_SPACING, &frames, &icp)?,
        sampled(&mesh, FINE_SPACING, &frames, &icp)?,
        exact(&mesh, &frames, &icp)?,
        field(&mesh, &frames, &icp)?,
    ] {
        writeln!(
            text,
            "| {} | {:.3} | {:.2} | {:.0} | {} |",
            row.name, row.error_mm, row.milliseconds, row.build_ms, row.bytes
        )?;
    }
    std::fs::write(page, text)?;
    Ok(())
}

fn sampled(mesh: &TriangleMesh, spacing: f64, frames: &[Frame], icp: &Icp) -> anyhow::Result<Row> {
    let started = Instant::now();
    let target = Target::from_mesh(mesh, Distance::new(spacing)?)?;
    let build_ms = 1e3 * started.elapsed().as_secs_f64();
    let bytes = target.cloud().count() * 6 * size_of::<f64>();
    let (error_mm, milliseconds) = scored(frames, |frame| {
        Ok(icp.align(&frame.source, &target, &frame.guess)?.transform)
    })?;
    Ok(Row {
        name: format!("Samples every {:.0} mm", 1e3 * spacing),
        error_mm,
        milliseconds,
        build_ms,
        bytes,
    })
}

fn exact(mesh: &TriangleMesh, frames: &[Frame], icp: &Icp) -> anyhow::Result<Row> {
    let started = Instant::now();
    let index = MeshIndex::new(mesh)?;
    let build_ms = 1e3 * started.elapsed().as_secs_f64();
    let (error_mm, milliseconds) = scored(frames, |frame| {
        Ok(icp.align_on(&frame.source, &index, &frame.guess)?.transform)
    })?;
    Ok(Row {
        name: "The mesh, exact".to_owned(),
        error_mm,
        milliseconds,
        build_ms,
        bytes: mesh.triangles().len() * 9 * size_of::<f64>(),
    })
}

fn field(mesh: &TriangleMesh, frames: &[Frame], icp: &Icp) -> anyhow::Result<Row> {
    let started = Instant::now();
    let field = DistanceField::from_mesh(mesh, Distance::new(FIELD_VOXEL)?, Distance::new(BAND)?)?;
    let build_ms = 1e3 * started.elapsed().as_secs_f64();
    let (error_mm, milliseconds) = scored(frames, |frame| {
        Ok(icp.align_on(&frame.source, &field, &frame.guess)?.transform)
    })?;
    Ok(Row {
        name: format!("A distance field, {:.0} mm", 1e3 * FIELD_VOXEL),
        error_mm,
        milliseconds,
        build_ms,
        bytes: field.bytes(),
    })
}

struct Frame {
    source: PointCloud,
    truth: RigidTransform,
    guess: RigidTransform,
}

fn frames(mesh: &TriangleMesh) -> anyhow::Result<Vec<Frame>> {
    let scan = mesh.sample_surface(Distance::new(SCAN_SPACING)?)?;
    (0..SEEDS)
        .map(|seed| {
            let mut random = ChaCha8Rng::seed_from_u64(seed);
            let truth =
                RigidTransform::from_axis_angle(drawn(&mut random, 1.5), [0.02, -0.01, 0.03]);
            let moved: Vec<Point> = scan
                .cloud()
                .points()
                .iter()
                .map(|point| truth.inverse().apply(point) + drawn(&mut random, 2.0 * NOISE))
                .collect();
            let tilt = drawn(&mut random, 1.0).normalize() * GUESS_ANGLE;
            Ok(Frame {
                source: PointCloud::new(moved)?,
                guess: RigidTransform::from_axis_angle(tilt, [0.005, 0.0, 0.0]).then(&truth),
                truth,
            })
        })
        .collect()
}

fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
    Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
}

/// The median error in mm and time in ms of `align` over `frames`.
fn scored(
    frames: &[Frame],
    align: impl Fn(&Frame) -> anyhow::Result<RigidTransform>,
) -> anyhow::Result<(f64, f64)> {
    let mut errors = Vec::new();
    let mut times = Vec::new();
    for frame in frames {
        let started = Instant::now();
        let found = align(frame)?;
        times.push(1e3 * started.elapsed().as_secs_f64());
        errors.push(1e3 * mean_error(&frame.source, &found, &frame.truth));
    }
    Ok((median(errors), median(times)))
}

fn mean_error(source: &PointCloud, found: &RigidTransform, truth: &RigidTransform) -> f64 {
    let total: f64 = source
        .points()
        .iter()
        .map(|point| (found.apply(point) - truth.apply(point)).norm())
        .sum();
    total / coordinate(source.count())
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
