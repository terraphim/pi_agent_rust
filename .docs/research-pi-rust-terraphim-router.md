# Research Document: pi-rust-terraphim-router Skill

**Status**: Draft
**Author**: AI Agent
**Date**: 2026-05-24
**Reviewers**: Pending

## Executive Summary

We need a skill that bridges Claude Code/Opencode with pi-rust using Terraphim's dynamic keyword-based capability routing. The skill will analyse user prompts, extract capabilities via Terraphim's `KeywordRouter`, map them to optimal pi-rust provider/model combinations, and invoke pi-rust programmatically. This enables intelligent model selection (e.g., routing "implement secure auth" to `gpt-5.5-codex` or `kimi-k2.6`) without manual `--provider`/`--model` flags.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energizing? | Yes | Eliminates repetitive manual model selection; leverages both systems' strengths |
| Leverages strengths? | Yes | Terraphim has proven keyword routing; pi-rust has 10+ providers with reasoning/cost diversity |
| Meets real need? | Yes | User explicitly requested; currently no skill enables pi-rust callout from Claude Code/Opencode |

**Proceed**: Yes - 3/3 YES

## Problem Statement

### Description
When working with pi-rust from Claude Code or Opencode, users must manually specify `--provider` and `--model` for every invocation. There is no automated way to match prompt intent (e.g., "think deeply", "implement quickly", "audit security") to the optimal pi-rust model. Terraphim-ai has a sophisticated keyword-based routing engine that extracts capabilities from prompts, but it is not connected to pi-rust's provider ecosystem.

### Impact
- **Agent efficiency**: Agents waste tokens on suboptimal models (e.g., using cheap models for deep reasoning, expensive models for simple summaries)
- **User friction**: Manual model selection interrupts flow
- **Capability waste**: Models with specialised capabilities (reasoning, vision, coding) are not utilised based on prompt needs

### Success Criteria
1. A skill can extract capabilities from any prompt using Terraphim's `KeywordRouter`
2. The skill maps capabilities to pi-rust provider/model combinations with >90% accuracy
3. The skill invokes pi-rust and returns structured output to the calling agent
4. Configuration is minimal (env vars for API keys, optional strategy selection)

## Current State Analysis

### Existing Implementation

#### pi-rust (pi_agent_rust)
- **Binary**: `pi` - CLI with interactive TUI, print mode (`-p`), RPC mode (`--mode rpc`), and ACP mode
- **SDK**: `src/sdk.rs` exposes `AgentSessionHandle` with `prompt()`, `set_model()`, `subscribe()`
- **Provider metadata**: `src/provider_metadata.rs` has 80+ providers with `routing_defaults` (api, base_url, auth_header, reasoning, input, context_window, max_tokens)
- **Model registry**: `src/models.rs` - `ModelRegistry::load()` loads built-in + custom models; `available_models()` filters by credential readiness
- **New models added**: `glm-5.1` (zai), `minimax-m2.7-highspeed` (minimax), `kimi-k2.5/k2.6` (kimi-for-coding) - all with reasoning detection

#### terraphim-ai
- **`terraphim_router`**: Keyword-based routing with 11 capabilities (DeepThinking, CodeGeneration, SecurityAudit, etc.)
- **`terraphim_cli`**: Commands: `search`, `graph`, `replace`, `find`, `extract`, `coverage`, `roles`, `config`
- **`KeywordRouter`**: Extracts capabilities from text via substring matching with priority scoring
- **`RoutingEngine`**: `route(prompt, context) -> Result<RoutingDecision, RoutingError>`
- **`Provider`**: Supports `Llm { model_id, api_endpoint }` and `Agent { agent_id, cli_command, working_dir }`

### Code Locations

| Component | Location | Purpose |
|-----------|----------|---------|
| pi-rust CLI | `pi_agent_rust/src/main.rs` | Entry point with `--provider`, `--model`, `--mode rpc` |
| pi-rust SDK | `pi_agent_rust/src/sdk.rs` | `AgentSessionHandle`, `SessionOptions`, `SessionTransport` |
| pi-rust providers | `pi_agent_rust/src/provider_metadata.rs` | `PROVIDER_METADATA` array with 80+ providers |
| pi-rust models | `pi_agent_rust/src/models.rs` | `ModelRegistry`, `ModelEntry`, built-in seeds |
| terraphim router | `terraphim-ai/crates/terraphim_router/src/` | `RoutingEngine`, `KeywordRouter`, `ProviderRegistry` |
| terraphim types | `terraphim-ai/crates/terraphim_types/src/capability.rs` | `Capability`, `Provider`, `ProviderType` |
| terraphim CLI | `terraphim-ai/crates/terraphim_cli/src/main.rs` | Command-line interface for automation |

### Data Flow (Current)
```
User Prompt -> pi-rust -> Manual --provider/--model selection -> LLM API
```

### Data Flow (Target)
```
User Prompt -> terraphim KeywordRouter -> Capabilities -> pi-rust Provider Mapping -> Optimal Model -> LLM API
```

