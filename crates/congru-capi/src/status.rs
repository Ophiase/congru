use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char, c_int};

use congru::Error;

/// What a function of the C ABI reports; `congru_status` in the header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok = 0,
    NullPointer = 1,
    InvalidInput = 2,
    InvalidParameter = 3,
    TooFewCorrespondences = 4,
    Degenerate = 5,
    MissingNormals = 6,
    Panic = 7,
}

impl From<&Error> for Status {
    fn from(error: &Error) -> Self {
        match error {
            Error::InvalidParameter { .. } => Self::InvalidParameter,
            Error::TooFewCorrespondences { .. } => Self::TooFewCorrespondences,
            Error::Degenerate => Self::Degenerate,
            Error::MissingNormals => Self::MissingNormals,
            _ => Self::InvalidInput,
        }
    }
}

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::default());
}

pub fn remember(message: &str) {
    let text = CString::new(message.replace('\0', " ")).unwrap_or_default();
    LAST_ERROR.with(|last| *last.borrow_mut() = text);
}

const fn message(status: c_int) -> &'static CStr {
    match status {
        0 => c"success",
        1 => c"a required pointer was null",
        2 => c"the points are empty, not finite, or not whole 3D points",
        3 => c"a parameter is out of its range",
        4 => c"too few correspondences to determine a rigid motion",
        5 => c"degenerate geometry: the rigid motion is not determined",
        6 => c"point-to-plane needs target normals",
        7 => c"a panic was caught at the C boundary",
        _ => c"unknown status",
    }
}

/// A static, NUL-terminated description of `status`; never null, never to
/// be freed. An unknown value gets a message too.
#[unsafe(no_mangle)]
pub extern "C" fn congru_status_message(status: c_int) -> *const c_char {
    message(status).as_ptr()
}

/// The core's message for the last failure on the calling thread, empty
/// before any. Valid until the next failing call on this thread; never
/// null, never to be freed.
#[unsafe(no_mangle)]
pub extern "C" fn congru_last_error_message() -> *const c_char {
    LAST_ERROR.with(|last| last.borrow().as_ptr())
}
