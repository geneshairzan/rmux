# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

| Task | Command |
|---|---|
| System deps (Ubuntu) | `sudo apt install -y build-essential libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev` |
| Debug run | `cargo run` |
| Release run | `cargo run --release` |
| Full test suite | `cargo test` |
| Single test | `cargo test <test_name>` (example: `cargo test test_default_config`) |
| Install for current user | `make install-user` |

- No lint or format task is configured. `Makefile` targets are `build`, `run`, `test`, `install-user`, `clean`.
- Single binary target: `rmux`, entry `src/main.rs` (`Cargo.toml` sets `default-run = "rmux"`).
- GUI app. Tests run headless (`config`, `model` logic). Building needs GTK4/libadwaita/VTE headers and a display for `cargo run`.

## Release (standing authorization)

- "Go release" runs the full flow with no further confirmation: commit, tag, push commit plus tag, build the `.deb` via `packaging/deb/build-deb.sh`, create the GitHub release on `github.com/geneshairzan/rmux` with the `.deb` attached.
- Bump `Cargo.toml` first when the tag already exists remotely. Never move a published tag.

## Architecture

Single Rust binary. `MainWindow` (`src/window.rs`) owns the sidebar and one `TabView` per workspace.

| Area | Files | Notes |
|---|---|---|
| App entry | `src/main.rs` | libadwaita `Application`, loads `AppConfig`, applies `CUSTOM_CSS`, sets color scheme |
| Window shell | `src/window.rs` | Header bar, workspace switching, capture-phase shortcut engine via `matches_key` |
| Split layout | `src/split.rs`, `src/pane.rs` | `TabView` holds nested `gtk4::Paned` tree and `panes: HashMap<pane_id, Pane>` plus focused pane id |
| Terminal | `src/vte.rs` | Unsafe wrapper over `vte-sys` (`VteTerminal` raw pointer), spawns shell, applies font/colors |
| State models | `src/model.rs` | `Workspace` holds `Vec<Tab>`; `Tab` holds `LayoutNode` tree (`Leaf` / `Split { orientation, ratio, first, second }`) |
| Config | `src/config.rs` | `AppConfig` JSON at `~/.config/rmux/config.json`; atomic save via tmp file plus rename |
| Themes | `src/theme.rs` | 7 presets (`get_available_themes`, `find_theme`), applied live with no restart |
| Settings UI | `src/settings.rs`, `src/sidebar.rs` | 2-column modal (appearance, shortcuts, shell, about); sidebar owns workspace list plus Settings button |
| Agent detection | `src/agent.rs` | Reads `/proc/<pid>/comm` and `cmdline` to flag background coding agents |

## Patterns

- Shortcuts are strings (`Ctrl+Shift+R`). Parsing and matching live in `config::matches_key`. Window-level capture runs before VTE so shell keybindings do not swallow them.
- Split changes must unparent GTK widgets before reparenting into a new `Paned`. Skipping this triggers GTK critical assertions.
- `LayoutNode::Leaf` carries `pane_id`, `working_directory`, `command`. New panes inherit the focused pane working directory.
- Config load is fallible-tolerant: missing or corrupt file returns `AppConfig::default`.
