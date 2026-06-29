# Auth

Credential storage and OAuth flows (`src/auth.rs`). `auth.json` (perms 600) at `~/.pi/agent/auth.json` holds OAuth tokens (openai-codex via localhost:1455 callback, anthropic, google gemini-cli via :8085, github copilot, kimi-for-coding) and API keys. `/login <provider>` runs the in-TUI OAuth flow; `pi doctor` reports token validity. Expired tokens must be re-logged-in.

**Key files:** `src/auth.rs`, `~/.pi/agent/auth.json`

Related: provider, interactive-tui

synonyms:: auth, authentication, oauth, auth.json, login, /login, credentials, oauth token, token expired, api key, openai-codex login, kimi oauth, copilot login, auth storage, re-login
