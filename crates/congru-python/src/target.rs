use numpy::PyArray2;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use congru::{OrientedCloud, PointCloud, TriangleMesh, Vector};

use crate::align::distance;
use crate::arrays::{Directions, Points, Triangles, cloud_array, vectors_array};
use crate::error::registration_error;

enum Normals {
    Absent,
    Known(Vec<Vector>),
    Estimated { neighbors: usize },
}

impl Normals {
    fn from_arguments(normals: Option<Directions>, neighbors: Option<usize>) -> PyResult<Self> {
        match (normals, neighbors) {
            (None, None) => Ok(Self::Absent),
            (Some(Directions(normals)), None) => Ok(Self::Known(normals)),
            (None, Some(neighbors)) => Ok(Self::Estimated { neighbors }),
            (Some(_), Some(_)) => Err(PyValueError::new_err(
                "give either normals or neighbors to estimate them, not both",
            )),
        }
    }

    fn attach(self, cloud: PointCloud) -> Result<congru::Target, congru::Error> {
        match self {
            Self::Absent => congru::Target::new(cloud),
            Self::Known(normals) => {
                congru::Target::from_oriented(OrientedCloud::new(cloud, &normals)?)
            }
            Self::Estimated { neighbors } => {
                congru::Target::with_estimated_normals(cloud, neighbors)
            }
        }
    }
}

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Target(congru::Target);

impl Target {
    pub const fn core(&self) -> &congru::Target {
        &self.0
    }
}

#[pymethods]
impl Target {
    /// Normals are given one per point, fitted to each point's `neighbors`
    /// nearest points, or absent, which allows point-to-point only.
    #[new]
    #[pyo3(signature = (points, *, normals = None, neighbors = None))]
    fn new(
        py: Python<'_>,
        points: Points,
        normals: Option<Directions>,
        neighbors: Option<usize>,
    ) -> PyResult<Self> {
        let normals = Normals::from_arguments(normals, neighbors)?;
        py.detach(|| normals.attach(points.0))
            .map(Self)
            .map_err(registration_error)
    }

    /// The surface sampled about every `spacing`, each sample with its face
    /// normal; `triangles` holds vertex indices.
    #[staticmethod]
    fn from_mesh(
        py: Python<'_>,
        vertices: Points,
        triangles: Triangles,
        spacing: f64,
    ) -> PyResult<Self> {
        let spacing = distance(spacing)?;
        py.detach(|| {
            let mesh = TriangleMesh::new(vertices.0, triangles.0)?;
            congru::Target::from_mesh(&mesh, spacing)
        })
        .map(Self)
        .map_err(registration_error)
    }

    #[getter]
    fn points<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        cloud_array(py, self.0.cloud())
    }

    #[getter]
    fn normals<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyArray2<f64>>>> {
        self.0
            .normals()
            .map(|normals| vectors_array(py, normals.iter().map(|normal| normal.into_inner())))
            .transpose()
    }
}
