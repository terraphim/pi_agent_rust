# Implementation Plan: pi_terraphim_router Review Fixes

**Status**: Draft  
**Research Doc**: `.docs/research-pi-rust-terraphim-router-review-fixes.md`  
**Author**: AI Agent  
**Date**: 2026-05-25  
**Estimated Effort**: 2-3 hours

## Overview

### Summary

Fix the structured review findings for the KgRouter-style `pi_terraphim_router` implementation. The plan keeps the markdown taxonomy architecture intact while making provider readiness truthful, restoring the documented helper API, and preventing embedded fallback tempdir leakage.

### Approach

Make the smallest correct changes:

1. Keep `Router::route(prompt)` as a pure taxonomy match for existing tests and fast usage.
2. Add readiness-aware selection that accepts a `ModelRegistry` and chooses the first ready route in the matched rule's fallback chain.
3. Use the readiness-aware path in `demo-route` and `route_and_execute()` where model execution or diagnostic JSON is involved.
4. Restore `get_provider_for_capability()` as a compatibility helper over taxonomy concept IDs.
5. Store embedded `TempDir` ownership inside `Router` instead of persisting temp directories.

### Scope

**In Scope:**
- Truthful `provider_ready` when a `ModelRegistry` is supplied.
- First-ready fallback route selection.
- Compatibility restoration for `get_provider_for_capability()`.
- Embedded taxonomy tempdir lifecycle fix.
- Focused tests for the three review findings.
- CLI `demo-route` update to use readiness-aware output.

**Out of Scope:**
- Online provider health checks or network probes.
- Taxonomy priority redesign.
- Adaptive routing or learning.
- SQLite/persistence cache.
- New CLI flags.

**Avoid At All Cost** (from 5/25 analysis):
- Pulling in `terraphim_orchestrator`.
- Adding a daemon or async service wrapper.
- Introducing compatibility shims for old capability enums unless explicitly required.
- Broad provider catalogue refactors.
- Rewriting tests to use mocks.

## Architecture

### Component Diagram

```text
CLI / Library
    |
    v
Router::route(prompt)  ------------------------------+
    |                                                |
    | pure KG match                                  | no registry needed
    v                                                |
Matched RoutingRule + RouteDirective list             |
    |                                                |
    +--> Router::route_with_registry(prompt, registry)
             |
             v
       find first route where ModelRegistry::find + model_entry_is_ready
             |
             v
       RouteDecision { provider_ready: true/false }
```

### Data Flow

```text
Prompt
 -> Router::route_rule(prompt)
 -> highest priority matched RoutingRule
 -> pure route: first route, provider_ready=false/unknown-safe default
 -> readiness route: first ready route from fallback chain, provider_ready=true
 -> CLI JSON/text or route_and_execute()
```

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Add readiness-aware method instead of changing `route()` signature | Preserves existing pure API and tests while enabling truthful JSON/execution. | Making every route call load auth/model registry. |
| Store `TempDir` in `Router` | Avoids permanent tempdir leaks while keeping parser API unchanged. | `TempDir::keep()`; global cache; new parser abstraction. |
| Restore `get_provider_for_capability()` over taxonomy concept IDs | Minimal compile compatibility for documented helper. | Removing helper and documenting breakage. |
| Do not add provider network health checks | Review finding is credential/model readiness, not live endpoint health. | HTTP probes, daemon health monitor. |

### Eliminated Options (Essentialism)

| Option Rejected | Why Rejected | Risk of Including |
|-----------------|--------------|-------------------|
| Full legacy capability enum compatibility | Old hardcoded mapper was intentionally replaced. | Reintroduces hardcoded routing and conflicts with KG taxonomy. |
| Loading `ModelRegistry` inside `Router::route()` | Pure route matching should remain cheap and deterministic. | Slower tests/CLI; hidden IO in a matching method. |
| Static global embedded taxonomy directory | Avoids repeated writes but introduces cache lifecycle and concurrency questions. | Cross-process races and cleanup ambiguity. |
| Removing `provider_ready` | Avoids false metadata but loses useful diagnostic output. | Less informative CLI/API. |

