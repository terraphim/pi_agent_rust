#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("no provider found for capabilities: {0:?}")]
    NoProviderFound(Vec<String>),

    #[error("provider not ready (missing credentials): {provider}/{model}")]
    ProviderNotReady { provider: String, model: String },

    #[error("RPC communication failed: {0}")]
    RpcError(String),

    #[error("pi-rust subprocess failed: {0}")]
    SubprocessError(String),

    #[error("invalid capability: {0}")]
    InvalidCapability(String),

    #[error("taxonomy directory not found: {0}")]
    TaxonomyNotFound(String),

    #[error("failed to parse taxonomy: {0}")]
    ParseError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

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
    fn test_taxonomy_not_found_display() {
        let err = RouterError::TaxonomyNotFound("/path/to/taxonomy".to_string());
        assert!(err.to_string().contains("/path/to/taxonomy"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = RouterError::ParseError("bad format".to_string());
        assert!(err.to_string().contains("bad format"));
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
