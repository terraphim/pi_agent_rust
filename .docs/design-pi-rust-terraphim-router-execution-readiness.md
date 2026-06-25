# Implementation Plan: pi_terraphim_router Execution Readiness

**Status**: Draft  
**Research Doc**: `.docs/research-pi-rust-terraphim-router-execution-readiness.md`  
**Author**: AI Agent  
**Date**: 2026-05-26  
**Estimated Effort**: 1.5-2.5 hours (estimated) / ~2h (actual, in progress)

## Overview

### Summary

Close the remaining structured review P1 by making routed execution use the same readiness-aware fallback selection that `demo-route` already uses. Keep pure routing untouched, preserve the existing `route_and_execute(input)` API, and add a minimal registry-aware execution entry point for callers that can provide `ModelRegistry`.

### Approach

Use an explicit execution boundary:

1. Keep `Router::route(prompt)` pure and unchanged.
2. Add `route_and_execute_with_registry(input, registry)` that calls `default_router()?.route_with_registry(&input.prompt, registry)`.
3. Share the actual `RpcClient::spawn()` and `RouterOutput` construction through small private helpers only where duplication would otherwise be direct copy-paste.
4. Keep `route_and_execute(input)` compiling and behaviourally compatible. It can continue using pure routing unless a registry-loading dependency already exists in this module without undesirable coupling.
5. Rename text output labels in both DemoRoute handlers from `Primary route` to `Selected route`.

### Scope

**In Scope:**

- Add a registry-aware execution function.
- Ensure registry-aware execution selects the first ready fallback route.
- Preserve explicit provider/model preference behaviour.
- Preserve existing `route_and_execute(input)` public signature.
- Rename DemoRoute text labels to avoid saying fallback selections are primary.
- Add focused tests for route selection and output metadata.

**Out of Scope:**

- Network health checks.
- Retry/failover after `RpcClient` errors.
- Changing `RouterInput` structure.
- Changing taxonomy files or route priority rules.
- Broad CLI output redesign.
- Removing or renaming `route_and_execute(input)`.

**Avoid At All Cost** (from 5/25 analysis):

- Pulling auth/config loading deeply into pure router matching.
- Adding SQLite or persistent readiness caches.
- Introducing mocks in tests.
- Rewriting router architecture.
- Adding speculative provider health abstractions.

## Architecture

### Component Diagram

```text
Caller with explicit provider/model
    -> route_and_execute_with_registry(input, registry)
    -> execute_selected(provider, model, input, reason, metadata)
    -> RpcClient

Caller without explicit provider/model
    -> default_router()
    -> route_with_registry(prompt, registry)
    -> first ready fallback RouteDecision
    -> execute_decision(input, decision)
    -> RpcClient

Compatibility caller
    -> route_and_execute(input)
    -> existing pure route path, or thin compatibility delegate if registry loading is approved
```

### Data Flow

```text
RouterInput + ModelRegistry
 -> explicit provider/model? yes
    -> provider/model from input
    -> RouterOutput { reason: "explicit preference", fallback_used: false }
 -> explicit provider/model? no
    -> default_router()
    -> route_with_registry(prompt, registry)
    -> selected ready fallback or primary if none ready
    -> RpcClient::spawn(selected.provider, selected.model, working_dir)
    -> RouterOutput { provider, model, matched_concept, confidence, reason }
 -> no taxonomy match
    -> hardcoded no-match fallback anthropic/claude-sonnet-4-6
```

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Add `route_and_execute_with_registry()` | Makes readiness dependency explicit and avoids hidden auth/config IO in pure routing. | Changing `route_and_execute(input)` signature; adding registry to `RouterInput`. |
| Keep `route_and_execute(input)` | Avoids breaking public consumers. | Removing the function or requiring all callers to change immediately. |
| Reuse `route_with_registry()` for selection | Avoids duplicating fallback readiness logic. | Reimplementing route readiness inside execution. |
| Rename CLI label to `Selected route` | Accurately describes both primary and fallback selections. | Printing both primary and selected route in this patch. |
| Do not fail when no route is ready | Preserves existing behaviour and avoids broad semantic change. | Returning `ProviderNotReady` when none are ready. |

