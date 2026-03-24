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
/// Supported languages for diagnostic messages.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
pub enum Language {
    /// English (default).
    #[default]
    English,
    /// French.
    French,
}

impl Language {
    /// Returns the language from a string (e.g., "en", "fr").
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fr" | "french" => Language::French,
            _ => Language::English,
        }
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
    /// Diagnoses the error and returns a structured diagnostic in English.
    pub fn diagnose(&self) -> Diagnostic {
        self.diagnose_with_language(Language::English)
    }

    /// Diagnoses the error and returns a structured diagnostic in the specified language.
    pub fn diagnose_with_language(&self, lang: Language) -> Diagnostic {
        match lang {
            Language::English => self.diagnose_en(),
            Language::French => self.diagnose_fr(),
        }
    }

    fn diagnose_en(&self) -> Diagnostic {
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
                    "Access is forbidden. You might be trying to access a restricted endpoint."
                        .to_string()
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
                advice: "Verify the ID and endpoint. List folders/devices to see valid IDs."
                    .to_string(),
            },
            Error::Network(msg) => {
                let advice = if msg.contains("refused") {
                    "SyncThing instance is not running or is listening on a different port."
                        .to_string()
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
                        advice: "Specified folder ID is incorrect. List folders to see valid IDs."
                            .to_string(),
                    }
                } else if msg.contains("device") && msg.contains("not found") {
                    Diagnostic {
                        category: "Configuration".to_string(),
                        explanation: format!("SyncThing error: {}", msg),
                        advice: "Specified device ID is incorrect. List devices to see valid IDs."
                            .to_string(),
                    }
                } else if msg.contains("disk space") {
                    Diagnostic {
                        category: "Resource".to_string(),
                        explanation: format!("SyncThing error: {}", msg),
                        advice:
                            "SyncThing cannot write data. Check disk space on the target machine."
                                .to_string(),
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

    fn diagnose_fr(&self) -> Diagnostic {
        match self {
            Error::Unauthorized(_) => Diagnostic {
                category: "Permission".to_string(),
                explanation: "L'authentification a échoué.".to_string(),
                advice: "La clé API est manquante ou invalide. Vérifiez votre configuration.".to_string(),
            },
            Error::Forbidden(msg) => {
                let advice = if msg.contains("CSRF") {
                    "La protection CSRF est active. Assurez-vous d'utiliser une clé API, car elle contourne les vérifications CSRF.".to_string()
                } else {
                    "L'accès est interdit. Vous essayez peut-être d'accéder à un point de terminaison restreint.".to_string()
                };
                Diagnostic {
                    category: "Permission".to_string(),
                    explanation: format!("Permission refusée : {}", msg),
                    advice,
                }
            }
            Error::NotFound(_) => Diagnostic {
                category: "Configuration".to_string(),
                explanation: "La ressource ou le point de terminaison demandé n'a pas été trouvé.".to_string(),
                advice: "Vérifiez l'ID et le point de terminaison. Listez les dossiers/périphériques pour voir les IDs valides.".to_string(),
            },
            Error::Network(msg) => {
                let advice = if msg.contains("refused") {
                    "L'instance SyncThing ne fonctionne pas ou écoute sur un port différent.".to_string()
                } else if msg.contains("timeout") || msg.contains("deadline exceeded") {
                    "La requête a pris trop de temps. Vérifiez si le serveur est surchargé ou si le réseau est instable.".to_string()
                } else {
                    "Vérifiez votre connexion réseau et l'URL du serveur SyncThing.".to_string()
                };
                Diagnostic {
                    category: "Network".to_string(),
                    explanation: format!("Erreur réseau : {}", msg),
                    advice,
                }
            }
            Error::SyncThing(msg) => {
                if msg.contains("folder") && msg.contains("not found") {
                    Diagnostic {
                        category: "Configuration".to_string(),
                        explanation: format!("Erreur SyncThing : {}", msg),
                        advice: "L'ID du dossier spécifié est incorrect. Listez les dossiers pour voir les IDs valides.".to_string(),
                    }
                } else if msg.contains("device") && msg.contains("not found") {
                    Diagnostic {
                        category: "Configuration".to_string(),
                        explanation: format!("Erreur SyncThing : {}", msg),
                        advice: "L'ID du périphérique spécifié est incorrect. Listez les périphériques pour voir les IDs valides.".to_string(),
                    }
                } else if msg.contains("disk space") {
                    Diagnostic {
                        category: "Ressource".to_string(),
                        explanation: format!("Erreur SyncThing : {}", msg),
                        advice: "SyncThing ne peut pas écrire de données. Vérifiez l'espace disque sur la machine cible.".to_string(),
                    }
                } else {
                    Diagnostic {
                        category: "Interne".to_string(),
                        explanation: format!("Erreur technique SyncThing : {}", msg),
                        advice: "Inspectez les journaux SyncThing pour plus de détails.".to_string(),
                    }
                }
            }
            _ => Diagnostic {
                category: "Interne".to_string(),
                explanation: self.to_string(),
                advice: "Inspectez les journaux et vérifiez l'état de l'instance SyncThing.".to_string(),
            },
        }
    }
}

/// A specialized Result type for SyncThing MCP operations.
pub type Result<T> = std::result::Result<T, Error>;
