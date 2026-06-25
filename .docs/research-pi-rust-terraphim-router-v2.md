# Research Document: pi-rust-terraphim-router Dynamic Routing

**Status**: Approved (updated -- correcting prior misconceptions)  
**Author**: AI Agent  
**Date**: 2026-05-25  
**Reviewers**: Pending  

## Executive Summary

The `pi_terraphim_router` module is declared in `src/lib.rs:224` behind the `terraphim-routing` feature flag, but the implementation file (`src/pi_terraphim_router.rs`) is entirely missing. A research document from 2026-05-24 incorrectly assumed Terraphim knowledge graph routing requires SQLite, and failed to reference the production-ready `KgRouter` already present in `terraphim_orchestrator`. This corrected research identifies the actual architecture: in-memory Aho-Corasick synonym matching against markdown-defined routing rules, with zero database dependency.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energizing? | Yes | Eliminates manual `--provider`/`--model` selection; enables intelligent prompt-to-model routing |
| Leverages strengths? | Yes | `terraphim_orchestrator::KgRouter` is already production-tested with ADF 3-tier taxonomy; pi-rust has 10+ providers with `model_entry_is_ready()` credential checking |
| Meets real need? | Yes | The `demo-route` subcommand is declared in CLI but unimplemented; the `pi-rust-terraphim-router` skill references a non-existent module |

**Proceed**: Yes -- 3/3 YES

## Problem Statement

### Description
The `terraphim-routing` Cargo feature and `pi_terraphim_router` module are declared but the source file is absent. Users cannot use `--demo-route`, and the skill document describes an API that does not exist. The prior research doc incorrectly eliminated knowledge graph integration based on a false SQLite dependency assumption, missing the `KgRouter` reference implementation entirely.

### Impact
- **Broken feature flag**: `cargo build --features terraphim-routing` compiles but `pi_terraphim_router` module has zero exports
- **Skill references non-existent API**: `pi::pi_terraphim_router::{extract_capabilities, route_and_execute}` are described but not implemented
- **Missed opportunity**: `terraphim_orchestrator` already solved this exact problem with `KgRouter` + markdown taxonomy

### Success Criteria
1. `src/pi_terraphim_router.rs` exists and compiles behind `terraphim-routing` feature
2. A markdown taxonomy directory can be loaded and used for routing
3. `pi demo-route "<prompt>"` CLI subcommand works and outputs JSON with routing decision
4. Routing uses Aho-Corasick synonym matching (as per `KgRouter` reference)
5. No SQLite or persistence dependency introduced

## Current State Analysis

### Existing Implementation

#### pi-rust (pi_agent_rust)
- **Module declaration**: `src/lib.rs:224` -- `#[cfg(feature = "terraphim-routing")] pub mod pi_terraphim_router;`
- **Missing file**: `src/pi_terraphim_router.rs` does not exist
- **CLI subcommand**: `demo-route` listed in `src/cli.rs:46` but no handler exists
- **Provider metadata**: `src/provider_metadata.rs` -- 80+ providers with `routing_defaults`
- **Model registry**: `src/models.rs` -- `ModelRegistry`, `model_entry_is_ready()` for credential checking
- **Example**: `examples/terraphim_router.rs` imports `pi::pi_terraphim_router::{extract_capabilities, get_provider_for_capability}` -- these do not exist

#### terraphim_orchestrator (reference implementation)
- **`KgRouter`** (`src/kg_router.rs`): Production-ready KG-driven router
  - Loads markdown files from taxonomy directory
  - Parses `route::`, `action::`, `synonyms::`, `priority::`, `trigger::` directives
  - Builds `Thesaurus` from synonyms for Aho-Corasick matching
  - `route_agent(task_description) -> Option<KgRouteDecision>`
  - `render_action()` substitutes `{{ model }}` and `{{ prompt }}`
  - `first_healthy_route()` skips unhealthy providers
  - Hot-reload via `reload_if_changed()`
- **ADF taxonomy** (`docs/taxonomy/routing_scenarios/adf/`):
  - `planning_tier.md` (priority 80) -> anthropic/opus
  - `implementation_tier.md` (priority 50) -> anthropic/sonnet
  - `review_tier.md` (priority 40) -> anthropic/haiku

