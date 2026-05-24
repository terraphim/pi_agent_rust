# Implementation Plan: pi-rust-terraphim-router Skill

**Status**: Draft
**Research Doc**: `.docs/research-pi-rust-terraphim-router.md`
**Author**: AI Agent
**Date**: 2026-05-24
**Estimated Effort**: 2-3 days

## Overview

### Summary
Build a Claude Code/Opencode skill that uses Terraphim's keyword-based capability extraction to intelligently route prompts to the optimal pi-rust provider and model. The skill spawns pi-rust in RPC mode, sends the routed prompt, and returns structured JSON output with the LLM response and routing metadata.

### Approach
**Subprocess RPC approach** (chosen from research):
- Terraphim `KeywordRouter` extracts capabilities from the prompt
- Static capability-to-provider mapping selects optimal provider/model
- pi-rust is spawned as `--mode rpc` subprocess
- JSON-RPC 2.0 protocol sends prompt and receives streaming response
- Structured JSON output returned to calling agent

### Scope

**In Scope:**
- Keyword-based capability extraction from prompts
- Capability-to-provider/model mapping for 11 capabilities
- pi-rust RPC subprocess spawning and communication
- Structured JSON output with response + routing metadata
- Fallback chain when routing fails
- Configuration via environment variables

**Out of Scope:**
- In-process SDK integration (runtime mismatch)
- terraphim knowledge graph search
- Real-time provider health monitoring
- Custom keyword mapping UI
- ACP/Zed editor integration

**Avoid At All Cost** (from 5/25 analysis):
- In-process integration with asupersync/tokio runtime bridging (too complex, fragile)
- HTTP daemon/service (deployment overhead, YAGNI)
- terraphim-cli pipe approach (too slow for interactive use)
- Dynamic provider discovery (over-engineering; static mapping is sufficient)

## Architecture

### Component Diagram
```
+-------------------------------------------------+
|  Claude Code / Opencode Agent                   |
|  (calls skill via tool invocation)              |
+-------------------------------------------------+
                          |
                          v
+-------------------------------------------------+
|  pi-rust-terraphim-router Skill                 |
|                                                 |
|  +---------------------+  +------------------+  |
|  | KeywordExtractor    |->| ProviderMapper   |  |
|  | (terraphim_router)  |  | (static mapping) |  |
|  +---------------------+  +------------------+  |
|             |                      |            |
|             v                      v            |
|  +---------------------+  +------------------+  |
|  | FallbackChain       |->| RpcClient        |  |
|  | (default models)    |  | (pi --mode rpc)  |  |
|  +---------------------+  +------------------+  |
|                                    |            |
+------------------------------------|------------+
                                     |
                                     v
+-------------------------------------------------+
|  pi-rust (subprocess)                           |
|  --mode rpc --provider X --model Y              |
+-------------------------------------------------+
                                     |
                                     v
+-------------------------------------------------+
|  LLM API (Anthropic/OpenAI/etc.)                |
+-------------------------------------------------+
```

### Data Flow
```
Prompt -> KeywordExtractor.extract_capabilities() -> Vec<Capability>
  -> ProviderMapper.map_to_provider() -> ProviderSelection
    -> RpcClient.spawn_and_send() -> JsonRpcRequest
      -> pi-rust --mode rpc -> LLM API
        -> Streaming Response -> JsonRpcResponse
          -> Skill.format_output() -> Structured JSON
```

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Subprocess RPC instead of SDK | Avoids asupersync/tokio runtime mismatch; simpler; stable protocol | In-process SDK (runtime mismatch, complex) |
| Static capability mapping | Sufficient for 11 capabilities; no runtime config needed | Dynamic provider discovery (over-engineering) |
| JSON output format | Agents consume structured data; enables downstream routing decisions | Raw text (loses metadata) |
| Feature flag gating | Keeps pi-rust binary size under control; optional integration | Always-on (forces dependency on all users) |
| terraphim_router crate directly | Minimal dependencies; no CLI overhead | terraphim-cli subprocess (extra process, slower) |

### Eliminated Options (Essentialism)

| Option Rejected | Why Rejected | Risk of Including |
|-----------------|--------------|-------------------|
| In-process SDK integration | asupersync/tokio runtime mismatch; mutex incompatibility; complex error bridging | Fragile integration, hard to debug |
| terraphim-cli pipe approach | Extra process spawn; higher latency; fragile parsing | >100ms routing latency |
| HTTP service daemon | Deployment complexity; port management; not needed for local agent use | Operational overhead, YAGNI |
| Dynamic provider health monitoring | Over-engineering; static mapping covers current needs | Complexity without proportional value |
| Custom keyword mapping at runtime | YAGNI; default mappings cover 11 capabilities well | Scope creep, UI complexity |

