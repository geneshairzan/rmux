# RFC: Shift+Enter and Alt+Enter insert newline

| Item | Value |
|---|---|
| Status | Draft |
| Date | 2026-10-22 |
| Scope | `src/window.rs` capture handler, `src/vte.rs` feed wrapper |

## Problem

- `Enter` submits the current line in every hosted app. No key inserts a newline without submitting.
- VTE defaults collapse the combos: `Shift+Enter` arrives as plain `CR`, `Alt+Enter` arrives as `ESC` + `CR`.
- Agent CLIs and REPLs distinguish submit (`CR`) from newline (`LF`). rmux cannot express the second.

## Proposal

- Intercept `Shift+Return` and `Alt+Return` in the capture-phase shortcut handler (`src/window.rs`).
- Stop propagation. Feed one `LF` (`\n`) to the focused pane via a new `TerminalWidget::feed_child_text` wrapper around `vte_terminal_feed_child` (present in locked `vte4-sys 0.10`).
- Plain `Enter` unchanged. Route per pane through `focused_pane_id`, same pattern as the copy/paste handlers.

| Combo | Today | After |
|---|---|---|
| `Enter` | `CR`, shell executes | Unchanged |
| `Shift+Enter` | Same as `Enter` | `LF`, newline without submit |
| `Alt+Enter` | `ESC` + `CR` (Meta) | `LF`, same as `Shift+Enter` |

## Side effects

- `Alt+Return` loses its Meta binding. Vim, tmux, emacs, and fzf mappings on `M-RET` stop working. Narrow to Shift-only if that binding matters.
- Plain shells barely change. Bash submits on both `CR` and `LF`. The visible difference appears only in apps that separate the two.
- No shortcut conflict. No existing shortcut uses `Return`. `matches_key` requires exact modifiers, so `Ctrl+Shift+Return` still passes through.
- Two details need confirmation in code: `Key::from_name("return")` resolution (headless unit test), and whether numpad `KP_Enter` is included.

## Verification

- `cargo test` for the key-name unit test.
- `cargo run` plus `showkey -a`: confirm bytes per combo.
- Manual checks in bash, vim, tmux, and the agent CLI.
