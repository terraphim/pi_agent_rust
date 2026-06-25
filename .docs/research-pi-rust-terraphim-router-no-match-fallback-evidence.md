# Research Document: pi_terraphim_router No-Match Fallback Evidence

**Status**: Draft  
**Author**: AI Agent  
**Date**: 2026-05-26  
**Reviewers**: User approval required

## Executive Summary

The latest structured review found no active P0/P1 runtime issue, but identified a P2 verification gap: `test_execution_selection_with_registry_uses_no_match_fallback` does not actually exercise a no-match path. The implementation contains a no-match fallback branch in `route_and_execute_with_registry()`, but current test evidence only proves matched-route selection with no ready registry entry. This research scopes a minimal follow-up to align test names, behaviour, and validation evidence.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energising? | Yes | It closes the last known review evidence gap and improves trust in the router execution proof. |
| Leverages strengths? | Yes | This is a precise Rust testability and traceability issue. |
| Meets real need? | Yes | Review and validation require evidence that actual no-match fallback behaviour is covered. |

**Proceed**: Yes - 3/3 YES.

## Problem Statement

### Description

The current test named `test_execution_selection_with_registry_uses_no_match_fallback` creates a rule with `synonyms:: implement` and then passes `RouterInput::new("implement something")`. Because the prompt matches the taxonomy synonym, `select_execution_route()` returns a matched `ExecutionSelection` for `unknown-provider/unknown-model`. That validates matched taxonomy behaviour, not no-match fallback behaviour.

The real no-match fallback branch lives in `route_and_execute_with_registry()` after `select_execution_route()` returns `None`. Directly testing that public async function would attempt to spawn `RpcClient`, which is outside the intended no-mock unit test boundary.

### Impact

The implementation may be correct, but validation evidence is incomplete. A future regression could break the no-match fallback branch without the current test suite noticing, while the test name would continue to suggest the branch is covered.

### Success Criteria

- A test explicitly proves that `select_execution_route()` returns `None` when no taxonomy rule matches.
- A test or helper proves that the no-match fallback selection metadata is Anthropic `claude-sonnet-4-6`, `fallback_used=true`, `confidence=0.0`, and `reason="fallback: no kg route matched"` without launching RPC.
- The existing matched-route-with-no-ready-registry behaviour remains covered under an accurate test name.
- No network calls, process spawning, mocks, or new dependencies are introduced.
- Focused router tests, `cargo check`, clippy, and fmt remain clean except for already-known repository baseline scanner noise.

## Current State Analysis

### Existing Implementation

`select_execution_route(input, router, registry)` returns `Some(ExecutionSelection)` for explicit preferences and taxonomy matches. It returns `None` if the prompt does not match any rule. `route_and_execute_with_registry(input, registry)` handles that `None` by constructing and executing the hardcoded no-match fallback provider/model.

The test suite includes new execution-selection tests, but the no-match fallback test still uses a matching prompt. Therefore it only proves the fallback-to-primary behaviour when a matching taxonomy route has no ready registry entry.

### Code Locations

| Component | Location | Purpose |
|-----------|----------|---------|
| Execution planner | `src/pi_terraphim_router/mod.rs:297` | Selects explicit or taxonomy route metadata without spawning RPC. |
| Registry-aware execution API | `src/pi_terraphim_router/mod.rs:410` | Executes registry-aware selection and handles no-match fallback branch. |
| No-match fallback branch | `src/pi_terraphim_router/mod.rs:435` | Constructs Anthropic fallback metadata and spawns RPC. |
| Existing misleading test | `src/pi_terraphim_router/mod.rs:912` | Claims no-match fallback, but uses matching prompt. |
| Existing test helpers | `src/pi_terraphim_router/mod.rs` test module | `write_rule`, `test_entry`, and real `ModelRegistry` construction. |

### Data Flow

Current no-match behaviour:

```text
RouterInput prompt
 -> select_execution_route(input, router, Some(registry))
 -> router.route_with_registry(prompt, registry)
 -> no taxonomy match
 -> None
 -> route_and_execute_with_registry fallback branch
 -> provider=anthropic, model=claude-sonnet-4-6, fallback_used=true
 -> RpcClient::spawn(...)
```

Current test behaviour:

```text
Prompt "implement something"
 -> matches synonym "implement"
 -> select_execution_route returns Some(unknown-provider/unknown-model)
 -> assertions expect kg route match
```

### Integration Points

- `Router::route_with_registry()` is already tested for first-ready fallback selection.
- `select_execution_route()` is private but testable from the internal test module.
- `route_and_execute_with_registry()` is public but not ideal for pure unit tests because it invokes `RpcClient::spawn()`.

## Constraints

### Technical Constraints

- Do not introduce mocks for `RpcClient` or provider processes.
- Do not alter public API shape unless needed for testability.
- Keep no-match fallback values identical to existing execution behaviour.
- Preserve `Router::route()` and `route_with_registry()` semantics.
- Avoid broad refactors or changes outside `src/pi_terraphim_router/mod.rs` unless formatting requires it.

### Business Constraints

- This is a review evidence fix, not a feature expansion.
- The change should be small, low-risk, and suitable for immediate re-review.

### Non-Functional Requirements

| Requirement | Target | Current |
|-------------|--------|---------|
| Test truthfulness | Test names match exercised behaviour | One test name does not match behaviour. |
| No external execution in unit tests | No provider process spawn | Current planner tests avoid spawn; public no-match branch remains untested. |
| Minimal review surface | One module plus docs at most | Achievable. |

## Vital Few (Essentialism)

