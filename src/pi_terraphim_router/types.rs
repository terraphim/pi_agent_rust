//! Input and output types for the Terraphim router skill.

use serde::Serialize;
use std::path::PathBuf;

/// Input to the router skill.
#[derive(Debug, Clone)]
pub struct RouterInput {
    /// User prompt.
    pub prompt: String,

    /// Optional strategy override (cost_optimized, latency_optimized, capability_first).
    pub strategy: Option<String>,

    /// Optional preferred provider (bypasses routing).
    pub preferred_provider: Option<String>,

    /// Optional preferred model (bypasses routing).
    pub preferred_model: Option<String>,

    /// Optional system prompt.
    pub system_prompt: Option<String>,

    /// Working directory for pi-rust.
    pub working_dir: Option<PathBuf>,
}

impl RouterInput {
    /// Create a new router input with just a prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            strategy: None,
            preferred_provider: None,
            preferred_model: None,
            system_prompt: None,
            working_dir: None,
        }
    }

    /// Set strategy override.
    #[must_use]
    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    /// Set preferred provider.
    #[must_use]
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.preferred_provider = Some(provider.into());
        self
    }

    /// Set preferred model.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.preferred_model = Some(model.into());
        self
    }

    /// Set system prompt.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set working directory.
    #[must_use]
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }
}

/// Output from the router skill.
#[derive(Debug, Clone, Serialize)]
pub struct RouterOutput {
    /// LLM response text.
    pub response: String,

    /// Selected provider.
    pub provider: String,

    /// Selected model.
    pub model: String,

    /// Capabilities extracted from prompt.
    pub capabilities: Vec<String>,

    /// Routing confidence (0.0 - 1.0).
    pub confidence: f32,

    /// Routing reason.
    pub reason: String,

    /// Whether fallback was used.
    pub fallback_used: bool,
}

/// Selection of a provider and model with confidence.
#[derive(Debug, Clone)]
pub struct ProviderSelection {
    /// Provider ID (e.g., "anthropic", "openai-codex").
    pub provider: String,

    /// Model ID (e.g., "claude-sonnet-4-6", "gpt-5.5").
    pub model: String,

    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
}
