use numpy::PyArray2;
use pyo3::prelude::*;

use crate::arrays::transform_array;

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Certificate(pub congru::Certificate);

#[pymethods]
impl Certificate {
    #[getter]
    const fn lower(&self) -> f64 {
        self.0.lower
    }

    #[getter]
    const fn upper(&self) -> f64 {
        self.0.upper
    }

    #[getter]
    fn pose<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        transform_array(py, &self.0.pose)
    }

    /// `False` when the budget ran out: `lower` holds, and may be far.
    #[getter]
    const fn closed(&self) -> bool {
        self.0.closed
    }

    #[getter]
    const fn measured(&self) -> usize {
        self.0.measured
    }

    #[getter]
    fn survivors<'py>(&self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyArray2<f64>>>> {
        self.0
            .survivors
            .iter()
            .map(|pose| transform_array(py, pose))
            .collect()
    }

    /// How much better than `value` any pose in the domain can be.
    fn gap(&self, value: f64) -> f64 {
        self.0.gap(value)
    }
}
