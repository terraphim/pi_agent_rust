# Research Document: pi_terraphim_router Review Fixes

**Status**: Draft  
**Author**: AI Agent  
**Date**: 2026-05-25  
**Reviewers**: User approval required

## Executive Summary

The latest structured review found three issues in the KgRouter-style `pi_terraphim_router` implementation: route readiness is reported without being checked, a documented public helper API was removed, and embedded fallback routing persists temporary directories. These issues are fixable with a small compatibility and correctness pass that preserves the markdown taxonomy architecture while aligning runtime behaviour with the design contract.

The essential fix is to separate pure taxonomy matching from provider readiness selection, restore or explicitly preserve the legacy helper API, and avoid leaking filesystem artefacts when loading embedded taxonomy rules.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energizing? | Yes | The fixes convert a working prototype into a trustworthy router contract for CLI and SDK users. |
| Leverages strengths? | Yes | This is a Rust API and integration boundary problem with clear type, test, and behavioural contracts. |
| Meets real need? | Yes | The structured review identified active P1 findings around truthfulness of routing metadata and public API compatibility. |

**Proceed**: Yes - 3/3 YES.

## Problem Statement

### Description

The new KG routing path correctly loads markdown taxonomy files and matches prompts through `terraphim_automata`, but it still has review-blocking gaps:

1. `RouteDecision.provider_ready` is always `true` even when no credentials or model registry evidence has been checked.
2. Fallback routes are returned but never used to select a ready provider/model.
3. `get_provider_for_capability()` disappeared from the public API despite existing documentation and examples referencing it.
4. Embedded fallback taxonomy writes to a temp directory and calls `TempDir::keep()`, leaving persistent temp directories after each load.

### Impact

CLI users can receive misleading JSON readiness fields. Library consumers can fail to compile if they depend on the documented helper API. Long-running callers can accumulate unnecessary temporary taxonomy directories.

### Success Criteria

- `RouteDecision.provider_ready` reflects actual model registry readiness where a registry is supplied.
- Route selection can choose the first ready fallback route while still supporting pure taxonomy matching.
- `get_provider_for_capability()` exists again, or a clearly documented compatibility replacement exists with tests.
- Embedded fallback loading does not intentionally leak temporary directories.
- Existing `demo-route` behaviour and all current router tests remain passing.

## Current State Analysis

### Existing Implementation

`src/pi_terraphim_router/mod.rs` now owns the routing flow:

- `Router::load(path)` parses markdown directives from a taxonomy directory.
- `Router::route(prompt)` runs `find_matches()` over an in-memory `Thesaurus`.
- The highest priority matched rule wins.
- The selected provider/model is always the first route in `directives.routes`.
- `provider_ready` is set to `true` unconditionally.
- `Router::from_embedded()` writes embedded markdown files into a temp directory and calls `tmp_dir.keep()`.

Provider readiness exists in `src/models.rs` as `model_entry_is_ready(entry: &ModelEntry) -> bool`, but it is currently `pub(crate)` and only works on a `ModelEntry`. `ModelRegistry::find(provider, id)` can locate entries, and `ModelRegistry::available_models()` already filters by readiness.

### Code Locations

| Component | Location | Purpose |
|-----------|----------|---------|
| Router core | `src/pi_terraphim_router/mod.rs` | Taxonomy loading, matching, route selection, embedded fallback. |
| Router types | `src/pi_terraphim_router/types.rs` | `RouteDecision`, `RouterConfig`, `RouterInput`, `RouterOutput`, `ProviderSelection`. |
| Router errors | `src/pi_terraphim_router/error.rs` | Error variants for routing and subprocess/RPC failures. |
| Demo CLI integration | `src/main.rs` | `Commands::DemoRoute` handler for text/JSON route output. |
| Model registry | `src/models.rs` | `ModelRegistry`, `ModelEntry`, `model_entry_is_ready()`, credential readiness. |
| Embedded taxonomy | `resources/routing_taxonomy/*.md` | Planning, implementation, and review default tiers. |
| Router skill docs | `skills/pi-rust-terraphim-router/SKILL.md` | Still documents `get_provider_for_capability()`. |

### Data Flow

Current `demo-route` flow:

```text
Prompt -> default_router() -> embedded/user taxonomy -> Thesaurus -> find_matches()
       -> highest priority concept -> first route -> RouteDecision(provider_ready=true)
       -> JSON/text output
```

Current `route_and_execute()` flow:

