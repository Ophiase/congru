use congru::{Distance, Point, RigidTransform, Vector};
use congru_sim::{Degradation, Solids, Turntable, bracket, surroundings, turntable};
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

#[test]
fn the_bracket_is_found_among_surroundings_in_a_noisy_frame() -> Result<(), congru::Error> {
    let part = bracket()?;
    let table = turntable()?;
    let around = surroundings()?;
    let camera = Camera::with_field_of_view(160, 120, 60.0);
    let finder = Finder::new(&part, Distance::new(0.0125)?)?;
    let sensor = Degradation {
        blur: 1,
        noise: 0.025,
        holes: 0.08,
        flying_pixels: true,
        lens: 0.0,
    };
    let path = Turntable {
        frames: 12,
        turns: 1.0,
        shake_degrees: 0.0,
        handheld: congru_synth::sequence::Handheld::default(),
        knock_degrees: 0.0,
    };
    let mut right = 0;
    for frame in [1, 4, 8, 10] {
        let truth = path.pose(frame);
        let exact = render(
            &camera,
            &[
                (&part, truth),
                (&table, truth),
                (&around, RigidTransform::identity()),
            ],
        )
        .depth;
        let depth = sensor.apply(&exact, frame as u64);
        let found = finder.find(&depth, &camera, f64::NEG_INFINITY);
        right += usize::from(found.is_some_and(|found| {
            found.pose.rotation_error_to(&truth).to_degrees() < 5.0
                && found.pose.translation_error_to(&truth) < 0.02
        }));
    }
    assert!(right >= 3, "{right} of 4 frames found");
    Ok(())
}

#[test]
fn the_bracket_is_found_with_its_flat_back_to_the_camera() -> Result<(), congru::Error> {
    let part = bracket()?;
    let camera = Camera::with_field_of_view(160, 120, 60.0);
    let finder = Finder::new(&part, Distance::new(0.0125)?)?;
    let truth =
        RigidTransform::from_axis_angle(Vector::x() * std::f64::consts::PI, [0.0, 0.0, 0.5]);
    let depth = render(&camera, &[(&part, truth)]).depth;
    let found = finder
        .find(&depth, &camera, 0.3)
        .ok_or(congru::Error::Degenerate)?;
    let flat_side = found.pose.apply(&Point::new(0.1, 0.06, 0.0)).z;
    assert!(
        (flat_side - 0.5).abs() < 0.01 && found.pose.translation_error_to(&truth) < 0.3,
        "the flat side at {flat_side:.3}, confirms {:.2}",
        found.agreement
    );
    Ok(())
}

#[test]
fn a_can_in_front_hides_a_pose_where_an_empty_frame_refutes_it() -> Result<(), congru::Error> {
    let part = bracket()?;
    let camera = Camera::with_field_of_view(160, 120, 60.0);
    let finder = Finder::new(&part, Distance::new(0.0125)?)?;
    let pose = RigidTransform::from_axis_angle(Vector::new(-1.9, 0.4, 0.3), [0.0, 0.0, 0.6]);
    let wall_behind = Solids::default()
        .cuboid([-2.0, -2.0, 1.2], [2.0, 2.0, 1.3])
        .build()?;
    let screen_before = Solids::default()
        .cuboid([-2.0, -2.0, 0.2], [2.0, 2.0, 0.3])
        .build()?;
    let identity = RigidTransform::identity();
    let refuted = render(&camera, &[(&wall_behind, identity)]).depth;
    let hidden = render(&camera, &[(&screen_before, identity)]).depth;
    assert!(finder.through_share(&refuted, &camera, &pose) > 0.9);
    assert!(finder.through_share(&hidden, &camera, &pose) < 0.01);
    Ok(())
}

#[test]
fn a_part_knocked_on_its_turntable_is_found_again_where_it_stood() -> Result<(), congru::Error> {
    let part = bracket()?;
    let table = turntable()?;
    let camera = Camera::with_field_of_view(160, 120, 60.0);
    let finder = Finder::new(&part, Distance::new(0.0125)?)?;
    let sensor = Degradation {
        blur: 1,
        noise: 0.025,
        holes: 0.08,
        flying_pixels: true,
        lens: 0.0,
    };
    let path = Turntable {
        frames: 12,
        turns: 1.0,
        shake_degrees: 0.0,
        handheld: congru_synth::sequence::Handheld::default(),
        knock_degrees: 0.0,
    };
    for frame in [2, 7] {
        let truth = path.pose(frame);
        let before = RigidTransform::from_axis_angle(Vector::z() * 1.2, [0.0; 3]).then(&truth);
        let exact = render(&camera, &[(&part, truth), (&table, truth)]).depth;
        let depth = sensor.apply(&exact, frame as u64);
        let found = finder
            .turned_again(&depth, &camera, &before)
            .ok_or(congru::Error::Degenerate)?;
        let degrees = found.pose.rotation_error_to(&truth).to_degrees();
        assert!(degrees < 8.0, "frame {frame}: {degrees:.1}° off");
    }
    Ok(())
}

#[test]
fn a_twin_beside_the_box_does_not_win() -> Result<(), congru::Error> {
    let part = bracket()?;
    let camera = Camera::with_field_of_view(160, 120, 60.0);
    let turn = Vector::new(-1.9, 0.4, 0.3);
    let target = RigidTransform::from_axis_angle(turn, [-0.12, 0.0, 0.7]);
    let twin = RigidTransform::from_axis_angle(turn, [0.12, 0.0, 0.7]);
    let depth = render(&camera, &[(&part, target), (&part, twin)]).depth;
    let mut region = depth.clone();
    for (column, row) in depth.positions() {
        if column >= camera.width / 2 {
            region.set(column, row, f64::NAN);
        }
    }
    let finder = Finder::new(&part, Distance::new(0.0125)?)?;
    let found = finder
        .find_in(&depth, &region, &camera, 0.1)
        .ok_or(congru::Error::Degenerate)?;
    let offset = found.pose.translation_error_to(&target);
    assert!(offset < 0.02, "{offset:.3} off");
    Ok(())
}
