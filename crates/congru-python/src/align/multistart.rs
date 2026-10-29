use pyo3::prelude::*;

use super::Icp;
use crate::arrays::Points;
use crate::error::registration_error;
use crate::results::Alignment;
use crate::target::Target;

fn defaults() -> congru::Multistart {
    congru::Multistart::default()
}

#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Multistart(congru::Multistart);

#[pymethods]
impl Multistart {
    #[new]
    #[pyo3(signature = (*, starts = defaults().starts, icp = Icp(defaults().icp)))]
    const fn new(starts: usize, icp: Icp) -> Self {
        Self(congru::Multistart { starts, icp: icp.0 })
    }

    fn align(&self, py: Python<'_>, source: Points, target: &Target) -> PyResult<Alignment> {
        let (multistart, Points(source)) = (self.0, source);
        py.detach(|| multistart.align(&source, target.core()))
            .map(Alignment)
            .map_err(registration_error)
    }

    #[getter]
    const fn starts(&self) -> usize {
        self.0.starts
    }

    #[getter]
    const fn icp(&self) -> Icp {
        Icp(self.0.icp)
    }
}
