# Product Requirements Document (PRD)
## Native Ubuntu Terminal Emulator for AI-Assisted Development

**Status:** Draft v1.0  
**Date:** October 20, 2026  
**Target platform:** Ubuntu 26.04 LTS  
**Product category:** Native Linux desktop application  
**Working name:** TBD

---

## 1. Product Overview

### 1.1 Vision

Build a fast, stable, native Ubuntu terminal application inspired by cmux, optimized for developers who run multiple terminal sessions, coding agents, and development workflows simultaneously.

The application combines a modern terminal emulator with a workspace-management interface, allowing developers to organize shells, projects, and AI coding agents in a single desktop application.

Unlike Electron-based terminals, the application should use native Linux UI technology and avoid embedding a browser engine in the primary application interface.

### 1.2 Problem Statement

Existing terminal applications often force developers to choose between:

- Traditional terminals with limited workspace organization.
- Multiplexers such as tmux that require learning keyboard-driven workflows and configuration.
- Modern desktop terminals whose rendering, window management, or webview dependencies may introduce stability issues.
- Separate terminals for multiple AI coding agents, making project context and session management harder.

The product should make multi-session development straightforward without sacrificing terminal compatibility or Linux integration.

### 1.3 Product Goals

1. Provide a stable native terminal experience on Ubuntu.
2. Support multiple workspaces, tabs, and split terminal panes.
3. Make it easy to run multiple AI coding agents concurrently.
4. Provide a polished GTK-based desktop interface.
5. Keep the application responsive during intensive terminal activity.
6. Integrate naturally with Git repositories, local shells, and existing development tools.
7. Minimize configuration and setup required for everyday use.

### 1.4 Non-Goals

The initial release will not attempt to:

- Build a new terminal protocol implementation from scratch.
- Replace tmux or provide a complete remote terminal management platform.
- Provide its own AI model, coding agent, or LLM subscription.
- Implement a full IDE, debugger, or code editor.
- Require Docker, a cloud service, or an account to operate.
- Support Windows and macOS in the MVP.
- Bundle proprietary coding agents or their credentials.

---

## 2. Target Users

### Primary persona: AI-assisted software developer

A developer working on several repositories who runs Claude Code, Codex, OpenCode, or other CLI agents alongside normal shell commands.

**Needs**
- Multiple terminal sessions visible simultaneously.
- Separate agent sessions for different tasks or Git branches.
- Fast switching between projects.
- Clear visibility into running and completed processes.
- Reliable copy/paste, keyboard shortcuts, and terminal compatibility.

### Secondary persona: Linux power user

A developer who wants the convenience of a graphical terminal workspace while retaining access to shell tools, scripts, SSH, tmux, Git, and custom shell configuration.

**Needs**
- Native Linux behavior.
- Minimal resource overhead.
- Configurable keyboard shortcuts.
- Compatibility with existing shell environments.
- Persistent workspace and session organization.

---

## 3. Product Scope

### 3.1 MVP Features

| ID | Feature | Priority |
|---|---|---|
| F-01 | Native GTK application window | P0 |
| F-02 | Terminal emulation and shell integration | P0 |
| F-03 | Multiple tabs and workspaces | P0 |
| F-04 | Horizontal and vertical split panes | P0 |
| F-05 | Create, close, and switch terminal sessions | P0 |
| F-06 | Keyboard shortcuts | P0 |
| F-07 | Dark and light themes | P0 |
| F-08 | Persistent workspace layout | P0 |
| F-09 | Configurable default shell | P0 |
| F-10 | Copy, paste, search, and scrollback | P0 |
| F-11 | Basic running-process and exit-status indicators | P1 |
| F-12 | Git branch and repository indicators | P1 |
| F-13 | AI-agent activity notifications | P1 |
| F-14 | Customizable key bindings and appearance | P1 |
| F-15 | Session recovery after application restart | P1 |

P0 features define the first usable release. P1 features may follow after terminal reliability and workspace management are validated.

### 3.2 Future Features

- Named project workspaces and workspace templates.
- Agent-specific status indicators and completion notifications.
- Optional persistent terminal sessions.
- Remote SSH workspace support.
- Configurable command launch profiles.
- Search across workspace and session metadata.
- Optional plugin or extension system.
- Multi-monitor window and workspace management.
- Session export and import.

