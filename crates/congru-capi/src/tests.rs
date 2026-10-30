use std::ffi::CStr;
use std::ptr::{null, null_mut};

use congru::RigidTransform;
use congru_synth::shapes::lumpy_sphere;
use nalgebra::Vector3;

use crate::{
    AlignmentOut, FitOut, Status, TargetHandle, congru_fit_rigid, congru_icp_align,
    congru_icp_options_default, congru_last_error_message, congru_status_message,
    congru_target_free, congru_target_new_from_points,
};

/// Noise-free points: the motion is recovered to the solver's tolerance.
const EXACT: f64 = 1e-6;

const EMPTY_ALIGNMENT: AlignmentOut = AlignmentOut {
    transform: [0.0; 16],
    rmse: 0.0,
    inlier_fraction: 0.0,
    iterations: 0,
    converged: false,
    conditioning: 0.0,
    weak_motions: 0,
};

fn last_error() -> String {
    unsafe { CStr::from_ptr(congru_last_error_message()) }
        .to_string_lossy()
        .into_owned()
}

#[test]
fn icp_through_the_abi_recovers_a_known_motion() {
    let model = lumpy_sphere(2000).unwrap();
    let truth =
        RigidTransform::from_axis_angle(Vector3::new(0.08, -0.05, 0.1), [0.03, 0.01, -0.02]);
    let source = model.transformed(&truth.inverse()).to_xyz();
    let coordinates = model.to_xyz();
    let mut target: *mut TargetHandle = null_mut();
    let status = unsafe {
        congru_target_new_from_points(coordinates.as_ptr(), model.count(), 12, &raw mut target)
    };
    assert_eq!(status, Status::Ok);
    let options = congru_icp_options_default();
    let mut alignment = EMPTY_ALIGNMENT;
    let status = unsafe {
        congru_icp_align(
            target,
            source.as_ptr(),
            model.count(),
            null(),
            &raw const options,
            &raw mut alignment,
        )
    };
    unsafe { congru_target_free(target) };
    assert_eq!(status, Status::Ok, "{}", last_error());
    let found = RigidTransform::from_row_major(&alignment.transform).unwrap();
    assert!(found.rotation_error_to(&truth) < EXACT);
    assert!(found.translation_error_to(&truth) < EXACT);
    assert!(alignment.conditioning > 0.0);
}

#[test]
fn fit_through_the_abi_recovers_a_known_motion() {
    let truth = RigidTransform::from_axis_angle(Vector3::new(0.4, 1.0, -0.3), [1.0, -2.0, 0.5]);
    let marks =
        congru::PointCloud::from_xyz(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0])
            .unwrap();
    let source = marks.to_xyz();
    let target = marks.transformed(&truth).to_xyz();
    let mut fit = FitOut {
        transform: [0.0; 16],
        rmse: 1.0,
    };
    let status =
        unsafe { congru_fit_rigid(source.as_ptr(), target.as_ptr(), 4, null(), &raw mut fit) };
    assert_eq!(status, Status::Ok);
    let found = RigidTransform::from_row_major(&fit.transform).unwrap();
    assert!(found.rotation_error_to(&truth) < EXACT);
    assert!(fit.rmse < EXACT);
}

#[test]
fn a_null_pointer_is_a_status_with_a_message() {
    let mut alignment = EMPTY_ALIGNMENT;
    let status = unsafe { congru_icp_align(null(), null(), 0, null(), null(), &raw mut alignment) };
    assert_eq!(status, Status::NullPointer);
    assert!(last_error().contains("target"));
    let generic = unsafe { CStr::from_ptr(congru_status_message(status as i32)) };
    assert!(!generic.is_empty());
}

#[test]
fn the_core_s_refusal_carries_its_message() {
    let flat = [0.0_f64; 9];
    let mut target: *mut TargetHandle = null_mut();
    let status = unsafe { congru_target_new_from_points(flat.as_ptr(), 3, 40, &raw mut target) };
    assert_eq!(status, Status::InvalidParameter);
    assert!(target.is_null());
    assert_ne!(last_error(), "");
}