#### terraphim_router crate (available dependency)
- **`terraphim_router::knowledge_graph::KnowledgeGraphRouter`**: In-memory thesaurus-based router
- **`terraphim_router::engine::RoutingEngine`**: Keyword-based capability extraction + provider mapping
- **Zero SQLite**: The `terraphim_router` crate has no SQLite dependency; `terraphim_persistence` is optional behind `persistence` feature
- **Dependency chain**: `terraphim_router` -> `terraphim_types` (core types) only

### Code Locations

| Component | Location | Purpose |
|-----------|----------|---------|
| Module declaration | `pi_agent_rust/src/lib.rs:224` | Conditional module export |
| CLI subcommand list | `pi_agent_rust/src/cli.rs:46` | `demo-route` is listed but unhandled |
| Example (broken) | `pi_agent_rust/examples/terraphim_router.rs` | References non-existent API |
| Reference router | `terraphim-ai/crates/terraphim_orchestrator/src/kg_router.rs` | `KgRouter` -- production-ready |
| Reference taxonomy | `terraphim-ai/docs/taxonomy/routing_scenarios/adf/*.md` | 3-tier routing rules |
| terraphim_router crate | `terraphim-ai/crates/terraphim_router/` | Available dependency with `KnowledgeGraphRouter` |

### Data Flow (Target)
```
User prompt / task description
    |
    v
Load taxonomy directory (.md files with route:: / synonyms:: / priority::)
    |
    v
Build Thesaurus from synonyms (Aho-Corasick automaton)
    |
    v
terraphim_automata::find_matches(prompt, thesaurus)
    |
    v
Select highest-priority matched rule
    |
    v
KgRouteDecision { provider, model, confidence, action, fallback_routes }
    |
    v
Check provider readiness (model_entry_is_ready())
    |
    v
Select first healthy route (skip unavailable providers)
    |
    v
Render action template with {{ model }} + {{ prompt }}
    |
    v
JSON output or CLI dispatch
```

### Integration Points
- **pi-rust model registry**: `models.rs` -- `ModelRegistry::available_models()` filters by credential readiness
- **pi-rust provider metadata**: `provider_metadata.rs` -- canonical provider IDs and aliases
- **terraphim_automata**: `find_matches()` for Aho-Corasick matching against `Thesaurus`
- **terraphim_types**: `Thesaurus`, `NormalizedTerm`, `NormalizedTermValue`, `MarkdownDirectives`

## Constraints

### Technical Constraints
- **No unsafe code**: `pi_agent_rust` has `#![forbid(unsafe_code)]`; all dependencies must comply
- **Feature flag gating**: All terraphim routing code behind `terraphim-routing` feature
- **Runtime**: pi-rust uses `asupersync` (structured concurrency); terraphim uses `tokio` -- keep integration synchronous where possible
- **Binary size**: Release target is <22 MiB; in-memory structures only, no heavy deps
- **Startup time**: <100ms target; taxonomy load must be lazy

### Business Constraints
- **No breaking changes**: Existing CLI/SDK must remain unchanged when feature is disabled
- **Optional integration**: terraphim routing must be opt-in via feature flag
- **Cross-platform**: macOS (local) and Linux (bigbox)

### Non-Functional Requirements
| Requirement | Target | Current |
|-------------|--------|---------|
| Routing latency | <10ms | N/A (new feature) |
| Taxonomy load time | <50ms | N/A (new feature) |
| Memory overhead | <1MB | N/A (new feature) |
| Binary size overhead | <500KB | pi-rust is ~18-23 MiB |

## Vital Few (Essentialism)