### Essential Constraints (Max 3)

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| Prove no-match planner behaviour | Without this, validation evidence remains conditional. | Review finding on `test_execution_selection_with_registry_uses_no_match_fallback`. |
| Preserve no-mock unit tests | Repository instruction forbids mocks and public execution spawns RPC. | Existing plan used private planning helper for this reason. |
| Keep runtime behaviour unchanged | This is an evidence gap, not a runtime bug. | Structural review found no active P1 in latest diff. |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|----------------|
| Mocking `RpcClient` | Violates project instruction and adds unnecessary machinery. |
| Testing real RPC fallback execution | Requires external process/provider environment and is not needed to prove selection metadata. |
| Changing fallback provider/model | Existing behaviour is accepted; only evidence is incomplete. |
| Refactoring `route_and_execute()` compatibility path | Not related to the P2 evidence gap. |
| Addressing baseline clippy/UBS findings | These are broader repository issues outside the latest change. |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|------------|--------|------|
| `ExecutionSelection` | Holds route metadata used by execution. | Low: private type can be extended safely. |
| `select_execution_route()` | Returns `None` for no taxonomy match. | Low: current behaviour already supports planner proof. |
| `route_and_execute_with_registry()` | Owns current no-match fallback metadata. | Medium: metadata is duplicated and hard to unit test without spawn. |

### External Dependencies

| Dependency | Version | Risk | Alternative |
|------------|---------|------|-------------|
| None new | N/A | N/A | Use existing test helpers and private planner. |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Duplicating fallback metadata in test expectations can drift | Medium | Low | Extract a private `no_match_fallback_selection()` helper and test it directly. |
| Renaming the existing test without adding fallback coverage leaves branch unproven | Medium | Medium | Add a separate no-match `None` or fallback-selection test. |
| Over-refactoring execution path for a test gap | Low | Medium | Limit implementation to private helper plus test rename/addition. |

### Open Questions

1. Should no-match fallback metadata be represented as `ExecutionSelection` before RPC spawn? Recommended: yes, via a private helper to make behaviour testable without mocks.
2. Should `select_execution_route()` itself return fallback metadata instead of `None`? Recommended: no. It should remain a route planner for explicit/taxonomy selection; no-match fallback is an execution policy.

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|------------|-------|---------------|-----------|
| The no-match fallback branch should remain Anthropic `claude-sonnet-4-6`. | Existing `route_and_execute()` and `route_and_execute_with_registry()` use that fallback. | Changing it would alter runtime behaviour. | Yes |
| Unit tests should avoid `RpcClient::spawn()`. | Existing design explicitly avoided external execution in planner tests. | Public branch remains indirectly tested only via metadata helper. | Yes |
| A private helper is acceptable for testability. | The module already uses private helpers such as `select_execution_route()`. | Slightly more code, but lower drift. | Yes |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen/Rejected |
|----------------|--------------|---------------------|
| Fix only the test prompt to a non-matching prompt and assert `None` | Minimal, proves planner no-match but not fallback metadata. | Useful but incomplete alone. |
| Extract fallback metadata into private helper and test it | Proves no-match fallback metadata without RPC spawn. | Chosen as strongest small evidence fix. |
| Test public async function end-to-end | Strongest runtime proof but requires provider/RPC process. | Rejected for unit scope and no-mock constraint. |

## Research Findings

### Key Insights

1. The no-match fallback branch is not missing; the evidence is missing.
2. The current misleading test should be renamed or split so each test describes one behaviour accurately.
3. A tiny private fallback-selection helper can make the no-match fallback metadata testable without introducing mocks or launching RPC.

### Relevant Prior Art

- Existing execution planner tests validate metadata without invoking `RpcClient`.
- Existing `route_and_execute()` compatibility function duplicates no-match fallback metadata, so helper extraction may also reduce future drift.

### Technical Spikes Needed

| Spike | Purpose | Estimated Effort |
|-------|---------|------------------|
| None | Current code and tests reveal the issue clearly. | N/A |

## Recommendations

### Proceed/No-Proceed

Proceed. The change is small, targeted, and directly addresses the remaining review/validation gap.

### Scope Recommendations

- Add a private `no_match_fallback_selection()` returning `ExecutionSelection`.
- Use it in `route_and_execute_with_registry()` no-match branch.
- Rename the misleading existing test to describe matched primary selection when no route is ready.
- Add a new true no-match planner test using a non-matching prompt.
- Add a fallback metadata test for `no_match_fallback_selection()`.

### Risk Mitigation Recommendations

- Do not change public APIs.
- Do not change fallback provider/model values.
- Keep tests inside the existing router test module.
- Run focused router tests and standard feature-gated checks.

## Next Steps

If approved:

1. Implement the private fallback selection helper.
2. Rename/split the misleading test.
3. Add a true no-match planner test and fallback metadata test.
4. Run `cargo test --features terraphim-routing -- pi_terraphim_router`, `cargo check`, `cargo fmt --check`, and changed-line/static checks as practical.

## Appendix

### Reference Materials

- `src/pi_terraphim_router/mod.rs`
- `.docs/research-pi-rust-terraphim-router-execution-readiness.md`
- `.docs/design-pi-rust-terraphim-router-execution-readiness.md`

### Code Snippets

Current misleading test input:

```rust
write_rule(
    dir.path(),
    "impl",
    "# Impl\npriority:: 50\nsynonyms:: implement\nroute:: unknown-provider, unknown-model\n",
);

let input = RouterInput::new("implement something");
let selection = select_execution_route(&input, &router, Some(&registry));
```

Current fallback metadata branch:

```rust
let fallback_provider = "anthropic".to_string();
let fallback_model = "claude-sonnet-4-6".to_string();
```
