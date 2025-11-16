//! Error types for durable-filestor

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DurableError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("TOML parsing error: {source}")]
    Toml {
        #[from]
        source: toml::de::Error,
    },

    #[error("JSON parsing error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("Storage service error: {message}")]
    Storage { message: String },

    #[error("Protocol not found: {protocol}")]
    ProtocolNotFound { protocol: String },

    #[error("Service not found: {service}")]
    ServiceNotFound { service: String },

    #[error("Invalid key translation configuration: {message}")]
    InvalidKeyTranslation { message: String },

    #[error("URI parsing error: {message}")]
    Uri { message: String },

    #[error("Generic error: {message}")]
    Generic { message: String },
}

pub type Result<T> = std::result::Result<T, DurableError>;