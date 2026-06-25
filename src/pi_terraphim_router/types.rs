use serde::Serialize;
use std::path::PathBuf;
use terraphim_types::RouteDirective;

#[derive(Debug, Clone, Serialize)]
pub struct RouteDecision {
    pub provider: String,
    pub model: String,
    pub action: Option<String>,
    pub confidence: f64,
    pub matched_concept: String,
    pub priority: u8,
    pub fallback_routes: Vec<RouteDirective>,
    pub provider_ready: bool,
}

impl RouteDecision {
    pub fn render_action(&self, prompt: &str) -> Option<String> {
        self.action.as_ref().map(|template| {
            template
                .replace("{{ model }}", &self.model)
                .replace("{{model}}", &self.model)
                .replace("{{ prompt }}", prompt)
                .replace("{{prompt}}", prompt)
        })
    }
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub taxonomy_path: Option<PathBuf>,
    pub use_embedded_fallback: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            taxonomy_path: dirs::home_dir()
                .map(|h| h.join(".config").join("pi").join("routing_taxonomy")),
            use_embedded_fallback: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouterInput {
    pub prompt: String,
    pub strategy: Option<String>,
    pub preferred_provider: Option<String>,
    pub preferred_model: Option<String>,
    pub system_prompt: Option<String>,
    pub working_dir: Option<PathBuf>,
}

impl RouterInput {
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

    #[must_use]
    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    #[must_use]
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.preferred_provider = Some(provider.into());
        self
    }

    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.preferred_model = Some(model.into());
        self
    }

    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    #[must_use]
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RouterOutput {
    pub response: String,
    pub provider: String,
    pub model: String,
    pub capabilities: Vec<String>,
    pub confidence: f64,
    pub reason: String,
    pub fallback_used: bool,
}

#[derive(Debug, Clone)]
pub struct ProviderSelection {
    pub provider: String,
    pub model: String,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_router_input_new() {
        let input = RouterInput::new("test prompt");
        assert_eq!(input.prompt, "test prompt");
        assert!(input.strategy.is_none());
        assert!(input.preferred_provider.is_none());
        assert!(input.preferred_model.is_none());
        assert!(input.system_prompt.is_none());
        assert!(input.working_dir.is_none());
    }

    #[test]
    fn test_router_input_builder_pattern() {
        let input = RouterInput::new("test")
            .with_strategy("latency_optimized")
            .with_provider("anthropic")
            .with_model("claude-sonnet-4-6")
            .with_system_prompt("You are helpful")
            .with_working_dir(PathBuf::from("/tmp"));

        assert_eq!(input.strategy.as_deref(), Some("latency_optimized"));
        assert_eq!(input.preferred_provider.as_deref(), Some("anthropic"));
        assert_eq!(input.preferred_model.as_deref(), Some("claude-sonnet-4-6"));
        assert_eq!(input.system_prompt.as_deref(), Some("You are helpful"));
        assert_eq!(input.working_dir.as_deref(), Some(Path::new("/tmp")));
    }

    #[test]
    fn test_route_decision_render_action() {
        let decision = RouteDecision {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            action: Some("pi --model {{ model }} -p \"{{ prompt }}\"".to_string()),
            confidence: 0.8,
            matched_concept: "implementation_tier".to_string(),
            priority: 50,
            fallback_routes: vec![],
            provider_ready: true,
        };

        let rendered = decision.render_action("implement auth").unwrap();
        assert_eq!(
            rendered,
            "pi --model claude-sonnet-4-6 -p \"implement auth\""
        );
    }

    #[test]
    fn test_route_decision_render_action_no_spaces() {
        let decision = RouteDecision {
            provider: "kimi".to_string(),
            model: "k2p5".to_string(),
            action: Some("run {{model}} {{prompt}}".to_string()),
            confidence: 0.5,
            matched_concept: "test".to_string(),
            priority: 50,
            fallback_routes: vec![],
            provider_ready: true,
        };

        let rendered = decision.render_action("hello").unwrap();
        assert_eq!(rendered, "run k2p5 hello");
    }

    #[test]
    fn test_route_decision_render_action_none() {
        let decision = RouteDecision {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            action: None,
            confidence: 0.8,
            matched_concept: "test".to_string(),
            priority: 50,
            fallback_routes: vec![],
            provider_ready: true,
        };

        assert!(decision.render_action("test").is_none());
    }

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert!(config.use_embedded_fallback);
        assert!(config.taxonomy_path.is_some());
    }

    #[test]
    fn test_provider_selection_fields() {
        let sel = ProviderSelection {
            provider: "test-provider".to_string(),
            model: "test-model".to_string(),
            confidence: 0.85,
        };
        assert_eq!(sel.provider, "test-provider");
        assert_eq!(sel.model, "test-model");
        assert!((sel.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_router_output_serialization() {
        let output = RouterOutput {
            response: "hello".to_string(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            capabilities: vec!["CodeGeneration".to_string()],
            confidence: 0.95,
            reason: "capability match".to_string(),
            fallback_used: false,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("anthropic"));
        assert!(json.contains("claude-sonnet-4-6"));
        assert!(json.contains("CodeGeneration"));
    }
}
