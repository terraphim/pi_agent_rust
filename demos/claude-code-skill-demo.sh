#!/usr/bin/env bash
set -euo pipefail

cd /Users/alex/projects/pi_agent_rust

clear
printf 'Claude Code + pi-rust dynamic routing\n'
printf '======================================\n\n'
printf 'Same assistant. Different prompts. Different models.\n\n'

printf '$ pi --version\n'
pi --version
printf '\n'

run_claude() {
  local task_label="$1"
  local prompt="$2"

  printf -- '--- %s ---\n' "$task_label"
  printf 'Prompt: %s\n' "$prompt"
  printf '$ claude -p "Use pi-rust-terraphim-router skill to route: %s"\n' "$prompt"
  result="$(claude -p \
    --permission-mode bypassPermissions \
    --allowedTools "Bash(pi demo-route *)" \
    -- \
    "Use the pi-rust-terraphim-router skill. Run: pi demo-route --format json '$prompt'. Report ONLY the top provider/model and confidence on one line. Do not edit files.")"
  printf '%s\n\n' "$result"
}

run_claude \
  "Deep reasoning" \
  "Think carefully about the tradeoffs between RAFT and Paxos"

run_claude \
  "Security audit" \
  "Audit this SQL query builder for injection vulnerabilities"

run_claude \
  "Write tests" \
  "Write tests for the authentication module"

run_claude \
  "Architecture design" \
  "Design an event-driven architecture for order processing"

printf 'Each prompt selected a different specialist model.\n'
printf 'No manual model switching required.\n'
