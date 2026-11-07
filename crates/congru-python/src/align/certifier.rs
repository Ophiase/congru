use pyo3::prelude::*;

use super::units::distance;
use crate::arrays::{Points, Transform};
use crate::error::registration_error;
use crate::results::Certificate;

/// A search with a proof, for a source of tens of points: see `congru::Certifier`.
#[pyclass(module = "congru", frozen)]
#[derive(Debug)]
pub struct Certifier(congru::Certifier);

#[pymethods]
impl Certifier {
    #[new]
    #[pyo3(signature = (*, truncation, reach, margin = 0.0, rotation_resolution = 0.01, translation_resolution = 0.001, budget = 3_000_000))]
    fn new(
        truncation: f64,
        reach: f64,
        margin: f64,
        rotation_resolution: f64,
        translation_resolution: f64,
        budget: usize,
    ) -> PyResult<Self> {
        Ok(Self(congru::Certifier {
            truncation: distance(truncation)?,
            reach: distance(reach)?,
            margin,
            resolution: [rotation_resolution, translation_resolution],
            budget,
        }))
    }

    /// `seeds` are `(4, 4)` transforms, the answers to grade.
    #[pyo3(signature = (source, target, seeds = Vec::new()))]
    fn run(
        &self,
        py: Python<'_>,
        source: Points,
        target: Points,
        seeds: Vec<Transform>,
    ) -> PyResult<Certificate> {
        let (certifier, Points(source), Points(target)) = (self.0, source, target);
        let seeds: Vec<congru::RigidTransform> = seeds.into_iter().map(|seed| seed.0).collect();
        py.detach(|| certifier.run(&source, &target, &seeds))
            .map(Certificate)
            .map_err(registration_error)
    }
}
