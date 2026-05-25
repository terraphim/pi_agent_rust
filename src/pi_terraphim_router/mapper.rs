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
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.6".to_string(),
                    confidence: 0.95,
                },
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.5".to_string(),
                    confidence: 0.93,
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
            vec![ProviderSelection {
                provider: "kimi-for-coding".to_string(),
                model: "kimi-k2.6".to_string(),
                confidence: 0.92,
            }],
        );

        // Fast thinking: quick/simple tasks
        mappings.insert(
            Capability::FastThinking,
            vec![
                ProviderSelection {
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.5".to_string(),
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
                    provider: "kimi-for-coding".to_string(),
                    model: "kimi-k2.6".to_string(),
                    confidence: 0.90,
                },
                ProviderSelection {
                    provider: "openai-codex".to_string(),
                    model: "gpt-5.3-codex-spark".to_string(),
                    confidence: 0.88,
                },
            ],
        );

        // Architecture: system design
        mappings.insert(
            Capability::Architecture,
            vec![ProviderSelection {
                provider: "kimi-for-coding".to_string(),
                model: "kimi-k2.6".to_string(),
                confidence: 0.93,
            }],
        );

        // Performance: optimization tasks
        mappings.insert(
            Capability::Performance,
            vec![ProviderSelection {
                provider: "kimi-for-coding".to_string(),
                model: "kimi-k2.6".to_string(),
                confidence: 0.88,
            }],
        );

        // Code review: review tasks
        mappings.insert(
            Capability::CodeReview,
            vec![ProviderSelection {
                provider: "kimi-for-coding".to_string(),
                model: "kimi-k2.6".to_string(),
                confidence: 0.90,
            }],
        );

        // Refactoring: restructuring tasks
        mappings.insert(
            Capability::Refactoring,
            vec![ProviderSelection {
                provider: "kimi-for-coding".to_string(),
                model: "kimi-k2.6".to_string(),
                confidence: 0.89,
            }],
        );

        // Documentation: doc generation
        mappings.insert(
            Capability::Documentation,
            vec![ProviderSelection {
                provider: "kimi-for-coding".to_string(),
                model: "kimi-k2.5".to_string(),
                confidence: 0.87,
            }],
        );

        // Explanation: teaching/clarification
        mappings.insert(
            Capability::Explanation,
            vec![ProviderSelection {
                provider: "kimi-for-coding".to_string(),
                model: "kimi-k2.5".to_string(),
                confidence: 0.88,
            }],
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
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
    }

    #[test]
    fn test_map_multiple_capabilities() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::CodeGeneration, Capability::SecurityAudit];
        let selection = mapper.map(&caps);
        assert!(selection.is_some());
        // CodeGeneration kimi-for-coding (0.95) vs SecurityAudit anthropic (0.92)
        let sel = selection.unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
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

    #[test]
    fn test_map_security_audit() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::SecurityAudit];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
        assert!((sel.confidence - 0.92).abs() < f32::EPSILON);
    }

    #[test]
    fn test_map_fast_thinking() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::FastThinking];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.5");
    }

    #[test]
    fn test_map_testing() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::Testing];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
    }

    #[test]
    fn test_map_architecture() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::Architecture];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
    }

    #[test]
    fn test_map_performance() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::Performance];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
    }

    #[test]
    fn test_map_code_review() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::CodeReview];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
    }

    #[test]
    fn test_map_refactoring() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::Refactoring];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.6");
    }

    #[test]
    fn test_map_documentation() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::Documentation];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.5");
    }

    #[test]
    fn test_map_explanation() {
        let mapper = ProviderMapper::new();
        let caps = vec![Capability::Explanation];
        let sel = mapper.map(&caps).unwrap();
        assert_eq!(sel.provider, "kimi-for-coding");
        assert_eq!(sel.model, "kimi-k2.5");
    }

    #[test]
    fn test_all_capability_names_resolvable() {
        let mapper = ProviderMapper::new();
        let all_names = [
            "DeepThinking",
            "FastThinking",
            "CodeGeneration",
            "CodeReview",
            "Architecture",
            "Testing",
            "Refactoring",
            "Documentation",
            "Explanation",
            "SecurityAudit",
            "Performance",
        ];
        for name in &all_names {
            assert!(
                mapper.get_by_name(name).is_some(),
                "capability {name} should be resolvable"
            );
        }
    }

    #[test]
    fn test_confidence_range() {
        let mapper = ProviderMapper::new();
        let all_caps = [
            Capability::DeepThinking,
            Capability::FastThinking,
            Capability::CodeGeneration,
            Capability::CodeReview,
            Capability::Architecture,
            Capability::Testing,
            Capability::Refactoring,
            Capability::Documentation,
            Capability::Explanation,
            Capability::SecurityAudit,
            Capability::Performance,
        ];
        for cap in &all_caps {
            let sel = mapper.map(&[*cap]).unwrap();
            assert!(
                (0.0..=1.0).contains(&sel.confidence),
                "confidence for {cap:?} should be 0.0-1.0, got {}",
                sel.confidence
            );
        }
    }

    #[test]
    fn test_default_trait() {
        let default = ProviderMapper::default();
        let explicit = ProviderMapper::new();
        let sel_default = default.map(&[Capability::CodeGeneration]).unwrap();
        let sel_explicit = explicit.map(&[Capability::CodeGeneration]).unwrap();
        assert_eq!(sel_default.provider, sel_explicit.provider);
        assert_eq!(sel_default.model, sel_explicit.model);
    }
}
