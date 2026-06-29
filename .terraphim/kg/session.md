# Session

JSONL session persistence (version 3 format) with a tree structure for branching. Entries are typed: Message, ModelChange, ThinkingLevel, Compaction, etc. Per-project session directories under `~/.pi/agent/sessions/` with a session-index metadata cache. SQLite backend is default-on via the `sqlite-sessions` feature (opt out with `--no-default-features`). Session replay/index correctness is covered by the `session` test module.

**Key files:** `src/session.rs`, `src/session_index.rs`, `src/session_test.rs`

Related: model-registry, interactive-tui, rpc

synonyms:: session, sessions, jsonl session, session persistence, session tree, session branch, session index, session replay, sqlite session, sqlite-sessions, compaction, session entry, session migrate
