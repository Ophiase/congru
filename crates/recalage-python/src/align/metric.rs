use pyo3::pyclass;

#[pyclass(
    module = "recalage",
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

impl From<Metric> for recalage::Metric {
    fn from(metric: Metric) -> Self {
        match metric {
            Metric::PointToPoint => Self::PointToPoint,
            Metric::PointToPlane => Self::PointToPlane,
        }
    }
}

impl From<recalage::Metric> for Metric {
    fn from(metric: recalage::Metric) -> Self {
        match metric {
            recalage::Metric::PointToPoint => Self::PointToPoint,
            recalage::Metric::PointToPlane => Self::PointToPlane,
        }
    }
}
