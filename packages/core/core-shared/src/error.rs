use thiserror::Error;
use tracing::error;

#[derive(Debug, Error, Clone)]
pub enum OrbitError {
    #[error("Nickname is already in use")]
    NickTaken,

    #[error("{0}")]
    SaslFailed(String),

    #[error("Capability '{0}' is not enabled on this server")]
    CapabilityDisabled(&'static str),

    #[error("Not found")]
    NotFound,

    #[error("{0}")]
    Generic(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<anyhow::Error> for OrbitError {
    fn from(error: anyhow::Error) -> Self {
        let err = error.chain().skip(1).fold(error.to_string(), |acc, cause| {
            format!("{}: {}\n", acc, cause)
        });
        error!("Unexpected Orbit error: {}", err);

        Self::Unknown(error.to_string())
    }
}
