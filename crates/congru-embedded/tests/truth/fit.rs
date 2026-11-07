use congru_embedded::{Error, Pair, Real, Vector3, kabsch, similarity};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

use crate::fixtures::{cast, cube, errors, motion, moved, rotation_floor, translation_floor};

const POINTS: usize = 400;
const HALF: f64 = 0.5;

fn pairs_of<Float: Real>(source: &[[f64; 3]], target: &[[f64; 3]]) -> Vec<Pair<Float>> {
    source
        .iter()
        .zip(target)
        .map(|(from, to)| Pair {
            source: Vector3(cast(*from)),
            target: Vector3(cast(*to)),
            weight: Float::ONE,
        })
        .collect()
}

fn noiseless_at<Float: Real>(centre: [f64; 3]) {
    let truth = motion();
    let source = cube(POINTS, HALF, centre, 1);
    let target: Vec<_> = source.iter().map(|point| moved(&truth, *point)).collect();
    let pairs = pairs_of::<Float>(&source, &target);
    let found = kabsch(pairs.iter().copied()).unwrap();
    let (rotation, translation) = errors(&found, &truth);
    let magnitude = centre.iter().fold(HALF, |sum, entry| sum + entry.abs());
    let rotation_bound = rotation_floor::<Float>(magnitude, HALF);
    let translation_bound = translation_floor::<Float>(magnitude, HALF);
    assert!(rotation < rotation_bound, "{rotation} / {rotation_bound}");
    assert!(
        translation < translation_bound,
        "{translation} / {translation_bound}"
    );
}

#[test]
fn a_noiseless_motion_is_recovered_to_the_rounding_floor() {
    noiseless_at::<f32>([0.0; 3]);
    noiseless_at::<f64>([0.0; 3]);
}

#[test]
fn a_cloud_far_from_the_origin_loses_only_what_its_coordinates_lose() {
    noiseless_at::<f32>([40.0, -30.0, 20.0]);
    noiseless_at::<f64>([40.0, -30.0, 20.0]);
}

/// With uniform noise of half-width `h`, so `σ = h/√3` per axis, on a cube of
/// half-side `a`: the small rotation's variance per axis is `σ² / (n · 2a²/3)`
/// and the translation's `σ²/n`. Five standard deviations over three axes.
fn noisy<Float: Real>() {
    const NOISE: f64 = 0.01;
    let truth = motion();
    let source = cube(POINTS, HALF, [0.0; 3], 2);
    let mut random = ChaCha8Rng::seed_from_u64(3);
    let target: Vec<_> = source
        .iter()
        .map(|point| {
            moved(&truth, *point).map(|entry| entry + NOISE * (random.random::<f64>() * 2.0 - 1.0))
        })
        .collect();
    let pairs = pairs_of::<Float>(&source, &target);
    let found = kabsch(pairs.iter().copied()).unwrap();
    let (rotation, translation) = errors(&found, &truth);
    let sigma = NOISE / 3.0_f64.sqrt();
    let root = f64::of_count(POINTS).sqrt();
    let three_axes = 5.0 * 3.0_f64.sqrt();
    let rotation_bound =
        three_axes * sigma * 1.5_f64.sqrt() / (HALF * root) + rotation_floor::<Float>(HALF, HALF);
    let translation_bound = three_axes * sigma / root + translation_floor::<Float>(HALF, HALF);
    assert!(rotation < rotation_bound, "{rotation} / {rotation_bound}");
    assert!(
        translation < translation_bound,
        "{translation} / {translation_bound}"
    );
}

#[test]
fn a_noisy_motion_is_recovered_within_what_the_noise_allows() {
    noisy::<f32>();
    noisy::<f64>();
}

fn weightless_pairs_are_ignored<Float: Real>() {
    let truth = motion();
    let source = cube(POINTS, HALF, [0.0; 3], 4);
    let target: Vec<_> = source.iter().map(|point| moved(&truth, *point)).collect();
    let mut pairs = pairs_of::<Float>(&source, &target);
    let strays = cube(POINTS, 3.0, [1.0; 3], 5);
    let wrong = pairs_of::<Float>(&source, &strays);
    pairs.extend(wrong.into_iter().map(|pair| Pair {
        weight: Float::ZERO,
        ..pair
    }));
    let (rotation, translation) = errors(&kabsch(pairs.iter().copied()).unwrap(), &truth);
    assert!(rotation < rotation_floor::<Float>(HALF, HALF));
    assert!(translation < translation_floor::<Float>(HALF, HALF));
}

#[test]
fn pairs_of_zero_weight_change_nothing() {
    weightless_pairs_are_ignored::<f32>();
    weightless_pairs_are_ignored::<f64>();
}

fn scale_recovered<Float: Real>() {
    let truth = motion();
    let source = cube(POINTS, HALF, [0.0; 3], 6);
    for scale in [0.5, 0.98, 1.0, 1.05, 3.0] {
        let target: Vec<_> = source
            .iter()
            .map(|point| moved(&truth, point.map(|entry| entry * scale)))
            .collect();
        let pairs = pairs_of::<Float>(&source, &target);
        let found = similarity(pairs.iter().copied()).unwrap();
        let (rotation, translation) = errors(&found.pose, &truth);
        let floor = rotation_floor::<Float>(HALF, HALF);
        assert!((found.scale.as_f64() - scale).abs() < floor * scale);
        assert!(rotation < floor);
        assert!(translation < translation_floor::<Float>(HALF * scale, HALF * scale));
    }
}

#[test]
fn a_known_scale_and_motion_are_recovered() {
    scale_recovered::<f32>();
    scale_recovered::<f64>();
}

fn refuses<Float: Real>() {
    let line: Vec<[f64; 3]> = (0..8).map(|step| [f64::from(step), 0.0, 0.0]).collect();
    let pairs = pairs_of::<Float>(&line, &line);
    assert_eq!(kabsch(pairs.iter().copied()), Err(Error::Degenerate));
    assert_eq!(
        kabsch(pairs[..2].iter().copied()),
        Err(Error::TooFewPairs {
            found: 2,
            needed: 3
        })
    );
}

fn scaled_axis_matches_truth<Float: Real>() {
    for angle in [0.0, 1e-6, 1e-3, 0.2, 1.0, 2.5, 3.1] {
        let axis = congru::Vector::new(0.48, -0.6, 0.64) * angle;
        let truth = congru::RigidTransform::from_axis_angle(axis, [0.0; 3]);
        let found = congru_embedded::Pose {
            rotation: congru_embedded::UnitQuaternion::from_scaled_axis(Vector3(cast([
                axis.x, axis.y, axis.z,
            ]))),
            translation: Vector3::ZERO,
        };
        let (rotation, _) = errors::<Float>(&found, &truth);
        assert!(rotation < rotation_floor::<Float>(1.0, 1.0), "{angle}");
    }
}

#[test]
fn a_scaled_axis_gives_the_rotation_it_names() {
    scaled_axis_matches_truth::<f32>();
    scaled_axis_matches_truth::<f64>();
}

#[test]
fn collinear_or_too_few_pairs_are_refused() {
    refuses::<f32>();
    refuses::<f64>();
}
