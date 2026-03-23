use thiserror::Error;

/// The error type for the SyncThing MCP server.
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred during configuration loading or validation.
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// An error occurred during an API request.
    #[error("API error: {0}")]
    Api(reqwest::Error),

    /// A network error occurred (e.g., timeout, connection failure).
    #[error("Network error: {0}")]
    Network(String),

    /// The request was unauthorized (401).
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// The request was forbidden (403).
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// The requested resource was not found (404).
    #[error("Not Found: {0}")]
    NotFound(String),

    /// An error occurred during JSON serialization or deserialization.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),

    /// An error returned by the SyncThing API.
    #[error("SyncThing error: {0}")]
    SyncThing(String),

    /// The specified SyncThing instance was not found.
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    /// A validation error occurred.
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

/// A specialized Result type for SyncThing MCP operations.
pub type Result<T> = std::result::Result<T, Error>;
