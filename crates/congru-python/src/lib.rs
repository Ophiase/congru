//! Python bindings of `congru`: NumPy arrays in, NumPy arrays out, the
//! core's defaults and errors, the interpreter released while aligning.

mod align;
mod arrays;
mod error;
mod results;
mod target;

use pyo3::pymodule;

#[pymodule(name = "_congru")]
mod module {
    #[pymodule_export]
    use crate::align::{Certifier, Icp, Kernel, Metric, Multistart};
    #[pymodule_export]
    use crate::error::RegistrationError;
    #[pymodule_export]
    use crate::results::{
        Alignment, Certificate, Fit, Observability, Rotation, SimilarFit, Translation, WeakMotion,
        fit_rigid, fit_similarity,
    };
    #[pymodule_export]
    use crate::target::Target;
}
