# Research Document: pi_terraphim_router Execution Readiness

**Status**: Draft  
**Author**: AI Agent  
**Date**: 2026-05-26  
**Reviewers**: User approval required

## Executive Summary

The latest structured PR review confirmed that the previous router fixes solved readiness reporting for `demo-route`, restored `get_provider_for_capability()`, and fixed embedded taxonomy tempdir ownership. One P1 correctness gap remains: `route_and_execute()` still calls pure `Router::route()`, so execution can select an unready primary route even when a ready fallback route exists. The fix should extend readiness-aware routing to the execution path without adding hidden IO to pure taxonomy matching.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energising? | Yes | This closes the last P1 review gap in the routing execution contract. |
| Leverages strengths? | Yes | The change is a Rust boundary/API design problem involving routing, registry readiness, and tests. |
| Meets real need? | Yes | Users invoking routed execution should not fail on an unconfigured primary provider when a configured fallback exists. |

**Proceed**: Yes - 3/3 YES.

## Problem Statement

### Description

`Router::route_with_registry()` now performs the correct readiness-aware selection, but `route_and_execute()` still calls `Router::route()`. Pure routing intentionally returns the first taxonomy route with `provider_ready=false`; it does not consult `ModelRegistry`. Because `route_and_execute()` immediately spawns the selected provider/model, the execution path can still ignore ready fallback routes and attempt to execute against an unavailable provider.

The PR review also identified a smaller diagnostic issue: both `demo-route` text handlers print the selected route as `Primary route`. After readiness-aware fallback selection, the selected route can be a later fallback rather than the taxonomy primary route.

### Impact

The affected users are callers of `pi_terraphim_router::route_and_execute()` and any CLI or library path built on it. If a taxonomy rule lists an unready provider first and a ready fallback second, diagnostics now show the correct route in `demo-route`, but execution can still use the unready first provider and fail.

The text label issue affects CLI trust: a user can see a fallback provider printed as primary, making troubleshooting confusing.

### Success Criteria

- Routed execution uses first-ready fallback selection when a model registry is available.
- Pure `Router::route()` remains synchronous, deterministic, and registry-free.
- Existing explicit provider/model preferences keep their current behaviour.
- `route_and_execute()` retains a simple compatibility path or delegates to a new registry-aware execution function.
- `demo-route` text output labels the selected route accurately.
- Focused tests cover ready fallback execution selection without using mocks.

## Current State Analysis

### Existing Implementation

`src/pi_terraphim_router/mod.rs` currently exposes three relevant paths:

- `Router::route(prompt)`: pure taxonomy match; selects the first route; sets `provider_ready=false`.
- `Router::route_with_registry(prompt, registry)`: taxonomy match plus readiness-aware selection; selects the first ready fallback route.
- `route_and_execute(input)`: executes explicit provider/model preferences directly; otherwise calls `default_router()?.route(&input.prompt)` and spawns the selected route.

`src/main.rs` has two `demo-route` handlers that correctly load `AuthStorage`, load `ModelRegistry`, and call `route_with_registry()`. Their text output still says `Primary route` for the selected route.

### Code Locations

| Component | Location | Purpose |
|-----------|----------|---------|
| Pure routing API | `src/pi_terraphim_router/mod.rs:99` | In-memory taxonomy match without registry readiness. |
| Readiness routing API | `src/pi_terraphim_router/mod.rs:105` | Selects the first ready route using `ModelRegistry`. |
| Execution API | `src/pi_terraphim_router/mod.rs:287` | Spawns an RPC client for explicit or routed provider/model execution. |
| Router input/output types | `src/pi_terraphim_router/types.rs:45` | Defines `RouterInput` and `RouterOutput`; no registry-bearing type exists. |
| Fast-path demo command | `src/main.rs:441` | Uses readiness-aware routing for CLI diagnostics. |
| Demo helper command | `src/main.rs:8329` | Uses readiness-aware routing for CLI diagnostics. |
| Model registry readiness | `src/models.rs:610` | `model_entry_is_ready(&ModelEntry)` determines configured credential readiness. |

### Data Flow

Current execution flow:

```text
RouterInput
 -> explicit provider/model? -> RpcClient::spawn -> send_prompt
 -> otherwise default_router()
 -> Router::route(prompt)
 -> first taxonomy route, provider_ready=false
 -> RpcClient::spawn -> send_prompt
 -> fallback anthropic/claude-sonnet-4-6 only when no taxonomy match
```

Desired execution flow when a registry is supplied:

```text
RouterInput + ModelRegistry
 -> explicit provider/model? -> RpcClient::spawn -> send_prompt
 -> otherwise default_router()
 -> Router::route_with_registry(prompt, registry)
 -> first ready fallback route, or primary route if none ready
 -> RpcClient::spawn -> send_prompt
 -> fallback anthropic/claude-sonnet-4-6 only when no taxonomy match
```

