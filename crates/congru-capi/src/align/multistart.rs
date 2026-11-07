use congru::Multistart;

use super::icp::{IcpOptions, congru_icp_options_default, icp, translated};
use crate::boundary::{Failure, cloud, guard, write};
use crate::status::Status;
use crate::target::TargetHandle;

/// Aligns without an initial pose, by ICP from `starts` rotations spread over all
/// of them; `starts` zero takes the core's default, `options` null the defaults.
///
/// # Safety
/// `target` is a live handle; `xyz` points at `3 × count` readable doubles;
/// `options` is null or points at one options struct; `out` at writable memory for
/// one alignment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_multistart_align(
    target: *const TargetHandle,
    xyz: *const f64,
    count: usize,
    starts: u32,
    options: *const IcpOptions,
    out: *mut super::AlignmentOut,
) -> Status {
    guard(|| {
        let target = unsafe { target.as_ref() }.ok_or_else(|| Failure::null("target"))?;
        let source = unsafe { cloud(xyz, count, "xyz") }?;
        let options = unsafe { options.as_ref() }
            .copied()
            .unwrap_or_else(|| congru_icp_options_default());
        let defaults = Multistart::default();
        let search = Multistart {
            starts: if starts == 0 {
                defaults.starts
            } else {
                usize::try_from(starts).unwrap_or(defaults.starts)
            },
            icp: icp(&options)?,
        };
        let alignment = search.align(&source, &target.0)?;
        unsafe { write(out, translated(&alignment), "out") }
    })
}
