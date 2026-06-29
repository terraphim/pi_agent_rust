# Provider

Abstract LLM backend implementing the `Provider` trait (`src/provider.rs`). pi_agent_rust ships 10 native provider modules in `src/providers/`: Anthropic (streaming + extended thinking), OpenAI Chat Completions, OpenAI Responses/Codex Responses, Gemini, Cohere, Azure OpenAI, Bedrock, Vertex AI, GitHub Copilot, and GitLab Duo. Extension-provided providers bridge via stream-simple. Each provider maps a `ModelEntry` to an HTTP transport (`anthropic-messages`, `openai-completions`, `google-generative-ai`, `cohere-chat`, `bedrock-converse-stream`). Auth comes from `auth.json` (OAuth for codex/kimi/copilot; API keys otherwise).

**Key files:** `src/provider.rs`, `src/providers/mod.rs`, `src/providers/{anthropic,openai,openai_responses,gemini,cohere,azure,bedrock,vertex,copilot,gitlab}.rs`

Related: model-registry, sse, auth, tool

synonyms:: provider, llm backend, llm provider, anthropic, openai, openai-codex, codex, gemini, google, cohere, azure, bedrock, vertex, github copilot, copilot, gitlab duo, gitlab, kimi, kimi-for-coding, deepseek, minimax, zai, provider trait, streaming provider