### Simplicity Check

> "Minimum code that solves the problem. Nothing speculative."

The easy version is: keep the existing matcher, add a helper that turns a `RouteDirective` into a `RouteDecision`, and pass an optional `ModelRegistry` only where readiness matters. The embedded tempdir fix is a single private field on `Router`.

**Senior Engineer Test**: This is not overcomplicated if the readiness path is an additive method and does not alter the pure matching API.

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
| None | All fixes belong in existing router files. |

### Modified Files

| File | Changes |
|------|---------|
| `src/pi_terraphim_router/mod.rs` | Add readiness-aware route selection, restore helper, store embedded tempdir, add tests. |
| `src/pi_terraphim_router/types.rs` | Optionally add small helper methods or clarify `RouteDecision` construction if needed. |
| `src/main.rs` | Update `demo-route` to load `AuthStorage`/`ModelRegistry` and use readiness-aware decision. |
| `examples/terraphim_router.rs` | Update only if restored helper documentation example is reintroduced. |
| `skills/pi-rust-terraphim-router/SKILL.md` | Update docs if helper semantics change from legacy capability names to taxonomy concepts. |

### Deleted Files

| File | Reason |
|------|--------|
| None | No deletion needed. |

## API Design

### Public Types

No new public structs are required. Existing public structs remain:

```rust
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
```

Internal router state changes:

```rust
pub struct Router {
    rules: Vec<RoutingRule>,
    thesaurus: Thesaurus,
    taxonomy_path: PathBuf,
    last_mtime: Option<SystemTime>,
    embedded_tempdir: Option<tempfile::TempDir>,
}
```

### Public Functions

Keep existing functions and add/restore the following:

```rust
impl Router {
    /// Route a prompt using taxonomy only. Does not check provider credentials.
    pub fn route(&self, prompt: &str) -> Option<RouteDecision>;

    /// Route a prompt and select the first ready provider/model from the matched fallback chain.
    pub fn route_with_registry(
        &self,
        prompt: &str,
        registry: &crate::models::ModelRegistry,
    ) -> Option<RouteDecision>;
}

/// Restore documented helper. The input is a taxonomy concept id such as
/// `planning_tier`, `implementation_tier`, or `review_tier`.
pub fn get_provider_for_capability(capability: &str) -> Option<ProviderSelection>;

/// Registry-aware helper for callers that need truthful readiness.
pub fn check_provider_readiness(
    decision: &RouteDecision,
    registry: &crate::models::ModelRegistry,
) -> Vec<(String, String, bool)>;
```

Private helpers:

```rust
fn route_rule(&self, prompt: &str) -> Option<(&RoutingRule, f64)>;

fn route_is_ready(
    route: &terraphim_types::RouteDirective,
    registry: &crate::models::ModelRegistry,
) -> bool;

fn decision_from_route(
    rule: &RoutingRule,
    route: &terraphim_types::RouteDirective,
    score: f64,
    provider_ready: bool,
) -> RouteDecision;
```

### Error Types

No new error variants are required. Existing `RouterError::ProviderNotReady` can remain for future execution-specific errors, but this plan does not require returning errors from pure matching.

## Test Strategy

### Unit Tests

| Test | Location | Purpose |
|------|----------|---------|
| `test_route_with_registry_marks_ready_provider` | `src/pi_terraphim_router/mod.rs` | A ready registry entry sets `provider_ready=true`. |
| `test_route_with_registry_uses_first_ready_fallback` | `src/pi_terraphim_router/mod.rs` | If first route is unready and second is ready, second route is selected. |
| `test_route_with_registry_falls_back_to_primary_when_none_ready` | `src/pi_terraphim_router/mod.rs` | Preserve deterministic output when no route is ready. |
| `test_check_provider_readiness_reports_all_routes` | `src/pi_terraphim_router/mod.rs` | Readiness list matches each fallback route. |
| `test_get_provider_for_capability_returns_taxonomy_concept_route` | `src/pi_terraphim_router/mod.rs` | Restored helper returns a provider/model for `implementation_tier`. |
| `test_embedded_router_does_not_keep_tempdir_path_after_drop` | `src/pi_terraphim_router/mod.rs` | Embedded tempdir is cleaned when router drops. |

