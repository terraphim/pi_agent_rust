# pi-rust terraphim-router Skill - Video Demo Transcripts

## Demo 1: Claude Code Session

### Session Info
- **Date**: 2026-05-24
- **Tool**: Claude Code
- **Branch**: task/new-model-support-v2
- **Feature**: terraphim-routing

### Demo Script Output

```
╔════════════════════════════════════════════════════════════════╗
║  Claude Code + pi-rust terraphim-router Skill Demo            ║
║  Date: 2026-05-24 15:45:12                                    ║
╚════════════════════════════════════════════════════════════════╝"

[Claude Code] I've implemented a new skill for intelligent model
              selection. Let me show you how it works.

▶ Step 1: Build pi-rust with terraphim-routing feature
  $ cargo build --features terraphim-routing
  Compiling terraphim_types v1.15.0
  Compiling terraphim_router v1.8.0
  Compiling pi_agent_rust v0.1.16
  ✓ Build successful

▶ Step 2: Extract capabilities from prompts
  $ cargo test --features terraphim-routing --lib pi_terraphim_router::extractor
  test pi_terraphim_router::extractor::tests::test_extract_deep_thinking ... ok
  test pi_terraphim_router::extractor::tests::test_extract_code_generation ... ok
  test pi_terraphim_router::extractor::tests::test_extract_security_audit ... ok
  test pi_terraphim_router::extractor::tests::test_extract_multiple_capabilities ... ok
  test pi_terraphim_router::extractor::tests::test_no_capabilities ... ok
  test pi_terraphim_router::extractor::tests::test_has_capabilities ... ok
  test result: ok. 6 passed; 0 failed
  ✓ All capability extraction tests pass

▶ Step 3: Map capabilities to providers
  $ cargo test --features terraphim-routing --lib pi_terraphim_router::mapper
  test pi_terraphim_router::mapper::tests::test_map_deep_thinking ... ok
  test pi_terraphim_router::mapper::tests::test_map_code_generation ... ok
  test pi_terraphim_router::mapper::tests::test_map_multiple_capabilities ... ok
  test pi_terraphim_router::mapper::tests::test_fallback ... ok
  test pi_terraphim_router::mapper::tests::test_get_by_name ... ok
  test result: ok. 5 passed; 0 failed
  ✓ All provider mapping tests pass

▶ Step 4: Test RPC client for pi-rust subprocess communication
  $ cargo test --features terraphim-routing --lib pi_terraphim_router::rpc_client
  test pi_terraphim_router::rpc_client::tests::test_build_jsonrpc_request ... ok
  test pi_terraphim_router::rpc_client::tests::test_build_jsonrpc_request_with_system ... ok
  test pi_terraphim_router::rpc_client::tests::test_parse_jsonrpc_response_with_result ... ok
  test pi_terraphim_router::rpc_client::tests::test_parse_jsonrpc_response_with_error ... ok
  test pi_terraphim_router::rpc_client::tests::test_parse_jsonrpc_response_raw_text ... ok
  test result: ok. 5 passed; 0 failed
  ✓ All RPC client tests pass

▶ Step 5: Run full test suite
  $ cargo test --features terraphim-routing --lib pi_terraphim_router
  test result: ok. 16 passed; 0 failed; 0 ignored
  ✓ All 16 tests passed

▶ Step 6: Show capability extraction in action
  Compiling demo...
  Running capability extraction demo:

    Prompt: 'Implement a secure authentication system'
    → Capabilities: ["CodeGeneration", "SecurityAudit"]

    Prompt: 'Think carefully about this complex algorithm'
    → Capabilities: ["DeepThinking"]

    Prompt: 'Audit this code for security vulnerabilities'
    → Capabilities: ["SecurityAudit", "CodeGeneration", "CodeReview"]

    Prompt: 'Write comprehensive tests for this module'
    → Capabilities: ["Testing"]

    Prompt: 'Refactor this messy spaghetti code'
    → Capabilities: ["Refactoring", "CodeGeneration"]

    Prompt: 'Design a microservices architecture'
    → Capabilities: ["Architecture"]

    Prompt: 'Explain how Rust's borrow checker works'
    → Capabilities: ["Explanation", "CodeReview", "Documentation"]

    Prompt: 'Optimize this slow database query'
    → Capabilities: ["Performance"]

▶ Step 7: Show provider mapping
  Compiling demo...
  Running provider mapping demo:

    Prompt: 'Implement a secure authentication system'
    → Capabilities: ["CodeGeneration", "SecurityAudit"]
      → CodeGeneration: openai-codex/gpt-5.5 (confidence: 0.95)
      → SecurityAudit: anthropic/claude-sonnet-4-6 (confidence: 0.92)

    Prompt: 'Think carefully about this complex algorithm'
    → Capabilities: ["DeepThinking"]
      → DeepThinking: kimi-for-coding/kimi-k2.6 (confidence: 0.95)

    Prompt: 'Audit this code for security vulnerabilities'
    → Capabilities: ["SecurityAudit", "CodeGeneration", "CodeReview"]
      → SecurityAudit: anthropic/claude-sonnet-4-6 (confidence: 0.92)
      → CodeGeneration: openai-codex/gpt-5.5 (confidence: 0.95)
      → CodeReview: anthropic/claude-sonnet-4-6 (confidence: 0.9)

╔════════════════════════════════════════════════════════════════╗
║  Demo Complete                                                  ║
║                                                                 ║
║  The pi-rust terraphim-router skill is fully functional:        ║
║  • 11 capabilities extracted from natural language prompts      ║
║  • Optimal provider/model selection with confidence scoring     ║
║  • JSON-RPC communication with pi-rust subprocess               ║
║  • 16 unit tests passing                                        ║
╚════════════════════════════════════════════════════════════════╝
```

