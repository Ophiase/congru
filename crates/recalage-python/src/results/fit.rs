use numpy::{AllowTypeChange, PyArray2, PyArrayLike1};
use pyo3::prelude::*;

use crate::arrays::{Points, transform_array};
use crate::error::registration_error;

#[pyclass(module = "recalage", frozen)]
#[derive(Debug)]
pub struct Fit(recalage::Fit);

#[pymethods]
impl Fit {
    #[getter]
    fn transform<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        transform_array(py, &self.0.transform)
    }

    #[getter]
    const fn rmse(&self) -> f64 {
        self.0.rmse
    }
}

/// `source[i]` is paired with `target[i]`; uniform weights when `None`.
#[pyfunction]
#[pyo3(signature = (source, target, weights = None))]
pub fn fit_rigid(
    source: Points,
    target: Points,
    weights: Option<PyArrayLike1<'_, f64, AllowTypeChange>>,
) -> PyResult<Fit> {
    let (Points(source), Points(target)) = (source, target);
    let weights = weights.map(|weights| weights.as_array().to_vec());
    recalage::fit_rigid(source.points(), target.points(), weights.as_deref())
        .map(Fit)
        .map_err(registration_error)
}
