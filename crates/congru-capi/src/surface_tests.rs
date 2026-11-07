use std::ptr::null_mut;

use congru::RigidTransform;
use nalgebra::Vector3;

use crate::{
    AlignmentOut, CertificateOut, CertifierOptions, SimilarFitOut, Status, SurfaceHandle,
    congru_certify, congru_fit_similarity, congru_icp_options_default, congru_surface_align,
    congru_surface_free, congru_surface_new_field, congru_surface_new_mesh,
};

const EMPTY: AlignmentOut = AlignmentOut {
    transform: [0.0; 16],
    rmse: 0.0,
    inlier_fraction: 0.0,
    iterations: 0,
    converged: false,
    conditioning: 0.0,
    weak_motions: 0,
};

/// A box 20 by 13 by 7 cm, its faces outwards.
const VERTICES: [f64; 24] = [
    0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 0.0, 0.13, 0.0, 0.2, 0.13, 0.0, 0.0, 0.0, 0.07, 0.2, 0.0, 0.07,
    0.0, 0.13, 0.07, 0.2, 0.13, 0.07,
];
const TRIANGLES: [u32; 36] = [
    0, 2, 1, 1, 2, 3, 4, 5, 6, 5, 7, 6, 0, 1, 4, 1, 5, 4, 2, 6, 3, 3, 6, 7, 0, 4, 2, 2, 4, 6, 1, 3,
    5, 3, 7, 5,
];

fn truth() -> RigidTransform {
    RigidTransform::from_axis_angle(Vector3::new(0.03, -0.02, 0.04), [0.004, 0.0, 0.002])
}

/// Points on the box's faces, moved away by the inverse of the truth.
fn scan() -> Vec<f64> {
    let mut points = Vec::new();
    for across in 0..12 {
        for along in 0..8 {
            let (first, second) = (f64::from(across) / 11.0, f64::from(along) / 7.0);
            for point in [
                [0.2 * first, 0.13 * second, 0.0],
                [0.2 * first, 0.0, 0.07 * second],
                [0.0, 0.13 * first, 0.07 * second],
            ] {
                let moved = truth().inverse().apply(&point.into());
                points.extend([moved.x, moved.y, moved.z]);
            }
        }
    }
    points
}

fn aligned_on(surface: *mut SurfaceHandle) -> AlignmentOut {
    let points = scan();
    let mut out = EMPTY;
    let options = congru_icp_options_default();
    let status = unsafe {
        congru_surface_align(
            surface,
            points.as_ptr(),
            points.len() / 3,
            std::ptr::null(),
            &raw const options,
            &raw mut out,
        )
    };
    assert_eq!(status, Status::Ok);
    unsafe { congru_surface_free(surface) };
    out
}

fn assert_truth(out: &AlignmentOut) {
    let found = RigidTransform::from_row_major(&out.transform).unwrap();
    assert!(found.rotation_error_to(&truth()) < 0.01);
}

#[test]
fn a_motion_is_recovered_on_a_mesh_through_the_abi() {
    let mut surface = null_mut();
    let status = unsafe {
        congru_surface_new_mesh(
            VERTICES.as_ptr(),
            8,
            TRIANGLES.as_ptr(),
            12,
            &raw mut surface,
        )
    };
    assert_eq!(status, Status::Ok);
    assert_truth(&aligned_on(surface));
}

#[test]
fn a_motion_is_recovered_on_a_field_through_the_abi() {
    let mut surface = null_mut();
    let status = unsafe {
        congru_surface_new_field(
            VERTICES.as_ptr(),
            8,
            TRIANGLES.as_ptr(),
            12,
            0.002,
            0.03,
            &raw mut surface,
        )
    };
    assert_eq!(status, Status::Ok);
    assert_truth(&aligned_on(surface));
}

#[test]
fn a_dangling_triangle_is_refused() {
    let mut surface = null_mut();
    let wrong = [0_u32, 1, 99];
    let status = unsafe {
        congru_surface_new_mesh(VERTICES.as_ptr(), 8, wrong.as_ptr(), 1, &raw mut surface)
    };
    assert_ne!(status, Status::Ok);
    assert!(surface.is_null());
}

#[test]
fn a_scale_is_recovered_through_the_abi() {
    let source = scan();
    let target: Vec<f64> = source.iter().map(|value| 2.5 * value).collect();
    let mut out = SimilarFitOut {
        scale: 0.0,
        transform: [0.0; 16],
        rmse: 0.0,
    };
    let status = unsafe {
        congru_fit_similarity(
            source.as_ptr(),
            target.as_ptr(),
            source.len() / 3,
            &raw mut out,
        )
    };
    assert_eq!(status, Status::Ok);
    assert!((out.scale - 2.5).abs() < 1e-9);
}

#[test]
fn a_certificate_comes_through_the_abi() {
    let source: Vec<f64> = scan().into_iter().take(3 * 40).collect();
    let target = scan();
    let options = CertifierOptions {
        truncation: 0.02,
        reach: 0.15,
        margin: -1e-4,
        rotation_resolution: 0.05,
        translation_resolution: 0.005,
        budget: 20_000,
    };
    let mut out = CertificateOut {
        lower: 0.0,
        upper: 0.0,
        pose: [0.0; 16],
        closed: false,
        measured: 0,
        survivors: 0,
    };
    let status = unsafe {
        congru_certify(
            source.as_ptr(),
            source.len() / 3,
            target.as_ptr(),
            target.len() / 3,
            &raw const options,
            &raw mut out,
        )
    };
    assert_eq!(status, Status::Ok);
    assert!(out.lower <= out.upper);
    assert!(out.measured > 0);
}
