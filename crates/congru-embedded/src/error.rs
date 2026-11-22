/// Why a fit, an index or an alignment refused its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("no points")]
    Empty,
    #[error("a coordinate or a weight is not finite")]
    NotFinite,
    #[error("{found} points, an index holds fewer than 2^32")]
    TooManyPoints { found: usize },
    #[error("{left} points against {right}")]
    LengthMismatch { left: usize, right: usize },
    #[error("{found} usable pairs, {needed} needed")]
    TooFewPairs { found: usize, needed: usize },
    #[error("the pairs do not determine a motion")]
    Degenerate,
    #[error("point-to-plane needs target normals")]
    MissingNormals,
    #[error("the workspace holds {capacity} points, the source has {needed}")]
    WorkspaceTooSmall { capacity: usize, needed: usize },
    #[error("{name} is out of range")]
    InvalidParameter { name: &'static str },
}
