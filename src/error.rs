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

/// Diagnostic information for an error.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    /// The category of the error (e.g., Network, Permission).
    pub category: String,
    /// A human-readable explanation of the error.
    pub explanation: String,
    /// Actionable advice for the user or AI agent.
    pub advice: String,
}

impl Error {
    /// Diagnoses the error and returns a structured diagnostic.
    pub fn diagnose(&self) -> Diagnostic {
        match self {
            Error::Unauthorized(_) => Diagnostic {
                category: "Permission".to_string(),
                explanation: "Authentication failed.".to_string(),
                advice: "API key is missing or invalid. Check your configuration.".to_string(),
            },
            Error::Forbidden(msg) => {
                let advice = if msg.contains("CSRF") {
                    "CSRF protection is active. Ensure you're using an API key, as it bypasses CSRF checks.".to_string()
                } else {
                    "Access is forbidden. You might be trying to access a restricted endpoint.".to_string()
                };
                Diagnostic {
                    category: "Permission".to_string(),
                    explanation: format!("Permission denied: {}", msg),
                    advice,
                }
            }
            Error::NotFound(_) => Diagnostic {
                category: "Configuration".to_string(),
                explanation: "The requested resource or endpoint was not found.".to_string(),
                advice: "Verify the ID and endpoint. List folders/devices to see valid IDs.".to_string(),
            },
            Error::Network(msg) => {
                let advice = if msg.contains("refused") {
                    "SyncThing instance is not running or is listening on a different port.".to_string()
                } else if msg.contains("timeout") || msg.contains("deadline exceeded") {
                    "The request took too long. Check if the server is under heavy load or network is unstable.".to_string()
                } else {
                    "Check your network connection and the SyncThing server URL.".to_string()
                };
                Diagnostic {
                    category: "Network".to_string(),
                    explanation: format!("Network error: {}", msg),
                    advice,
                }
            }
            Error::SyncThing(msg) => {
                if msg.contains("folder") && msg.contains("not found") {
                    Diagnostic {
                        category: "Configuration".to_string(),
                        explanation: format!("SyncThing error: {}", msg),
                        advice: "Specified folder ID is incorrect. List folders to see valid IDs.".to_string(),
                    }
                } else if msg.contains("device") && msg.contains("not found") {
                    Diagnostic {
                        category: "Configuration".to_string(),
                        explanation: format!("SyncThing error: {}", msg),
                        advice: "Specified device ID is incorrect. List devices to see valid IDs.".to_string(),
                    }
                } else if msg.contains("disk space") {
                    Diagnostic {
                        category: "Resource".to_string(),
                        explanation: format!("SyncThing error: {}", msg),
                        advice: "SyncThing cannot write data. Check disk space on the target machine.".to_string(),
                    }
                } else {
                    Diagnostic {
                        category: "Internal".to_string(),
                        explanation: format!("SyncThing technical error: {}", msg),
                        advice: "Inspect the SyncThing logs for more details.".to_string(),
                    }
                }
            }
            _ => Diagnostic {
                category: "Internal".to_string(),
                explanation: self.to_string(),
                advice: "Inspect the logs and check the SyncThing instance status.".to_string(),
            },
        }
    }
}

/// A specialized Result type for SyncThing MCP operations.
pub type Result<T> = std::result::Result<T, Error>;
