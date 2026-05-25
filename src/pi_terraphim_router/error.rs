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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_provider_found_display() {
        let err = RouterError::NoProviderFound(vec!["CodeGeneration".to_string()]);
        let msg = err.to_string();
        assert!(msg.contains("no provider found"));
        assert!(msg.contains("CodeGeneration"));
    }

    #[test]
    fn test_provider_not_ready_display() {
        let err = RouterError::ProviderNotReady {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("anthropic"));
        assert!(msg.contains("claude-sonnet-4-6"));
    }

    #[test]
    fn test_rpc_error_display() {
        let err = RouterError::RpcError("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn test_subprocess_error_display() {
        let err = RouterError::SubprocessError("spawn failed".to_string());
        assert!(err.to_string().contains("spawn failed"));
    }

    #[test]
    fn test_invalid_capability_display() {
        let err = RouterError::InvalidCapability("Unknown".to_string());
        assert!(err.to_string().contains("Unknown"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let router_err: RouterError = io_err.into();
        assert!(matches!(router_err, RouterError::Io(_)));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let router_err: RouterError = json_err.into();
        assert!(matches!(router_err, RouterError::Json(_)));
    }
}