### Integration Tests

| Test | Location | Purpose |
|------|----------|---------|
| Existing `demo-route` CLI smoke evidence | Manual/CLI | Verify JSON includes truthful `provider_ready` after registry load. |

No new integration test file is required unless existing CLI test infrastructure has a natural place for `demo-route`.

### Test Data Construction

Use real structs and existing test constructors, not mocks:

```rust
let registry = ModelRegistry::from_entries_for_tests(vec![entry]);
```

If `ModelEntry` construction is verbose, add a private test helper inside `#[cfg(test)]` in `mod.rs`.

## Implementation Steps

### Step 1: Factor Matched Rule Selection

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Extract the current matching/priority logic from `Router::route()` into a private `route_rule(prompt)` helper. Keep `Router::route()` behaviour unchanged after refactor.  
**Tests:** Existing route tests must still pass.  
**Estimated:** 20 minutes

Key code:

```rust
fn route_rule(&self, prompt: &str) -> Option<(&RoutingRule, f64)> {
    // Existing find_matches + highest priority selection.
}
```

### Step 2: Add RouteDecision Construction Helper

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Add a private helper to construct `RouteDecision` from a matched rule and selected `RouteDirective`. This prevents readiness-aware and pure routing paths from drifting.  
**Tests:** Existing `test_route_by_synonym`, `test_confidence_normalised`, `test_fallback_routes`.  
**Dependencies:** Step 1  
**Estimated:** 15 minutes

Key code:

```rust
fn decision_from_route(
    rule: &RoutingRule,
    route: &RouteDirective,
    score: f64,
    provider_ready: bool,
) -> RouteDecision;
```

### Step 3: Add Registry-Aware Readiness Selection

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Implement `route_with_registry()` and `route_is_ready()`. Select the first ready fallback route; if none are ready, return the primary route with `provider_ready=false` so callers still see the taxonomy match truthfully.  
**Tests:** Add the three readiness route tests listed above.  
**Dependencies:** Steps 1-2  
**Estimated:** 45 minutes

Key code:

```rust
pub fn route_with_registry(
    &self,
    prompt: &str,
    registry: &crate::models::ModelRegistry,
) -> Option<RouteDecision> {
    let (rule, score) = self.route_rule(prompt)?;
    let primary = rule.directives.routes.first()?;
    let selected = rule
        .directives
        .routes
        .iter()
        .find(|route| Self::route_is_ready(route, registry))
        .unwrap_or(primary);
    let provider_ready = Self::route_is_ready(selected, registry);
    Some(Self::decision_from_route(rule, selected, score, provider_ready))
}
```

### Step 4: Add Provider Readiness Reporting Helper

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Implement `check_provider_readiness(decision, registry)` for consumers that want the readiness state of all fallback routes.  
**Tests:** `test_check_provider_readiness_reports_all_routes`.  
**Dependencies:** Step 3  
**Estimated:** 20 minutes

### Step 5: Restore `get_provider_for_capability()`

**Files:** `src/pi_terraphim_router/mod.rs`, optionally `skills/pi-rust-terraphim-router/SKILL.md`  
**Description:** Restore the documented helper as a taxonomy concept lookup. Return the first route for the concept with confidence derived from priority.  
**Tests:** `test_get_provider_for_capability_returns_taxonomy_concept_route`.  
**Dependencies:** Step 2  
**Estimated:** 25 minutes

Key code:

