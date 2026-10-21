<div align="center">

# ⚡ rmux

**Native Linux Terminal Emulator for AI-Assisted Developers & Power Users**  
*Built in 100% Pure Rust for Ubuntu 26.04 LTS using GTK4, libadwaita, and VTE*

[![Rust](https://img.shields.io/badge/Language-Rust%202021-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GUI-GTK4%20%2B%20libadwaita-4f8df7.svg?style=for-the-badge&logo=gnome)](https://gtk.org/)
[![Platform](https://img.shields.io/badge/Platform-Ubuntu%20%2F%20Linux-E95420.svg?style=for-the-badge&logo=ubuntu)](https://ubuntu.com/)
[![License](https://img.shields.io/badge/License-MIT-10b981.svg?style=for-the-badge)](LICENSE)

<br/>

> **rmux** combines high-performance native terminal emulation with vertical workspace organization and a minimalist Google Antigravity-inspired aesthetic. Unlike webview- or Electron-based multiplexers, rmux compiles directly into a standalone native binary with sub-millisecond input response, zero telemetry, and minimal memory footprint.

</div>

---

## 🎨 Visual Philosophy: Minimalist Dark UI

rmux is crafted with a distraction-free aesthetic inspired by the **Google Antigravity** interface:

- **Clean Typography & Matte Backdrops**: Deep obsidian and carbon backgrounds (`#0c0d10`, `#111215`, `#131418`) that minimize eye fatigue during extended coding sessions.
- **Uncluttered Header Bar**: No redundant split icons or crowded top tab strips. The top bar features only workspace navigation, clickable rename pill, and native window controls.
- **Subtle 1px Borders**: No neon cyan borders. Active panes are highlighted with a restrained, modern border (`#2e313d`).
- **Sidebar Settings Placement**: Accessible directly from the bottom-left of the workspaces sidebar, just like modern developer IDEs.

```
┌────────────────────────────────────────────────────────────────────────┐
│ [▤] [+]                [ Workspace 1 ▾ ]                      [—] [□] [✕]│
├──────────────┬─────────────────────────────────────────────────────────┤
│ rmux     [+] │ 🟢 workspace                                         [✕] │
│              │ ─────────────────────────────────────────────────────── │
│ 🟢 Worksp..1 │ $ cargo test                                            │
│    ~/project │                                                         │
│              │ running 5 tests                                         │
│ ⚪ Worksp..2 │ test config::tests::test_default_config ... ok          │
│    ~/backend │ test model::tests::test_workspace_serialization ... ok  │
│              │ test result: ok. 5 passed; 0 failed                     │
│              │                                                         │
│              │                                                         │
│ ──────────── │                                                         │
│ ⚙ Settings   │                                                         │
├──────────────┴─────────────────────────────────────────────────────────┤
│ ~/project_terminal2                                             1 pane │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 🌟 Key Features

### 📑 Vertical Workspaces Sidebar
Organize concurrent tasks into named workspaces. Each workspace retains its own split layout, active working directory, and PTY processes.
- **Quick Switch**: Instant zero-lag workspace switching via sidebar click or `Ctrl+B`.
- **In-Place Renaming**: Double-click any workspace row or click the active workspace pill in the top header bar to rename it.
- **Context Actions**: Right-click any workspace row to open contextual options (*Rename Workspace...*, *Close Workspace*).

### 🪟 Arbitrary Nested Split Panes
Split your active workspace horizontally or vertically without being confined to a rigid 2-pane grid.
- **Split Right (Side-by-Side)**: `Ctrl + Shift + R`
- **Split Down (Top-and-Bottom)**: `Ctrl + Shift + D`
- **Close Focused Pane**: `Ctrl + W` (closes only the active split pane without terminating other panes).
- **Close All Workspaces**: `Ctrl + Shift + W` (cleanly exits all open workspaces and resets to a clean session).

### ⚙️ Antigravity 2-Column Settings Modal
Click **⚙ Settings** at the bottom-left of the sidebar to summon an overlay modal dialog:
- **Left Column Navigation**: Categories for *Appearance & Themes*, *Keyboard Shortcuts*, *Terminal & Shell*, and *About*.
- **Live Theme Presets**: Pick from 7 popular developer color palettes. Themes apply across all open panes dynamically with zero restart.
- **Interactive Shortcut Editor**: Click **Edit** next to any shortcut, press your desired key combination, and rmux immediately captures, validates, and persists the binding.
- **Typography & Environment**: Configure custom monospace fonts, point sizes, and your preferred interactive shell (`/bin/bash`, `/bin/zsh`, `/bin/fish`).

---

## 🎨 Built-in Color Themes

| Theme Name | Background | Foreground | Description |
|---|---|---|---|
| **rmux Dark** (Default) | `#09090b` | `#e4e4e7` | Precision balanced high-contrast zinc palette |
| **Catppuccin Mocha** | `#1e1e2e` | `#cdd6f4` | Soothing pastel theme for night owls |
| **Tokyo Night** | `#1a1b26` | `#c0caf5` | Celebrated neon-dark Tokyo cityscape palette |
| **Dracula** | `#282a36` | `#f8f8f2` | Classic vibrant Gothic dark theme |
| **Nord** | `#2e3440` | `#d8dee9` | Arctic, north-bluish clean color palette |
| **Gruvbox Dark** | `#282828` | `#ebdbb2` | Retro groove warm autumnal contrast |
| **One Dark** | `#21252b` | `#abb2bf` | Iconic Atom & VS Code dark theme |

---

## ⌨️ Default Keyboard Shortcuts

| Key Binding | Action | Description |
|---|---|---|
| `Ctrl + Shift + T` | **New Workspace** | Creates a fresh workspace tab |
| `Ctrl + Shift + W` | **Close All Open Tabs** | Closes all open workspaces and resets to clean session |
| `Ctrl + W` | **Close Focused Pane** | Closes the current terminal split pane |
| `Ctrl + Shift + R` | **Split Right** | Splits focused pane vertically (side-by-side) |
| `Ctrl + Shift + D` | **Split Down** | Splits focused pane horizontally (top-and-bottom) |
| `Ctrl + B` | **Toggle Sidebar** | Hides or shows the left workspaces sidebar |
| `Ctrl + Plus` / `Ctrl + =` | **Zoom In** | Increases terminal font scale |
| `Ctrl + Minus` | **Zoom Out** | Decreases terminal font scale |
| `Ctrl + 0` | **Reset Zoom** | Restores terminal font size to default |
| `Ctrl + Shift + C` | **Copy** | Copies selected terminal text to clipboard |
| `Ctrl + Shift + V` | **Paste** | Pastes text from clipboard into terminal |

> *All shortcuts are fully customizable via the built-in **Shortcuts Editor** in Settings (`~/.config/rmux/config.json`).*

---

## 🛠️ Architecture & Tech Stack

```
                        ┌───────────────────────────────┐
                        │        rmux (Rust Bin)        │
                        └───────────────┬───────────────┘
                                        │
           ┌────────────────────────────┼────────────────────────────┐
           ▼                            ▼                            ▼
┌─────────────────────┐      ┌─────────────────────┐      ┌─────────────────────┐
│  GTK4 & libadwaita  │      │     vte-sys / VTE   │      │    Serde & Config   │
│  - Application      │      │  - PTY Spawning     │      │  - Atomic JSON I/O  │
│  - HeaderBar        │      │  - ANSI 24-bit True │      │  - Dynamic Themes   │
│  - Custom CSS Theme │      │  - Font Description │      │  - Rebindable Keys  │
│  - Event Capture    │      │  - POSIX Signals    │      │  - XDG Compliance   │
└─────────────────────┘      └─────────────────────┘      └─────────────────────┘
```

- **Compiled Native Binary**: 100% Rust code. Zero Electron, zero webviews, zero Python runtimes.
- **Capture-Phase Keyboard Pipeline**: Window-level event capture ensures global shortcuts (`Ctrl+Shift+R`, `Ctrl+W`) are intercepted reliably without being swallowed by inner shell processes (e.g. `bash` reverse-i-search).
- **Safe Reparenting Engine**: Paned hierarchies unparent widgets safely before reparenting, avoiding memory leaks or GTK critical assertions.

---

## 🚀 Building & Installation

### System Requirements (Ubuntu 26.04 / Modern Linux)

Install the required GTK4, libadwaita, and VTE development headers:

```bash
sudo apt update
sudo apt install -y build-essential libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev
```

### 1. Compile & Run

```bash
# Debug run
cargo run

# Optimized release run
cargo run --release
```

### 2. Run Test Suite

```bash
cargo test
```
*All core models, configuration serialization, themes, and workspace layouts are covered with automated unit tests.*

### 3. Install for Current User

```bash
make install-user
```

This installs:
- Binary: `~/.local/bin/rmux` (with a backwards-compatible `cmux-native` symlink)
- Desktop Entry: `~/.local/share/applications/rmux.desktop`
- Application Icon: `~/.local/share/icons/hicolor/scalable/apps/rmux.svg`

You can then launch **rmux** from your terminal or search for `rmux` in the GNOME Application launcher.

---

## ⚙️ Configuration Format

Preferences are stored in `~/.config/rmux/config.json`:

```json
{
  "terminal": {
    "font_family": "Monospace",
    "font_size": 11,
    "scrollback_lines": 10000,
    "default_shell": "/bin/bash",
    "theme": "rmux Dark"
  },
  "appearance": {
    "theme": "rmux Dark",
    "sidebar_visible": true,
    "sidebar_width": 230
  },
  "shortcuts": {
    "new_workspace": "Ctrl+Shift+T",
    "close_all_tabs": "Ctrl+Shift+W",
    "close_pane": "Ctrl+W",
    "split_right": "Ctrl+Shift+R",
    "split_down": "Ctrl+Shift+D",
    "toggle_sidebar": "Ctrl+B",
    "zoom_in": "Ctrl+Plus",
    "zoom_out": "Ctrl+Minus",
    "zoom_reset": "Ctrl+0"
  }
}
```

---

## 📂 Source Code Layout

```
project_terminal2/
├── Cargo.toml            # Manifest with single rmux binary target
├── Makefile              # User installation and compilation automation
├── .gitignore            # Clean source-only Git filter
├── README.md             # Project documentation
├── bin/
│   ├── rmux              # Direct release launcher script
│   └── cmux-native       # Legacy compatibility launcher
├── data/
│   ├── rmux.desktop      # Freedesktop application entry
│   └── icons/
│       └── rmux.svg      # Scalable application icon
├── docs/
│   ├── prd.md            # Original requirements specification
│   └── cmux-guide.webp   # Design reference
└── src/
    ├── main.rs           # Application initialization and stylesheet setup
    ├── window.rs         # Root window, headerbar, and capture-phase shortcut engine
    ├── sidebar.rs        # Vertical workspaces sidebar with bottom Settings button
    ├── settings.rs       # Antigravity 2-column modal settings & shortcuts editor
    ├── pane.rs           # Individual terminal pane container & header
    ├── split.rs          # Nested Paned layout manager with safe reparenting
    ├── vte.rs            # Safe Rust abstraction over vte-sys
    ├── theme.rs          # 7 preset developer color palettes
    ├── config.rs         # Configuration schema, shortcuts parser, and persistence
    ├── model.rs          # Tab, Workspace, and Layout tree data models
    └── agent.rs          # Linux /proc inspector for background coding agents
```

---

## 📄 License

Licensed under the [MIT License](LICENSE).
