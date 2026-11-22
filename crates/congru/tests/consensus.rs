use congru::{Aligner, Consensus, Distance, Fraction, Frame, Icp};
use congru_synth::{Corruption, MotionBounds, Scenario};

const SEEDS: u64 = 4;
const NOISE: f64 = 0.002;
const SPACING: f64 = 0.003;
const VOXEL: f64 = 0.008;
const TOLERANCE_DEGREES: f64 = 2.0;

#[test]
fn features_find_any_rotation_of_a_machined_part_seen_in_part() {
    let model = congru_sim::bracket()
        .unwrap()
        .sample_surface(Distance::new(SPACING).unwrap())
        .unwrap();
    let mut consensus = Consensus::new(
        "consensus",
        &model,
        Distance::new(VOXEL).unwrap(),
        48,
        Icp::default(),
    )
    .unwrap();
    let corruption = Corruption {
        noise: NOISE,
        visible: Fraction::new(0.6).unwrap(),
        ..Corruption::default()
    };
    for seed in 0..SEEDS {
        let scenario = Scenario::generate(
            model.cloud(),
            &corruption,
            &MotionBounds::any_rotation(0.2),
            seed,
        )
        .unwrap();
        let estimate = consensus
            .align(&Frame {
                scan: &scenario.observed,
                guess: None,
            })
            .unwrap();
        let error = estimate
            .pose
            .rotation_error_to(&scenario.truth)
            .to_degrees();
        assert!(error < TOLERANCE_DEGREES, "seed {seed}: {error}°");
    }
}