### Integration Points

- `ModelRegistry` is already loaded by `demo-route` from `AuthStorage::load(Config::auth_path())` and `default_models_path(&Config::global_dir())`.
- `route_and_execute()` currently lives in the library module and does not have direct access to `Config` or CLI auth loading decisions.
- `RpcClient::spawn(provider, model, working_dir)` remains the execution boundary.
- Tests can construct real registry entries via `ModelRegistry::from_entries_for_tests()` under `#[cfg(test)]`.

## Constraints

### Technical Constraints

- Preserve `Router::route()` as pure, synchronous, and registry-free.
- Do not add SQLite, background daemons, network readiness probes, or persistence caches.
- Keep the `terraphim-routing` feature boundary intact.
- Avoid mocks in tests; use real `ModelEntry` and `ModelRegistry` test constructors.
- Avoid hidden CLI config dependencies inside low-level routing logic unless compatibility requires it.
- Maintain `#![forbid(unsafe_code)]` and clippy `-D warnings`.

### Business Constraints

- The change is a review-fix closure, not a new router redesign.
- Backwards compatibility is only needed where there is a concrete existing API caller. `route_and_execute(input)` is a public function and should continue compiling.
- The implementation should be small enough to review as a targeted follow-up commit.

### Non-Functional Requirements

| Requirement | Target | Current |
|-------------|--------|---------|
| Execution correctness | Select first ready fallback where registry is supplied | `route_and_execute()` ignores registry readiness. |
| Pure matching latency | No extra IO in `Router::route()` | Already pure and should stay that way. |
| Diagnostic clarity | Text output distinguishes selected route from primary route | Text currently says `Primary route` for selected route. |
| API clarity | Registry-aware execution boundary is explicit | No registry-aware execution function exists. |

## Vital Few (Essentialism)

### Essential Constraints (Max 3)

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| Execution must use readiness-aware selection | This is the active P1 review finding and affects runtime behaviour. | `route_and_execute()` calls `route()`, not `route_with_registry()`. |
| Pure routing must stay pure | Prevents hidden auth/model loading in fast deterministic matching paths. | Existing design intentionally separated `route()` and `route_with_registry()`. |
| Public compatibility must remain simple | Existing callers of `route_and_execute(input)` should not break. | `route_and_execute` is public in the feature module. |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|----------------|
| Network provider health checks | Review finding is configured readiness, not endpoint liveness. |
| Retry/failover after RPC execution failure | Larger behaviour change; current issue is pre-execution selection. |
| Adding registry fields to `RouterInput` | Would couple transport input to process-local registry state and complicate serialisation. |
| Changing taxonomy route semantics | Current issue is execution use of existing route semantics. |
| Broad CLI route output redesign | Only the selected-vs-primary label is misleading. |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|------------|--------|------|
| `ModelRegistry` | Required for first-ready fallback selection. | Medium: callers must provide it, or compatibility function must load it. |
| `AuthStorage` and `Config` | Needed only if the compatibility `route_and_execute(input)` loads a registry itself. | Medium: importing CLI config into router module may create undesirable coupling. |
| `RpcClient` | Execution boundary remains unchanged. | Low: selected provider/model are still strings. |
| `RouteDecision` | Carries selected route and fallback chain. | Low: no type change is necessary. |

### External Dependencies

| Dependency | Version | Risk | Alternative |
|------------|---------|------|-------------|
| None new | N/A | N/A | Use existing `ModelRegistry`, `AuthStorage`, and `RpcClient`. |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Low-level router module imports CLI config to load registry | Medium | Medium | Prefer a new `route_and_execute_with_registry(input, registry)` function and keep compatibility wrapper minimal. |
| Existing callers expect `route_and_execute()` to require no auth load | Medium | Medium | Preserve current signature and add the new explicit registry-aware API; decide whether wrapper remains pure or loads registry. |
| Tests accidentally mock readiness rather than using real registry structures | Low | Medium | Reuse existing test helper pattern with concrete `ModelEntry`. |
| `demo-route` text fix drifts between two handlers | Medium | Low | Update both handlers with the same wording and keep existing JSON unchanged. |

### Open Questions

