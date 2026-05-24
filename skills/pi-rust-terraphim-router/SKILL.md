# Skill: pi-rust-terraphim-router

Intelligent model selection for pi-rust using Terraphim's keyword-based routing engine. Automatically routes prompts to the optimal LLM provider based on prompt intent.

## When to Use

Use this skill when you want pi-rust to automatically select the best model for a given prompt rather than using a fixed model. The router extracts capabilities from the prompt and maps them to the optimal provider/model combination.

## Activation

The skill is available when the `terraphim-routing` feature is enabled:

```bash
# Build with routing support
cargo build --features terraphim-routing

# Or add to your Cargo.toml
[dependencies]
pi_agent_rust = { path = "../pi_agent_rust", features = ["terraphim-routing"] }
```

## API

### `route_and_execute(input: RouterInput) -> RouterResult<RouterOutput>`

Main entry point. Extracts capabilities from the prompt, selects the optimal provider, spawns pi-rust in RPC mode, and returns the LLM response with routing metadata.

```rust
use pi::pi_terraphim_router::{RouterInput, route_and_execute};

let input = RouterInput::new("Implement a secure authentication system");
let output = route_and_execute(input).await?;

println!("Provider: {}", output.provider);  // "openai-codex"
println!("Model: {}", output.model);        // "gpt-5.5"
println!("Response: {}", output.response);
```

### `extract_capabilities(prompt: &str) -> Vec<String>`

Extract capabilities from a prompt without executing.

```rust
use pi::pi_terraphim_router::extract_capabilities;

let caps = extract_capabilities("Audit this code for security vulnerabilities");
// ["SecurityAudit"]
```

### `get_provider_for_capability(capability: &str) -> Option<ProviderSelection>`

Get the provider mapping for a specific capability.

```rust
use pi::pi_terraphim_router::get_provider_for_capability;

let selection = get_provider_for_capability("DeepThinking");
// Provider: "kimi-for-coding", Model: "kimi-k2.6"
```

## Capability Mapping

| Capability | Primary Provider | Model | Confidence |
|-----------|-----------------|-------|-----------|
| DeepThinking | kimi-for-coding | kimi-k2.6 | 0.95 |
| CodeGeneration | openai-codex | gpt-5.5 | 0.95 |
| SecurityAudit | anthropic | claude-sonnet-4-6 | 0.92 |
| FastThinking | google | gemini-3-flash | 0.92 |
| Architecture | anthropic | claude-opus-4-6 | 0.93 |
| Testing | openai-codex | gpt-5.3-codex-spark | 0.90 |
| CodeReview | anthropic | claude-sonnet-4-6 | 0.90 |
| Refactoring | openai-codex | gpt-5.5 | 0.89 |
| Documentation | anthropic | claude-sonnet-4-6 | 0.87 |
| Explanation | anthropic | claude-sonnet-4-6 | 0.88 |
| Performance | openai-codex | gpt-5.5 | 0.88 |

## Configuration

### Environment Variables

- `PI_BINARY_PATH` - Path to the pi binary (default: "pi" in PATH)

### Provider Credentials

Providers require their respective API keys:
- `ANTHROPIC_API_KEY`
- `OPENAI_API_KEY`
- `KIMI_API_KEY`
- `GOOGLE_API_KEY`
- `ZAI_API_KEY`

## Architecture

```
Prompt -> KeywordExtractor -> ProviderMapper -> RpcClient -> pi-rust --mode rpc -> LLM API
```

1. **CapabilityExtractor** wraps `terraphim_router::KeywordRouter` to extract capabilities from prompts
2. **ProviderMapper** maps capabilities to provider/model selections with confidence scores
3. **RpcClient** spawns pi-rust subprocess in RPC mode and communicates via JSON-RPC 2.0
4. **route_and_execute** orchestrates the flow and returns structured output

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| NoProviderFound | No provider matches capabilities | Check capability extraction |
| ProviderNotReady | Missing API credentials | Set required env var |
| RpcError | pi-rust subprocess failed | Check pi binary availability |
| SubprocessError | Failed to spawn pi-rust | Verify pi is installed |

## Performance

- Capability extraction: <1ms (keyword matching)
- Provider mapping: <1ms (HashMap lookup)
- RPC spawn: ~50-100ms (process startup)
- Total routing overhead: <100ms

## Examples

### Basic Usage

```rust
use pi::pi_terraphim_router::{RouterInput, route_and_execute};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let input = RouterInput::new("Implement a function to parse JSON");
    let output = route_and_execute(input).await?;
    println!("Selected: {}/{}", output.provider, output.model);
    println!("Response: {}", output.response);
    Ok(())
}
```

### With Overrides

```rust
use pi::pi_terraphim_router::RouterInput;

let input = RouterInput::new("Complex reasoning task")
    .with_provider("anthropic")
    .with_model("claude-opus-4-6")
    .with_system_prompt("You are an expert in formal logic");
```

### Batch Processing

```rust
use pi::pi_terraphim_router::{extract_capabilities, get_provider_for_capability};

let prompts = vec![
    "Implement auth",
    "Audit for security",
    "Explain borrow checker",
];

for prompt in prompts {
    let caps = extract_capabilities(prompt);
    for cap in caps {
        if let Some(sel) = get_provider_for_capability(&cap) {
            println!("{} -> {}/{}", prompt, sel.provider, sel.model);
        }
    }
}
```

## Testing

```bash
# Run router tests
cargo test --features terraphim-routing --lib pi_terraphim_router

# Run all tests with routing
cargo test --features terraphim-routing
```

## Dependencies

- `terraphim_router` - Keyword-based routing engine
- `terraphim_types` - Core types (Capability, Provider)
- `serde_json` - JSON-RPC serialization

## License

Apache-2.0