### Integration Points
- **pi-rust RPC mode**: `--mode rpc` accepts JSON-RPC 2.0 over stdio; can be spawned as subprocess
- **pi-rust SDK**: `AgentSessionHandle` can be created in-process via `create_agent_session()`
- **terraphim CLI**: `terraphim-cli extract "text" --json` outputs capabilities/entities
- **terraphim router**: `RoutingEngine::route()` is synchronous; can be embedded in any Rust code

## Constraints

### Technical Constraints
- **Runtime mismatch**: pi-rust uses `asupersync` (structured concurrency); terraphim uses `tokio` - async boundaries need bridging
- **Mutex incompatibility**: `asupersync::sync::Mutex` vs `tokio::sync::Mutex` - cannot share locked state
- **Binary size**: pi-rust release target is <22 MiB; adding terraphim_router as dependency may impact this
- **Unsafe**: pi-rust forbids unsafe code (`#![forbid(unsafe_code)]`); terraphim must comply
- **Startup time**: pi-rust targets <100ms startup; terraphim router initialisation must be lazy
- **Error types**: pi-rust uses `pi::error::Error`; terraphim uses `anyhow::Result` - conversion needed

### Business Constraints
- **No breaking changes**: pi-rust's existing CLI and SDK must remain unchanged
- **Optional integration**: terraphim routing must be opt-in (feature flag or config)
- **Cross-platform**: Must work on macOS (local) and Linux (bigbox)

### Non-Functional Requirements
| Requirement | Target | Current |
|-------------|--------|---------|
| Routing latency | <50ms | N/A (new feature) |
| Capability accuracy | >90% | N/A (new feature) |
| Binary size overhead | <2 MiB | pi-rust is ~18-23 MiB |
| Startup overhead | <10ms | pi-rust targets <100ms |

## Vital Few (Essentialism)

### Essential Constraints (Max 3)

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| No breaking changes to pi-rust CLI/SDK | pi-rust has users and CI pipelines; breakage is unacceptable | AGENTS.md specifies backwards compat policy |
| Optional/opt-in integration | terraphim routing adds complexity; not all users need it | pi-rust standalone must remain functional |
| Sub-50ms routing latency | Routing happens on every prompt; >50ms degrades UX | pi-rust targets <100ms total startup |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|----------------|
| terraphim knowledge graph search integration | Not vital for routing; adds SQLite dependency complexity |
| terraphim synonym replacement in prompts | Out of scope for model selection; could be future enhancement |
| Custom keyword mapping UI | YAGNI - default mappings cover 11 capabilities well |
| ACP/Zed editor integration | ACP is for Zed specifically; our target is Claude Code/Opencode |
| Real-time provider health monitoring | Over-engineering; static capability mapping is sufficient |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|------------|--------|------|
| `terraphim_router` | Core routing logic | Medium - needs feature flag gating |
| `terraphim_types` | `Capability`, `Provider` types | Low - small crate, stable types |
| `pi_agent_rust::sdk` | `AgentSessionHandle`, `SessionOptions` | Low - existing public API |
| `pi_agent_rust::provider_metadata` | `PROVIDER_METADATA` for mapping | Low - compile-time static data |

### External Dependencies

| Dependency | Version | Risk | Alternative |
|------------|---------|------|-------------|
| `terraphim_router` | 0.1.x | Medium - may add to binary size | Use terraphim-cli subprocess instead |
| `terraphim_types` | 0.1.x | Low | Inline capability enum |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Binary size exceeds budget | Medium | High | Feature flag `terraphim-routing`; measure with `cargo bloat` |
| Runtime mismatch (asupersync vs tokio) | High | Medium | Spawn pi-rust as subprocess (RPC mode) instead of in-process SDK |
| Keyword mapping misses specialised prompts | Medium | Medium | Fallback to default model; allow custom keyword overrides |
| Provider credential not configured | High | Low | Check `model_entry_is_ready()` before selection; fallback chain |
| Terraphim router returns no match | Low | Medium | Fallback to `PROVIDER_DEFAULT_MODELS` ordering |

### Open Questions

1. **Should the skill use pi-rust RPC mode (subprocess) or SDK (in-process)?**
   - RPC: avoids runtime mismatch, simpler, higher latency
   - SDK: lower latency, more complex integration, mutex/runtime issues
   - *Recommendation*: Start with RPC mode for simplicity

2. **How should capabilities map to pi-rust providers?**
   - Static mapping in config file?
   - Dynamic mapping based on `PROVIDER_METADATA` reasoning flags?
   - *Recommendation*: Static mapping with provider metadata augmentation

