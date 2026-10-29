use congru::{PointCloud, RigidTransform, Vector};
use numpy::{PyArray1, PyArray2, PyArrayMethods};
use pyo3::prelude::*;

pub fn transform_array<'py>(
    py: Python<'py>,
    transform: &RigidTransform,
) -> PyResult<Bound<'py, PyArray2<f64>>> {
    PyArray1::from_slice(py, &transform.to_row_major()).reshape([4, 4])
}

pub fn cloud_array<'py>(
    py: Python<'py>,
    cloud: &PointCloud,
) -> PyResult<Bound<'py, PyArray2<f64>>> {
    PyArray1::from_slice(py, &cloud.to_xyz()).reshape([cloud.count(), 3])
}

pub fn vectors_array(
    py: Python<'_>,
    vectors: impl ExactSizeIterator<Item = Vector>,
) -> PyResult<Bound<'_, PyArray2<f64>>> {
    let count = vectors.len();
    let flat: Vec<f64> = vectors
        .flat_map(|vector| [vector.x, vector.y, vector.z])
        .collect();
    PyArray1::from_vec(py, flat).reshape([count, 3])
}

pub fn vector_array<'py>(py: Python<'py>, vector: &Vector) -> Bound<'py, PyArray1<f64>> {
    PyArray1::from_slice(py, vector.as_slice())
}
