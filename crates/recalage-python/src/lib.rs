//! Python bindings of `recalage`: NumPy arrays in, NumPy arrays out, the
//! core's defaults and errors, the interpreter released while aligning.

mod align;
mod arrays;
mod error;
mod results;
mod target;

use pyo3::pymodule;

#[pymodule(name = "_recalage")]
mod module {
    #[pymodule_export]
    use crate::align::{Icp, Kernel, Metric, Multistart};
    #[pymodule_export]
    use crate::error::RegistrationError;
    #[pymodule_export]
    use crate::results::{
        Alignment, Fit, Observability, Rotation, Translation, WeakMotion, fit_rigid,
    };
    #[pymodule_export]
    use crate::target::Target;
}
