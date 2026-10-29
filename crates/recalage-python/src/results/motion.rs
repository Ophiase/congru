use numpy::PyArray1;
use pyo3::prelude::*;
use recalage::Vector;

use crate::arrays::vector_array;

/// A turn about the line along `axis` (unit) through `through`, the point of
/// that line closest to the source's centroid.
#[pyclass(module = "recalage", frozen)]
#[derive(Debug)]
pub struct Rotation {
    pub axis: Vector,
    pub through: Vector,
}

#[pymethods]
impl Rotation {
    #[classattr]
    #[pyo3(name = "__match_args__")]
    const MATCH_ARGUMENTS: (&'static str, &'static str) = ("axis", "through");

    #[getter]
    fn axis<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        vector_array(py, &self.axis)
    }

    #[getter]
    fn through<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        vector_array(py, &self.through)
    }
}

/// A slide along `direction` (unit).
#[pyclass(module = "recalage", frozen)]
#[derive(Debug)]
pub struct Translation {
    pub direction: Vector,
}

#[pymethods]
impl Translation {
    #[classattr]
    #[pyo3(name = "__match_args__")]
    const MATCH_ARGUMENTS: (&'static str,) = ("direction",);

    #[getter]
    fn direction<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        vector_array(py, &self.direction)
    }
}
