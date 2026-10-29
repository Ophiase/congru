use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use congru::{PointCloud, RigidTransform, Vector};

use super::shape::Shape;
use crate::error::registration_error;

const TRIPLES: Shape = Shape {
    rows: None,
    columns: 3,
};

const HOMOGENEOUS: Shape = Shape {
    rows: Some(4),
    columns: 4,
};

/// An `(n, 3)` array of finite coordinates.
#[derive(Debug)]
pub struct Points(pub PointCloud);

/// An `(n, 3)` array of vectors, one per point.
#[derive(Debug)]
pub struct Directions(pub Vec<Vector>);

/// An `(m, 3)` array of vertex indices.
#[derive(Debug)]
pub struct Triangles(pub Vec<[u32; 3]>);

/// A `(4, 4)` homogeneous matrix of a proper rigid motion.
#[derive(Debug)]
pub struct Transform(pub RigidTransform);

impl<'py> FromPyObject<'_, 'py> for Points {
    type Error = PyErr;

    fn extract(object: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        TRIPLES.read(object, |entries| {
            PointCloud::from_xyz(entries)
                .map(Self)
                .map_err(registration_error)
        })
    }
}

impl<'py> FromPyObject<'_, 'py> for Directions {
    type Error = PyErr;

    fn extract(object: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        TRIPLES.read(object, |entries: &[f64]| {
            Ok(Self(
                entries
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|&row| Vector::from(row))
                    .collect(),
            ))
        })
    }
}

impl<'py> FromPyObject<'_, 'py> for Triangles {
    type Error = PyErr;

    fn extract(object: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        TRIPLES.read(object, |entries: &[u32]| {
            Ok(Self(entries.as_chunks::<3>().0.to_vec()))
        })
    }
}

impl<'py> FromPyObject<'_, 'py> for Transform {
    type Error = PyErr;

    fn extract(object: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        HOMOGENEOUS.read(object, |entries: &[f64]| {
            let entries: &[f64; 16] = entries
                .try_into()
                .map_err(|_| PyValueError::new_err("a transform holds 16 entries"))?;
            RigidTransform::from_row_major(entries)
                .map(Self)
                .map_err(registration_error)
        })
    }
}
