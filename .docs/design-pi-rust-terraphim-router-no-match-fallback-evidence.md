# Implementation Plan: pi_terraphim_router No-Match Fallback Evidence

**Status**: Draft  
**Research Doc**: `.docs/research-pi-rust-terraphim-router-no-match-fallback-evidence.md`  
**Author**: AI Agent  
**Date**: 2026-05-26  
**Estimated Effort**: 45-75 minutes

## Overview

### Summary

Close the remaining P2 verification gap by making no-match fallback evidence accurate and testable. The implementation should not alter public APIs or runtime behaviour; it should align private execution-selection tests with the behaviour they claim to prove.

### Approach

Make the fallback metadata a private, testable `ExecutionSelection`, then adjust tests so each scenario is explicit:

1. Keep `select_execution_route()` returning `None` for no taxonomy match.
2. Add a private `no_match_fallback_selection()` helper that returns the existing Anthropic fallback metadata.
3. Use that helper in `route_and_execute_with_registry()` so fallback metadata has one source of truth.
4. Rename the current misleading test to describe matched primary selection when no registry route is ready.
5. Add a true no-match planner test and a fallback metadata test.

### Scope

**In Scope:**

- Private helper for no-match fallback metadata.
- Test rename for accurate behaviour description.
- New test proving no-match planner returns `None`.
- New test proving no-match fallback metadata.
- Focused quality gates for router tests and formatting.

**Out of Scope:**

- Public API changes.
- Real RPC/provider process tests.
- Mocks or fake `RpcClient` implementations.
- Changing fallback provider/model.
- Addressing unrelated baseline clippy/UBS findings.

**Avoid At All Cost** (from 5/25 analysis):

- Adding test-only production abstractions around `RpcClient`.
- Refactoring broader execution flow.
- Converting no-match fallback into an error.
- Loading auth/model registry inside pure route helpers.

## Architecture

### Component Diagram

```text
RouterInput + Router + optional ModelRegistry
    |
    v
select_execution_route()
    |-- explicit provider/model --> ExecutionSelection
    |-- taxonomy match ----------> ExecutionSelection
    '-- no taxonomy match -------> None
                                      |
                                      v
                         no_match_fallback_selection()
                                      |
                                      v
                              ExecutionSelection
                                      |
                                      v
                              execute_selection()
```

### Data Flow

```text
Non-matching prompt
 -> select_execution_route(input, router, Some(registry))
 -> None
 -> no_match_fallback_selection()
 -> ExecutionSelection {
      provider: "anthropic",
      model: "claude-sonnet-4-6",
      capabilities: [],
      confidence: 0.0,
      reason: "fallback: no kg route matched",
      fallback_used: true
    }
 -> execute_selection(input, selection)
```

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Keep `select_execution_route()` returning `None` for no match | Preserves separation between route planning and execution fallback policy. | Making planner always return fallback selection. |
| Add `no_match_fallback_selection()` | Makes fallback metadata testable without RPC spawn and avoids string drift. | Duplicating fallback strings in tests and execution. |
| Rename the misleading test | Test names should describe the exercised behaviour. | Keeping a misleading test name and adding comments. |
| Do not test public async fallback by spawning RPC | Avoids external process dependency and mocks. | Mocking `RpcClient`; requiring provider process in unit tests. |

### Eliminated Options (Essentialism)

| Option Rejected | Why Rejected | Risk of Including |
|-----------------|--------------|-------------------|
| Mock `RpcClient` | Project instruction says no mocks in tests. | Brittle test-only abstraction. |
| Change fallback execution behaviour | Runtime behaviour is not the problem. | New regression risk. |
| Add integration harness for RPC fallback | Too large for a P2 evidence gap. | Slow, environment-dependent test. |
| Fix unrelated scanner baselines | Outside this review gap. | Scope creep. |

### Simplicity Check

> "Minimum code that solves the problem. Nothing speculative."

The simplest correct design is one private helper and two targeted tests. It proves the actual no-match condition and the exact fallback metadata without changing public API or requiring a provider process.

**Senior Engineer Test**: This is not overcomplicated if the helper has no parameters and simply returns the existing fallback metadata as `ExecutionSelection`.

**Nothing Speculative Checklist**:

- [x] No features the user did not request
- [x] No abstractions in case we need them later
- [x] No flexibility just in case
- [x] No new runtime error handling
- [x] No premature optimisation

## File Changes

### New Files

| File | Purpose |
|------|---------|
| None | Existing router module is sufficient. |

### Modified Files

| File | Changes |
|------|---------|
| `src/pi_terraphim_router/mod.rs` | Add private fallback metadata helper, update registry-aware fallback branch, rename/split tests. |
| `.docs/research-pi-rust-terraphim-router-no-match-fallback-evidence.md` | Phase 1 research for this evidence gap. |
| `.docs/design-pi-rust-terraphim-router-no-match-fallback-evidence.md` | This implementation plan. |

### Deleted Files

| File | Reason |
|------|--------|
| None | No deletion required. |

## API Design

### Public Types

No public type changes.

### Public Functions

No public function changes.

### Private Functions

Add:

