use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::motion::{Rotation, Translation};

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Observability(pub congru::Observability);

#[pymethods]
impl Observability {
    #[getter]
    const fn conditioning(&self) -> f64 {
        self.0.conditioning
    }

    #[getter]
    fn weak_motions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.0.weak_motions.iter().copied().map(WeakMotion))
    }
}

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct WeakMotion(congru::WeakMotion);

#[pymethods]
impl WeakMotion {
    #[getter]
    const fn stiffness(&self) -> f64 {
        self.0.stiffness
    }

    /// A `Rotation` or a `Translation`.
    #[getter]
    fn kind<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self.0.kind {
            congru::MotionKind::Rotation { axis, through } => Ok(Bound::new(
                py,
                Rotation {
                    axis: axis.into_inner(),
                    through: through.coords,
                },
            )?
            .into_any()),
            congru::MotionKind::Translation { direction } => Ok(Bound::new(
                py,
                Translation {
                    direction: direction.into_inner(),
                },
            )?
            .into_any()),
        }
    }
}
