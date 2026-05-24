# Demo Video Creation Plan

## Based on RLM Demo Learnings

### Tool Choice: VHS
- **Winner**: VHS over asciinema+agg and Remotion
- **Why**: Fully deterministic, declarative .tape files, readable source, single command to render
- **Version**: vhs 0.11.0 (installed at /opt/homebrew/bin/vhs)

### Theme: Monokai Remastered
- **Background**: Near-black #0C0C0C
- **Why**: Highest contrast for feed previews, smallest GIF size (compression-friendly)
- **VHS Setting**: `Set Theme { "name": "monokai", "background": "#0C0C0C" }`

### Content Style: Real Captured Output
- **Rule**: Use actual tmux session output, not staging or fabrication
- **Source**: 
  - `tmux:claude_demo` - Claude Code session output
  - `tmux:opencode_demo` - Opencode session output
- **Method**: Export tmux buffer to file, replay via VHS `Type` and `Sleep` commands

### Playback Settings
- **Speed**: ~1.4-1.5x (fast enough to hold attention, slow enough to read)
- **Typing Speed**: 18-22ms per character
- **Resolution**: 1280x720 or 1280x800
- **Output**: .gif for social media, .mp4 for documentation

## Demo 1: Claude Code + pi-rust terraphim-router

### Script Structure
1. **Intro** (3s): Show terminal with project path
2. **Build** (5s): `cargo build --features terraphim-routing`
3. **Tests** (8s): Run all 16 tests with checkmarks
4. **Capability Extraction** (15s): Show 8 prompts with extracted capabilities
5. **Provider Mapping** (12s): Show routing decisions with confidence scores
6. **Outro** (3s): Summary of features

### Tape File: `demos/claude-code-demo.tape`

## Demo 2: Opencode + pi-rust terraphim-router

### Script Structure
1. **Intro** (3s): Show terminal with feature flag verification
2. **API Surface** (10s): Show all 3 public APIs
3. **Performance** (8s): Show microsecond-level benchmarks
4. **Provider Map** (10s): Show capability-to-provider table
5. **Outro** (3s): Call to action

### Tape File: `demos/opencode-demo.tape`

## Execution Steps

1. **Export tmux output** to text files
2. **Create .tape files** with real output
3. **Render with VHS**: `vhs demos/claude-code-demo.tape`
4. **Verify output** (GIF/MP4 quality)
5. **Commit** tape files and generated demos

## Files to Create

- `demos/claude-code-demo.tape` - VHS tape for Claude Code demo
- `demos/opencode-demo.tape` - VHS tape for Opencode demo
- `demos/claude-code-demo.gif` - Generated GIF
- `demos/opencode-demo.gif` - Generated GIF
