# Implementation Plan: pi_terraphim_router Dynamic Routing Module

**Status**: Draft  
**Research Doc**: `.docs/research-pi-rust-terraphim-router-v2.md`  
**Author**: AI Agent  
**Date**: 2026-05-25  
**Estimated Effort**: 6-8 hours  

## Overview

### Summary
Implement the missing `src/pi_terraphim_router.rs` module behind the existing `terraphim-routing` feature flag. The module provides KG-driven prompt routing using markdown-defined taxonomy rules with Aho-Corasick synonym matching. Wire up the already-declared `demo-route` CLI subcommand to output routing decisions as JSON or text.

### Approach
Replicate the proven `KgRouter` pattern from `terraphim_orchestrator` using only `terraphim_automata` and `terraphim_types` as dependencies (minimal footprint, no SQLite, no orchestrator bloat). Load `.md` taxonomy files from a user-configurable directory, build an in-memory `Thesaurus`, match prompts via Aho-Corasick, and return structured routing decisions with provider health checking.

### Scope

**In Scope:**
- `src/pi_terraphim_router.rs` module (new file)
- `terraphim_automata` + `terraphim_types` dependency wiring in `Cargo.toml`
- `demo-route` CLI handler in `src/main.rs`
- Default taxonomy directory at `~/.config/pi/routing_taxonomy/`
- Embedded fallback taxonomy (3-tier ADF rules baked into binary)
- JSON and text output formats for `demo-route`
- Provider readiness checking via `model_entry_is_ready()`
- Hot-reload support (`reload_if_changed()`)

**Out of Scope:**
- Persistence / SQLite backend
- Online learning / adaptive routing
- Multi-hop graph traversal beyond synonym matching
- Web UI for editing taxonomy rules
- Real-time provider health monitoring daemon

**Avoid At All Cost** (from 5/25 analysis):
- Adding `terraphim_orchestrator` as a dependency (pulls in scheduler, dispatcher, cost tracker -- massive bloat)
- Adding `terraphim_persistence` / SQLite (not needed; in-memory only)
- Async runtime boundaries between asupersync and tokio (keep routing synchronous)

## Architecture

### Component Diagram
```
+------------------+        +-----------------------+
|   CLI (main.rs)  |        |  pi_terraphim_router  |
| Commands::DemoRoute| ----> |                       |
+------------------+        |  +-----------------+  |
                            |  | TaxonomyLoader  |  |
                            |  | (markdown .md)  |  |
                            |  +--------+--------+  |
                            |           |           |
                            |  +--------v--------+  |
                            |  | ThesaurusBuilder|  |
                            |  | (Aho-Corasick)  |  |
                            |  +--------+--------+  |
                            |           |           |
                            |  +--------v--------+  |
                            |  | KgRouter        |  |
                            |  | (find_matches)  |  |
                            |  +--------+--------+  |
                            |           |           |
                            |  +--------v--------+  |
                            |  | HealthFilter    |  |
                            |  | (model_entry_   |  |
                            |  |  is_ready)      |  |
                            |  +--------+--------+  |
                            |           |           |
                            |  +--------v--------+  |
                            |  | RouteDecision   |  |
                            |  | (JSON / text)   |  |
                            |  +-----------------+  |
                            +-----------------------+
```

### Data Flow
```
User: pi demo-route "implement auth system" --format json
    |
    v
CLI parses prompt + format
    |
    v
Lazy-load taxonomy (first call):
  - Try ~/.config/pi/routing_taxonomy/
  - Fallback to embedded default rules
    |
    v
Build Thesaurus from synonyms in all .md files
    |
    v
terraphim_automata::find_matches(prompt, thesaurus)
    |
    v
Group matches by concept, select highest priority
    |
    v
Check provider readiness (model_entry_is_ready)
    |
    v
Select first healthy route from fallback chain
    |
    v
Render action template (substitute {{ model }}, {{ prompt }})
    |
    v
Output JSON or text
```

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Use `terraphim_automata` + `terraphim_types` only | Minimal deps; `find_matches` and `parse_markdown_directives_dir` provide everything needed | `terraphim_orchestrator` (too heavy), `terraphim_router` (too limited) |
| Synchronous routing API | Avoids async runtime mismatch between asupersync and tokio | Async wrapper (unnecessary complexity for <10ms operation) |
| Lazy taxonomy loading | Keeps startup time <100ms; load on first route request | Eager loading (adds latency to every startup, even when routing unused) |
| Embedded fallback taxonomy | Works out-of-box without user configuration; user taxonomy overrides | Require user to create taxonomy directory first (friction) |
| In-memory only (no persistence) | Zero SQLite dependency; hot-reload via mtime check | SQLite cache (complexity not justified for small taxonomy) |

