use pyo3::PyResult;
use congru::{Distance, Fraction};

use crate::error::registration_error;

pub fn distance(value: f64) -> PyResult<Distance> {
    Distance::new(value).map_err(registration_error)
}

pub fn fraction(value: f64) -> PyResult<Fraction> {
    Fraction::new(value).map_err(registration_error)
}
