//! Error types for the Terraphim router integration.

/// Errors that can occur during routing.
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    /// No provider found for the extracted capabilities.
    #[error("no provider found for capabilities: {0:?}")]
    NoProviderFound(Vec<String>),

    /// Provider not ready (missing credentials).
    #[error("provider not ready (missing credentials): {provider}/{model}")]
    ProviderNotReady { provider: String, model: String },

    /// RPC communication with pi-rust failed.
    #[error("RPC communication failed: {0}")]
    RpcError(String),

    /// pi-rust subprocess failed.
    #[error("pi-rust subprocess failed: {0}")]
    SubprocessError(String),

    /// Invalid capability name.
    #[error("invalid capability: {0}")]
    InvalidCapability(String),

    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Result type for router operations.
pub type RouterResult<T> = Result<T, RouterError>;