### Eliminated Options (Essentialism)

| Option Rejected | Why Rejected | Risk of Including |
|-----------------|--------------|-------------------|
| terraphim_orchestrator dependency | Would add ~50+ transitive deps, scheduler, dispatcher, cost tracker | Binary size increase >5MB, compile time increase, maintenance burden |
| terraphim_router::KnowledgeGraphRouter | Lacks render_action(), first_healthy_route(), hot-reload; would need to wrap/extend anyway | Incomplete solution; users would hit missing features quickly |
| SQLite-backed taxonomy cache | Not needed; markdown files are the source of truth; mtime check is sufficient | Adds rusqlite, opendal deps; prior false assumption blocked this feature |
| Async routing API | find_matches is sync; adding async buys nothing for <10ms operation | Runtime complexity, tokio/asupersync boundary issues |
| HTTP service wrapper | Over-engineering; CLI subprocess approach is sufficient | Deployment complexity, daemon lifecycle management |

### Simplicity Check

> "Minimum code that solves the problem. Nothing speculative."

**What if this could be easy?**

The simplest design is: load markdown files, build a `Thesaurus`, call `find_matches`, pick highest priority, check provider readiness, output result. No database, no async, no daemon, no learning. This is exactly what `KgRouter` does in ~310 lines.

**Senior Engineer Test**: A senior engineer would recognise this as a straightforward text matching problem. The design uses standard Terraphim primitives (`Thesaurus`, `find_matches`) with a thin routing layer on top. No over-engineering.

**Nothing Speculative Checklist**:
- [x] No features the user didn't request
- [x] No abstractions "in case we need them later"
- [x] No flexibility "just in case"
- [x] No error handling for scenarios that cannot occur
- [x] No premature optimization

## File Changes

### New Files
| File | Purpose |
|------|---------|
| `src/pi_terraphim_router.rs` | Core routing module: taxonomy loading, Aho-Corasick matching, route selection |

### Modified Files
| File | Changes |
|------|---------|
| `Cargo.toml` | Add `terraphim_automata` to `terraphim-routing` feature dependencies |
| `src/lib.rs` | No changes (module already declared at line 224) |
| `src/main.rs` | Add `Commands::DemoRoute` handler in the ultra-fast path match block |

### Deleted Files
| File | Reason |
|------|--------|
| None | |

## API Design

### Public Types

```rust
/// A routing decision from KG matching.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RouteDecision {
    /// Provider name (e.g., "anthropic", "kimi")
    pub provider: String,
    /// Model identifier (e.g., "claude-sonnet-4-6", "kimi-for-coding/k2p5")
    pub model: String,
    /// CLI action template with placeholders
    pub action: Option<String>,
    /// Match confidence (0.0-1.0)
    pub confidence: f64,
    /// Concept that matched (filename stem)
    pub matched_concept: String,
    /// Priority from the matched rule (0-100)
    pub priority: u8,
    /// All routes from the matched file (primary + fallbacks)
    pub fallback_routes: Vec<terraphim_types::RouteDirective>,
    /// Whether the selected provider has credentials configured
    pub provider_ready: bool,
}

impl RouteDecision {
    /// Render the action template by substituting `{{ model }}` and `{{ prompt }}`.
    pub fn render_action(&self, prompt: &str) -> Option<String>;
}

/// KG-based model router that loads routing rules from markdown files.
pub struct Router {
    /* internal fields */
}

impl Router {
    /// Load routing rules from a taxonomy directory.
    pub fn load(taxonomy_path: impl Into<PathBuf>) -> Result<Self, RouterError>;
    
    /// Route a prompt to the best provider+model.
    pub fn route(&self, prompt: &str) -> Option<RouteDecision>;
    
    /// Reload rules from the taxonomy directory.
    pub fn reload(&mut self) -> Result<(), RouterError>;
    
    /// Reload rules only if any file has been modified.
    pub fn reload_if_changed(&mut self) -> bool;
    
    /// Number of loaded routing rules.
    pub fn rule_count(&self) -> usize;
}

/// Router configuration and defaults.
pub struct RouterConfig {
    /// Path to taxonomy directory (default: ~/.config/pi/routing_taxonomy)
    pub taxonomy_path: Option<PathBuf>,
    /// Whether to use embedded fallback rules if no taxonomy exists
    pub use_embedded_fallback: bool,
}

impl Default for RouterConfig {
    fn default() -> Self;
}

/// Errors that can occur during router operations.
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("taxonomy directory not found: {0}")]
    TaxonomyNotFound(String),
    #[error("failed to parse taxonomy: {0}")]
    ParseError(String),
}
```

