use congru::Target;

use crate::boundary::{Failure, cloud, guard, write};
use crate::status::Status;

/// `congru_target` in the header: opaque to C, freed by
/// [`congru_target_free`].
#[derive(Debug)]
pub struct TargetHandle(pub(crate) Target);

/// Builds a reusable target from `count` points, `x y z` after `x y z`.
/// With `normal_neighbors` at zero it has no normals and serves
/// point-to-point only; otherwise normals are fitted to that many nearest
/// points, 8 to 20 suiting most scans.
///
/// # Safety
/// `xyz` points at `3 × count` readable doubles; `out` at a writable
/// pointer, which receives the handle on success and is left alone on
/// failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_target_new_from_points(
    xyz: *const f64,
    count: usize,
    normal_neighbors: u32,
    out: *mut *mut TargetHandle,
) -> Status {
    guard(|| {
        if out.is_null() {
            return Err(Failure::null("out"));
        }
        let points = unsafe { cloud(xyz, count, "xyz") }?;
        let target = match usize::try_from(normal_neighbors) {
            Ok(0) => Target::new(points)?,
            Ok(neighbors) => Target::with_estimated_normals(points, neighbors)?,
            Err(_) => {
                return Err(Failure::new(
                    Status::InvalidParameter,
                    "normal_neighbors does not fit this platform",
                ));
            }
        };
        let handle = Box::into_raw(Box::new(TargetHandle(target)));
        unsafe { write(out, handle, "out") }
    })
}

/// Frees a target; null is ignored.
///
/// # Safety
/// `target` is null or a handle from [`congru_target_new_from_points`] not
/// freed yet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_target_free(target: *mut TargetHandle) {
    if !target.is_null() {
        drop(unsafe { Box::from_raw(target) });
    }
}
