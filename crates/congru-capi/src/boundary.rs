use std::panic::{AssertUnwindSafe, catch_unwind};

use congru::{Error, PointCloud, RigidTransform};

use crate::status::{Status, remember};

/// A failure on its way to C: the status returned, the message kept.
#[derive(Debug)]
pub struct Failure {
    status: Status,
    message: String,
}

impl Failure {
    pub fn new(status: Status, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn null(name: &str) -> Self {
        Self::new(Status::NullPointer, format!("{name} is null"))
    }
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self::new(Status::from(&error), error.to_string())
    }
}

/// Runs `body`, turning its failure or its panic into a status and keeping
/// the message for `congru_last_error_message`.
pub fn guard(body: impl FnOnce() -> Result<(), Failure>) -> Status {
    let outcome = catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|_| {
        Err(Failure::new(
            Status::Panic,
            "a panic was caught at the C boundary",
        ))
    });
    match outcome {
        Ok(()) => Status::Ok,
        Err(failure) => {
            remember(&failure.message);
            failure.status
        }
    }
}

/// # Safety
/// `values` is null, or points at `length` readable, initialised doubles
/// that stay unchanged while the slice lives.
pub unsafe fn doubles<'caller>(
    values: *const f64,
    length: usize,
    name: &str,
) -> Result<&'caller [f64], Failure> {
    if values.is_null() {
        return Err(Failure::null(name));
    }
    if !values.is_aligned() {
        return Err(Failure::new(
            Status::InvalidInput,
            format!("{name} is not aligned for doubles"),
        ));
    }
    // SAFETY: non-null and aligned, checked above; readable for `length`
    // doubles by the caller's contract.
    Ok(unsafe { std::slice::from_raw_parts(values, length) })
}

/// # Safety
/// As [`doubles`], for `3 × count` doubles.
pub unsafe fn cloud(xyz: *const f64, count: usize, name: &str) -> Result<PointCloud, Failure> {
    let length = count.checked_mul(3).ok_or_else(|| {
        Failure::new(
            Status::InvalidInput,
            format!("{name}: {count} points overflow"),
        )
    })?;
    let coordinates = unsafe { doubles(xyz, length, name) }?;
    Ok(PointCloud::from_xyz(coordinates)?)
}

/// # Safety
/// As [`doubles`], for 16 doubles; null gives the identity.
pub unsafe fn transform(entries: *const f64, name: &str) -> Result<RigidTransform, Failure> {
    if entries.is_null() {
        return Ok(RigidTransform::identity());
    }
    let entries = unsafe { doubles(entries, 16, name) }?;
    let row_major: &[f64; 16] = entries
        .try_into()
        .map_err(|_| Failure::new(Status::InvalidInput, "a transform has 16 entries"))?;
    Ok(RigidTransform::from_row_major(row_major)?)
}

/// # Safety
/// `out` is null, or points at writable, aligned memory for one `Value`.
pub unsafe fn write<Value>(out: *mut Value, value: Value, name: &str) -> Result<(), Failure> {
    if out.is_null() {
        return Err(Failure::null(name));
    }
    // SAFETY: non-null, checked above; writable by the caller's contract.
    unsafe { out.write(value) };
    Ok(())
}
