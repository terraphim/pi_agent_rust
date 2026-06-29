# SSE

Server-Sent Events parser for streaming LLM responses (`src/sse.rs`). Parses the chunked event stream from providers into deltas (text, tool-call, thinking). Drives streaming + tool-call parity across all providers; covered by the `sse` and `provider_streaming` test modules. Also underpins `src/providers/model_fetch.rs` dynamic model discovery.

**Key files:** `src/sse.rs`, `src/providers/model_fetch.rs`, `tests/e2e_provider_streaming.rs`

Related: provider, model-registry

synonyms:: sse, server-sent events, sse parser, streaming, stream parser, event stream, delta, streaming response, provider streaming, provider_streaming, model fetch
