#!/usr/bin/env bash
set -euo pipefail

cd /Users/alex/projects/pi_agent_rust

clear
printf 'pi-rust dynamic model routing\n'
printf '=============================\n\n'
printf 'Same command. Different prompts. Different specialist models.\n\n'
printf '$ pi --version\n'
pi --version
printf '\n'

run_route() {
  local label="$1"
  local prompt="$2"
  local routing

  printf '%s\n' "$label"
  printf 'prompt: %s\n' "$prompt"
  printf '$ pi demo-route --format json <prompt>\n'
  routing="$(pi demo-route --format json "$prompt")"
  printf '%s\n' "$routing" | jq -r '
    "capabilities: " + (.capabilities | sort | join(", ")),
    "selected model(s):",
    (.providers | sort_by(.capability)[] | "  - " + .capability + " -> " + .provider + "/" + .model + " (confidence " + (((.confidence * 100) | round) / 100 | tostring) + ")")
  '
  printf '\n'
}

run_route '1. Deep reasoning prompt routes to Kimi' \
  'Think carefully about the tradeoffs in this distributed consensus algorithm'

run_route '2. Security prompt routes to Claude Sonnet' \
  'Audit this login handler for security vulnerabilities'

run_route '3. Testing prompt routes to Codex Spark' \
  'Write comprehensive unit tests for this parser'

run_route '4. Architecture prompt routes to Claude Opus' \
  'Design a resilient microservices architecture for checkout'

printf 'Result: pi-rust selects the model from the prompt, not from a hard-coded CLI flag.\n'
