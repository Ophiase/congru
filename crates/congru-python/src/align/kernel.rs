use pyo3::prelude::*;

use super::units::distance;

/// Each constructor takes the residual `scale` past which a pair starts to
/// count as an outlier, in the unit of the point coordinates.
#[pyclass(module = "congru", frozen, eq, from_py_object)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kernel(pub congru::Kernel);

#[pymethods]
impl Kernel {
    #[staticmethod]
    fn quadratic() -> Self {
        Self(congru::Kernel::Quadratic)
    }

    #[staticmethod]
    fn huber(scale: f64) -> PyResult<Self> {
        Ok(Self(congru::Kernel::Huber(distance(scale)?)))
    }

    #[staticmethod]
    fn cauchy(scale: f64) -> PyResult<Self> {
        Ok(Self(congru::Kernel::Cauchy(distance(scale)?)))
    }

    #[staticmethod]
    fn geman_mcclure(scale: f64) -> PyResult<Self> {
        Ok(Self(congru::Kernel::GemanMcClure(distance(scale)?)))
    }

    #[staticmethod]
    fn tukey(scale: f64) -> PyResult<Self> {
        Ok(Self(congru::Kernel::Tukey(distance(scale)?)))
    }

    /// The weight of a pair whose residual is `residual`, in `[0, 1]`.
    fn weight(&self, residual: f64) -> f64 {
        self.0.weight(residual)
    }

    fn __repr__(&self) -> String {
        match self.0 {
            congru::Kernel::Quadratic => "Kernel.quadratic()".to_owned(),
            congru::Kernel::Huber(scale) => format!("Kernel.huber({})", scale.get()),
            congru::Kernel::Cauchy(scale) => format!("Kernel.cauchy({})", scale.get()),
            congru::Kernel::GemanMcClure(scale) => {
                format!("Kernel.geman_mcclure({})", scale.get())
            }
            congru::Kernel::Tukey(scale) => format!("Kernel.tukey({})", scale.get()),
        }
    }
}
