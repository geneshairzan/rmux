# RFC: Session Restore — Workspaces + Split Layout (No History)

**Status:** Draft
**Date:** 2026-10-23
**Author:** Project Architect

## Problem Statement

`rmux` currently loses all workspace state on real quit. `MainWindow::new` (`src/window.rs:547`) always creates a single `Tab::default()`. The `Tab.root_layout` / `Workspace` model in `src/model.rs` is dead code — `TabView` (`src/split.rs`) builds the `gtk4::Paned` tree imperatively and never syncs back to the model.

User request: on app close and reopen, restore **how many workspaces and split terminals** existed. Explicitly **no need to restore scrollback / history / running processes** — fresh shells are fine.

PRD already requires this: F-08 (Persistence and Recovery, §4), F-15 (Session recovery), and MVP Done #5 ("Workspace layout and preferences survive application restarts"). §7 Security forbids auto-executing restored commands.

## Proposed Solution

Add lightweight **layout-only session persistence**:

- New file `~/.config/rmux/session.json` (XDG-compliant, same dir as `config.json`), **separate** from user prefs so a corrupt session never wipes settings.
- Schema reuses existing `LayoutNode` (`src/model.rs:15-30`) — already `Serialize/Deserialize` with `leaf`/`split` tags. Wrap it:

```json
{
  "version": 1,
  "active_tab_id": "<uuid>",
  "tabs": [
    { "id": "<uuid>", "title": "Workspace 1", "root_layout": { "type": "leaf", ... } }
  ]
}
```

- Leaf stores only `{ pane_id (ignored on load, regenerated), working_directory, command: always None }`. No scrollback, no env, no history, no command re-execution.
- `TabView` gains two methods (single source of truth = live GTK tree):
  - `snapshot_layout() -> LayoutNode` — recursively walk `container` child: `Pane container (gtk4::Box)` → `Leaf{cwd}`; `gtk4::Paned` → `Split{orientation, ratio, first, second}` where `ratio = position / allocation-size` clamped `0.1..=0.9`, fallback `0.5` when unrealized.
  - `TabView::from_layout(config, LayoutNode) -> Rc<TabView>` (or `restore_layout`) — rebuild Paned tree from a layout, spawning one fresh `Pane`/shell per leaf in saved `working_directory` (validate: exists + is dir, else fallback `$HOME` → `/`). Cap: ≤16 leaves/tab, ≤20 tabs.
- `MainWindow` loads session at startup: `SessionStore::load()` → if `Some` and non-empty, build tabs + `TabView::from_layout` for each; else single `Tab::default()` (current behavior). Sidebar + `switch_tab` unchanged.
- Save triggers (debounced ~500 ms via `glib::timeout_add_local`, plus immediate on quit):
  - add / close / rename workspace, split / close pane, `Paned` `position-notify`, focused-pane change (to persist `active_tab_id`).
  - `connect_close_request` (both hide-to-tray and real quit paths) and `TrayAction::Quit` handler.
  - Atomic write: tmp file + `rename` (same pattern as `AppConfig::save`).
- Corrupt/missing/empty session → fall back to default single workspace, never crash, never overwrite good config.

Text diagram:

```
close/quit ──► collect: tabs[].title + TabView.snapshot_layout()
                  ──► session.json (atomic)
open ──► SessionStore::load() ──► for each tab: TabView::from_layout(node)
                                   spawn fresh shell per leaf in saved cwd
```

## Implementation Plan

1. **New module `src/session.rs`** — `SavedTab {id,title,root_layout}`, `SessionFile {version, active_tab_id, tabs}`, `SessionStore::load/save/snapshot` helpers, path `~/.config/rmux/session.json`, validation + caps. Unit tests (round-trip, corrupt fallback, cap enforcement). Delegatable to `project-backend`.
2. **`TabView` snapshot/restore (`src/split.rs`)** — add `snapshot_layout()`, `from_layout()` / `restore_layout()`, `working_directory()` accessor for save, ratio read via `Paned::position` + `width/height`. Handle unrealized widgets (fallback 0.5 + `idle_add` centering already exists). Delegatable to `project-backend`.
3. **`MainWindow` wiring (`src/window.rs`)** — load session in `new()` instead of hardcoded `Tab::default()`; add `save_session_soon()` debounced helper; hook all mutation sites (add/close/rename/split/close/ratio/focus); save on `close_request` + `TrayAction::Quit`. Register `mod session` in `src/main.rs`. Delegatable to `project-backend`.
4. **Tests + manual verify** — `cargo test` (new session tests + existing model/config tests); manual: 3 workspaces × nested splits → Quit from indicator → relaunch → same count/titles/splits/cwds, fresh prompts, no history. Delegatable to `project-backend`.

## Affected Files

- NEW `src/session.rs` — session schema, load/save, validation.
- `src/split.rs` — add `snapshot_layout`, `from_layout`/`restore_layout`.
- `src/window.rs` — startup restore, save hooks, close/quit save.
- `src/main.rs` — `mod session;` (one line).
- `src/model.rs` — reuse `LayoutNode` as-is; optionally move `SavedTab`/`SessionFile` here (prefer `session.rs` to keep model stable). No format change → backward compatible.
- NEW `~/.config/rmux/session.json` at runtime (not in repo).

## Alternatives Considered

- **Store layout inside `config.json`**: rejected — mixes fast-changing layout with slow-changing prefs; corrupt layout could wipe shortcuts/themes. Separate file isolates failure domains (PRD §6.4).
- **Maintain parallel `LayoutNode` model in `State.tabs` on every mutation**: rejected — dual source of truth drifts from GTK reality (ratios, drag-resize). Snapshot-from-widget is always accurate with less code.
- **SQLite / full process restore (tmux-style)**: rejected — overkill for MVP, conflicts with "no history" request and PRD §6.4 ("database not required"), plus security risk of reviving processes.
- **Restore shell commands per pane**: rejected — PRD §7 explicitly forbids auto-executing restored commands. `command` forced to `None` on save and ignored on load.

## Risks & Trade-offs

- **Stale cwd**: `working_directory` is captured at pane spawn (`vte.rs:35`), not tracked through later `cd`. Restored pane opens in the *original* cwd. Acceptable for MVP; future: OSC-7 / `/proc/<pid>/cwd` polling (see `src/agent.rs` pattern). Document as known limitation.
- **Ratio before realize**: `width/height == 0` on hidden tabs → ratio fallback 0.5. Mitigated by existing `idle_add` centering in `split_pane`.
- **Corrupt session fork-bomb**: mitigated by caps (20 tabs / 16 leaves) + path validation + ignoring `command`.
- **Save churn on drag-resize**: mitigated by 500 ms debounce; writes are small JSON, atomic.
- **Hide-to-tray vs quit**: X only hides (in-memory state survives), but we save anyway so a crash/X-kill still recovers on next launch.

## Acceptance Criteria

- [ ] Quit via top-bar indicator → relaunch restores same workspace count, titles, order, active workspace.
- [ ] Each workspace restores same nested split structure (vertical/horizontal) with approximately same ratios (±10%).
- [ ] Each restored pane spawns a fresh shell in the saved cwd (or `$HOME` fallback); prompt is fresh, no scrollback restored.
- [ ] No command is auto-executed on restore.
- [ ] Corrupt/missing `session.json` → app launches with single default workspace, no crash.
- [ ] `cargo test` passes, including new `session` round-trip + fallback tests.
