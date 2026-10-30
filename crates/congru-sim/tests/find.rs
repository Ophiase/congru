use congru::{Distance, RigidTransform, Vector};
use congru_sim::bracket;
use congru_vision::{Camera, Finder, render};

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
