use congru::{Distance, Point, PointCloud, RigidTransform, TriangleMesh, Vector};
use congru_sim::{bracket, gear};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

const SPACING: f64 = 0.006;
const SEEDS: u64 = 2;
const NOISES: [f64; 2] = [0.001, 0.003];
const MARKS: [usize; 2] = [12, 30];

/// Marks made from the target by `truth`, some of them strays.
pub(super) struct Instance {
    pub(super) name: String,
    pub(super) source: PointCloud,
    pub(super) target: PointCloud,
    pub(super) truth: RigidTransform,
}

pub(super) fn instances() -> anyhow::Result<Vec<Instance>> {
    let mut all = Vec::new();
    for (shape, mesh) in [("bracket", bracket()?), ("gear", gear()?)] {
        let target = points_of(&mesh)?;
        for marks in MARKS {
            for noise in NOISES {
                for strays in [0, marks / 4] {
                    for seed in 0..SEEDS {
                        let millimetres = noise * 1e3;
                        all.push(Instance {
                            name: format!(
                                "{shape}, {marks} marks, {millimetres:.0} mm, {strays} strays"
                            ),
                            ..drawn_instance(&target, (marks, strays), noise, seed)?
                        });
                    }
                }
            }
        }
    }
    Ok(all)
}

fn points_of(mesh: &TriangleMesh) -> anyhow::Result<PointCloud> {
    Ok(mesh
        .sample_surface(Distance::new(SPACING)?)?
        .cloud()
        .clone())
}

fn drawn_instance(
    target: &PointCloud,
    (marks, strays): (usize, usize),
    noise: f64,
    seed: u64,
) -> anyhow::Result<Instance> {
    let mut random = ChaCha8Rng::seed_from_u64(seed + 41 * u64::try_from(marks)?);
    let truth = RigidTransform::from_axis_angle(drawn(&mut random, 1.5), [0.01, -0.02, 0.015]);
    let back = truth.inverse();
    let points: Vec<Point> = (0..marks)
        .map(|index| {
            if index < strays {
                back.apply(&(target.centroid() + drawn(&mut random, 0.12)))
            } else {
                let at = random.random_range(0..target.count());
                back.apply(&target.points()[at]) + gaussian(&mut random, noise)
            }
        })
        .collect();
    Ok(Instance {
        name: String::new(),
        source: PointCloud::new(points)?,
        target: target.clone(),
        truth,
    })
}

fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
    Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
}

/// Normal draws, by Box and Muller.
fn gaussian(random: &mut ChaCha8Rng, deviation: f64) -> Vector {
    Vector::from_fn(|_, _| {
        let radius = (-2.0 * (1.0 - random.random::<f64>()).ln()).sqrt();
        deviation * radius * (std::f64::consts::TAU * random.random::<f64>()).cos()
    })
}
