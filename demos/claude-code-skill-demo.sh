#!/usr/bin/env bash
set -euo pipefail

cd /Users/alex/projects/pi_agent_rust

clear
printf 'Claude Code + pi-rust-terraphim-router\n'
printf '======================================\n\n'
printf 'This is a real Claude Code run using the installed skill.\n\n'

printf '$ test -f ~/.claude/skills/pi-rust-terraphim-router/SKILL.md && echo "skill installed"\n'
test -f ~/.claude/skills/pi-rust-terraphim-router/SKILL.md
printf 'skill installed\n\n'

printf '$ pi --version\n'
pi --version
printf '\n'

printf '$ claude -p ... "Use the pi-rust-terraphim-router skill"\n'
claude -p \
  --permission-mode bypassPermissions \
  --allowedTools "Bash(pi demo-route *)" \
  -- \
  "Use the pi-rust-terraphim-router skill. Run exactly: pi demo-route --format json 'Write comprehensive unit tests for this parser'. Report only the selected provider/model from the output. Do not edit files."

printf '\nClaude Code delegated routing to pi-rust and selected the test specialist.\n'