### Public Functions

```rust
/// Create a router with default configuration.
/// Loads from ~/.config/pi/routing_taxonomy/ if it exists,
/// otherwise uses embedded fallback taxonomy.
pub fn default_router() -> Result<Router, RouterError>;

/// Create a router from explicit configuration.
pub fn router_from_config(config: RouterConfig) -> Result<Router, RouterError>;

/// Extract capabilities from a prompt without routing.
/// Returns the list of matched concept names.
pub fn extract_capabilities(prompt: &str, router: &Router) -> Vec<String>;

/// Get the provider readiness status for all routes in a decision.
pub fn check_provider_readiness(
    decision: &RouteDecision,
    model_registry: &pi::models::ModelRegistry,
) -> Vec<(String, String, bool)>;
```

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("taxonomy directory not found: {0}")]
    TaxonomyNotFound(String),
    #[error("failed to parse taxonomy: {0}")]
    ParseError(String),
}
```

## Test Strategy

### Unit Tests
| Test | Location | Purpose |
|------|----------|---------|
| `test_load_taxonomy` | `pi_terraphim_router.rs` | Verify markdown files are loaded correctly |
| `test_route_by_synonym` | `pi_terraphim_router.rs` | Verify Aho-Corasick matching works |
| `test_priority_selection` | `pi_terraphim_router.rs` | Higher priority wins when multiple rules match |
| `test_no_match_returns_none` | `pi_terraphim_router.rs` | Unknown prompts return None gracefully |
| `test_render_action` | `pi_terraphim_router.rs` | Template substitution works correctly |
| `test_reload_picks_up_changes` | `pi_terraphim_router.rs` | Hot-reload detects file modifications |
| `test_fallback_routes` | `pi_terraphim_router.rs` | Multiple routes are preserved in decision |
| `test_embedded_fallback` | `pi_terraphim_router.rs` | Default router works without user taxonomy |

### Integration Tests
| Test | Location | Purpose |
|------|----------|---------|
| `test_demo_route_cli_json` | `tests/` or inline in main.rs | CLI outputs valid JSON |
| `test_demo_route_cli_text` | `tests/` or inline in main.rs | CLI outputs readable text |
| `test_provider_readiness_integration` | `tests/` | Routing respects credential availability |

### Conformance Tests
Add a new conformance fixture for routing:
```json
{
  "version": "1.0",
  "tool": "terraphim_router",
  "cases": [
    {
      "name": "route_implementation_task",
      "input": {"prompt": "implement authentication module"},
      "expected": {
        "content_contains": ["implementation_tier", "anthropic"],
        "content_regex": "confidence.*0\.[0-9]+"
      }
    }
  ]
}
```

## Implementation Steps

### Step 1: Add Dependency
**Files:** `Cargo.toml`
**Description:** Add `terraphim_automata` to the `terraphim-routing` feature
**Tests:** `cargo check --features terraphim-routing` compiles
**Estimated:** 15 minutes

```toml
# In [dependencies] section:
terraphim_automata = { path = "../terraphim-ai/crates/terraphim_automata", optional = true }

# In [features] section:
terraphim-routing = ["dep:terraphim_router", "dep:terraphim_types", "dep:terraphim_automata"]
```

### Step 2: Core Types and Errors
**Files:** `src/pi_terraphim_router.rs` (first half)
**Description:** Define `RouteDecision`, `RouterConfig`, `RouterError`
**Tests:** Unit tests for type construction and error formatting
**Estimated:** 45 minutes

```rust
// Key code to write
#[derive(Debug, Clone, serde::Serialize)]
pub struct RouteDecision { ... }

