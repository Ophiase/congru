#![expect(
    clippy::unwrap_used,
    reason = "a broken fixture should fail the test loudly"
)]

use congru::{RigidTransform, TriangleMesh, Vector};
use congru_sim::bracket;
use congru_track::{Answer, Follower, MotionModel, Settings, Sight};
use congru_vision::{Camera, DepthImage, render};

const SPAN: f64 = 0.25;
const HIDDEN: std::ops::Range<usize> = 10..14;
const KNOCK: usize = 10;
const KNOCK_RADIANS: f64 = 1.2;
const MOST_DEGREES: f64 = 5.0;
const MOST_METRES: f64 = 0.015;

fn camera() -> Camera {
    Camera::with_field_of_view(100, 75, 60.0)
}

fn truth(frame: usize) -> RigidTransform {
    let time = congru_vision::pixel::coordinate(frame);
    RigidTransform::from_axis_angle(
        Vector::new(-1.9, 0.4, 0.3 + 0.01 * time),
        [0.02, -0.01 + 0.002 * time, 0.6],
    )
}

fn depth_of(part: &TriangleMesh, pose: &RigidTransform) -> DepthImage {
    render(&camera(), &[(part, *pose)]).depth
}

fn step(follower: &mut Follower, depth: &DepthImage) -> Answer {
    follower.step(Sight {
        depth,
        region: depth,
    })
}

fn off(answer: &Answer, truth: &RigidTransform) -> (f64, f64) {
    answer.pose.map_or((f64::INFINITY, f64::INFINITY), |pose| {
        (
            pose.rotation_error_to(truth).to_degrees(),
            pose.translation_error_to(truth),
        )
    })
}

fn follower(part: &TriangleMesh) -> Follower {
    let settings = Settings::for_part(SPAN).unwrap();
    Follower::new(part, camera(), SPAN, settings).unwrap()
}

#[test]
fn a_hidden_object_is_held_along_its_trajectory() {
    let part = bracket().unwrap();
    let mut follower = follower(&part);
    for frame in 0..HIDDEN.end {
        let pose = truth(frame);
        let mut depth = depth_of(&part, &pose);
        if HIDDEN.contains(&frame) {
            let before = depth.map(|value| if value.is_finite() { 0.1 } else { value });
            depth = before;
        }
        let answer = step(&mut follower, &depth);
        let (degrees, metres) = off(&answer, &pose);
        assert!(
            degrees < MOST_DEGREES && metres < MOST_METRES,
            "frame {frame}: {degrees:.1}° and {metres:.3} m off"
        );
    }
}

#[test]
fn an_object_that_left_is_lost() {
    let part = bracket().unwrap();
    let mut follower = follower(&part);
    for frame in 0..5 {
        let depth = depth_of(&part, &truth(frame));
        assert!(step(&mut follower, &depth).pose.is_some());
    }
    let wall = DepthImage::filled(camera().width, camera().height, 2.0);
    let answers: Vec<Answer> = (0..30).map(|_| step(&mut follower, &wall)).collect();
    assert!(answers.iter().any(|answer| answer.pose.is_none()));
}

#[test]
fn a_knocked_object_is_found_again_within_a_few_frames() {
    let part = bracket().unwrap();
    let mut follower = follower(&part);
    let turn = RigidTransform::from_axis_angle(Vector::z() * KNOCK_RADIANS, [0.0; 3]);
    let posed = |frame| {
        if frame < KNOCK {
            truth(frame)
        } else {
            turn.then(&truth(frame))
        }
    };
    let mut last = (f64::INFINITY, f64::INFINITY);
    for frame in 0..KNOCK + 4 {
        let pose = posed(frame);
        last = off(&step(&mut follower, &depth_of(&part, &pose)), &pose);
    }
    assert!(
        last.0 < MOST_DEGREES && last.1 < MOST_METRES,
        "{:.1}° and {:.3} m off",
        last.0,
        last.1
    );
}

#[test]
fn a_part_at_rest_stays_forecast_where_it_is() {
    let part = bracket().unwrap();
    let settings = Settings::for_part(SPAN).unwrap();
    let motion: MotionModel = settings.motion.at_rest();
    let settings = Settings { motion, ..settings };
    let mut follower = Follower::new(&part, camera(), SPAN, settings).unwrap();
    let pose = truth(0);
    let depth = depth_of(&part, &pose);
    for _ in 0..6 {
        step(&mut follower, &depth);
    }
    let forecast = follower.forecast().unwrap();
    assert!(forecast.translation_error_to(&pose) < MOST_METRES);
    assert!(forecast.rotation_error_to(&pose).to_degrees() < MOST_DEGREES);
}
