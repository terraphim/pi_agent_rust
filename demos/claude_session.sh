#!/bin/bash
# Simulate Claude Code session analyzing pi-rust terraphim-router skill

cd /Users/alex/projects/pi_agent_rust

sleep 1

cat << 'OUTPUT'
[Claude Code] Analyzing pi_terraphim_router module...

## Analysis of `pi_terraphim_router` module

**Location:** `src/pi_terraphim_router/` (6 files, ~700 LOC)

**Purpose:** Intelligent model/provider selection — extracts capabilities from user prompts and routes to optimal LLM providers.

### Architecture

```
Prompt → KeywordExtractor → ProviderMapper → RpcClient → pi-rust --mode rpc → LLM API
```

### Components

1. **CapabilityExtractor** (`extractor.rs`)
   - Wraps `terraphim_router::KeywordRouter`
   - Extracts 11 capabilities from natural language prompts
   - Examples: DeepThinking, CodeGeneration, SecurityAudit, Testing

2. **ProviderMapper** (`mapper.rs`)
   - Static HashMap of Capability → Vec<ProviderSelection>
   - Confidence scores: 0.84–0.95
   - Fallback: openai-codex/gpt-5.5

3. **RpcClient** (`rpc_client.rs`)
   - Spawns `pi --mode rpc` subprocess
   - JSON-RPC 2.0 over stdio
   - Synchronous I/O (avoids tokio/asupersync runtime mismatch)

### Issues Found

1. **Blocking RPC in async context** — `RpcClient` uses `std::process::Command` inside async `send_prompt`
2. **JSON-RPC framing bug** — reads until blank line, truncates multi-line responses
3. **Hard-coded models** — `gpt-5.5`, `claude-opus-4-6` don't exist yet
4. **No readiness check** — never verifies provider credentials before routing
5. **Strategy field ignored** — `RouterInput.strategy` is never read
6. **No streaming** — buffers full response before return
7. **String-matching duplication** — `get_by_name` duplicates Capability enum
8. **Missing integration tests** — no mocked subprocess tests for end-to-end path

### Quick Wins

- Make `RpcClient` async-native (`tokio::process`)
- Fix stdout-reading termination (read one JSON line, not "until blank")
- Wire the `strategy` field or remove it
- Move provider tables to config (TOML/JSON) for runtime updates

[Claude Code] Analysis complete. 8 issues found, 4 quick wins identified.
OUTPUT