#[derive(Debug, thiserror::Error)]
pub enum RouterError { ... }
```

### Step 3: Taxonomy Loading
**Files:** `src/pi_terraphim_router.rs`
**Description:** Implement `Router::load()` using `terraphim_automata::markdown_directives::parse_markdown_directives_dir`
**Tests:** Unit tests for loading from temp directory, handling missing directory, parsing warnings
**Dependencies:** Step 2
**Estimated:** 1 hour

```rust
// Key code to write
impl Router {
    pub fn load(taxonomy_path: impl Into<PathBuf>) -> Result<Self, RouterError> {
        let parse_result = terraphim_automata::parse_markdown_directives_dir(&path)
            .map_err(|e| RouterError::ParseError(e.to_string()))?;
        // Build thesaurus from synonyms...
    }
}
```

### Step 4: Routing Logic
**Files:** `src/pi_terraphim_router.rs`
**Description:** Implement `Router::route()` with Aho-Corasick matching and priority selection
**Tests:** Unit tests for synonym matching, priority tiebreaking, no-match fallback
**Dependencies:** Step 3
**Estimated:** 1.5 hours

```rust
// Key code to write
pub fn route(&self, prompt: &str) -> Option<RouteDecision> {
    let matches = terraphim_automata::find_matches(prompt, self.thesaurus.clone(), false).ok()?;
    // Group by concept, select highest priority...
}
```

### Step 5: Embedded Fallback Taxonomy
**Files:** `src/pi_terraphim_router.rs`
**Description:** Include default 3-tier ADF rules as compile-time strings; build router from embedded data when user taxonomy missing
**Tests:** Unit test verifying `default_router()` works without filesystem access
**Dependencies:** Step 3
**Estimated:** 1 hour

```rust
// Key code to write
const EMBEDDED_TAXONOMY: &[(&str, &str)] = &[
    ("planning_tier.md", include_str!("../resources/routing_taxonomy/planning_tier.md")),
    // ...
];
```

### Step 6: CLI Handler
**Files:** `src/main.rs`
**Description:** Add `Commands::DemoRoute` handler in the ultra-fast path match block (around line 299)
**Tests:** Unit tests in `cli.rs` already exist for parsing; add integration test for output
**Dependencies:** Steps 2-5
**Estimated:** 45 minutes

```rust
// In main.rs match block:
cli::Commands::DemoRoute { prompt, format } => {
    let router = pi::pi_terraphim_router::default_router()?;
    match router.route(&prompt) {
        Some(decision) => {
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&decision)?),
                _ => println!("{}", format_route_decision(&decision)),
            }
        }
        None => {
            eprintln!("No route matched for prompt: {}", prompt);
            std::process::exit(1);
        }
    }
    return Ok(());
}
```

### Step 7: Provider Readiness Integration
**Files:** `src/pi_terraphim_router.rs`
**Description:** Add `check_provider_readiness()` function that filters routes by `model_entry_is_ready()`
**Tests:** Unit tests with mock model registry
**Dependencies:** Step 4
**Estimated:** 30 minutes

### Step 8: Hot Reload
**Files:** `src/pi_terraphim_router.rs`
**Description:** Implement `reload_if_changed()` using directory mtime
**Tests:** Unit test: modify file, verify reload detects change
**Dependencies:** Step 3
**Estimated:** 30 minutes

### Step 9: Documentation and Example Update
**Files:** `examples/terraphim_router.rs`, inline docs
**Description:** Update example to use actual API; add module-level documentation
**Tests:** `cargo run --example terraphim_router --features terraphim-routing`
**Dependencies:** Steps 2-8
**Estimated:** 30 minutes

### Step 10: Quality Gates
**Files:** All changed files
**Description:** Run compiler checks, tests, UBS scanner
**Tests:** `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo test --features terraphim-routing`, `ubs --staged --only=rust .`
**Dependencies:** All previous steps
**Estimated:** 30 minutes

## Rollback Plan

If issues discovered:
1. Disable feature: remove `terraphim-routing` from default features (already not in default)
2. Remove module declaration from `src/lib.rs:224`
3. Remove CLI handler from `src/main.rs`

Feature is already behind `terraphim-routing` flag -- zero impact when disabled.

## Dependencies

### New Dependencies
| Crate | Version | Justification |
|-------|---------|---------------|
| `terraphim_automata` | 1.x | Aho-Corasick matching (`find_matches`) + markdown directives parsing |

Already declared (no change needed):
| Crate | Version | Justification |
|-------|---------|---------------|
| `terraphim_types` | 1.x | `Thesaurus`, `NormalizedTerm`, `RouteDirective` |
| `terraphim_router` | 1.x | Already in Cargo.toml (optional) -- may remove if unused |

### Dependency Updates
| Crate | From | To | Reason |
|-------|------|-----|--------|
| None | | | |

### Dependency Removal Consideration
`terraphim_router` is currently in `Cargo.toml` but the new design uses `terraphim_automata` + `terraphim_types` directly. Consider removing `terraphim_router` from the feature to reduce compile-time overhead, unless `KnowledgeGraphRouter` is needed for future expansion.

## Performance Considerations

### Expected Performance
| Metric | Target | Measurement |
|--------|--------|-------------|
| Routing latency | <10ms | `cargo bench` or simple timing in tests |
| Taxonomy load time | <50ms | Timing in `Router::load()` |
| Memory overhead | <1MB | `sizeof` analysis or heap profiling |
| Binary size overhead | <500KB | `cargo bloat --features terraphim-routing` |

### Benchmarks to Add
```rust
#[bench]
fn bench_route_prompt(b: &mut Bencher) {
    let router = Router::load("test_taxonomy").unwrap();
    b.iter(|| router.route("implement authentication"));
}

