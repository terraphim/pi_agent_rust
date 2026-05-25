use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use terraphim_automata::{find_matches, parse_markdown_directives_dir};
use terraphim_types::{MarkdownDirectives, NormalizedTerm, NormalizedTermValue, Thesaurus};

pub mod error;
pub mod rpc_client;
pub mod types;

pub use error::{RouterError, RouterResult};
pub use rpc_client::RpcClient;
pub use types::{ProviderSelection, RouteDecision, RouterConfig, RouterInput, RouterOutput};

struct RoutingRule {
    concept: String,
    directives: MarkdownDirectives,
}

pub struct Router {
    rules: Vec<RoutingRule>,
    thesaurus: Thesaurus,
    taxonomy_path: PathBuf,
    last_mtime: Option<SystemTime>,
}

impl std::fmt::Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("taxonomy_path", &self.taxonomy_path)
            .field("rules_count", &self.rules.len())
            .field("thesaurus_size", &self.thesaurus.len())
            .finish_non_exhaustive()
    }
}

const EMBEDDED_PLANNING_TIER: &str =
    include_str!("../../resources/routing_taxonomy/planning_tier.md");
const EMBEDDED_IMPLEMENTATION_TIER: &str =
    include_str!("../../resources/routing_taxonomy/implementation_tier.md");
const EMBEDDED_REVIEW_TIER: &str = include_str!("../../resources/routing_taxonomy/review_tier.md");

impl Router {
    pub fn load(taxonomy_path: impl Into<PathBuf>) -> RouterResult<Self> {
        let taxonomy_path = taxonomy_path.into();
        if !taxonomy_path.exists() {
            return Err(RouterError::TaxonomyNotFound(
                taxonomy_path.display().to_string(),
            ));
        }

        let parse_result = parse_markdown_directives_dir(&taxonomy_path)
            .map_err(|e| RouterError::ParseError(e.to_string()))?;

        let mut rules = Vec::new();
        let mut thesaurus = Thesaurus::new("pi_router".to_string());
        let mut term_id: u64 = 1;

        for (concept, directives) in &parse_result.directives {
            if directives.routes.is_empty() {
                continue;
            }

            for synonym in &directives.synonyms {
                let key = NormalizedTermValue::from(synonym.clone());
                let term = NormalizedTerm {
                    id: term_id,
                    value: NormalizedTermValue::from(concept.clone()),
                    display_value: None,
                    url: None,
                    action: None,
                    priority: None,
                    trigger: None,
                    pinned: false,
                };
                thesaurus.insert(key, term);
                term_id += 1;
            }

            rules.push(RoutingRule {
                concept: concept.clone(),
                directives: directives.clone(),
            });
        }

        let last_mtime = Self::dir_mtime(&taxonomy_path);

        Ok(Self {
            rules,
            thesaurus,
            taxonomy_path,
            last_mtime,
        })
    }

    pub fn route(&self, prompt: &str) -> Option<RouteDecision> {
        if self.thesaurus.is_empty() {
            return None;
        }

        let matches = match find_matches(prompt, self.thesaurus.clone(), false) {
            Ok(m) if !m.is_empty() => m,
            Ok(_) | Err(_) => return None,
        };

        let mut best: Option<(&RoutingRule, f64)> = None;

        for matched in &matches {
            let concept = matched.normalized_term.value.to_string();
            if let Some(rule) = self.rules.iter().find(|r| r.concept == concept) {
                let priority = f64::from(rule.directives.priority.unwrap_or(50));
                match &best {
                    Some((_, best_score)) if priority <= *best_score => {}
                    _ => best = Some((rule, priority)),
                }
            }
        }

        let (rule, score) = best?;
        let primary = rule.directives.routes.first()?;

        Some(RouteDecision {
            provider: primary.provider.clone(),
            model: primary.model.clone(),
            action: primary.action.clone(),
            confidence: score / 100.0,
            matched_concept: rule.concept.clone(),
            priority: rule.directives.priority.unwrap_or(50),
            fallback_routes: rule.directives.routes.clone(),
            provider_ready: true,
        })
    }

    pub fn reload(&mut self) -> RouterResult<()> {
        let reloaded = Self::load(&self.taxonomy_path)?;
        self.rules = reloaded.rules;
        self.thesaurus = reloaded.thesaurus;
        self.last_mtime = reloaded.last_mtime;
        Ok(())
    }

    pub fn reload_if_changed(&mut self) -> bool {
        let current_mtime = Self::dir_mtime(&self.taxonomy_path);
        if current_mtime != self.last_mtime && self.reload().is_ok() {
            return true;
        }
        false
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    fn dir_mtime(path: &Path) -> Option<SystemTime> {
        fs::read_dir(path)
            .ok()?
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext == "md")
            })
            .filter_map(|e| e.metadata().ok()?.modified().ok())
            .max()
    }

    fn from_embedded() -> RouterResult<Self> {
        let tmp_dir = tempfile::tempdir().map_err(|e| RouterError::ParseError(e.to_string()))?;
        let embedded: &[(&str, &str)] = &[
            ("planning_tier.md", EMBEDDED_PLANNING_TIER),
            ("implementation_tier.md", EMBEDDED_IMPLEMENTATION_TIER),
            ("review_tier.md", EMBEDDED_REVIEW_TIER),
        ];

        for (name, content) in embedded {
            fs::write(tmp_dir.path().join(name), content)
                .map_err(|e| RouterError::ParseError(e.to_string()))?;
        }

        Self::load(tmp_dir.keep())
    }
}

