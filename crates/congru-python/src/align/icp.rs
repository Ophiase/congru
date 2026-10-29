use congru::{Convergence, Distance};
use pyo3::prelude::*;

use super::units::{distance, fraction};
use super::{Kernel, Metric};
use crate::arrays::{Points, Transform};
use crate::error::registration_error;
use crate::results::Alignment;
use crate::target::Target;

fn defaults() -> congru::Icp {
    congru::Icp::default()
}

#[pyclass(module = "congru", frozen, from_py_object)]
#[derive(Debug, Clone, Copy)]
pub struct Icp(pub congru::Icp);

#[pymethods]
impl Icp {
    #[new]
    #[pyo3(signature = (
        *,
        metric = defaults().metric.into(),
        kernel = Kernel(defaults().kernel),
        max_correspondence_distance = defaults().max_correspondence_distance.map(Distance::get),
        overlap = defaults().overlap.get(),
        max_iterations = defaults().convergence.max_iterations,
    ))]
    fn new(
        metric: Metric,
        kernel: Kernel,
        max_correspondence_distance: Option<f64>,
        overlap: f64,
        max_iterations: u32,
    ) -> PyResult<Self> {
        Ok(Self(congru::Icp {
            metric: metric.into(),
            kernel: kernel.0,
            max_correspondence_distance: max_correspondence_distance.map(distance).transpose()?,
            overlap: fraction(overlap)?,
            convergence: Convergence {
                max_iterations,
                ..defaults().convergence
            },
        }))
    }

    /// `initial` is a `(4, 4)` transform, the identity when `None`.
    #[pyo3(signature = (source, target, initial = None))]
    fn align(
        &self,
        py: Python<'_>,
        source: Points,
        target: &Target,
        initial: Option<Transform>,
    ) -> PyResult<Alignment> {
        let initial = initial.map(|initial| initial.0).unwrap_or_default();
        let (icp, Points(source)) = (self.0, source);
        py.detach(|| icp.align(&source, target.core(), &initial))
            .map(Alignment)
            .map_err(registration_error)
    }

    #[getter]
    fn metric(&self) -> Metric {
        self.0.metric.into()
    }

    #[getter]
    const fn kernel(&self) -> Kernel {
        Kernel(self.0.kernel)
    }

    #[getter]
    fn max_correspondence_distance(&self) -> Option<f64> {
        self.0.max_correspondence_distance.map(Distance::get)
    }

    #[getter]
    const fn overlap(&self) -> f64 {
        self.0.overlap.get()
    }

    #[getter]
    const fn max_iterations(&self) -> u32 {
        self.0.convergence.max_iterations
    }
}