1. Should the public `route_and_execute(input)` compatibility wrapper load `AuthStorage` and `ModelRegistry` itself, or remain pure and delegate only when a caller provides registry via a new function? Recommended: add `route_and_execute_with_registry(input, registry)` and have `route_and_execute(input)` keep existing behaviour unless a natural registry-loading helper already exists in the library layer.
2. Should execution refuse to run when a taxonomy route matches but no provider is ready? Recommended: no for this fix; preserve existing fallback-to-primary behaviour when none are ready, because changing it to an error is a larger behavioural decision.
3. Should text output expose both primary and selected routes? Recommended: for now, rename `Primary route` to `Selected route`; adding both is a larger diagnostic redesign.

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|------------|-------|---------------|-----------|
| `route_with_registry()` already implements correct first-ready route selection. | Existing tests cover ready primary, ready fallback, and none-ready cases. | Execution helper would inherit bad selection. | Yes |
| Execution should preserve explicit provider/model preference behaviour. | Explicit preference branch bypasses taxonomy by design. | Users could lose deterministic overrides. | Yes |
| Public API compatibility matters for `route_and_execute(input)`. | Function is public in the router module. | Removing/changing signature would break consumers. | Yes |
| Adding a new explicit function is safer than hiding registry loading in `RouterInput`. | Keeps routing input serialisable and simple. | If users only call existing function, they may not benefit until wrapper is updated. | Partially |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen/Rejected |
|----------------|--------------|---------------------|
| Fix `route_and_execute()` by loading auth/registry internally | One-function fix for all callers, but introduces hidden config IO in library execution. | Rejected as first choice due to coupling; acceptable only if product wants old API to gain readiness automatically. |
| Add `route_and_execute_with_registry()` and update callers | Clear explicit readiness boundary; preserves pure compatibility API. | Chosen as the minimal clean design. |
| Change `Router::route()` to readiness-aware | Simplifies call sites but violates the pure API constraint. | Rejected. |
| Error when no ready route exists | Prevents attempts against unready providers but changes behaviour beyond the review finding. | Rejected for this targeted patch. |

## Research Findings

### Key Insights

1. The remaining P1 is not in matching logic; it is in execution using the wrong matching API.
2. The clean boundary is a new execution function that accepts `&ModelRegistry` and delegates to shared execution construction.
3. `RouteDecision` already contains all fields needed for `RouterOutput`; no new output type is required.
4. The DemoRoute label fix is independent and should not affect JSON output.

### Relevant Prior Art

- `demo-route` already demonstrates the expected registry-loading flow and call to `route_with_registry()`.
- Existing router tests provide `test_model()` and `test_entry()` helpers that can be reused for execution-selection tests.
- `route_with_registry()` tests prove selection without invoking actual providers, which is sufficient for the routing branch. Full RPC execution tests would require a real provider process and are outside the targeted review fix.

### Technical Spikes Needed

| Spike | Purpose | Estimated Effort |
|-------|---------|------------------|
| Identify current callers of `route_and_execute()` | Decide whether compatibility wrapper must change immediately. | 10 minutes |
| Check `RpcClient` test seams | Determine whether execution branch tests can avoid launching a real provider process. | 20 minutes |

## Recommendations

### Proceed/No-Proceed

Proceed. The gap is narrow, the risk is known, and the implementation can be a small targeted follow-up.

### Scope Recommendations

- Add `route_and_execute_with_registry(input, registry)` in `src/pi_terraphim_router/mod.rs`.
- Refactor shared execution response construction into private helpers only if it reduces duplication without adding abstraction.
- Keep `route_and_execute(input)` public and compiling. Decide in design whether it remains old behaviour or delegates through a registry-loading wrapper.
- Rename DemoRoute text label from `Primary route` to `Selected route` in both handlers.

### Risk Mitigation Recommendations

- Do not modify `Router::route()`.
- Do not add a registry field to `RouterInput`.
- Add focused tests for route selection in execution planning without launching external providers.
- Run `cargo check --all-targets --features terraphim-routing`, focused router tests, clippy, and fmt.

## Next Steps

If approved:

1. Create the implementation plan for explicit registry-aware execution.
2. Add tests that fail if execution planning uses pure `route()` instead of `route_with_registry()`.
3. Implement the new execution function and update any suitable call sites.
4. Update DemoRoute text labels.
5. Run quality gates and request re-review.

## Appendix

### Reference Materials

- `src/pi_terraphim_router/mod.rs`
- `src/pi_terraphim_router/types.rs`
- `src/main.rs`
- `.docs/research-pi-rust-terraphim-router-review-fixes.md`
- `.docs/design-pi-rust-terraphim-router-review-fixes.md`

### Code Snippets

Current problematic execution selection:

```rust
let router = default_router()?;
if let Some(decision) = router.route(&input.prompt) {
    let mut client = RpcClient::spawn(
        &decision.provider,
        &decision.model,
        input.working_dir.as_deref(),
    )?;
}
```

Existing correct selection primitive:

```rust
let decision = router.route_with_registry(prompt, registry);
```
