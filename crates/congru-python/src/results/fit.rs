use numpy::{AllowTypeChange, PyArray2, PyArrayLike1};
use pyo3::prelude::*;

use crate::arrays::{Points, transform_array};
use crate::error::registration_error;

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Fit(congru::Fit);

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

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct SimilarFit(congru::SimilarFit);

#[pymethods]
impl SimilarFit {
    #[getter]
    const fn scale(&self) -> f64 {
        self.0.scale
    }

    /// The rotation and the translation, the scale left out.
    #[getter]
    fn transform<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        transform_array(py, &self.0.transform)
    }

    #[getter]
    const fn rmse(&self) -> f64 {
        self.0.rmse
    }
}

/// `source[i]` is paired with `target[i]`: `scale * R @ source + t` nearest `target`.
#[pyfunction]
pub fn fit_similarity(source: Points, target: Points) -> PyResult<SimilarFit> {
    let (Points(source), Points(target)) = (source, target);
    congru::fit_similarity(source.points(), target.points())
        .map(SimilarFit)
        .map_err(registration_error)
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
    congru::fit_rigid(source.points(), target.points(), weights.as_deref())
        .map(Fit)
        .map_err(registration_error)
}
