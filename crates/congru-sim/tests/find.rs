use congru::{Distance, RigidTransform, Vector};
use congru_sim::{Degradation, bracket, turntable};
use congru_vision::{Camera, Finder, Observed, render};

#[test]
fn the_bracket_alone_is_found_where_it_stands() -> Result<(), congru::Error> {
    let part = bracket()?;
    let camera = Camera {
        width: 320,
        height: 240,
        focal: 300.0,
    };
    let truth = RigidTransform::from_axis_angle(Vector::new(-1.9, 0.4, 0.3), [0.02, -0.01, 0.6]);
    let depth = render(&camera, &[(&part, truth)]).depth;
    let finder = Finder::new(&part, Distance::new(0.0125)?)?;
    let found = finder
        .find(&depth, &camera, 0.3)
        .ok_or(congru::Error::Degenerate)?;
    let degrees = found.pose.rotation_error_to(&truth).to_degrees();
    let offset = found.pose.translation_error_to(&truth);
    assert!(
        degrees < 15.0 && offset < 0.02,
        "{degrees:.1}° and {offset:.3} off, agreement {:.2}",
        found.agreement
    );
    Ok(())
}

#[test]
fn the_bracket_is_found_on_a_turntable_that_takes_most_votes() -> Result<(), congru::Error> {
    let part = bracket()?;
    let table = turntable()?;
    let camera = Camera::with_field_of_view(160, 120, 60.0);
    let truth = RigidTransform::from_axis_angle(Vector::new(-1.9, 0.4, 0.3), [0.02, -0.01, 0.6]);
    let depth = render(&camera, &[(&part, truth), (&table, truth)]).depth;
    let finder = Finder::new(&part, Distance::new(0.0125)?)?;
    let found = finder
        .find(&depth, &camera, 0.3)
        .ok_or(congru::Error::Degenerate)?;
    let degrees = found.pose.rotation_error_to(&truth).to_degrees();
    let offset = found.pose.translation_error_to(&truth);
    assert!(
        degrees < 5.0 && offset < 0.01,
        "{degrees:.1}° and {offset:.3} off, agreement {:.2}",
        found.agreement
    );
    Ok(())
}

#[test]
fn the_truth_outscores_the_part_flipped_flush_on_its_turntable() -> Result<(), congru::Error> {
    let part = bracket()?;
    let table = turntable()?;
    let camera = Camera::with_field_of_view(160, 120, 60.0);
    let truth = RigidTransform::from_axis_angle(Vector::new(-1.9, 0.4, 0.3), [0.02, -0.01, 0.6]);
    let flip = RigidTransform::from_axis_angle(Vector::x() * std::f64::consts::PI, [0.0; 3]);
    let flipped = flip.then(&truth);
    let exact = render(&camera, &[(&part, truth), (&table, truth)]).depth;
    let step = 0.0125;
    let tolerance = step / 2.0;
    let sensor = Degradation {
        blur: 1,
        noise: 0.01,
        holes: 0.04,
        flying_pixels: true,
        lens: 0.0,
    };
    let finder = Finder::new(&part, Distance::new(step)?)?;
    for seed in 0..4 {
        let seen = Observed::new(sensor.apply(&exact, seed), &camera, tolerance);
        let right = finder.score(&seen, &camera, &truth, tolerance);
        let wrong = finder.score(&seen, &camera, &flipped, tolerance);
        assert!(right > 2.0 * wrong, "seed {seed}: {right} against {wrong}");
    }
    Ok(())
}