3. **What output format should the skill return?**
   - Raw LLM text?
   - Structured JSON with metadata (provider used, confidence, capabilities)?
   - *Recommendation*: Structured JSON for agent consumption

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|------------|-------|---------------|-----------|
| terraphim_router crate can be compiled standalone | It has minimal dependencies | Build fails if not | No |
| pi-rust RPC mode is stable for programmatic use | `--mode rpc` exists in CLI | RPC protocol changes break integration | Partial - exists but not heavily tested |
| Keyword mappings are sufficient for coding tasks | Default mappings cover code, test, review, architecture | Misses specialised domains | No |
| Users have API keys configured for target providers | `model_entry_is_ready()` checks this | Routing selects provider without credentials | No |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen/Rejected |
|----------------|--------------|---------------------|
| **A. Library integration** (terraphim_router as pi-rust dep) | Tight coupling, binary size risk, runtime issues | Rejected - too invasive |
| **B. CLI pipe** (terraphim-cli | pi-rust) | Loose coupling, Unix philosophy, higher latency | Rejected - too slow for interactive use |
| **C. SDK wrapper skill** (Rust skill calling pi-rust SDK) | Clean API, in-process, complex | Rejected - runtime mismatch |
| **D. Subprocess skill** (spawn pi-rust --mode rpc) | Moderate coupling, stable, acceptable latency | **Chosen** - best balance |
| **E. HTTP service** (daemon wrapping both) | Over-engineering, deployment complexity | Rejected - YAGNI |

## Research Findings

### Key Insights

1. **RPC mode is the sweet spot**: pi-rust's `--mode rpc` provides JSON-RPC 2.0 over stdio. This avoids all runtime/mutex issues while keeping latency acceptable (~20-50ms spawn overhead).

2. **Keyword-to-provider mapping is straightforward**: Terraphim's 11 capabilities map cleanly to pi-rust provider strengths:
   - `DeepThinking` -> `claude-opus-4-6`, `kimi-k2.6`, `gpt-5.5`
   - `CodeGeneration` -> `gpt-5.5-codex`, `kimi-k2.5`, `glm-5.1`
   - `FastThinking` -> `gemini-3-flash`, `gpt-5.4`
   - `SecurityAudit` -> `claude-sonnet-4-6`, `gpt-5.5`

3. **Provider readiness check exists**: pi-rust's `model_entry_is_ready()` verifies credentials. The skill should check this before selecting a provider.

4. **Strategy selection matters**: Terraphim supports `CostOptimized`, `LatencyOptimized`, `CapabilityFirst`. For coding agents, `CapabilityFirst` or `CostOptimized` are most relevant.

### Relevant Prior Art

- **pi-agent-rust skill** (`~/.claude/skills/pi-agent-rust/`): Development skill for pi-rust repo; NOT a callout skill
- **terraphim-cli**: Existing automation interface; could be used for capability extraction
- **pi-rust SDK** (`src/sdk.rs`): Programmatic API; useful for future in-process integration

### Technical Spikes Needed

| Spike | Purpose | Estimated Effort |
|-------|---------|------------------|
| Compile terraphim_router standalone | Verify it builds without full terraphim-ai workspace | 1 hour |
| Test pi-rust RPC mode programmatically | Verify JSON-RPC protocol stability | 2 hours |
| Measure subprocess spawn latency | Validate <50ms routing target | 1 hour |
| Test keyword-to-provider accuracy | Validate capability mapping with sample prompts | 2 hours |

## Recommendations

### Proceed/No-Proceed
**Proceed** - The integration is feasible, valuable, and aligns with both systems' strengths. The subprocess approach (RPC mode) minimises risk.

### Scope Recommendations
- **In scope**: Keyword extraction, capability-to-provider mapping, pi-rust RPC invocation, structured JSON output
- **Out of scope**: Knowledge graph search, synonym replacement, real-time provider monitoring, ACP integration

### Risk Mitigation Recommendations
1. Use feature flag `terraphim-routing` to keep integration optional
2. Implement fallback chain: terraphim routing -> provider defaults -> manual selection
3. Add `cargo bloat` check to CI to monitor binary size impact
4. Cache `KeywordRouter` initialisation (it's stateless after construction)

## Next Steps

If approved:
1. Conduct technical spikes (compile terraphim_router, test RPC mode, measure latency)
2. Proceed to Phase 2: Design the skill architecture and API
3. Define capability-to-provider mapping table
4. Specify JSON output schema for agent consumption

## Appendix

### Reference Materials
- `pi_agent_rust/src/sdk.rs` - SDK API
- `pi_agent_rust/src/provider_metadata.rs` - Provider metadata
- `terraphim-ai/crates/terraphim_router/src/engine.rs` - Routing engine
- `terraphim-ai/crates/terraphim_router/src/keyword.rs` - Keyword router
- `terraphim-ai/crates/terraphim_types/src/capability.rs` - Capability types

### Code Snippets

**pi-rust RPC mode spawn:**
```rust
let mut child = Command::new("pi")
    .args(["--mode", "rpc", "--provider", "anthropic", "--model", "claude-sonnet-4-6"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
```

**terraphim routing:**
```rust
let engine = RoutingEngine::new();
let decision = engine.route("Implement a secure auth function", &RoutingContext::default())?;
println!("Selected: {} (confidence: {})", decision.provider.id, decision.confidence);
```

**Capability extraction:**
```rust
let router = KeywordRouter::new();
let caps = router.extract_capabilities("Implement a secure auth function");
// caps = [CodeGeneration, SecurityAudit]
```
