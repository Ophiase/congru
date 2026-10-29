use pyo3::PyErr;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;

create_exception!(
    recalage,
    RegistrationError,
    PyValueError,
    "The input was refused, or the motion could not be determined from it."
);

#[expect(
    clippy::needless_pass_by_value,
    reason = "taken by value to be used as `map_err(registration_error)`"
)]
pub fn registration_error(error: recalage::Error) -> PyErr {
    RegistrationError::new_err(error.to_string())
}
