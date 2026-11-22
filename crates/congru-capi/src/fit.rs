use congru::fit_rigid;

use crate::boundary::{cloud, doubles, guard, write};
use crate::status::Status;

/// `congru_fit` in the header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitOut {
    /// Takes each source point onto its target point, row-major.
    pub transform: [f64; 16],
    /// Weighted root mean square distance left; for fiducials, the
    /// fiducial registration error.
    pub rmse: f64,
}

/// `congru_similar_fit` in the header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarFitOut {
    pub scale: f64,
    /// The rotation and translation, row-major; the scale is not in it.
    pub transform: [f64; 16],
    pub rmse: f64,
}

/// The rotation, translation and scale best taking `source[i]` onto `target[i]`.
///
/// # Safety
/// `source` and `target` point at `3 × count` readable doubles; `out` points at
/// writable memory for one fit.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_fit_similarity(
    source: *const f64,
    target: *const f64,
    count: usize,
    out: *mut SimilarFitOut,
) -> Status {
    guard(|| {
        let source = unsafe { cloud(source, count, "source") }?;
        let target = unsafe { cloud(target, count, "target") }?;
        let fit = congru::fit_similarity(source.points(), target.points())?;
        let answer = SimilarFitOut {
            scale: fit.scale,
            transform: fit.transform.to_row_major(),
            rmse: fit.rmse,
        };
        unsafe { write(out, answer, "out") }
    })
}

/// The rigid motion best taking `source[i]` onto `target[i]` for `count`
/// pairs, each weighted by `weights[i]`, uniformly when `weights` is null.
///
/// # Safety
/// `source` and `target` point at `3 × count` readable doubles; `weights`
/// is null or points at `count`; `out` points at writable memory for one
/// fit.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_fit_rigid(
    source: *const f64,
    target: *const f64,
    count: usize,
    weights: *const f64,
    out: *mut FitOut,
) -> Status {
    guard(|| {
        let source = unsafe { cloud(source, count, "source") }?;
        let target = unsafe { cloud(target, count, "target") }?;
        let weights = if weights.is_null() {
            None
        } else {
            Some(unsafe { doubles(weights, count, "weights") }?)
        };
        let fit = fit_rigid(source.points(), target.points(), weights)?;
        let answer = FitOut {
            transform: fit.transform.to_row_major(),
            rmse: fit.rmse,
        };
        unsafe { write(out, answer, "out") }
    })
}
