use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("API error: {0}")]
    Api(reqwest::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("SyncThing error: {0}")]
    SyncThing(String),

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_status() {
            match err.status() {
                Some(reqwest::StatusCode::UNAUTHORIZED) => Error::Unauthorized(err.to_string()),
                Some(reqwest::StatusCode::FORBIDDEN) => Error::Forbidden(err.to_string()),
                Some(reqwest::StatusCode::NOT_FOUND) => Error::NotFound(err.to_string()),
                _ => Error::Api(err),
            }
        } else if err.is_timeout() || err.is_connect() {
            Error::Network(err.to_string())
        } else {
            Error::Api(err)
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::SyncThing(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
