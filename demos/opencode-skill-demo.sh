#!/usr/bin/env bash
set -euo pipefail

cd /Users/alex/projects/pi_agent_rust

clear
printf 'Opencode + pi-rust dynamic routing\n'
printf '===================================\n\n'
printf 'Same assistant. Different prompts. Different models.\n\n'

printf '$ pi --version\n'
pi --version
printf '\n'

run_opencode() {
  local task_label="$1"
  local prompt="$2"

  printf -- '--- %s ---\n' "$task_label"
  printf 'Prompt: %s\n' "$prompt"
  printf '$ opencode run "Use pi-rust-terraphim-router: %s"\n' "$prompt"
  result="$(opencode run --dangerously-skip-permissions \
    "Use the pi-rust-terraphim-router skill. Run: pi demo-route --format json '$prompt'. Report ONLY the top provider/model and confidence on one line. Do not edit files.")"
  printf '%s\n\n' "$result"
}

run_opencode \
  "Deep reasoning" \
  "Think carefully about the tradeoffs between RAFT and Paxos"

run_opencode \
  "Refactor code" \
  "Refactor this legacy code to use async/await"

run_opencode \
  "Write tests" \
  "Write tests for the authentication module"

run_opencode \
  "Performance optimisation" \
  "Optimize this database query for performance"

printf 'Each prompt selected a different specialist model.\n'
printf 'No manual model switching required.\n'