```rust
fn no_match_fallback_selection() -> ExecutionSelection {
    ExecutionSelection {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-6".to_string(),
        capabilities: vec![],
        confidence: 0.0,
        reason: "fallback: no kg route matched".to_string(),
        fallback_used: true,
    }
}
```

Use in `route_and_execute_with_registry()`:

```rust
let selection = select_execution_route(&input, &router, Some(registry))
    .unwrap_or_else(no_match_fallback_selection);
execute_selection(input, selection).await
```

Do not change `route_and_execute(input)` unless a separate design decision approves sharing fallback metadata there too. Optional minimal sharing is allowed only if it does not increase diff size meaningfully.

### Error Types

No error type changes.

## Test Strategy

### Unit Tests

| Test | Location | Purpose |
|------|----------|---------|
| `test_execution_selection_with_registry_uses_primary_when_none_ready` | `src/pi_terraphim_router/mod.rs` | Renamed current test; proves matched route with no ready registry still selects primary. |
| `test_execution_selection_with_registry_returns_none_without_match` | `src/pi_terraphim_router/mod.rs` | Uses a non-matching prompt and proves planner returns `None`. |
| `test_no_match_fallback_selection_uses_default_anthropic_route` | `src/pi_terraphim_router/mod.rs` | Proves fallback metadata without launching RPC. |

### Integration Tests

No new integration test required. The public async function still spawns RPC, so its fallback branch is covered through private metadata helper plus compile-time checks rather than external execution.

### Coverage Expectations

- No test should claim no-match fallback while using a matching prompt.
- Fallback metadata should have one assertion each for provider, model, capabilities, confidence, reason, and `fallback_used`.
- Existing first-ready fallback and explicit preference tests remain unchanged.

## Implementation Steps

### Step 1: Add Private Fallback Selection Helper

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Add `no_match_fallback_selection() -> ExecutionSelection` beside `select_execution_route()`. It must return the same fallback metadata currently hardcoded in `route_and_execute_with_registry()`.  
**Tests:** Add `test_no_match_fallback_selection_uses_default_anthropic_route`.  
**Estimated:** 15 minutes

### Step 2: Use Helper in Registry-Aware Execution

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Replace the manual no-match fallback branch in `route_and_execute_with_registry()` with `unwrap_or_else(no_match_fallback_selection)` and `execute_selection(input, selection).await`. This preserves runtime behaviour while making metadata single-source.  
**Tests:** Existing tests plus fallback metadata helper test.  
**Dependencies:** Step 1  
**Estimated:** 15 minutes

### Step 3: Fix Misleading Test Coverage

**Files:** `src/pi_terraphim_router/mod.rs`  
**Description:** Rename `test_execution_selection_with_registry_uses_no_match_fallback` to `test_execution_selection_with_registry_uses_primary_when_none_ready`. Add `test_execution_selection_with_registry_returns_none_without_match` using a prompt that does not contain any configured synonym.  
**Tests:** The renamed and new tests.  
**Dependencies:** None  
**Estimated:** 15 minutes

### Step 4: Quality Gates

**Files:** All changed files  
**Description:** Run focused and standard checks.  
**Tests:**

```bash
rch exec -- cargo check --all-targets --features terraphim-routing
rch exec -- cargo test --features terraphim-routing -- pi_terraphim_router
cargo fmt --check
```

Run clippy and UBS as evidence, but document known repository baseline noise if unchanged-file findings dominate:

```bash
rch exec -- cargo clippy --all-targets --features terraphim-routing -- -D warnings
ubs --only=rust src/pi_terraphim_router/mod.rs src/main.rs
```

If committing staged Rust changes, prefer changed-line UBS gate when full-file scan is noisy:

```bash
python3 scripts/check_ubs_staged_delta.py
```

**Dependencies:** Steps 1-3  
**Estimated:** 20-30 minutes

## Rollback Plan

If issues are discovered:

1. Revert the no-match evidence follow-up commit.
2. Keep `91b16777` intact because it added the registry-aware execution API.
3. If helper extraction proves undesirable, use the minimal test-only approach: assert `select_execution_route()` returns `None` for a non-matching prompt and leave runtime code unchanged.

## Migration

No migration required. No public API changes.

## Dependencies

### New Dependencies

| Crate | Version | Justification |
|-------|---------|---------------|
| None | N/A | Existing private types are sufficient. |

### Dependency Updates

| Crate | From | To | Reason |
|-------|------|----|--------|
| None | N/A | N/A | No dependency changes needed. |

## Performance Considerations

### Expected Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Route planning latency | No material change | One private helper only used on no-match branch. |
| RPC execution latency | No change | Same provider/model fallback and same `execute_selection()` path. |
| Memory | No material change | One short-lived `ExecutionSelection`. |

### Benchmarks to Add

No benchmark required. This is a testability/evidence fix with no meaningful hot-path change.

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| Decide whether to also use `no_match_fallback_selection()` in compatibility `route_and_execute(input)` | Optional, pending implementation judgement | Implementer |
| Decide whether baseline scanner findings need separate issue tracking | Outside scope | User/repo owner |

## Approval

- [ ] Technical review complete
- [ ] Test strategy approved
- [ ] Performance targets agreed
- [ ] Human approval received
