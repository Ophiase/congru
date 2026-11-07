use congru::{Metric, Point, RigidTransform, Vector};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

pub const SEEDS: u64 = 16;
pub const GATE: f64 = 0.03;
pub const KERNEL_SCALE: f64 = 0.005;
pub const ITERATIONS: u32 = 50;
const STRAY_REACH: f64 = 0.2;

/// How far a frame is from what the aligner expects.
pub struct Situation {
    pub name: &'static str,
    noise: f64,
    strays_percent: usize,
    half_seen: bool,
    guess_degrees: f64,
}

pub const SITUATIONS: [Situation; 5] = [
    Situation {
        name: "a guess 10° off",
        noise: 0.001,
        strays_percent: 0,
        half_seen: false,
        guess_degrees: 10.0,
    },
    Situation {
        name: "5 mm of noise",
        noise: 0.005,
        strays_percent: 0,
        half_seen: false,
        guess_degrees: 10.0,
    },
    Situation {
        name: "30 % strays",
        noise: 0.001,
        strays_percent: 30,
        half_seen: false,
        guess_degrees: 10.0,
    },
    Situation {
        name: "half of it seen",
        noise: 0.001,
        strays_percent: 0,
        half_seen: true,
        guess_degrees: 10.0,
    },
    Situation {
        name: "all of them, 20° off",
        noise: 0.005,
        strays_percent: 30,
        half_seen: true,
        guess_degrees: 20.0,
    },
];

/// Which aligner is run, the same in each implementation.
#[derive(Clone, Copy)]
pub struct Setup {
    pub name: &'static str,
    pub metric: Metric,
    pub robust: bool,
}

pub const SETUPS: [Setup; 3] = [
    Setup {
        name: "point-to-plane",
        metric: Metric::PointToPlane,
        robust: false,
    },
    Setup {
        name: "point-to-plane, Cauchy",
        metric: Metric::PointToPlane,
        robust: true,
    },
    Setup {
        name: "point-to-point",
        metric: Metric::PointToPoint,
        robust: false,
    },
];

/// How finely the bracket is sampled.
pub struct Scale {
    pub name: &'static str,
    pub target_spacing: f64,
    pub source_spacing: f64,
    pub setups: &'static [usize],
}

pub const SCALES: [Scale; 2] = [
    Scale {
        name: "as in the comparison",
        target_spacing: 0.003,
        source_spacing: 0.0047,
        setups: &[0, 1, 2],
    },
    Scale {
        name: "on a board's budget",
        target_spacing: 0.012,
        source_spacing: 0.02,
        setups: &[0, 1],
    },
];

/// The frame of the main comparison: the scan pulled back by a known motion,
/// noise on each point, strays around, a guess tilted away from the truth.
pub fn frame(
    scan: &congru::PointCloud,
    situation: &Situation,
    seed: u64,
) -> (Vec<Point>, RigidTransform, RigidTransform) {
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
    (points, truth, guess)
}

fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
    Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
}