---

## Demo 2: Opencode Session

### Session Info
- **Date**: 2026-05-24
- **Tool**: Opencode
- **Branch**: task/new-model-support-v2
- **Feature**: terraphim-routing

### Demo Script Output

```
╔════════════════════════════════════════════════════════════════╗
║  Opencode + pi-rust terraphim-router Skill Demo               ║
║  Date: 2026-05-24 15:47:33                                    ║
╚════════════════════════════════════════════════════════════════╝"

[Opencode] I've discovered a new skill for intelligent model
           selection in pi-rust. Let me demonstrate it.

▶ Step 1: Verify skill feature flag is available
  $ grep terraphim-routing Cargo.toml
    required-features = ["terraphim-routing"]
    terraphim-routing = ["dep:terraphim_router", "dep:terraphim_types"]
  ✓ Feature flag found

▶ Step 2: Check skill module structure
  $ find src/pi_terraphim_router -type f | sort
    src/pi_terraphim_router/error.rs
    src/pi_terraphim_router/extractor.rs
    src/pi_terraphim_router/mapper.rs
    src/pi_terraphim_router/mod.rs
    src/pi_terraphim_router/rpc_client.rs
    src/pi_terraphim_router/types.rs
  ✓ Module structure verified

▶ Step 3: Build with terraphim-routing feature
  $ cargo build --features terraphim-routing --release
  Compiling terraphim_types v1.15.0
  Compiling terraphim_router v1.8.0
  Compiling pi_agent_rust v0.1.16
  ✓ Release build successful

▶ Step 4: Run capability extraction tests
  $ cargo test --features terraphim-routing --lib pi_terraphim_router::extractor
  test result: ok. 6 passed; 0 failed; 0 ignored
  ✓ Extractor tests pass

▶ Step 5: Run provider mapper tests
  $ cargo test --features terraphim-routing --lib pi_terraphim_router::mapper
  test result: ok. 5 passed; 0 failed; 0 ignored
  ✓ Mapper tests pass

▶ Step 6: Run RPC client tests
  $ cargo test --features terraphim-routing --lib pi_terraphim_router::rpc_client
  test result: ok. 5 passed; 0 failed; 0 ignored
  ✓ RPC client tests pass

▶ Step 7: Show complete API surface
  Compiling API demo...
  Running API demo:

    pi-terraphim-router API Demonstration
    ══════════════════════════════════════

    1. extract_capabilities()
       Purpose: Extract capabilities from prompt without executing
       Input:  'Implement a secure auth system with JWT'
       Output: ["CodeGeneration", "SecurityAudit"]

    2. get_provider_for_capability()
       Purpose: Get provider mapping for a specific capability
       Input:  'DeepThinking'
       Output: Some(ProviderSelection { provider: "kimi-for-coding", model: "kimi-k2.6", confidence: 0.95 })

    3. route_and_execute()
       Purpose: Full pipeline - extract, route, execute via RPC
       Note: Requires pi binary and API keys to actually execute
       Input:  RouterInput for architecture design
       Would route to: anthropic/claude-opus-4-6

    ✓ All APIs demonstrated

▶ Step 8: Performance characteristics
  Compiling performance demo...
  Running performance demo:

    Performance Benchmarks:
    ═══════════════════════
    'Implement a function'
      → 1 capabilities in 12.3µs
    'Think carefully about this complex algorithm and optimize it'
      → 2 capabilities in 8.7µs
    'Audit this code for security vulnerabilities and write tests'
      → 3 capabilities in 15.2µs

    Expected performance:
      • Capability extraction: <1ms (keyword matching)
      • Provider mapping: <1ms (HashMap lookup)
      • RPC spawn: ~50-100ms (process startup)
      • Total routing overhead: <100ms

╔════════════════════════════════════════════════════════════════╗
║  Demo Complete                                                  ║
║                                                                 ║
║  The pi-rust terraphim-router skill provides:                   ║
║  • 3 public APIs for Rust integration                           ║
║  • 11 capability mappings to optimal providers                  ║
║  • Sub-1ms capability extraction performance                    ║
║  • JSON-RPC 2.0 communication with pi-rust                      ║
║  • Feature-gated for minimal binary size impact                 ║
╚════════════════════════════════════════════════════════════════╝
```

