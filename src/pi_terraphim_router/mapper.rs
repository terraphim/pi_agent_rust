//! Maps extracted capabilities to optimal pi-rust provider/model combinations.

use std::collections::HashMap;
use terraphim_types::capability::Capability;

use crate::pi_terraphim_router::types::ProviderSelection;

/// Maps capabilities to optimal provider/model selections.
pub struct ProviderMapper {
    mappings: HashMap<Capability, Vec<ProviderSelection>>,
}

impl ProviderMapper {
    /// Create a new provider mapper with default mappings.
    #[allow(clippy::too_many_lines)]
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Deep thinking: reasoning-intensive tasks
        mappings.insert(
            Capability::DeepThinking,
            vec![
                ProviderSelection {
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.6".to_string(),
                    confidence: 0.95,
                },
                ProviderSelection {
                    provider: "anthropic".to_string(),
                    model: "claude-opus-4-6".to_string(),
                    confidence: 0.90,
                },
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.5".to_string(),
                    confidence: 0.88,
                },
            ],
        );

        // Code generation: implementation tasks
        mappings.insert(
            Capability::CodeGeneration,
            vec![
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.5".to_string(),
                    confidence: 0.95,
                },
                ProviderSelection {
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.5".to_string(),
                    confidence: 0.90,
                },
                ProviderSelection {
                    provider: "zai".to_string(),
                    model: "glm-5.1".to_string(),
                    confidence: 0.85,
                },
            ],
        );

        // Security audit: security-focused tasks
        mappings.insert(
            Capability::SecurityAudit,
            vec![
                ProviderSelection {
                    provider: "anthropic".to_string(),
                    model: "claude-sonnet-4-6".to_string(),
                    confidence: 0.92,
                },
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.5".to_string(),
                    confidence: 0.88,
                },
            ],
        );

        // Fast thinking: quick/simple tasks
        mappings.insert(
            Capability::FastThinking,
            vec![
                ProviderSelection {
                    provider: "google".to_string(),
                    model: "gemini-3-flash".to_string(),
                    confidence: 0.92,
                },
                ProviderSelection {
                    provider: "openai".to_string(),
                    model: "gpt-5.4".to_string(),
                    confidence: 0.88,
                },
            ],
        );

        // Testing: test generation
        mappings.insert(
            Capability::Testing,
            vec![
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.3-codex-spark".to_string(),
                    confidence: 0.90,
                },
                ProviderSelection {
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.5".to_string(),
                    confidence: 0.85,
                },
            ],
        );

        // Architecture: system design
        mappings.insert(
            Capability::Architecture,
            vec![
                ProviderSelection {
                    provider: "anthropic".to_string(),
                    model: "claude-opus-4-6".to_string(),
                    confidence: 0.93,
                },
                ProviderSelection {
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.6".to_string(),
                    confidence: 0.88,
                },
            ],
        );

        // Performance: optimization tasks
        mappings.insert(
            Capability::Performance,
            vec![
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.5".to_string(),
                    confidence: 0.88,
                },
                ProviderSelection {
                    provider: "anthropic".to_string(),
                    model: "claude-sonnet-4-6".to_string(),
                    confidence: 0.85,
                },
            ],
        );

        // Code review: review tasks
        mappings.insert(
            Capability::CodeReview,
            vec![
                ProviderSelection {
                    provider: "anthropic".to_string(),
                    model: "claude-sonnet-4-6".to_string(),
                    confidence: 0.90,
                },
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.4".to_string(),
                    confidence: 0.87,
                },
            ],
        );

        // Refactoring: restructuring tasks
        mappings.insert(
            Capability::Refactoring,
            vec![
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.5".to_string(),
                    confidence: 0.89,
                },
                ProviderSelection {
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.5".to_string(),
                    confidence: 0.85,
                },
            ],
        );

        // Documentation: doc generation
        mappings.insert(
            Capability::Documentation,
            vec![
                ProviderSelection {
                    provider: "anthropic".to_string(),
                    model: "claude-sonnet-4-6".to_string(),
                    confidence: 0.87,
                },
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.4".to_string(),
                    confidence: 0.84,
                },
            ],
        );

        // Explanation: teaching/clarification
        mappings.insert(
            Capability::Explanation,
            vec![
                ProviderSelection {
                    provider: "anthropic".to_string(),
                    model: "claude-sonnet-4-6".to_string(),
                    confidence: 0.88,
                },
                ProviderSelection {
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.5".to_string(),
                    confidence: 0.84,
                },
            ],
        );

        Self { mappings }
    }

    /// Map extracted capabilities to the best provider selection.
    ///
    /// Selects the highest confidence provider across all capabilities.
    pub fn map(&self, capabilities: &[Capability]) -> Option<ProviderSelection> {
        capabilities
            .iter()
            .filter_map(|cap| self.mappings.get(cap))
            .flat_map(|selections| selections.iter().cloned())
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get provider selection by capability name.
    pub fn get_by_name(&self, capability_name: &str) -> Option<ProviderSelection> {
        let capability = match capability_name {
            "DeepThinking" => Capability::DeepThinking,
            "FastThinking" => Capability::FastThinking,
            "CodeGeneration" => Capability::CodeGeneration,
            "CodeReview" => Capability::CodeReview,
            "Architecture" => Capability::Architecture,
            "Testing" => Capability::Testing,
            "Refactoring" => Capability::Refactoring,
            "Documentation" => Capability::Documentation,
            "Explanation" => Capability::Explanation,
            "SecurityAudit" => Capability::SecurityAudit,
            "Performance" => Capability::Performance,
            _ => return None,
        };

        self.mappings
            .get(&capability)
            .and_then(|selections| selections.first().cloned())
    }

    /// Get the fallback selection when no capabilities match.
    pub fn fallback_selection() -> ProviderSelection {
        ProviderSelection {
            provider: "openai-codex".to_string(),
            model: "gpt-5.5".to_string(),
            confidence: 0.0,
        }
    }
}

impl Default for ProviderMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_deep_thinking() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::DeepThinking];
        let selection = mapper.map(&caps);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
    }

    #[test]
    fn test_map_code_generation() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::CodeGeneration];
        let selection = mapper.map(&caps);
        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.provider, "openai-codex");
        assert_eq!(sel.model, "gpt-5.5");
    }

    #[test]
    fn test_map_multiple_capabilities() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::CodeGeneration, Capability::SecurityAudit];
        let selection = mapper.map(&caps);
        assert!(selection.is_some());
        // CodeGeneration has higher confidence (0.95 vs 0.92)
        let sel = selection.unwrap();
        assert_eq!(sel.provider, "openai-codex");
        assert_eq!(sel.model, "gpt-5.5");
    }

    #[test]
    fn test_fallback() {
        let mapper = ProviderMapper::new();
        let caps: Vec<Capability> = vec![];
        let selection = mapper.map(&caps);
        assert!(selection.is_none());

        let fallback = ProviderMapper::fallback_selection();
        assert_eq!(fallback.provider, "openai-codex");
        assert_eq!(fallback.model, "gpt-5.5");
    }

    #[test]
    fn test_get_by_name() {
        let mapper = ProviderMapper::new();
        let selection = mapper.get_by_name("DeepThinking");
        assert!(selection.is_some());
        assert_eq!(selection.unwrap().provider, "kimi-for-coding");

        assert!(mapper.get_by_name("UnknownCapability").is_none());
    }
}