### Essential Constraints (Max 3)

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| No breaking changes to pi-rust CLI/SDK | pi-rust has active users and CI pipelines | AGENTS.md backwards compat policy |
| Feature-flag gating (`terraphim-routing`) | Not all users need routing; must be opt-in | Cargo.toml already declares this |
| Zero persistence/database dependency | Prior research incorrectly rejected KG routing for this reason | terraphim_router Cargo.toml confirms no SQLite in default build |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|----------------|
| terraphim_persistence / SQLite backend | Not needed; KgRouter uses in-memory Thesaurus only |
| Real-time provider health monitoring | Over-engineering; static capability mapping with `model_entry_is_ready()` check is sufficient |
| Custom keyword mapping UI / config editor | YAGNI -- markdown files are the editing interface |
| ACP / Zed editor integration | Out of scope for pi-rust CLI |
| Online learning / adaptive routing | Complex; static taxonomy with hot-reload is sufficient |
| Multi-hop graph traversal | The `KnowledgeGraphRouter` in terraphim_router only does single-hop synonym expansion; sufficient for routing |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|------------|--------|------|
| `terraphim_automata` | Aho-Corasick matching (`find_matches`) | Low -- stable, tested |
| `terraphim_types` | `Thesaurus`, `NormalizedTerm`, `MarkdownDirectives` | Low -- core types, minimal deps |
| `pi_agent_rust::models` | `ModelRegistry`, `model_entry_is_ready()` | Low -- existing public API |
| `pi_agent_rust::provider_metadata` | `PROVIDER_METADATA`, canonical IDs | Low -- compile-time static data |

### External Dependencies

| Dependency | Version | Risk | Alternative |
|------------|---------|------|-------------|
| `terraphim_automata` | 1.x | Low | None -- required for Aho-Corasick |
| `terraphim_types` | 1.x | Low | None -- required for types |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| terraphim_automata depends on tokio | Medium | Medium | `find_matches` is synchronous; verify no async boundary issues |
| Provider credential not configured | High | Low | Check `model_entry_is_ready()` before selection; fallback chain |
| Taxonomy directory not found | Medium | Low | Graceful fallback to default provider; warn user |
| Binary size increase | Low | Medium | Feature-flag gating isolates impact; measure with `cargo bloat` |

### Open Questions

1. **Where should the taxonomy directory live?**
   - Option A: `~/.config/pi/routing_taxonomy/` (user-configurable)
   - Option B: Embedded in binary at compile time (no runtime files)
   - Option C: `resources/routing_taxonomy/` in repo, copied on install
   - *Recommendation*: Option A with embedded defaults as fallback

