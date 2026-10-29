use pyo3::pyclass;

#[pyclass(
    module = "congru",
    frozen,
    eq,
    eq_int,
    hash,
    from_py_object,
    rename_all = "SCREAMING_SNAKE_CASE"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    PointToPoint,
    PointToPlane,
}

impl From<Metric> for congru::Metric {
    fn from(metric: Metric) -> Self {
        match metric {
            Metric::PointToPoint => Self::PointToPoint,
            Metric::PointToPlane => Self::PointToPlane,
        }
    }
}

impl From<congru::Metric> for Metric {
    fn from(metric: congru::Metric) -> Self {
        match metric {
            congru::Metric::PointToPoint => Self::PointToPoint,
            congru::Metric::PointToPlane => Self::PointToPlane,
        }
    }
}