### Eliminated Options (Essentialism)

| Option Rejected | Why Rejected | Risk of Including |
|-----------------|--------------|-------------------|
| Registry inside `RouterInput` | `RouterInput` is a simple data object and should not carry process-local registry state. | Type coupling and poor serialisation story. |
| Make `Router::route()` load registry | Violates pure matching contract. | Hidden IO and hard-to-test behaviour. |
| Automatic RPC retry through every fallback | Larger execution semantics change than the review finding. | Latency, duplicate side effects, unclear error reporting. |
| Provider readiness network probes | Not required to know whether credentials/models are configured. | Slow, flaky, external dependency in unit path. |
| Full CLI diagnostics redesign | Only the selected route label is wrong. | More review surface and user-facing churn. |

### Simplicity Check

> "Minimum code that solves the problem. Nothing speculative."

The easy version is to expose the execution equivalent of `route_with_registry()`: take the same `RouterInput`, add a `&ModelRegistry`, and call the existing readiness-aware selection before spawning the RPC client. The current pure `route_and_execute(input)` remains available, while new and updated callers can opt into truthful readiness-aware execution.

**Senior Engineer Test**: This should not look overcomplicated. It adds one explicit function and, if needed, one private helper to avoid duplicating `RpcClient` response construction.

**Nothing Speculative Checklist**:

- [x] No features the user did not request
- [x] No abstractions in case we need them later
- [x] No flexibility just in case
- [x] No new error handling for scenarios outside the review finding
- [x] No premature optimisation

## File Changes

### New Files

| File | Purpose |
|------|---------|
| None | Existing router and CLI files are sufficient. |

### Modified Files

| File | Changes |
|------|---------|
| `src/pi_terraphim_router/mod.rs` | Add `route_and_execute_with_registry()`, optionally add private execution helper, add focused tests. |
| `src/main.rs` | Rename DemoRoute text label from `Primary route` to `Selected route` in both handlers. |
| `.docs/design-pi-rust-terraphim-router-execution-readiness.md` | This implementation plan. |
| `.docs/research-pi-rust-terraphim-router-execution-readiness.md` | Phase 1 research for the remaining P1. |

### Deleted Files

| File | Reason |
|------|--------|
| None | No deletion is required. |

## API Design

### Public Types

No new public types are required.

Existing `RouterInput` stays unchanged:

```rust
pub struct RouterInput {
    pub prompt: String,
    pub strategy: Option<String>,
    pub preferred_provider: Option<String>,
    pub preferred_model: Option<String>,
    pub system_prompt: Option<String>,
    pub working_dir: Option<PathBuf>,
}
```

Existing `RouterOutput` stays unchanged:

```rust
pub struct RouterOutput {
    pub response: String,
    pub provider: String,
    pub model: String,
    pub capabilities: Vec<String>,
    pub confidence: f64,
    pub reason: String,
    pub fallback_used: bool,
}
```

### Public Functions

Add this public function in `src/pi_terraphim_router/mod.rs`:

```rust
pub async fn route_and_execute_with_registry(
    input: RouterInput,
    registry: &crate::models::ModelRegistry,
) -> RouterResult<RouterOutput>;
```

Behaviour:

- If `input.preferred_provider` and `input.preferred_model` are both present, execute those directly and return `reason: "explicit preference"`.
- Otherwise load `default_router()` and call `route_with_registry(&input.prompt, registry)`.
- If a route matches, execute the selected provider/model from the readiness-aware `RouteDecision`.
- If no route matches, preserve the current hardcoded fallback output.

Keep existing function:

```rust
pub async fn route_and_execute(input: RouterInput) -> RouterResult<RouterOutput>;
```

Compatibility decision:

- Preferred: keep its current signature and existing pure-route behaviour for now, and document that readiness-aware execution requires `route_and_execute_with_registry()`.
- Optional if approved: make it load `AuthStorage` and `ModelRegistry` only if doing so does not introduce unacceptable module coupling.