These features must not delay the core MVP.

---

## 4. Functional Requirements

### F-01: Native Application Window

The application MUST:

- Launch as a normal Ubuntu desktop application.
- Use GTK4 and libadwaita for its primary interface.
- Support window resizing, minimizing, maximizing, and closing.
- Follow system dark/light appearance where possible.
- Integrate with Ubuntu desktop keyboard and window behavior.
- Support normal application launchers and desktop entries.

**Acceptance criteria**
- The application launches without a browser window or embedded web UI.
- Closing and reopening the application does not corrupt saved workspace metadata.
- The UI remains responsive during normal terminal output.

### F-02: Terminal Emulation

The application MUST use an established terminal engine rather than implementing VT emulation independently.

The initial architecture SHOULD evaluate Ghostty's terminal library (`libghostty`) as the preferred engine.

Required terminal behavior includes:

- PTY-backed interactive shell sessions.
- ANSI escape sequences and common VT terminal behavior.
- UTF-8 and Unicode text.
- Terminal colors and standard color palettes.
- Alternate screen support for tools such as `vim`, `less`, and terminal-based applications.
- Mouse interaction and terminal selection.
- Scrollback history.
- Clipboard operations.
- Terminal resizing and proper PTY window-size updates.
- Configurable font family and size.

**Acceptance criteria**
- Bash and Zsh work with standard interactive configurations.
- Common terminal applications run without rendering corruption.
- Long output and rapid updates do not freeze the main interface.
- Keyboard input and terminal resizing work correctly.

### F-03: Workspaces

Users MUST be able to organize terminals into workspaces.

Each workspace contains a collection of terminal sessions and an associated layout.

Required operations:

- Create a workspace.
- Rename a workspace.
- Switch between workspaces.
- Close a workspace.
- Create a workspace with a specified starting directory.
- Save workspace ordering and layout.

A workspace should represent a project, task, or development context.

### F-04: Tabs and Split Panes

The application MUST support multiple tabs and multiple terminal panes.

Users MUST be able to:

- Open and close tabs.
- Split a terminal horizontally or vertically.
- Resize split panes.
- Focus a specific pane.
- Move between panes using keyboard shortcuts.
- Close individual panes without closing unrelated sessions.
- Create additional panes from the current working directory when available.

Layouts should support nested splits rather than limiting users to a fixed two-pane arrangement.

**Acceptance criteria**
- Each pane has its own terminal session and PTY.
- Resizing one pane does not corrupt other panes.
- Closing a pane does not terminate other unrelated sessions.
- Layouts can be serialized and restored.

### F-05: Shell and Process Management

The application MUST support the user's existing local shell environment.

Requirements:

- Detect the default shell where practical.
- Allow the user to select a shell executable.
- Launch shells with an appropriate working directory and environment.
- Detect process termination and exit status when available.
- Display whether a session is running, idle, or terminated when the state can be determined reliably.
- Provide a way to create a fresh session after termination.

The application MUST NOT assume that every shell command is observable or that all processes expose reliable activity information.

### F-06: Keyboard Shortcuts

The application MUST provide shortcuts for common operations, including:

- New tab.
- New workspace.
- Close tab or pane.
- Split horizontally and vertically.
- Focus adjacent pane.
- Switch tabs and workspaces.
- Increase and decrease terminal font size.
- Search terminal output.
- Copy and paste.
- Open application preferences.

Shortcuts SHOULD be configurable.

The application MUST avoid intercepting terminal-specific key combinations unnecessarily. Users must still be able to operate terminal editors, shells, and coding agents.

### F-07: Appearance and Preferences

The application MUST support:

- Dark, light, and system-following appearance.
- Configurable terminal font family and size.
- Configurable cursor appearance where supported.
- Adjustable scrollback limits.
- Configurable default shell and startup directory.
- Workspace and pane layout preferences.

Preferences should be stored locally using a documented, versioned configuration format.

### F-08: Persistence and Recovery

The application MUST save workspace metadata and layout information.

Saved metadata may include:

- Workspace names and order.
- Tab ordering.
- Split tree structure and pane proportions.
- Session working directories, when available.
- Selected theme and appearance settings.
- Other user preferences.

The application MUST distinguish between restoring a terminal layout and restoring a running process.

A restored pane MUST NOT be presented as a recovered live session unless the underlying process actually survived.

