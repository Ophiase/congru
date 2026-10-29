/// Why a scene could not be built.
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    #[error("cannot read {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("{path} is not a valid scene")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error("a scene needs exactly one object marked target = true, found {found}")]
    Target { found: usize },
    #[error(transparent)]
    Geometry(#[from] congru::Error),
}