pub fn default_router() -> RouterResult<Router> {
    let config = RouterConfig::default();
    router_from_config(config)
}

pub fn router_from_config(config: RouterConfig) -> RouterResult<Router> {
    if let Some(ref path) = config.taxonomy_path {
        if path.exists() {
            return Router::load(path);
        }
    }

    if config.use_embedded_fallback {
        Router::from_embedded()
    } else {
        Err(RouterError::TaxonomyNotFound(
            config.taxonomy_path.map_or_else(
                || "no path configured".to_string(),
                |p| p.display().to_string(),
            ),
        ))
    }
}

pub async fn route_and_execute(input: RouterInput) -> RouterResult<RouterOutput> {
    if let (Some(provider), Some(model)) = (&input.preferred_provider, &input.preferred_model) {
        let mut client = RpcClient::spawn(provider, model, input.working_dir.as_deref())?;
        let response = client
            .send_prompt(&input.prompt, input.system_prompt.as_deref())
            .await?;
        return Ok(RouterOutput {
            response,
            provider: provider.clone(),
            model: model.clone(),
            capabilities: vec![],
            confidence: 1.0,
            reason: "explicit preference".to_string(),
            fallback_used: false,
        });
    }

    let router = default_router()?;
    if let Some(decision) = router.route(&input.prompt) {
        let mut client = RpcClient::spawn(
            &decision.provider,
            &decision.model,
            input.working_dir.as_deref(),
        )?;
        let response = client
            .send_prompt(&input.prompt, input.system_prompt.as_deref())
            .await?;
        Ok(RouterOutput {
            response,
            provider: decision.provider,
            model: decision.model,
            capabilities: vec![decision.matched_concept],
            confidence: decision.confidence,
            reason: "kg route match".to_string(),
            fallback_used: false,
        })
    } else {
        let fallback_provider = "anthropic".to_string();
        let fallback_model = "claude-sonnet-4-6".to_string();
        let mut client = RpcClient::spawn(
            &fallback_provider,
            &fallback_model,
            input.working_dir.as_deref(),
        )?;
        let response = client
            .send_prompt(&input.prompt, input.system_prompt.as_deref())
            .await?;
        Ok(RouterOutput {
            response,
            provider: fallback_provider,
            model: fallback_model,
            capabilities: vec![],
            confidence: 0.0,
            reason: "fallback: no kg route matched".to_string(),
            fallback_used: true,
        })
    }
}

pub fn extract_capabilities(prompt: &str) -> Vec<String> {
    let Ok(router) = default_router() else {
        return vec![];
    };
    extract_capabilities_with_router(prompt, &router)
}

