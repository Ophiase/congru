use congru::{DistanceField, MeshIndex, TriangleMesh};
use pyo3::prelude::*;

use super::Icp;
use super::units::distance;
use crate::arrays::{Points, Transform, Triangles};
use crate::error::registration_error;
use crate::results::Alignment;

#[derive(Debug)]
enum Held {
    Exact(MeshIndex),
    Field(DistanceField),
}

/// A mesh held for alignment: exact, or a distance field, faster once built.
#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Surface(Held);

fn mesh_of(vertices: Points, triangles: Triangles) -> PyResult<TriangleMesh> {
    let (Points(vertices), Triangles(triangles)) = (vertices, triangles);
    TriangleMesh::new(vertices, triangles).map_err(registration_error)
}

#[pymethods]
impl Surface {
    /// Exact distances, by a tree of boxes: the most accurate, the slowest to query.
    #[staticmethod]
    fn exact(vertices: Points, triangles: Triangles) -> PyResult<Self> {
        let mesh = mesh_of(vertices, triangles)?;
        MeshIndex::new(&mesh)
            .map(|index| Self(Held::Exact(index)))
            .map_err(registration_error)
    }

    /// A signed distance field of cubes of side `voxel`, truthful within `band` of
    /// the surface, which must exceed the alignments' correspondence distance.
    #[staticmethod]
    fn field(vertices: Points, triangles: Triangles, voxel: f64, band: f64) -> PyResult<Self> {
        let mesh = mesh_of(vertices, triangles)?;
        DistanceField::from_mesh(&mesh, distance(voxel)?, distance(band)?)
            .map(|field| Self(Held::Field(field)))
            .map_err(registration_error)
    }

    /// Aligns `source` onto the surface itself with `icp`, from `initial`, a `(4, 4)`
    /// transform, the identity when `None`.
    #[pyo3(signature = (source, icp, initial = None))]
    fn align(
        &self,
        py: Python<'_>,
        source: Points,
        icp: Icp,
        initial: Option<Transform>,
    ) -> PyResult<Alignment> {
        let initial = initial.map(|initial| initial.0).unwrap_or_default();
        let Points(source) = source;
        py.detach(|| match &self.0 {
            Held::Exact(index) => icp.0.align_on(&source, index, &initial),
            Held::Field(field) => icp.0.align_on(&source, field, &initial),
        })
        .map(Alignment)
        .map_err(registration_error)
    }
}
