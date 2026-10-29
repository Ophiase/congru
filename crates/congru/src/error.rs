#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("{what} is empty")]
    Empty { what: &'static str },
    #[error("coordinate {index} is not finite")]
    NonFinite { index: usize },
    #[error("{length} coordinates do not make whole 3D points")]
    Ragged { length: usize },
    #[error("{what}: {left} against {right}")]
    LengthMismatch {
        what: &'static str,
        left: usize,
        right: usize,
    },
    #[error("triangle {triangle} refers to a missing vertex")]
    DanglingTriangle { triangle: usize },
    #[error("{name} must be {expected}, got {value}")]
    InvalidParameter {
        name: &'static str,
        expected: &'static str,
        value: f64,
    },
    #[error("{found} usable correspondences, at least {needed} needed")]
    TooFewCorrespondences { found: usize, needed: usize },
    #[error("degenerate geometry: the rigid motion is not determined")]
    Degenerate,
    #[error("point-to-plane needs target normals")]
    MissingNormals,
}
