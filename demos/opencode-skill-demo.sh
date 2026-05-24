#!/usr/bin/env bash
set -euo pipefail

cd /Users/alex/projects/pi_agent_rust

clear
printf 'Opencode + pi-rust-terraphim-router\n'
printf '===================================\n\n'
printf 'This is a real Opencode run using the installed skill.\n\n'

printf '$ test -f ~/.config/opencode/skill/pi-rust-terraphim-router/SKILL.md && echo "skill installed"\n'
test -f ~/.config/opencode/skill/pi-rust-terraphim-router/SKILL.md
printf 'skill installed\n\n'

printf '$ pi --version\n'
pi --version
printf '\n'

printf '$ opencode run ... "Use the pi-rust-terraphim-router skill"\n'
opencode run --dangerously-skip-permissions \
  "Use the pi-rust-terraphim-router skill. Run exactly: pi demo-route --format json 'Design a resilient microservices architecture for checkout'. Report only the selected provider/model from the output. Do not edit files."

printf '\nOpencode delegated routing to pi-rust and selected the architecture specialist.\n'