### Simplicity Check

> "Minimum code that solves the problem. Nothing speculative."

**What if this could be easy?**
The simplest design is: extract keywords -> lookup provider in a HashMap -> spawn pi-rust with that provider -> return JSON. This is exactly what we're building. No daemon, no in-process integration, no dynamic discovery.

**Senior Engineer Test**: A senior engineer would call this appropriately simple for the problem. It uses subprocess RPC (proven pattern), static mapping (sufficient for known capabilities), and returns JSON (standard for agent tools).

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
| `crates/pi_rust_terraphim_router/src/lib.rs` | Module root and public API |
| `crates/pi_rust_terraphim_router/src/extractor.rs` | Keyword-based capability extraction wrapper |
| `crates/pi_rust_terraphim_router/src/mapper.rs` | Capability-to-provider mapping |
| `crates/pi_rust_terraphim_router/src/rpc_client.rs` | pi-rust RPC subprocess client |
| `crates/pi_rust_terraphim_router/src/types.rs` | Input/output types for skill |
| `crates/pi_rust_terraphim_router/src/error.rs` | Error types |
| `crates/pi_rust_terraphim_router/Cargo.toml` | Crate manifest with terraphim deps |
| `skills/pi-rust-terraphim-router/SKILL.md` | Claude Code/Opencode skill definition |
| `skills/pi-rust-terraphim-router/config.json` | Default provider mapping configuration |

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` (workspace) | Add `crates/pi_rust_terraphim_router` to workspace members |
| `Cargo.toml` (pi_agent_rust) | Add optional `terraphim-routing` feature flag |

### Deleted Files
None.

## API Design

### Public Types

```rust
/// Input to the router skill
#[derive(Debug, Clone)]
pub struct RouterInput {
    /// User prompt
    pub prompt: String,
    /// Optional strategy override (cost_optimized, latency_optimized, capability_first)
    pub strategy: Option<String>,
    /// Optional preferred provider (bypasses routing)
    pub preferred_provider: Option<String>,
    /// Optional preferred model (bypasses routing)
    pub preferred_model: Option<String>,
    /// Optional system prompt
    pub system_prompt: Option<String>,
    /// Working directory for pi-rust
    pub working_dir: Option<PathBuf>,
}