### Private Functions

If implementation duplication appears, add one private helper:

```rust
async fn execute_provider_model(
    input: &RouterInput,
    provider: String,
    model: String,
    capabilities: Vec<String>,
    confidence: f64,
    reason: String,
    fallback_used: bool,
) -> RouterResult<RouterOutput>;
```

If this helper makes the call sites less readable due to many parameters, do not add it. Duplicate the small `RpcClient::spawn()` block instead. Minimal clarity wins.

### Error Types

No new error variants are required.

`RouterError::ProviderNotReady` should remain unused for this targeted patch unless a later product decision changes no-ready-route behaviour from "try primary" to "return error".

## Test Strategy

### Unit Tests

| Test | Location | Purpose |
|------|----------|---------|
| `test_route_with_registry_uses_first_ready_fallback` | Existing in `src/pi_terraphim_router/mod.rs` | Continue proving selection primitive. |
| `test_execution_selection_with_registry_uses_first_ready_fallback` | `src/pi_terraphim_router/mod.rs` | Verify the execution planning path chooses the ready fallback route. |
| `test_execution_selection_with_registry_preserves_explicit_preference` | `src/pi_terraphim_router/mod.rs` | Verify explicit provider/model still bypass taxonomy selection. |
| `test_execution_selection_with_registry_uses_no_match_fallback` | `src/pi_terraphim_router/mod.rs` | Verify no taxonomy match still uses existing hardcoded fallback metadata. |

### Testability Note

Directly testing `route_and_execute_with_registry()` may launch `RpcClient`, which can require an external provider process. To avoid mocks and still test the routing correctness, first extract a small private planning helper that performs provider/model selection without executing RPC:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutionSelection {
    provider: String,
    model: String,
    capabilities: Vec<String>,
    reason: String,
    fallback_used: bool,
}