For the initial implementation, the application may recreate terminal panes with their previous working directories while explaining that terminated shell processes cannot be recovered.

### F-09: Git Integration

The application SHOULD display repository information when available, including:

- Repository name.
- Current Git branch.
- Dirty working-tree indicator, if practical.

Git detection must not block the UI or execute expensive repository scans on every terminal output event.

The feature must work with ordinary local Git repositories and degrade gracefully outside repositories.

### F-10: AI Coding Agent Awareness

The application SHOULD provide useful visibility into coding-agent activity without becoming dependent on a specific vendor.

Initial support should include:

- Running agent sessions.
- Optional user-configured agent launch profiles.
- Notifications when a supported process exits.
- Optional notifications when a supported agent reports completion or requires attention.
- Per-workspace identification of agent sessions.

The MVP SHOULD prefer explicit integrations, configurable commands, and opt-in shell hooks over fragile parsing of terminal text.

The application MUST NOT capture or upload prompts, terminal output, or API credentials to external services by default.

---

## 5. User Experience and Interface

### 5.1 Main Window

The interface SHOULD contain:

1. **Sidebar**
   - Workspace list.
   - Workspace creation.
   - Workspace selection and reordering.

2. **Tab bar**
   - Tabs within the selected workspace.
   - New-tab control.
   - Session status indicators where available.

3. **Terminal area**
   - One or more split terminal panes.
   - Clear focus indication.
   - Pane resize handles.

4. **Status area**
   - Optional current working directory.
   - Git branch.
   - Process status.
   - Relevant notifications.

5. **Preferences**
   - Appearance.
   - Fonts.
   - Shell configuration.
   - Keyboard shortcuts.
   - Persistence settings.

### 5.2 Design Principles

- Native Ubuntu desktop look and behavior.
- Keyboard-first navigation without requiring keyboard-only operation.
- Minimal visual clutter.
- Clear distinction between workspaces, tabs, and panes.
- No modal dialog for routine terminal operations unless necessary.
- Immediate visual feedback when changing layout or switching sessions.
- Avoid animations or visual effects that compromise responsiveness.

### 5.3 Empty State

On first launch, the application SHOULD display a welcome screen with a primary action to open a terminal.

Users should be able to start using the application without creating an account, importing configuration, or configuring an AI provider.

---

## 6. Technical Architecture

### 6.1 Recommended Stack

| Component | Technology | Responsibility |
|---|---|---|
| Application language | Rust | Core application logic and lifecycle |
| GUI toolkit | GTK4 | Native Linux UI |
| Design system | libadwaita | GNOME-style components and conventions |
| Terminal engine | libghostty, subject to feasibility validation | Terminal emulation and rendering |
| Terminal processes | PTY implementation integrated with the engine | Shell process and terminal communication |
| Async operations | Tokio only where useful | Background tasks and asynchronous services |
| Serialization | Serde | Configuration and workspace persistence |
| Configuration | XDG-compliant files | Local preferences and layout |
| Packaging | Native Linux package and/or Flatpak | Distribution and installation |

**Important architectural constraint:** Ghostty's terminal library is not a trivial drop-in Rust widget. Its integration approach, API maturity, build requirements, licensing, and rendering ownership must be validated before committing to it. If the integration is impractical, evaluate VTE through its available Rust bindings as the fallback.

### 6.2 High-Level Components

The application SHOULD separate the following modules:

- `app`: application initialization and lifecycle.
- `window`: main window and top-level UI.
- `workspace`: workspace model and switching.
- `layout`: split tree and pane geometry.
- `terminal`: terminal-engine integration.
- `session`: shell process and PTY lifecycle.
- `config`: preferences and schema migration.
- `persistence`: workspace serialization and recovery.
- `git`: repository metadata.
- `notifications`: application notifications.
- `agent`: optional coding-agent detection and integration.

Modules SHOULD have clear boundaries so the terminal engine can be replaced without rewriting workspace management.

### 6.3 Data Model

**Workspace**
- `id`
- `name`
- `position`
- `layout`
- `created_at`
- `updated_at`

**Tab**
- `id`
- `workspace_id`
- `title`
- `layout`
- `position`

**Pane**
- `id`
- `tab_id`
- `session_id`
- `layout_node`
- `working_directory`