pub fn extract_capabilities_with_router(prompt: &str, router: &Router) -> Vec<String> {
    if router.thesaurus.is_empty() {
        return vec![];
    }

    let Ok(matches) = find_matches(prompt, router.thesaurus.clone(), false) else {
        return vec![];
    };

    let mut concepts: Vec<String> = matches
        .iter()
        .map(|m| m.normalized_term.value.to_string())
        .collect();
    concepts.sort();
    concepts.dedup();
    concepts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_rule(dir: &Path, name: &str, content: &str) {
        let path = dir.join(format!("{name}.md"));
        let mut f = fs::File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    #[test]
    fn test_load_taxonomy() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "implementation",
            "# Implementation\npriority:: 50\nsynonyms:: implement, build, code\nroute:: anthropic, claude-sonnet-4-6\n",
        );

        let router = Router::load(dir.path()).unwrap();
        assert_eq!(router.rule_count(), 1);
    }

    #[test]
    fn test_route_by_synonym() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "implementation_tier",
            "# Implementation Tier\npriority:: 50\nsynonyms:: implement, build, code, fix\nroute:: anthropic, claude-sonnet-4-6\nroute:: kimi, kimi-for-coding/k2p5\n",
        );

        let router = Router::load(dir.path()).unwrap();
        let decision = router.route("implement the new feature").unwrap();

        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.model, "claude-sonnet-4-6");
        assert_eq!(decision.matched_concept, "implementation_tier");
        assert_eq!(decision.fallback_routes.len(), 2);
    }

    #[test]
    fn test_priority_selection() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "implementation",
            "# Implementation\npriority:: 50\nsynonyms:: implement, build, review code\nroute:: kimi, k2p5\n",
        );
        write_rule(
            dir.path(),
            "code_review",
            "# Code Review\npriority:: 70\nsynonyms:: code review, architecture review\nroute:: anthropic, opus\n",
        );

        let router = Router::load(dir.path()).unwrap();
        let decision = router
            .route("do a code review of the architecture")
            .unwrap();

        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.matched_concept, "code_review");
        assert_eq!(decision.priority, 70);
    }

    #[test]
    fn test_no_match_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "security",
            "# Security\npriority:: 60\nsynonyms:: security audit, CVE\nroute:: kimi, k2p5\n",
        );

        let router = Router::load(dir.path()).unwrap();
        assert!(router.route("write documentation").is_none());
    }

    #[test]
    fn test_render_action() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "impl",
            "# Impl\nsynonyms:: build\nroute:: kimi, k2p5\naction:: pi --model {{ model }} -p \"{{ prompt }}\"\n",
        );

        let router = Router::load(dir.path()).unwrap();
        let decision = router.route("build it").unwrap();
        let rendered = decision.render_action("echo hello").unwrap();

        assert_eq!(rendered, "pi --model k2p5 -p \"echo hello\"");
    }

    #[test]
    fn test_embedded_fallback() {
        let router = default_router().unwrap();
        assert!(router.rule_count() >= 3);

        let decision = router.route("implement authentication").unwrap();
        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.matched_concept, "implementation_tier");
    }

    #[test]
    fn test_fallback_routes() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "impl",
            "# Impl\nsynonyms:: build\nroute:: anthropic, sonnet\nroute:: kimi, k2p5\nroute:: openai, gpt-5.4\n",
        );

        let router = Router::load(dir.path()).unwrap();
        let decision = router.route("build it").unwrap();

        assert_eq!(decision.fallback_routes.len(), 3);
        assert_eq!(decision.fallback_routes[0].provider, "anthropic");
        assert_eq!(decision.fallback_routes[1].provider, "kimi");
        assert_eq!(decision.fallback_routes[2].provider, "openai");
    }

    #[test]
    fn test_empty_dir_loads_zero_rules() {
        let dir = tempfile::tempdir().unwrap();
        let router = Router::load(dir.path()).unwrap();
        assert_eq!(router.rule_count(), 0);
        assert!(router.route("anything").is_none());
    }

    #[test]
    fn test_reload_picks_up_new_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut router = Router::load(dir.path()).unwrap();
        assert_eq!(router.rule_count(), 0);

        write_rule(
            dir.path(),
            "security",
            "# Security\nsynonyms:: CVE\nroute:: kimi, k2p5\n",
        );
        router.reload().unwrap();
        assert_eq!(router.rule_count(), 1);
        assert!(router.route("check CVE").is_some());
    }

    #[test]
    fn test_confidence_normalised() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "planning",
            "# Planning\npriority:: 80\nsynonyms:: strategic planning\nroute:: anthropic, opus\n",
        );

        let router = Router::load(dir.path()).unwrap();
        let decision = router.route("strategic planning session").unwrap();
        assert!((decision.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn test_planning_tier_embedded() {
        let router = default_router().unwrap();
        let decision = router.route("create a plan for the new feature").unwrap();
        assert_eq!(decision.matched_concept, "planning_tier");
        assert_eq!(decision.priority, 80);
        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.model, "claude-opus-4-6");
    }

    #[test]
    fn test_review_tier_embedded() {
        let router = default_router().unwrap();
        let decision = router.route("verify the results").unwrap();
        assert_eq!(decision.matched_concept, "review_tier");
        assert_eq!(decision.priority, 40);
        assert_eq!(decision.provider, "anthropic");
        assert_eq!(decision.model, "claude-haiku-4-6");
    }

    #[test]
    fn test_extract_capabilities() {
        let dir = tempfile::tempdir().unwrap();
        write_rule(
            dir.path(),
            "impl",
            "# Impl\nsynonyms:: implement, build\nroute:: anthropic, sonnet\n",
        );
        write_rule(
            dir.path(),
            "review",
            "# Review\nsynonyms:: verify\nroute:: anthropic, haiku\n",
        );

        let router = Router::load(dir.path()).unwrap();
        let caps = extract_capabilities_with_router("implement and verify", &router);
        assert!(caps.contains(&"impl".to_string()));
        assert!(caps.contains(&"review".to_string()));
    }

    #[test]
    fn test_load_nonexistent_dir() {
        let result = Router::load("/nonexistent/path/to/taxonomy");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RouterError::TaxonomyNotFound(_)
        ));
    }

    #[test]
    fn test_router_from_config_no_path() {
        let config = RouterConfig {
            taxonomy_path: None,
            use_embedded_fallback: true,
        };
        let router = router_from_config(config).unwrap();
        assert!(router.rule_count() >= 3);
    }

    #[test]
    fn test_router_from_config_no_fallback() {
        let config = RouterConfig {
            taxonomy_path: Some(PathBuf::from("/nonexistent")),
            use_embedded_fallback: false,
        };
        let result = router_from_config(config);
        assert!(result.is_err());
    }
}