fn select_execution_route(
    input: &RouterInput,
    router: &Router,
    registry: Option<&crate::models::ModelRegistry>,
) -> Option<ExecutionSelection>;
```

Guidance:

- Keep this helper private.
- Use `registry: Option<&ModelRegistry>` only if it lets both compatibility and registry-aware execution share the same planner without awkward duplication.
- If the helper becomes complex, split into two helpers: one pure compatibility planner and one registry-aware planner.
- Tests should assert selected provider/model and metadata, not RPC execution.

### Integration Tests

| Test | Location | Purpose |
|------|----------|---------|
| Existing focused router test command | `cargo test --features terraphim-routing -- pi_terraphim_router` | Verify router unit tests and type tests. |
| Manual CLI smoke test | Optional | Run `pi demo-route "implement auth"` and verify text says `Selected route`. |

### Coverage Expectations

- At least one test must fail if registry-aware execution accidentally calls `Router::route()` instead of `Router::route_with_registry()`.
- At least one test must cover explicit provider/model precedence.
- Existing 46 router tests must continue to pass.

## Implementation Steps

### Step 1: Add Execution Selection Helper

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Add a private helper that chooses the provider/model and output metadata for execution without spawning `RpcClient`. It should support explicit preferences, registry-aware taxonomy routing, pure taxonomy routing if needed, and no-match fallback.  
**Tests:** Add tests for ready fallback, explicit preference, and no-match fallback selection.  
**Estimated:** 35 minutes

Possible shape:

```rust
struct ExecutionSelection {
    provider: String,
    model: String,
    capabilities: Vec<String>,
    confidence: f64,
    reason: String,
    fallback_used: bool,
}
```

### Step 2: Add Registry-Aware Execution Function

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Implement `route_and_execute_with_registry(input, registry)` using the selection helper. Execute the selected provider/model via `RpcClient::spawn()` and return `RouterOutput`.  
**Tests:** Covered indirectly by selection tests; add a compile-level or minimal unit test only if it does not launch external processes.  
**Dependencies:** Step 1  
**Estimated:** 25 minutes

Key code:

```rust
pub async fn route_and_execute_with_registry(
    input: RouterInput,
    registry: &crate::models::ModelRegistry,
) -> RouterResult<RouterOutput> {
    let router = default_router()?;
    let selection = select_execution_route(&input, &router, Some(registry));
    execute_selection(input, selection).await
}
```

### Step 3: Preserve Compatibility Function

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Keep `route_and_execute(input)` public. Prefer leaving its no-registry behaviour intact while making its implementation share the new selection/execution helper. This avoids breaking callers and prevents hidden config loading in the router module.  
**Tests:** Existing tests plus a compatibility selection test if helper is shared.  
**Dependencies:** Steps 1-2  
**Estimated:** 20 minutes

Decision checkpoint:

- If reviewers require existing `route_and_execute(input)` itself to become readiness-aware, stop and decide where `AuthStorage` and `ModelRegistry` should be loaded. Do not silently import CLI config into router logic without approval.

### Step 4: Fix DemoRoute Text Labels

**Files:** `src/main.rs`  
**Description:** Replace `Primary route:` with `Selected route:` in both DemoRoute text handlers. JSON output remains unchanged.  
**Tests:** `cargo fmt --check`; optional snapshot/manual CLI smoke if a suitable test exists.  
**Dependencies:** None  
**Estimated:** 10 minutes

Exact replacements:

```text
Primary route: ...
```

to:

```text
Selected route: ...
```

### Step 5: Documentation Notes

**Files:** `.docs/design-pi-rust-terraphim-router-execution-readiness.md` and optionally router skill docs  
**Description:** If the new public function is added, mention that registry-aware execution should use `route_and_execute_with_registry()`. Do not claim strict drop-in behaviour beyond what is implemented.  
**Tests:** Documentation review only.  
**Dependencies:** Step 2  
**Estimated:** 10 minutes

### Step 6: Quality Gates

**Files:** All changed files  
**Description:** Run required checks.  
**Tests:**

```bash
rch exec -- cargo check --all-targets --features terraphim-routing
rch exec -- cargo test --features terraphim-routing -- pi_terraphim_router
rch exec -- cargo clippy --all-targets --features terraphim-routing -- -D warnings
cargo fmt --check
```

If committing, also run staged UBS according to repository workflow:

```bash
ubs --staged --only=rust .
```

**Dependencies:** Steps 1-5  
**Estimated:** 30-45 minutes

## Rollback Plan

If issues are discovered:

1. Revert only the execution-readiness follow-up commit.
2. Keep the previous router readiness API commit intact, since it fixed demo diagnostics and tempdir ownership.
3. If `route_and_execute_with_registry()` causes API concerns, remove only that new function and keep the private selection helper tests as design evidence until the API boundary is settled.

The feature remains behind `terraphim-routing`, so non-feature builds are unaffected.

## Migration

No data migration required.

Consumer migration recommendation:

```rust
// Existing, compatibility behaviour:
route_and_execute(input).await

// New, readiness-aware behaviour:
route_and_execute_with_registry(input, &registry).await
```

## Dependencies

### New Dependencies

| Crate | Version | Justification |
|-------|---------|---------------|
| None | N/A | Existing crates and modules are sufficient. |

### Dependency Updates

| Crate | From | To | Reason |
|-------|------|----|--------|
| None | N/A | N/A | No dependency changes needed. |

## Performance Considerations

### Expected Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Pure route latency | No change | `Router::route()` unchanged. |
| Registry-aware execution selection | O(number of fallback routes) registry lookups | Existing `route_with_registry()` behaviour. |
| RPC execution latency | No additional network calls before spawn | No health probes or retries added. |

### Benchmarks to Add

No benchmark is required for this targeted fix. The added selection work is bounded by the route fallback list and occurs before an RPC call, so it is not expected to be material.

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| Decide whether `route_and_execute(input)` should internally load registry or remain compatibility-pure | Pending approval | User/reviewer |
| Decide whether to document `route_and_execute_with_registry()` in user-facing router skill docs | Pending approval | Implementer |

## Approval

- [ ] Technical review complete
- [ ] Test strategy approved
- [ ] Performance targets agreed
- [ ] Human approval received
