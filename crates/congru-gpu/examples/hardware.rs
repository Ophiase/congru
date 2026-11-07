//! What asking "how far?" costs for a batch of points, on each kind of hardware:
//! writes `docs/benchmark/hardware.md`.

use std::fmt::Write;
use std::time::Instant;

use congru::{Distance, DistanceField, MeshIndex, NearestNeighbors, Point, Surface};
use congru_gpu::GpuField;
use congru_sim::bracket;
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

const SIZES: [usize; 4] = [1_000, 10_000, 100_000, 1_000_000];
const RUNS: usize = 9;
const INTRO: &str = "# What a batch of questions costs\n\n\
Written by `just bench-hardware`: how long to ask how far each point is, for a\n\
bracket of 20 cm. [Why it matters](../../ideas/architecture/acceleration.md).\n\n";

fn median_ms(mut work: impl FnMut()) -> f64 {
    let mut times = [0.0; RUNS];
    for time in &mut times {
        let started = Instant::now();
        work();
        *time = 1e3 * started.elapsed().as_secs_f64();
    }
    times.sort_by(f64::total_cmp);
    times[RUNS / 2]
}

fn near_the_surface(mesh: &congru::TriangleMesh, count: usize) -> anyhow::Result<Vec<Point>> {
    let samples = mesh.sample_surface(Distance::new(0.002)?)?;
    let mut random = ChaCha8Rng::seed_from_u64(1);
    let points = samples.cloud().points();
    Ok((0..count)
        .map(|_| {
            let at = random.random_range(0..points.len());
            points[at] + congru::Vector::from_fn(|_, _| 0.01 * (random.random::<f64>() * 2.0 - 1.0))
        })
        .collect())
}

fn main() -> anyhow::Result<()> {
    let mesh = bracket()?;
    let exact = MeshIndex::new(&mesh)?;
    let cloud = mesh.sample_surface(Distance::new(0.002)?)?.cloud().clone();
    let tree = NearestNeighbors::new(&cloud)?;
    let field = DistanceField::from_mesh(&mesh, Distance::new(0.002)?, Distance::new(0.035)?)?;
    let copy = DistanceField::from_mesh(&mesh, Distance::new(0.002)?, Distance::new(0.035)?)?;
    let gpu = GpuField::new(copy).ok();
    let device = gpu.as_ref().map_or("none", GpuField::device).to_owned();
    let mut page = String::from(INTRO);
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let crossover = gpu.as_ref().map_or(usize::MAX, GpuField::fewest_on_device);
    let choice = if crossover == usize::MAX {
        "stays on the CPU at every size".to_owned()
    } else {
        format!("goes to the device from {crossover} points")
    };
    writeln!(
        page,
        "The device here: {device}, with {threads} CPU threads.\n"
    )?;
    writeln!(
        page,
        "`GpuField` measures the machine when built; here, a batch\n{choice}.\n"
    )?;
    page += "| Points | Tree, all cores | Mesh, all cores | Field, one core | Field, all cores | Field, GPU |\n";
    page += "| ---: | ---: | ---: | ---: | ---: | ---: |\n";
    for size in SIZES {
        let queries = near_the_surface(&mesh, size)?;
        let trees = median_ms(|| {
            congru::map(&queries, |query| tree.nearest(query));
        });
        let meshes = median_ms(|| {
            exact.closest_all(&queries);
        });
        let one = median_ms(|| {
            for query in &queries {
                field.closest(query);
            }
        });
        let all = median_ms(|| {
            field.closest_all(&queries);
        });
        let device = gpu.as_ref().map_or(f64::NAN, |gpu| {
            median_ms(|| {
                gpu.closest_on_device(&queries);
            })
        });
        writeln!(
            page,
            "| {size} | {trees:.2} ms | {meshes:.2} ms | {one:.2} ms | {all:.2} ms | {device:.2} ms |"
        )?;
    }
    std::fs::write("docs/benchmark/hardware.md", page)?;
    Ok(())
}
