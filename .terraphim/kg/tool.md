# Tool

Built-in agent tools (`src/tools.rs`) invoked by the agent loop (`src/agent.rs`): `read` (files with line numbers + image support), `write`, `edit` (string replacement), `bash` (shell with timeout), `grep` (content search with context), `find` (glob discovery), `ls`, and `hashline_edit` (precise `LINE#HASH` edits). Tool definitions carry JSON Schema; results are truncated for context safety. Extension tools are registered alongside built-ins.

**Key files:** `src/tools.rs`, `src/agent.rs`

Related: extension, hashline-edit, agent loop

synonyms:: tool, tools, built-in tool, read tool, write tool, edit tool, bash tool, grep tool, find tool, ls tool, hashline_edit, agent tool, tool registry, tool call, tool definition