/// Output from the router skill
#[derive(Debug, Clone, Serialize)]
pub struct RouterOutput {
    /// LLM response text
    pub response: String,
    /// Selected provider
    pub provider: String,
    /// Selected model
    pub model: String,
    /// Capabilities extracted from prompt
    pub capabilities: Vec<String>,
    /// Routing confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Routing reason
    pub reason: String,
    /// Whether fallback was used
    pub fallback_used: bool,
    /// Token usage (if available)
    pub token_usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### Public Functions

```rust
/// Route a prompt to the optimal pi-rust provider and return the LLM response.
///
/// # Arguments
/// * `input` - Router input with prompt and optional overrides
///
/// # Returns
/// Structured output with LLM response and routing metadata
///
/// # Errors
/// Returns `RouterError::NoProviderFound` if no provider matches capabilities
/// Returns `RouterError::RpcError` if pi-rust subprocess fails
pub async fn route_and_execute(input: RouterInput) -> Result<RouterOutput, RouterError>;

/// Extract capabilities from a prompt without executing.
///
/// # Arguments
/// * `prompt` - User prompt text
///
/// # Returns
/// List of extracted capabilities
pub fn extract_capabilities(prompt: &str) -> Vec<String>;

/// Get the provider mapping for a given capability.
///
/// # Arguments
/// * `capability` - Capability name (e.g., "DeepThinking")
///
/// # Returns
/// Provider selection with provider name, model, and confidence
pub fn get_provider_for_capability(capability: &str) -> Option<ProviderSelection>;
```

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("no provider found for capabilities: {0:?}")]
    NoProviderFound(Vec<String>),

    #[error("provider not ready (missing credentials): {provider}/{model}")]
    ProviderNotReady { provider: String, model: String },

    #[error("RPC communication failed: {0}")]
    RpcError(String),

    #[error("pi-rust subprocess failed: {0}")]
    SubprocessError(String),

    #[error("invalid capability: {0}")]
    InvalidCapability(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

## Test Strategy

### Unit Tests

| Test | Location | Purpose |
|------|----------|---------|
| `test_extract_deep_thinking` | `extractor.rs` | Verify DeepThinking keyword detection |
| `test_extract_code_generation` | `extractor.rs` | Verify CodeGeneration keyword detection |
| `test_extract_security_audit` | `extractor.rs` | Verify SecurityAudit keyword detection |
| `test_extract_multiple_capabilities` | `extractor.rs` | Verify multiple capability extraction |
| `test_map_deep_thinking_to_provider` | `mapper.rs` | Verify capability-to-provider mapping |
| `test_map_code_generation_to_provider` | `mapper.rs` | Verify CodeGeneration mapping |
| `test_fallback_when_no_match` | `mapper.rs` | Verify fallback chain |
| `test_rpc_client_spawn` | `rpc_client.rs` | Verify subprocess spawning |
| `test_rpc_client_send_receive` | `rpc_client.rs` | Verify JSON-RPC communication |
| `test_format_output` | `types.rs` | Verify JSON output structure |

### Integration Tests

| Test | Location | Purpose |
|------|----------|---------|
| `test_route_and_execute_real_provider` | `tests/integration.rs` | Full flow with real pi-rust (requires API key) |
| `test_route_and_execute_mock_provider` | `tests/integration.rs` | Full flow with mock RPC |
| `test_end_to_end_with_fallback` | `tests/integration.rs` | Fallback chain verification |

### Property Tests

```rust
proptest! {
    #[test]
    fn extract_capabilities_never_panics(prompt in "\\PC{0,500}") {
        let _ = extract_capabilities(&prompt);
    }

    #[test]
    fn route_output_has_required_fields(output in router_output_strategy()) {
        prop_assert!(!output.response.is_empty());
        prop_assert!(!output.provider.is_empty());
        prop_assert!(!output.model.is_empty());
    }
}
```

## Implementation Steps

### Step 1: Create Crate Skeleton
**Files:** `crates/pi_rust_terraphim_router/Cargo.toml`, `src/lib.rs`, `src/error.rs`
**Description:** Create new crate with dependencies, module structure, and error types
**Tests:** Verify crate compiles
**Dependencies:** None
**Estimated:** 1 hour

```toml
[package]
name = "pi_rust_terraphim_router"
version = "0.1.0"
edition = "2024"

[dependencies]
terraphim_router = { path = "../../../terraphim-ai/crates/terraphim_router" }
terraphim_types = { path = "../../../terraphim-ai/crates/terraphim_types" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tokio = { version = "1", features = ["process", "io-util", "macros"] }
tracing = "0.1"

[dev-dependencies]
tokio-test = "0.4"
proptest = "1.0"
```

### Step 2: Implement Capability Extraction
**Files:** `src/extractor.rs`
**Description:** Wrap terraphim_router's KeywordRouter to extract capabilities from prompts
**Tests:** Unit tests for all 11 capabilities
**Dependencies:** Step 1
**Estimated:** 2 hours

```rust
use terraphim_router::keyword::KeywordRouter;
use terraphim_types::capability::Capability;

pub struct CapabilityExtractor {
    router: KeywordRouter,
}

impl CapabilityExtractor {
    pub fn new() -> Self {
        Self { router: KeywordRouter::new() }
    }

    pub fn extract(&self, prompt: &str) -> Vec<Capability> {
        self.router.extract_capabilities(prompt)
    }
}
```

### Step 3: Implement Provider Mapping
**Files:** `src/mapper.rs`
**Description:** Map extracted capabilities to pi-rust provider/model combinations with fallback chain
**Tests:** Unit tests for mapping and fallback
**Dependencies:** Step 2
**Estimated:** 3 hours

```rust
use std::collections::HashMap;
use terraphim_types::capability::Capability;

#[derive(Debug, Clone)]
pub struct ProviderSelection {
    pub provider: String,
    pub model: String,
    pub confidence: f32,
}

pub struct ProviderMapper {
    mappings: HashMap<Capability, Vec<ProviderSelection>>,
}

impl ProviderMapper {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert(Capability::DeepThinking, vec![
            ProviderSelection { provider: "kimi-for-coding".into(), model: "kimi-k2.6".into(), confidence: 0.95 },
            ProviderSelection { provider: "anthropic".into(), model: "claude-opus-4-6".into(), confidence: 0.90 },
            ProviderSelection { provider: "openai-codex".into(), model: "gpt-5.5".into(), confidence: 0.88 },
        ]);
        mappings.insert(Capability::CodeGeneration, vec![
            ProviderSelection { provider: "openai-codex".into(), model: "gpt-5.5".into(), confidence: 0.95 },
            ProviderSelection { provider: "kimi-for-coding".into(), model: "kimi-k2.5".into(), confidence: 0.90 },
            ProviderSelection { provider: "zai".into(), model: "glm-5.1".into(), confidence: 0.85 },
        ]);
        // ... other capabilities
        Self { mappings }
    }

    pub fn map(&self, capabilities: &[Capability]) -> Option<ProviderSelection> {
        // Select highest confidence provider across all capabilities
        capabilities.iter()
            .filter_map(|cap| self.mappings.get(cap))
            .flat_map(|selections| selections.iter().cloned())
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }
}
```

### Step 4: Implement RPC Client
**Files:** `src/rpc_client.rs`
**Description:** Spawn pi-rust in RPC mode and communicate via JSON-RPC 2.0
**Tests:** Integration tests with mock and real pi-rust
**Dependencies:** Step 1
**Estimated:** 4 hours

```rust
use tokio::process::{Command, Child};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

pub struct RpcClient {
    child: Child,
}

impl RpcClient {
    pub async fn spawn(provider: &str, model: &str) -> Result<Self, RouterError> {
        let child = Command::new("pi")
            .args(["--mode", "rpc", "--provider", provider, "--model", model])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        Ok(Self { child })
    }

    pub async fn send_prompt(&mut self, prompt: &str) -> Result<String, RouterError> {
        // Send JSON-RPC request
        // Read streaming response
        // Return accumulated text
        todo!()
    }
}
```

### Step 5: Implement Main Router
**Files:** `src/lib.rs` (update)
**Description:** Wire extractor, mapper, and RPC client into `route_and_execute()`
**Tests:** Integration tests
**Dependencies:** Steps 2, 3, 4
**Estimated:** 3 hours

### Step 6: Create Skill Definition
**Files:** `skills/pi-rust-terraphim-router/SKILL.md`, `skills/pi-rust-terraphim-router/config.json`
**Description:** Claude Code/Opencode skill definition with usage examples
**Tests:** Manual verification
**Dependencies:** Step 5
**Estimated:** 2 hours

### Step 7: Documentation
**Files:** `README.md`, inline docs
**Description:** User-facing documentation with setup instructions and examples
**Tests:** Doc tests
**Dependencies:** Step 6
**Estimated:** 2 hours

## Rollback Plan

If issues discovered:
1. Remove `crates/pi_rust_terraphim_router` from workspace
2. Remove `terraphim-routing` feature flag from pi_agent_rust
3. Delete skill directory `skills/pi-rust-terraphim-router/`
4. Revert any Cargo.toml changes

Feature flag: `terraphim-routing` (disabled by default)

## Dependencies

### New Dependencies

| Crate | Version | Justification |
|-------|---------|---------------|
| `terraphim_router` | 0.1.x | Core keyword extraction and routing logic |
| `terraphim_types` | 0.1.x | Capability and Provider types |
| `tokio` | 1.x | Async subprocess management (already in terraphim) |
| `serde_json` | 1.x | JSON-RPC protocol handling |

### Dependency Updates
None.

## Performance Considerations

### Expected Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Capability extraction | <5ms | Benchmark with 500-char prompts |
| Provider mapping | <1ms | Benchmark with 3 capabilities |
| pi-rust spawn overhead | <50ms | Measure subprocess spawn time |
| Total routing latency | <100ms | End-to-end from prompt to LLM request |

### Benchmarks to Add

```rust
#[bench]
fn bench_extract_capabilities(b: &mut Bencher) {
    let extractor = CapabilityExtractor::new();
    let prompt = "Implement a secure authentication system with JWT tokens";
    b.iter(|| extractor.extract(prompt));
}

#[bench]
fn bench_provider_mapping(b: &mut Bencher) {
    let mapper = ProviderMapper::new();
    let caps = vec![Capability::CodeGeneration, Capability::SecurityAudit];
    b.iter(|| mapper.map(&caps));
}
```

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| Verify terraphim_router compiles standalone | Pending | Agent |
| Measure actual pi-rust RPC spawn latency | Pending | Agent |
| Validate capability-to-provider mapping accuracy | Pending | Agent |
| Test on bigbox (Linux x86-64) | Pending | Agent |

## Approval

- [ ] Technical review complete
- [ ] Test strategy approved
- [ ] Performance targets agreed
- [ ] Human approval received
