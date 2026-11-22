#![expect(
    clippy::unwrap_used,
    reason = "a broken fixture should fail the test loudly"
)]

mod common;

use common::{Scratch, congru, row_major, transform, write_ply, write_xyz};
use congru::{PointCloud, RigidTransform, TriangleMesh};
use congru_synth::shapes::{lumpy_sphere, wavy_patch, wavy_patch_mesh};
use nalgebra::Vector3;

/// Noise-free points: the motion is recovered to the solver's tolerance,
/// far below a thousandth of the unit-sized shapes.
const EXACT: f64 = 1e-3;

fn moved(cloud: &PointCloud, truth: &RigidTransform) -> PointCloud {
    cloud.transformed(&truth.inverse())
}

#[test]
fn methods_lists_every_method_with_a_summary() {
    let (success, lines) = congru(&["methods"]);
    assert!(success);
    let names: Vec<&str> = lines[0]["methods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|method| method["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "local",
            "search",
            "annealed",
            "adaptive",
            "guarded",
            "consensus"
        ]
    );
}

#[test]
fn describe_reports_the_bounds_of_a_box() {
    let scratch = Scratch::new("describe");
    let corners: Vec<_> = (0..8)
        .map(|corner| {
            let along = |bit: u32| f64::from((corner >> bit) & 1);
            nalgebra::Point3::new(2.0 * along(0), 3.0 * along(1), 6.0 * along(2))
        })
        .collect();
    write_xyz(&scratch.path("box.xyz"), &PointCloud::new(corners).unwrap());
    let file = scratch.path("box.xyz");
    let (success, lines) = congru(&["describe", file.to_str().unwrap()]);
    assert!(success);
    assert_eq!(lines[0]["points"], 8);
    assert_eq!(
        lines[0]["bounds"]["max"],
        serde_json::json!([2.0, 3.0, 6.0])
    );
    assert_eq!(lines[0]["size"], 7.0);
}

#[test]
fn align_recovers_a_known_motion_onto_a_cloud() {
    let scratch = Scratch::new("align-cloud");
    let model = wavy_patch(30).unwrap();
    let truth =
        RigidTransform::from_axis_angle(Vector3::new(0.05, -0.03, 0.04), [0.02, 0.0, -0.01]);
    let (source, target) = (scratch.path("scan.xyz"), scratch.path("model.xyz"));
    write_xyz(&source, &moved(&model, &truth));
    write_xyz(&target, &model);
    let (success, lines) = congru(&[
        "align",
        source.to_str().unwrap(),
        target.to_str().unwrap(),
        "--method",
        "local",
    ]);
    assert!(success, "{lines:?}");
    let found = transform(&lines[0]["transform"]);
    assert!(found.rotation_error_to(&truth) < EXACT);
    assert!(found.translation_error_to(&truth) < EXACT);
    assert_eq!(lines[0]["trusted"], true);
    assert!(lines[0]["observability"]["conditioning"].as_f64().unwrap() > 0.0);
}

#[test]
fn align_starts_from_the_guess_onto_a_mesh() {
    let scratch = Scratch::new("align-mesh");
    let mesh = wavy_patch_mesh(25).unwrap();
    let truth = RigidTransform::from_axis_angle(Vector3::new(0.3, 1.2, -0.5), [0.4, -0.2, 1.0]);
    let (source, target) = (scratch.path("scan.ply"), scratch.path("model.ply"));
    let scan = TriangleMesh::new(moved(mesh.vertices(), &truth), mesh.triangles()).unwrap();
    write_ply(&source, &scan);
    write_ply(&target, &mesh);
    let guess = row_major(&truth);
    let (success, lines) = congru(&[
        "align",
        source.to_str().unwrap(),
        target.to_str().unwrap(),
        "--method",
        "local",
        "--guess",
        &guess,
    ]);
    assert!(success, "{lines:?}");
    let found = transform(&lines[0]["transform"]);
    assert!(found.rotation_error_to(&truth) < EXACT);
    assert!(found.translation_error_to(&truth) < EXACT);
}

#[test]
fn a_failure_is_one_error_object_and_exit_code_one() {
    let (success, lines) = congru(&["describe", "/nonexistent/scan.xyz"]);
    assert!(!success);
    assert!(lines[0]["error"].as_str().unwrap().contains("scan.xyz"));
    let (success, lines) = congru(&["align", "only-one.xyz"]);
    assert!(!success);
    assert!(lines[0]["error"].is_string());
}

#[test]
fn track_follows_a_model_frame_after_frame() {
    let scratch = Scratch::new("track");
    let model = lumpy_sphere(1500).unwrap();
    write_xyz(&scratch.path("model.xyz"), &model);
    let frames = scratch.path("frames");
    std::fs::create_dir(&frames).unwrap();
    let poses: Vec<RigidTransform> = (0..4)
        .map(|frame| {
            let turned = 0.03 * f64::from(frame);
            RigidTransform::from_axis_angle(Vector3::new(0.1, turned, 0.0), [0.0, 0.0, 4.0])
        })
        .collect();
    for (frame, pose) in poses.iter().enumerate().rev() {
        write_xyz(
            &frames.join(format!("frame-{frame:03}.xyz")),
            &model.transformed(pose),
        );
    }
    let model_file = scratch.path("model.xyz");
    let (success, lines) = congru(&[
        "track",
        frames.to_str().unwrap(),
        "--model",
        model_file.to_str().unwrap(),
    ]);
    assert!(success, "{lines:?}");
    assert_eq!(lines.len(), poses.len());
    for (frame, (line, pose)) in lines.iter().zip(&poses).enumerate() {
        assert_eq!(line["frame"], frame);
        assert_eq!(line["file"], format!("frame-{frame:03}.xyz"));
        assert_eq!(line["trusted"], true, "{line}");
        let found = transform(&line["transform"]);
        assert!(found.rotation_error_to(pose) < 2_f64.to_radians(), "{line}");
    }
}