```text
RouterInput -> explicit provider/model? -> RpcClient
            -> otherwise default_router().route(prompt)
            -> selected first route -> RpcClient
            -> fallback anthropic/claude-sonnet-4-6 only when no KG match
```

### Integration Points

- `ModelRegistry::load()` requires `AuthStorage` and optional models path.
- `ModelRegistry::find(provider, id)` returns an owned `ModelEntry` if present.
- `model_entry_is_ready(&ModelEntry)` can be called inside the crate, including from `pi_terraphim_router`.
- `demo-route` currently runs in the fast CLI path and should avoid surprising heavyweight side effects where possible.

## Constraints

### Technical Constraints

- The router module is behind the `terraphim-routing` feature flag.
- Routing must remain synchronous for taxonomy matching.
- No new SQLite/persistence dependency should be introduced for routing.
- Existing public APIs documented by the local skill should not disappear without a deliberate migration.
- Rust 2024 nightly and `#![forbid(unsafe_code)]` apply.
- Tests must not use mocks per repository instruction; use real `ModelRegistry` construction helpers or concrete fixtures.

### Business Constraints

- The feature is early-stage but still has SDK-facing public functions.
- The fix should be minimal and targeted to review findings rather than expanding router scope.

### Non-Functional Requirements

| Requirement | Target | Current |
|-------------|--------|---------|
| Route matching latency | Keep effectively in-memory and synchronous | `find_matches()` path is synchronous; readiness integration must not make pure route matching expensive. |
| Filesystem hygiene | No persistent temp directory per route load | Current embedded fallback calls `TempDir::keep()`. |
| Metadata truthfulness | `provider_ready` must reflect readiness or be unavailable | Current value is always `true`. |
| API compatibility | Preserve documented helper functions | `get_provider_for_capability()` removed. |

## Vital Few (Essentialism)

### Essential Constraints (Max 3)

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| Truthful readiness metadata | JSON route output should not claim unavailable providers are ready. | Structured review P1; `provider_ready` currently unconditional. |
| Public API compatibility | Existing documented helper usage should keep compiling. | `skills/pi-rust-terraphim-router/SKILL.md` references `get_provider_for_capability()`. |
| No tempdir leakage | Embedded taxonomy fallback is called by default and should not leave artefacts per invocation. | `Router::from_embedded()` calls `tmp_dir.keep()`. |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|----------------|
| Adaptive/online routing learning | Not part of review findings and explicitly out of original scope. |
| New daemon/service for routing | Over-engineering; CLI/library path is sufficient. |
| SQLite or persistence cache for taxonomy | Original design rejected persistence for routing. |
| Full provider health monitoring | Review needs credential readiness, not network liveness. |
| Taxonomy priority redesign | Broad synonym interactions are known behaviour and not a blocking review finding. |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|------------|--------|------|
| `ModelRegistry` | Needed to determine whether a taxonomy route is ready. | Loading it in every pure route call could add overhead if designed poorly. |
| `AuthStorage` | Supplies credentials for registry readiness. | CLI fast path may need to load auth only for readiness-aware route output. |
| `terraphim_automata` | Provides markdown directive parsing and Aho-Corasick matching. | No change expected. |
| `tempfile` | Current embedded fallback uses temp dirs. | Can keep if the `TempDir` is owned by `Router`; avoid `keep()`. |

### External Dependencies

| Dependency | Version | Risk | Alternative |
|------------|---------|------|-------------|
| `terraphim_automata` | Path dependency | API may lack direct parse-from-string helper. | Keep tempdir but own it in `Router`. |
| `terraphim_types` | Path dependency | `RouteDirective` model controls action/provider/model fields. | No alternative needed. |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Readiness checks make `demo-route` slower | Medium | Medium | Keep `Router::route()` pure; add readiness-aware method that accepts a preloaded registry. |
| Reintroduced compatibility helper has changed semantics | Medium | Medium | Define it as concept/taxonomy route lookup and add docs/tests for current semantics. |
| Tempdir ownership changes require extra field on `Router` | High | Low | Add an internal `embedded_tempdir: Option<TempDir>` field hidden from Debug. |
| Route IDs differ from legacy capability names | High | Medium | Preserve helper name but document it now accepts taxonomy concept IDs, not old enum variants. |

### Open Questions

