use std::ffi::c_int;

use congru::{Alignment, Convergence, Distance, Fraction, Icp, Kernel, Metric, StopReason};

use crate::boundary::{Failure, cloud, guard, transform, write};
use crate::status::Status;
use crate::target::TargetHandle;

/// `congru_icp_options` in the header. `metric` and `kernel` hold the
/// header's `CONGRU_METRIC_*` and `CONGRU_KERNEL_*` values; a
/// `max_correspondence_distance` of zero keeps every pair.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IcpOptions {
    pub metric: c_int,
    pub kernel: c_int,
    /// In the unit of the points; unused by the quadratic kernel.
    pub kernel_scale: f64,
    pub max_correspondence_distance: f64,
    pub overlap: f64,
    pub max_iterations: u32,
    /// In radians.
    pub rotation_tolerance: f64,
    pub translation_tolerance: f64,
}

/// `congru_alignment` in the header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignmentOut {
    /// Takes the source onto the target, row-major.
    pub transform: [f64; 16],
    pub rmse: f64,
    pub inlier_fraction: f64,
    pub iterations: u32,
    pub converged: bool,
    /// Smallest over largest stiffness of the six motions; NaN when the
    /// target has no normals or too few pairs were kept.
    pub conditioning: f64,
    pub weak_motions: u32,
}

const POINT_TO_POINT: c_int = 0;
const POINT_TO_PLANE: c_int = 1;

/// The core's defaults: point-to-plane, quadratic kernel, every pair kept.
#[unsafe(no_mangle)]
pub extern "C" fn congru_icp_options_default() -> IcpOptions {
    let icp = Icp::default();
    IcpOptions {
        metric: POINT_TO_PLANE,
        kernel: 0,
        kernel_scale: 0.0,
        max_correspondence_distance: icp.max_correspondence_distance.map_or(0.0, Distance::get),
        overlap: icp.overlap.get(),
        max_iterations: icp.convergence.max_iterations,
        rotation_tolerance: icp.convergence.rotation,
        translation_tolerance: icp.convergence.translation,
    }
}

/// Aligns `count` source points onto `target` from `initial`, the identity
/// when null; `options` null uses [`congru_icp_options_default`].
///
/// # Safety
/// `target` is a live handle; `xyz` points at `3 × count` readable doubles;
/// `initial` is null or points at 16; `options` is null or points at one
/// options struct; `out` points at writable memory for one alignment.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_icp_align(
    target: *const TargetHandle,
    xyz: *const f64,
    count: usize,
    initial: *const f64,
    options: *const IcpOptions,
    out: *mut AlignmentOut,
) -> Status {
    guard(|| {
        let target = unsafe { target.as_ref() }.ok_or_else(|| Failure::null("target"))?;
        let source = unsafe { cloud(xyz, count, "xyz") }?;
        let initial = unsafe { transform(initial, "initial") }?;
        let options = unsafe { options.as_ref() }
            .copied()
            .unwrap_or_else(|| congru_icp_options_default());
        let alignment = icp(&options)?.align(&source, &target.0, &initial)?;
        unsafe { write(out, translated(&alignment), "out") }
    })
}

fn icp(options: &IcpOptions) -> Result<Icp, Failure> {
    let metric = match options.metric {
        POINT_TO_POINT => Metric::PointToPoint,
        POINT_TO_PLANE => Metric::PointToPlane,
        _ => return Err(Failure::new(Status::InvalidParameter, "unknown metric")),
    };
    let gate = options.max_correspondence_distance;
    Ok(Icp {
        metric,
        kernel: kernel(options.kernel, options.kernel_scale)?,
        max_correspondence_distance: (gate != 0.0).then(|| Distance::new(gate)).transpose()?,
        overlap: Fraction::new(options.overlap)?,
        convergence: Convergence {
            max_iterations: options.max_iterations,
            rotation: options.rotation_tolerance,
            translation: options.translation_tolerance,
        },
        prior: None,
    })
}

fn kernel(kind: c_int, scale: f64) -> Result<Kernel, Failure> {
    let scaled = || Distance::new(scale);
    Ok(match kind {
        0 => Kernel::Quadratic,
        1 => Kernel::Huber(scaled()?),
        2 => Kernel::Cauchy(scaled()?),
        3 => Kernel::GemanMcClure(scaled()?),
        4 => Kernel::Tukey(scaled()?),
        _ => return Err(Failure::new(Status::InvalidParameter, "unknown kernel")),
    })
}

fn translated(alignment: &Alignment) -> AlignmentOut {
    let observability = alignment.observability.as_ref();
    AlignmentOut {
        transform: alignment.transform.to_row_major(),
        rmse: alignment.rmse,
        inlier_fraction: alignment.inlier_fraction,
        iterations: alignment.iterations,
        converged: alignment.stop == StopReason::Converged,
        conditioning: observability.map_or(f64::NAN, |report| report.conditioning),
        weak_motions: observability.map_or(0, |report| {
            u32::try_from(report.weak_motions.len()).unwrap_or(u32::MAX)
        }),
    }
}