#[bench]
fn bench_load_taxonomy(b: &mut Bencher) {
    b.iter(|| Router::load("test_taxonomy"));
}
```

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| Verify terraphim_automata compiles in pi-rust workspace | Pending | Implementer |
| Decide whether to keep terraphim_router dependency | Pending | Implementer |
| Create default taxonomy markdown files in resources/ | Pending | Implementer |
| Write conformance fixture for routing | Pending | Implementer |

## Approval

- [ ] Technical review complete
- [ ] Test strategy approved
- [ ] Performance targets agreed
- [ ] Human approval received

---

## Appendix: Reference Implementation Mapping

This table maps `terraphim_orchestrator::KgRouter` API to the planned `pi_terraphim_router::Router` API:

| KgRouter (reference) | pi_terraphim_router (this plan) | Notes |
|---------------------|----------------------------------|-------|
| `KgRouter::load(path)` | `Router::load(path)` | Identical |
| `KgRouter::route_agent(task)` | `Router::route(prompt)` | Renamed for generality |
| `KgRouteDecision` | `RouteDecision` | Same fields, added `provider_ready` |
| `KgRouteDecision::render_action()` | `RouteDecision::render_action()` | Identical |
| `KgRouteDecision::first_healthy_route()` | `check_provider_readiness()` | Extracted to free function for testability |
| `KgRouter::reload()` | `Router::reload()` | Identical |
| `KgRouter::reload_if_changed()` | `Router::reload_if_changed()` | Identical |
| `KgRouter::rule_count()` | `Router::rule_count()` | Identical |
| `KgRouterError` | `RouterError` | Same variants |
| `KgRouter::all_routes()` | `Router::all_routes()` | For probing/testing |

## Appendix: Embedded Fallback Taxonomy

If user taxonomy directory does not exist, the router will use embedded rules equivalent to:

**planning_tier.md:**
```markdown
# Planning Tier
priority:: 80
synonyms:: strategic planning, architecture design, create a plan
synonyms:: product vision, technical strategy, feasibility study
route:: anthropic, claude-opus-4-6
route:: kimi, kimi-for-coding/k2p6
route:: openai, openai/gpt-5.4
```

**implementation_tier.md:**
```markdown
# Implementation Tier
priority:: 50
synonyms:: implement, build, code, fix, test, security audit
synonyms:: bug fix, patch, enhancement, cargo build
route:: anthropic, claude-sonnet-4-6
route:: kimi, kimi-for-coding/k2p5
route:: openai, openai/gpt-5.3-codex
```

**review_tier.md:**
```markdown
# Review Tier
priority:: 40
synonyms:: verify, validate, check results, compliance check
synonyms:: review plan, quality gate, drift detection
route:: anthropic, claude-haiku-4-6
route:: kimi, kimi-for-coding/k2p5
route:: openai, openai/gpt-5.4-mini
```