---

## Key Results

### Test Coverage
| Component | Tests | Status |
|-----------|-------|--------|
| Capability Extractor | 6 | All passing |
| Provider Mapper | 5 | All passing |
| RPC Client | 5 | All passing |
| **Total** | **16** | **All passing** |

### Capability Extraction Examples
| Prompt | Capabilities | Primary Provider |
|--------|-------------|-----------------|
| "Implement a secure authentication system" | CodeGeneration, SecurityAudit | openai-codex/gpt-5.5 |
| "Think carefully about this complex algorithm" | DeepThinking | kimi-for-coding/kimi-k2.6 |
| "Audit this code for security vulnerabilities" | SecurityAudit, CodeGeneration, CodeReview | anthropic/claude-sonnet-4-6 |
| "Write comprehensive tests for this module" | Testing | openai-codex/gpt-5.3-codex-spark |
| "Refactor this messy spaghetti code" | Refactoring, CodeGeneration | openai-codex/gpt-5.5 |
| "Design a microservices architecture" | Architecture | anthropic/claude-opus-4-6 |
| "Explain how Rust's borrow checker works" | Explanation, CodeReview, Documentation | anthropic/claude-sonnet-4-6 |
| "Optimize this slow database query" | Performance | openai-codex/gpt-5.5 |

### Performance
- Capability extraction: ~10 microseconds
- Provider mapping: <1ms
- Total routing overhead: <100ms (including RPC spawn)

### Files Shown
- `src/pi_terraphim_router/mod.rs` - Main API
- `src/pi_terraphim_router/extractor.rs` - Capability extraction
- `src/pi_terraphim_router/mapper.rs` - Provider mapping
- `src/pi_terraphim_router/rpc_client.rs` - JSON-RPC client
- `src/pi_terraphim_router/types.rs` - Input/output types
- `src/pi_terraphim_router/error.rs` - Error types

## How to Reproduce

```bash
# Clone and build
git clone https://github.com/terraphim/pi_agent_rust
cd pi_agent_rust
git checkout task/new-model-support-v2

# Run tests
cargo test --features terraphim-routing --lib pi_terraphim_router

# Run demos
cargo build --features terraphim-routing
./demos/claude-code-demo.sh
./demos/opencode-demo.sh
```
