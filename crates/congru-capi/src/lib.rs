//! A C ABI over congru, declared by hand in `include/congru.h` and wrapped
//! for C++20 by `include/congru.hpp`.
//!
//! **Unstable**: names, structs and status codes may change in any version
//! until the project declares 1.0.
//!
//! Every function returns a status; every pointer is checked before use;
//! a panic is caught at the boundary and becomes a status, never an unwind
//! into C. The core's message of the last failure on the calling thread is
//! kept for `congru_last_error_message`.

#![expect(
    unsafe_code,
    reason = "a C ABI reads raw pointers and exports unmangled symbols"
)]

mod align;
mod boundary;
mod fit;
mod status;
#[cfg(test)]
mod surface_tests;
mod target;
#[cfg(test)]
mod tests;

pub use align::{
    AlignmentOut, CertificateOut, CertifierOptions, IcpOptions, SurfaceHandle, congru_certify,
    congru_icp_align, congru_icp_options_default, congru_multistart_align, congru_surface_align,
    congru_surface_free, congru_surface_new_field, congru_surface_new_mesh,
};
pub use fit::{FitOut, SimilarFitOut, congru_fit_rigid, congru_fit_similarity};
pub use status::{Status, congru_last_error_message, congru_status_message};
pub use target::{TargetHandle, congru_target_free, congru_target_new_from_points};
