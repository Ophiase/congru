use std::borrow::Cow;
use std::fmt;

use numpy::{AllowTypeChange, Element, PyArrayLikeDyn, PyUntypedArrayMethods};
use pyo3::conversion::FromPyObjectOwned;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// The `(rows, columns)` an argument must have; `rows` is `None` when any
/// count is accepted.
#[derive(Debug, Clone, Copy)]
pub struct Shape {
    pub rows: Option<usize>,
    pub columns: usize,
}

impl Shape {
    fn admits(self, shape: &[usize]) -> bool {
        matches!(shape, [rows, columns]
            if *columns == self.columns && self.rows.is_none_or(|expected| expected == *rows))
    }

    /// The entries of `object` in row-major order, whatever its memory
    /// layout, converted to `T` by NumPy if need be. Borrowed without a copy
    /// from a C-contiguous array of `T`.
    pub fn read<'py, T, Output>(
        self,
        object: Borrowed<'_, 'py, PyAny>,
        convert: impl FnOnce(&[T]) -> PyResult<Output>,
    ) -> PyResult<Output>
    where
        T: Element + Copy + FromPyObjectOwned<'py> + 'py,
    {
        let array: PyArrayLikeDyn<'py, T, AllowTypeChange> = object.extract()?;
        if !self.admits(array.shape()) {
            let found: Vec<String> = array.shape().iter().map(usize::to_string).collect();
            return Err(PyValueError::new_err(format!(
                "expected an array of shape {self}, got ({})",
                found.join(", ")
            )));
        }
        let view = array.as_array();
        let entries = view
            .to_slice()
            .map_or_else(|| Cow::Owned(view.iter().copied().collect()), Cow::Borrowed);
        convert(&entries)
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.rows {
            Some(rows) => write!(formatter, "({rows}, {})", self.columns),
            None => write!(formatter, "(n, {})", self.columns),
        }
    }
}