**Session**
- `id`
- `pane_id`
- `shell`
- `working_directory`
- `process_state`
- `exit_status`
- `created_at`

The actual implementation may normalize these models differently. Session identifiers must be treated as application metadata, not proof that a process can survive a restart.

### 6.4 Persistence

The initial implementation SHOULD use local JSON or another lightweight format for workspace state.

Requirements:

- Use an XDG-compliant application configuration/data location.
- Write state atomically.
- Version the configuration schema.
- Recover gracefully from invalid or partially written configuration.
- Avoid saving sensitive environment variables, command history, or terminal output unless explicitly enabled.
- Do not overwrite valid configuration with an empty default after a transient loading error.

A database is not required for the MVP.

### 6.5 Rendering and Performance

The rendering architecture MUST be tested early with the target Ubuntu desktop environment and NVIDIA hardware.

Requirements:

- No browser engine in the main UI.
- Avoid blocking the GTK main thread with process inspection or filesystem operations.
- Ensure background tasks can be cancelled during application shutdown.
- Prevent excessive UI refreshes during high-volume terminal output.
- Avoid unnecessary terminal surface recreation when switching workspaces.
- Test rendering behavior under both GPU acceleration and relevant fallback modes when supported by the selected engine.

The application SHOULD use the rendering path supported by the terminal engine instead of implementing a separate custom renderer.

---

## 7. Security and Privacy

The application operates with the user's normal Linux permissions.

It MUST:

- Never execute arbitrary commands from untrusted workspace metadata without explicit user action.
- Never automatically execute restored shell commands by default.
- Avoid logging terminal content or environment variables.
- Avoid transmitting data to external services by default.
- Validate saved configuration before using it.
- Handle paths and process identifiers defensively.
- Avoid requiring root privileges for normal operation.
- Keep agent integrations opt-in where they require additional hooks or permissions.

Remote access, synchronization, and external integrations are out of scope for the initial release.

---

## 8. Non-Functional Requirements

### Reliability

- No known reproducible crash during the defined core regression suite.
- Workspace state survives normal application restarts.
- Corrupt configuration does not prevent recovery into a usable application.
- A terminated shell does not cause unrelated terminal sessions to terminate.

### Performance

Initial engineering targets, to be validated on representative hardware:

- Cold startup: under 2 seconds on the target development machine.
- Workspace switching: under 100 ms when sessions are already initialized.
- Keyboard input: no noticeable delay under ordinary workloads.
- Background tasks must not block UI event handling.
- Memory consumption should scale reasonably with the number of open terminal sessions.

These are target budgets, not guarantees. Terminal output volume, font configuration, session count, and rendering backend will affect results.

### Compatibility

The initial target is Ubuntu 26.04 LTS on a supported GNOME desktop environment.

The application SHOULD be tested with:

- X11 and Wayland, where supported by the chosen GUI/rendering stack.
- NVIDIA and software-rendering fallback configurations where practical.
- Bash and Zsh.
- Common terminal applications and TUI tools.
- Multiple concurrent shell and AI coding-agent sessions.

### Maintainability

- Use stable, documented interfaces where practical.
- Pin and track terminal-engine dependencies.
- Include automated tests for workspace and layout logic.
- Document engine integration and native build dependencies.
- Keep platform-specific code isolated.

---

## 9. Testing Strategy

### Unit tests

- Workspace creation, deletion, and ordering.
- Nested split-tree serialization.
- Pane and tab lifecycle.
- Configuration parsing and schema migration.
- Recovery from malformed state.
- Process-state transitions.

### Integration tests

- Spawn a shell and send input.
- Capture expected terminal output.
- Resize a PTY.
- Handle shell exit and non-zero exit status.
- Open several concurrent sessions.
- Restore workspace layouts after application restart.
- Run common terminal applications.
- Test clipboard and Unicode handling.

### Manual desktop tests

- Launch and close on Ubuntu 26.04.
- Switch between workspaces during active terminal output.
- Resize and maximize windows.
- Exercise splits at different dimensions.
- Test NVIDIA rendering and investigate flickering.
- Verify Wayland behavior.
- Test font scaling and different display resolutions.

### Regression tests

Every fix for a crash, flicker, stuck UI, or terminal corruption issue SHOULD have a reproducible test case or documented manual regression scenario.

---

