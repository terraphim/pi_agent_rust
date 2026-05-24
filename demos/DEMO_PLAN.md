# Demo Video Creation Plan

## Integrity Rule

**All demos must use real, captured output. No fabrication, no hardcoded echo, no static cat heredocs.**
If a demo step cannot produce real output at recording time, it must be removed.

## Tool Choice: VHS

- **Why**: Deterministic, declarative `.tape` files, single command to render
- **Version**: vhs 0.11.0 (`/opt/homebrew/bin/vhs`)
- **Theme**: Monokai, background `#0C0C0C`
- **Playback**: 1.4-1.6x speed, 16ms typing, 1280x720

## Demo 1: Dynamic Routing (primary demo)

**Status: Working. Calls real `pi demo-route --format json`.**

| File | Description |
|------|-------------|
| `demos/dynamic-routing-demo.sh` | Runs `pi demo-route` with 4 different prompts |
| `demos/dynamic-routing-demo.tape` | VHS tape file |

**Prerequisites**: `pi` binary in PATH (built with `terraphim-routing` feature).

**What it demonstrates**: Same command, different prompts, different specialist models selected automatically.

## Demo 2: Claude Code + pi-rust terraphim-router

**Status: Working. Runs real `cargo test` commands.**

| File | Description |
|------|-------------|
| `demos/claude-code-demo.sh` | Builds + runs all 16 terraphim-router tests |
| `demos/claude-code-demo.tape` | VHS tape file (calls `claude-code-skill-demo.sh`) |

**Prerequisites**: Local build with `terraphim-routing` feature; sibling `terraphim_router`/`terraphim_types` crates available.

**What it demonstrates**: Test suite for the terraphim-router module (extractor, mapper, RPC client).

**Note**: The `.tape` file calls `claude-code-skill-demo.sh` which requires the external `claude` CLI. GIF re-recording needs `claude` installed and authenticated.

## Demo 3: Opencode + pi-rust terraphim-router

**Status: Working. Runs real `cargo test` and `cargo build` commands.**

| File | Description |
|------|-------------|
| `demos/opencode-demo.sh` | Verifies module structure + runs test suite |
| `demos/opencode-demo.tape` | VHS tape file (calls `opencode-skill-demo.sh`) |

**Prerequisites**: Same as Demo 2.

**What it demonstrates**: Module structure, build, and test verification for terraphim-router.

**Note**: The `.tape` file calls `opencode-skill-demo.sh` which requires the external `opencode` CLI. GIF re-recording needs `opencode` installed.

## Removed Files (2026-05-24 cleanup)

These files were fabricated (hardcoded output, fake performance numbers, static heredocs):

- `demos/claude_session.sh` — 100% fake `cat` heredoc
- `demos/route_demo.sh` — hardcoded `echo` statements
- `demos/DEMO_TRANSCRIPTS.md` — fabricated transcripts with fake numbers
- `demos/claude-code-demo.gif` — unverifiable, needs re-recording
- `demos/opencode-demo.gif` — unverifiable, needs re-recording
- `demos/dynamic-routing-demo.gif` — unverifiable, needs re-recording

## GIF Re-recording

To regenerate GIFs from real sessions:

```bash
# Requires VHS installed: brew install vhs
# Dynamic routing (self-contained, just needs pi in PATH)
vhs demos/dynamic-routing-demo.tape

# Claude Code skill demo (needs claude CLI + pi in PATH)
vhs demos/claude-code-demo.tape

# Opencode skill demo (needs opencode CLI + pi in PATH)
vhs demos/opencode-demo.tape
```