```rust
pub fn get_provider_for_capability(capability: &str) -> Option<ProviderSelection> {
    let router = default_router().ok()?;
    let rule = router.rules.iter().find(|rule| rule.concept == capability)?;
    let route = rule.directives.routes.first()?;
    Some(ProviderSelection {
        provider: route.provider.clone(),
        model: route.model.clone(),
        confidence: f64::from(rule.directives.priority.unwrap_or(50)) / 100.0,
    })
}
```

### Step 6: Fix Embedded Tempdir Lifecycle

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Add `embedded_tempdir: Option<TempDir>` to `Router`. Change `Router::load()` to set `embedded_tempdir: None`. Change `Router::from_embedded()` to load from `tmp_dir.path()` and then store `Some(tmp_dir)` in the returned router.  
**Tests:** Add embedded cleanup test if practical; otherwise assert `default_router().rule_count() >= 3` still works and inspect `Router` internal state in unit tests.  
**Dependencies:** Step 1  
**Estimated:** 30 minutes

Key code:

```rust
let mut router = Self::load(tmp_dir.path())?;
router.embedded_tempdir = Some(tmp_dir);
Ok(router)
```

### Step 7: Update CLI `demo-route` to Use Readiness-Aware Routing

**Files:** `src/main.rs`  
**Description:** Load `AuthStorage` and `ModelRegistry` in the `DemoRoute` handler, call `router.route_with_registry(prompt, &registry)`, and print the same JSON/text shape with truthful `provider_ready`.  
**Tests:** Existing build/clippy plus manual CLI smoke test.  
**Dependencies:** Step 3  
**Estimated:** 25 minutes

Key code:

```rust
let auth = AuthStorage::load(Config::auth_path())?;
let models_path = default_models_path(&Config::global_dir());
let registry = ModelRegistry::load(&auth, Some(models_path));
let decision = router.route_with_registry(prompt, &registry);
```

### Step 8: Quality Gates

**Files:** All changed files  
**Description:** Run focused and project-required checks.  
**Tests:**  
- `rch exec -- cargo test --features terraphim-routing -- pi_terraphim_router`
- `rch exec -- cargo clippy --features terraphim-routing --all-targets -- -D warnings`
- `cargo fmt --check`
- `ubs --staged --only=rust .` after staging if committing
**Dependencies:** Steps 1-7  
**Estimated:** 30 minutes

## Rollback Plan

If issues are discovered:

1. Revert only the latest fix commit, leaving the existing KgRouter rewrite intact.
2. Keep `Router::route()` pure fallback behaviour as the stable route path.
3. If readiness loading causes CLI problems, temporarily make `demo-route` use `Router::route()` but set `provider_ready=false` until registry integration is corrected.

Feature flag remains `terraphim-routing`, so non-feature builds are unaffected.

## Migration

No data migration required.

## Dependencies

### New Dependencies

| Crate | Version | Justification |
|-------|---------|---------------|
| None | N/A | Use existing `tempfile`, `ModelRegistry`, and `terraphim_*` crates. |

### Dependency Updates

| Crate | From | To | Reason |
|-------|------|----|--------|
| None | N/A | N/A | No dependency changes needed. |

## Performance Considerations

### Expected Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Pure `Router::route()` latency | No regression from current implementation | Focused unit tests and optional micro-benchmark later. |
| Readiness-aware route latency | Acceptable for CLI diagnostic/execution path | Loads registry once per command, not per fallback route. |
| Embedded fallback filesystem artefacts | No persistent tempdirs after `Router` drop | Unit test or code inspection around `TempDir` ownership. |

### Benchmarks to Add

No benchmark is required for this fix pass. Existing performance target can be benchmarked later once route API stabilises.

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| Confirm whether legacy capability aliases like `DeepThinking` must still work | Pending user/product decision | User |
| Decide whether to update `skills/pi-rust-terraphim-router/SKILL.md` in this same patch | Pending approval | Implementer |
| Confirm `demo-route` should load full registry rather than listing-lite registry | Pending implementation check | Implementer |

## Approval

- [ ] Technical review complete
- [ ] Test strategy approved
- [ ] Performance targets agreed
- [ ] Human approval received