2. **Should pi-rust import terraphim_automata directly or use terraphim_router's KnowledgeGraphRouter?**
   - `terraphim_router::KnowledgeGraphRouter` is lightweight but less featureful than `KgRouter`
   - `KgRouter` in terraphim_orchestrator has `render_action()`, `first_healthy_route()`, hot-reload
   - *Recommendation*: Replicate `KgRouter` pattern using `terraphim_automata` + `terraphim_types` directly in pi-rust (avoids pulling in terraphim_orchestrator's heavy deps)

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|------------|-------|---------------|-----------|
| terraphim_automata::find_matches is synchronous | Code inspection shows it takes `&str` and `Thesaurus`, returns `Result<Vec<Match>>` | Build fails if async required | Yes -- verified in source |
| terraphim_types has no tokio/async runtime dependency | Cargo.toml of terraphim_types shows only serde, schemars, chrono, ahash | Runtime mismatch if tokio sneaks in | Yes -- verified in source |
| Markdown directives parser is available in terraphim_automata | terraphim_orchestrator imports from `terraphim_automata::markdown_directives` | Cannot load taxonomy without it | Yes -- verified in source |
| Provider canonical IDs in pi-rust match those in taxonomy | Both use "anthropic", "openai", "kimi", etc. | Routing selects wrong provider | Partial -- naming conventions align |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen/Rejected |
|----------------|--------------|---------------------|
| **A. Add terraphim_orchestrator as dependency** | Gets `KgRouter` for free, but pulls in huge dependency tree (scheduler, dispatcher, cost tracker, etc.) | Rejected -- massive binary size impact |
| **B. Add terraphim_router as dependency** | Gets `KnowledgeGraphRouter`, but it lacks `render_action()`, `first_healthy_route()`, hot-reload | Rejected -- too limited; would need to extend anyway |
| **C. Add terraphim_automata + terraphim_types only** | Replicate `KgRouter` pattern in pi-rust with exactly the features needed | **Chosen** -- minimal deps, full control, matches reference implementation pattern |
| **D. Subprocess to terraphim-cli** | No code changes to pi-rust, but adds process spawn latency | Rejected -- too slow for per-prompt routing |

## Research Findings

### Key Insights

1. **The SQLite claim was false**: `terraphim_router` has zero SQLite dependency. The `terraphim_persistence` crate (which has SQLite) is only pulled in under the `persistence` feature, which is not in default features. Knowledge graph routing is purely in-memory.

2. **`KgRouter` is the reference implementation**: `terraphim_orchestrator::kg_router` already implements exactly what we need: markdown taxonomy loading, Aho-Corasick synonym matching, priority-based route selection, action template rendering, and health-aware fallback. The implementation is ~310 lines of Rust.

3. **Minimal dependency footprint**: Only `terraphim_automata` (for `find_matches` and `markdown_directives`) and `terraphim_types` (for `Thesaurus`, `NormalizedTerm`, `RouteDirective`) are needed. Both are lightweight, sync-only crates.

4. **The `demo-route` CLI subcommand is already declared**: `src/cli.rs:46` lists `demo-route` in `ROOT_SUBCOMMANDS`, confirming the intent. We just need the handler.

5. **Provider readiness checking exists**: `models.rs` has `model_entry_is_ready()` which verifies API keys. This should be integrated with `first_healthy_route()` logic.

### Relevant Prior Art

- **`terraphim_orchestrator::kg_router`**: Direct reference implementation. Replicate its API surface in pi-rust.
- **`terraphim-ai/docs/taxonomy/routing_scenarios/adf/`**: Existing 3-tier taxonomy that can serve as default rules.
- **`pi-rust examples/terraphim_router.rs`**: Shows the desired public API (`extract_capabilities`, `get_provider_for_capability`) -- but we should align with `KgRouter`'s API instead.

### Technical Spikes Needed

| Spike | Purpose | Estimated Effort |
|-------|---------|------------------|
| Verify terraphim_automata compiles in pi-rust workspace | Ensure no hidden tokio/async issues | 30 minutes |
| Test markdown directives parsing | Verify `parse_markdown_directives_dir` works with pi-rust's taxonomy format | 30 minutes |
| Measure binary size impact | `cargo bloat --features terraphim-routing` | 15 minutes |

## Recommendations

### Proceed/No-Proceed
**Proceed** -- The integration is feasible, valuable, and the reference implementation (`KgRouter`) proves the approach works. The dependency footprint is minimal (two lightweight crates).

### Scope Recommendations
- **In scope**: `pi_terraphim_router` module, markdown taxonomy loading, Aho-Corasick routing, `demo-route` CLI subcommand, JSON output, provider readiness checking
- **Out of scope**: Persistence, online learning, multi-hop graph traversal, health monitoring daemon

### Risk Mitigation Recommendations
1. Use feature flag `terraphim-routing` to keep integration optional
2. Implement fallback chain: KG routing -> default provider -> error
3. Add `cargo bloat` check to CI to monitor binary size impact
4. Lazy-load taxonomy on first route request (not at startup)

## Next Steps

If approved:
1. Verify `terraphim_automata` and `terraphim_types` compile cleanly in pi-rust workspace
2. Proceed to Phase 2: Design the `pi_terraphim_router` module architecture
3. Define public API (align with `KgRouter` pattern, not the broken example API)
4. Specify JSON output schema for `demo-route` CLI

## Appendix

### Reference Materials
- `terraphim-ai/crates/terraphim_orchestrator/src/kg_router.rs` -- Reference implementation
- `terraphim-ai/docs/taxonomy/routing_scenarios/adf/*.md` -- Default routing rules
- `terraphim-ai/crates/terraphim_router/src/knowledge_graph.rs` -- Lightweight KG router
- `terraphim-ai/crates/terraphim_router/Cargo.toml` -- Confirms no SQLite in default build
- `pi_agent_rust/src/lib.rs:224` -- Module declaration
- `pi_agent_rust/src/cli.rs:46` -- `demo-route` subcommand declaration

### Code Snippets

**KgRouter::load() pattern:**
```rust
let router = KgRouter::load("/path/to/taxonomy")?;
let decision = router.route_agent("implement the new feature").unwrap();
// decision.provider = "anthropic"
// decision.model = "sonnet"
// decision.confidence = 0.5
```

**Markdown rule format:**
```markdown
# Implementation Tier
priority:: 50
synonyms:: implement, build, code, fix
route:: anthropic, sonnet
action:: pi --provider {{ provider }} --model {{ model }} -p "{{ prompt }}"
```

**Aho-Corasick matching:**
```rust
let matches = terraphim_automata::find_matches(text, thesaurus.clone(), false)?;
```