1. Should `demo-route --format json` pay the cost to load `AuthStorage` + `ModelRegistry` so `provider_ready` is fully truthful? Recommended: yes, because it is a diagnostic route command.
2. Should `get_provider_for_capability()` retain legacy capability names like `DeepThinking`, or map only current taxonomy concepts like `planning_tier`? Recommended: map current taxonomy concepts and optionally preserve a small alias layer only if existing documented legacy names must work.

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|------------|-------|---------------|-----------|
| `pi_terraphim_router` can call `model_entry_is_ready()` because it is in the same crate. | Function is `pub(crate)` in `src/models.rs`. | If feature/module boundaries prevent access, a public crate-internal wrapper is needed. | Yes |
| `ModelRegistry::find(provider, id)` is sufficient to assess route readiness. | It returns a `ModelEntry` after canonical provider/model lookup. | Unknown models may need a separate unavailable state. | Partially |
| Restoring `get_provider_for_capability()` is preferable to documenting a breaking change. | Repository guidance says not to break CLI/SDK for this feature. | Function semantics may be slightly different from legacy mapper. | Yes |
| Owning `TempDir` inside `Router` is the smallest safe fix if parsing requires files. | Current parser accepts a directory path. | If parser has a string API, in-memory would be cleaner. | Partially |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen/Rejected |
|----------------|--------------|---------------------|
| `provider_ready` means taxonomy route exists | Simple, but misleading. | Rejected: field name and review expect credential readiness. |
| `provider_ready` means configured credentials/model available | Requires registry lookup. | Chosen: matches `model_entry_is_ready()` semantics. |
| `get_provider_for_capability()` must support old enum capabilities | Requires compatibility alias mapping. | Deferred unless user confirms legacy names are required. |
| `get_provider_for_capability()` maps taxonomy concept IDs | Minimal compatibility restoration. | Chosen as the smallest non-breaking compile fix. |

## Research Findings

### Key Insights

1. The KG router already has the correct matching architecture; the fixes are around contract truthfulness and lifecycle hygiene, not a rewrite.
2. Pure taxonomy matching and readiness-aware selection should be separate APIs so fast callers do not implicitly load auth/model state.
3. The existing `RouteDecision` type can remain the JSON output contract if readiness-aware methods populate `provider_ready` accurately.
4. The tempdir issue can be fixed without changing taxonomy parsing by storing the `TempDir` in `Router` instead of keeping it permanently.

### Relevant Prior Art

- Existing `ModelRegistry::available_models()` filters via `model_entry_is_ready()`.
- Existing `Router::route()` tests are good coverage for pure taxonomy behaviour and should remain valid.
- The previous public example and skill documentation established `get_provider_for_capability()` as a consumer-facing helper.

### Technical Spikes Needed

| Spike | Purpose | Estimated Effort |
|-------|---------|------------------|
| Confirm parse-from-string support in `terraphim_automata` | Determine whether tempdir can be eliminated entirely. | 15 minutes |
| Build readiness test with real `ModelRegistry::from_entries_for_tests()` | Verify no mocks are needed. | 30 minutes |

## Recommendations

### Proceed/No-Proceed

Proceed. The review findings are narrow, well-understood, and should be fixed before merging the router feature.

### Scope Recommendations

- Add readiness-aware route selection while preserving pure `Router::route()`.
- Restore `get_provider_for_capability()` with taxonomy concept semantics.
- Store embedded tempdir ownership in `Router`, or use a parser path that avoids disk entirely if available.
- Add tests specifically targeting each review finding.

### Risk Mitigation Recommendations

- Keep all changes inside `pi_terraphim_router` and the `demo-route` handler.
- Avoid introducing new dependencies.
- Add focused tests before/with each fix.
- Run `cargo test --features terraphim-routing -- pi_terraphim_router`, `cargo clippy --features terraphim-routing --all-targets -- -D warnings`, and `cargo fmt --check`.

## Next Steps

If approved:

1. Implement readiness-aware route selection helpers.
2. Restore `get_provider_for_capability()` and update example/docs if needed.
3. Fix embedded taxonomy tempdir lifecycle.
4. Add targeted tests for all three review findings.
5. Re-run quality gates and request re-review.

## Appendix

### Reference Materials

- `.docs/design-pi-rust-terraphim-router.md`
- `.docs/research-pi-rust-terraphim-router-v2.md`
- `src/pi_terraphim_router/mod.rs`
- `src/models.rs`
- `skills/pi-rust-terraphim-router/SKILL.md`

### Code Snippets

Current problematic readiness assignment:

```rust
provider_ready: true,
```

Current embedded fallback persistence:

```rust
Self::load(tmp_dir.keep())
```