## 10. Development Roadmap

### Phase 0 — Technical feasibility

**Deliverable:** A minimal native terminal prototype.

- Create a Rust desktop application using GTK4.
- Evaluate libghostty integration.
- Validate PTY lifecycle and rendering.
- Test on Ubuntu 26.04 with the target NVIDIA GPU.
- Compare VTE as a fallback.
- Measure startup, memory use, and terminal responsiveness.

**Exit criteria:** A reliable interactive terminal works without a webview, and the selected engine has a feasible integration path.

### Phase 1 — Core terminal

**Deliverable:** A usable single-window terminal.

- Shell startup.
- Terminal input and output.
- Copy/paste and scrollback.
- Font configuration.
- Application preferences.
- Graceful shell termination.

### Phase 2 — Workspace management

**Deliverable:** Multi-workspace development terminal.

- Sidebar and workspace switching.
- Tabs.
- Horizontal and vertical splits.
- Pane focus and resizing.
- Persistent layout and startup directories.

### Phase 3 — Developer experience

**Deliverable:** Daily-driver MVP.

- Keyboard shortcut customization.
- Git indicators.
- Session exit notifications.
- Startup and responsiveness optimization.
- Configuration recovery and regression testing.

### Phase 4 — AI agent workflows

**Deliverable:** Agent-aware terminal.

- Configurable agent launch profiles.
- Session identification.
- Process and completion notifications.
- Optional agent-specific integrations.
- Workspace templates.

---

## 11. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| libghostty integration is more complex than expected | Validate integration in Phase 0; retain VTE fallback |
| Rendering flicker on NVIDIA | Test rendering early; isolate renderer-specific issues |
| Terminal protocol incompatibility | Use a mature engine and test real-world TUI applications |
| Workspace recovery is confused with process recovery | Model process lifetime separately from saved layout |
| High CPU use with many panes | Benchmark concurrent sessions and avoid unnecessary redraws |
| Fragile AI-agent detection | Prefer explicit integrations and configurable hooks |
| GTK UI becomes difficult to maintain | Separate domain logic from UI and terminal-engine integration |
| Packaging introduces environment-specific issues | Test native packages and Flatpak independently |

---

## 12. MVP Definition of Done

The MVP is ready for release when:

1. It runs as a native Ubuntu desktop application.
2. A user can launch an interactive shell and use common terminal applications.
3. Workspaces, tabs, and nested split panes function correctly.
4. Keyboard navigation and copy/paste work reliably.
5. Workspace layout and preferences survive application restarts.
6. Shell termination is handled without crashing the application.
7. Core regression tests pass.
8. No known reproducible rendering or input defect remains in the supported test configurations.
9. The application does not require an account, cloud service, or AI provider.
10. Installation and troubleshooting instructions are documented.

---

## 13. Success Metrics

The following are proposed initial goals, to be evaluated during development:

- Zero known reproducible crashes in the core regression suite.
- At least 20 concurrent terminal panes tested successfully on the target development machine.
- No UI freeze during ordinary terminal workloads.
- Workspace switching within the defined performance budget.
- Successful layout recovery across repeated application restarts.
- Successful operation with at least two different coding-agent CLI tools.
- No mandatory network dependency for normal terminal operation.

---

## 14. Open Technical Decisions

These decisions should be resolved during Phase 0:

1. **Terminal engine:** libghostty versus VTE.
2. **Rendering:** Which renderer and acceleration modes are stable on the target NVIDIA setup?
3. **PTY ownership:** Whether the chosen terminal engine owns PTY creation or the application manages it.
4. **Session persistence:** Layout restoration only for MVP, with persistent background sessions deferred.
5. **Packaging:** Native `.deb`, Flatpak, or both.
6. **Agent detection:** Process metadata and explicit integrations versus optional shell hooks.
7. **Project identity:** Product name, application ID, icon, and repository structure.

## Final Recommendation

Build a Rust application with GTK4/libadwaita and validate libghostty before implementing the workspace UI. Treat terminal correctness and rendering stability as the foundation; then add tabs, split panes, persistence, and agent-aware workflows incrementally.

The first engineering milestone should be a single native window that launches a shell, renders correctly, accepts keyboard input, and exits cleanly on Ubuntu 26.04. Once that foundation is proven, the rest of the product can be developed as independent, testable modules.
