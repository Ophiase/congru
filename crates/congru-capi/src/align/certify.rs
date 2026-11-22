use congru::{Certifier, Distance};

use crate::boundary::{cloud, guard, write};
use crate::status::Status;

/// `congru_certifier_options` in the header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CertifierOptions {
    /// Past this distance a point counts as an outlier.
    pub truncation: f64,
    /// How far the source's centroid may land from the target's, per axis.
    pub reach: f64,
    /// Negative: asks for a gap of that size. Positive: keeps every pose this close.
    pub margin: f64,
    pub rotation_resolution: f64,
    pub translation_resolution: f64,
    pub budget: u64,
}

/// `congru_certificate` in the header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CertificateOut {
    /// No pose in the domain costs less, whether or not `closed`.
    pub lower: f64,
    /// The value of `pose`, the best seen.
    pub upper: f64,
    pub pose: [f64; 16],
    pub closed: bool,
    pub measured: u64,
    /// How many poses the bound could not exclude.
    pub survivors: u64,
}

/// Proves how far any pose can be below the best found, for tens of source points.
///
/// # Safety
/// `source` and `target` point at `3 ×` their count of readable doubles; `options`
/// at one options struct; `out` at writable memory for one certificate.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_certify(
    source: *const f64,
    source_count: usize,
    target: *const f64,
    target_count: usize,
    options: *const CertifierOptions,
    out: *mut CertificateOut,
) -> Status {
    guard(|| {
        let source = unsafe { cloud(source, source_count, "source") }?;
        let target = unsafe { cloud(target, target_count, "target") }?;
        let options =
            unsafe { options.as_ref() }.ok_or_else(|| crate::boundary::Failure::null("options"))?;
        let certificate = Certifier {
            truncation: Distance::new(options.truncation)?,
            reach: Distance::new(options.reach)?,
            margin: options.margin,
            resolution: [options.rotation_resolution, options.translation_resolution],
            budget: usize::try_from(options.budget).unwrap_or(usize::MAX),
        }
        .run(&source, &target, &[])?;
        let answer = CertificateOut {
            lower: certificate.lower,
            upper: certificate.upper,
            pose: certificate.pose.to_row_major(),
            closed: certificate.closed,
            measured: u64::try_from(certificate.measured).unwrap_or(u64::MAX),
            survivors: u64::try_from(certificate.survivors.len()).unwrap_or(u64::MAX),
        };
        unsafe { write(out, answer, "out") }
    })
}
