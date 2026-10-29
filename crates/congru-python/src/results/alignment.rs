use numpy::PyArray2;
use pyo3::prelude::*;
use congru::StopReason;

use super::Observability;
use crate::arrays::transform_array;

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Alignment(pub congru::Alignment);

#[pymethods]
impl Alignment {
    #[getter]
    fn transform<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        transform_array(py, &self.0.transform)
    }

    #[getter]
    const fn rmse(&self) -> f64 {
        self.0.rmse
    }

    #[getter]
    const fn inlier_fraction(&self) -> f64 {
        self.0.inlier_fraction
    }

    #[getter]
    const fn iterations(&self) -> u32 {
        self.0.iterations
    }

    /// `False` when the iteration budget ran out first.
    #[getter]
    fn converged(&self) -> bool {
        self.0.stop == StopReason::Converged
    }

    #[getter]
    fn observability(&self) -> Option<Observability> {
        self.0.observability.clone().map(Observability)
    }
}
