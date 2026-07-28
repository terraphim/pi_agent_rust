# Interactive TUI

Interactive terminal front-end (`src/interactive.rs`, `src/tui.rs`) built on the charmed_rust stack (bubbletea/lipgloss/bubbles/glamour) gated behind the default-on `tui` feature. 60fps differential rendering. Slash commands live in `src/interactive/commands.rs` (`/login`, `/model`, `/mcp`, `/help`). Mouse capture can be disabled with `PI_NO_MOUSE_CAPTURE=1` to let terminal-native copy/paste work during OAuth.

**Key files:** `src/interactive.rs`, `src/interactive/commands.rs`, `src/tui.rs`, `Cargo.toml` (`tui` feature)

Related: rpc, acp, model-registry

synonyms:: interactive, tui, interactive tui, terminal ui, bubbletea, charmed_rust, lipgloss, glamour, slash command, /login, /model, /mcp, mouse capture, pi no mouse capture, frontend, TreeSelectorRow, TreeSelectorState, PiApp, SessionPickerOverlay, ThemePickerOverlay, BranchPickerOverlay, ThemePickerItem
