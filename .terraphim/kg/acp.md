# ACP

Agent Client Protocol support. `pi --acp` runs pi as an ACP server (stdin JSON-RPC) so external clients drive it headlessly. Supports `session/set_model`, `session/set_config_option`, and `--session-dir` persistence. The runtime-parking fix (#107) stopped idle `--acp` from burning ~200% CPU. Distinct from the interactive TUI and the RPC/stdin bridge in `src/rpc.rs`.

**Key files:** `src/main.rs`, `src/acp.rs`, `src/rpc.rs`

Related: interactive-tui, session, rpc

synonyms:: acp, agent client protocol, --acp, acp server, acp session, set_model, set_config_option, session-dir, json-rpc server, rpc mode, stdin mode, rpc, AcpOptions
