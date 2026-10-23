//! Configuration management with JSON persistence.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub font_family: String,
    pub font_size: u32,
    pub scrollback_lines: i64,
    pub default_shell: String,
    pub theme: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_family: String::from("Monospace"),
            font_size: 11,
            scrollback_lines: 10000,
            default_shell: std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash")),
            theme: String::from("rmux Dark"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: String, // "rmux Dark", "Dark 2026", "Codex", etc.
    pub sidebar_visible: bool,
    pub sidebar_width: i32,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: String::from("rmux Dark"),
            sidebar_visible: true,
            sidebar_width: 240,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    pub new_workspace: String,
    pub close_pane: String,
    pub split_right: String,
    pub split_down: String,
    #[serde(default = "default_focus_next")]
    pub focus_next_pane: String,
    #[serde(default = "default_focus_prev")]
    pub focus_prev_pane: String,
    pub toggle_sidebar: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub zoom_reset: String,
}

fn default_focus_next() -> String {
    String::from("Ctrl+Tab")
}

fn default_focus_prev() -> String {
    String::from("Ctrl+Shift+Tab")
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            new_workspace: String::from("Ctrl+Shift+N"),
            close_pane: String::from("Ctrl+W"),
            split_right: String::from("Ctrl+Shift+T"),
            split_down: String::from("Ctrl+Shift+D"),
            focus_next_pane: default_focus_next(),
            focus_prev_pane: default_focus_prev(),
            toggle_sidebar: String::from("Ctrl+B"),
            zoom_in: String::from("Ctrl+Plus"),
            zoom_out: String::from("Ctrl+Minus"),
            zoom_reset: String::from("Ctrl+0"),
        }
    }
}

pub fn matches_key(shortcut_str: &str, keyval: gtk4::gdk::Key, state_flags: gtk4::gdk::ModifierType) -> bool {
    let lower_key = keyval.to_lower();
    let parts: Vec<&str> = shortcut_str.split('+').map(|s| s.trim()).collect();
    let mut req_ctrl = false;
    let mut req_shift = false;
    let mut req_alt = false;
    let mut target_key = "";

    for part in parts {
        if part.eq_ignore_ascii_case("ctrl") || part.eq_ignore_ascii_case("control") {
            req_ctrl = true;
        } else if part.eq_ignore_ascii_case("shift") {
            req_shift = true;
        } else if part.eq_ignore_ascii_case("alt") {
            req_alt = true;
        } else {
            target_key = part;
        }
    }

    let has_ctrl = state_flags.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
    let has_shift = state_flags.contains(gtk4::gdk::ModifierType::SHIFT_MASK);
    let has_alt = state_flags.contains(gtk4::gdk::ModifierType::ALT_MASK);

    if req_ctrl != has_ctrl || req_shift != has_shift || req_alt != has_alt {
        return false;
    }

    match target_key.to_lowercase().as_str() {
        "t" => lower_key == gtk4::gdk::Key::t,
        "n" => lower_key == gtk4::gdk::Key::n,
        "w" => lower_key == gtk4::gdk::Key::w,
        "r" => lower_key == gtk4::gdk::Key::r,
        "d" => lower_key == gtk4::gdk::Key::d,
        "e" => lower_key == gtk4::gdk::Key::e,
        "b" => lower_key == gtk4::gdk::Key::b,
        "plus" | "=" => lower_key == gtk4::gdk::Key::plus || lower_key == gtk4::gdk::Key::equal,
        "minus" | "-" => lower_key == gtk4::gdk::Key::minus,
        "0" => lower_key == gtk4::gdk::Key::_0,
        "tab" => {
            lower_key == gtk4::gdk::Key::Tab
                || lower_key == gtk4::gdk::Key::KP_Tab
                || lower_key == gtk4::gdk::Key::ISO_Left_Tab
        }
        other => {
            if let Some(k) = gtk4::gdk::Key::from_name(other) {
                return lower_key == k.to_lower();
            }
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub terminal: TerminalConfig,
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub shortcuts: ShortcutsConfig,
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let base = dirs_next_config().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("rmux");
        let _ = fs::create_dir_all(&dir);
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    #[allow(dead_code)]
    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("tmp");
            if fs::write(&tmp, json).is_ok() {
                let _ = fs::rename(tmp, path);
            }
        }
    }
}

fn dirs_next_config() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.terminal.font_family, "Monospace");
        assert_eq!(config.terminal.font_size, 11);
        assert_eq!(config.appearance.theme, "rmux Dark");
    }

    #[test]
    fn test_config_serialization() {
        let mut config = AppConfig::default();
        config.terminal.font_size = 14;
        let json = serde_json::to_string(&config).unwrap();
        let recovered: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.terminal.font_size, 14);
    }

    #[test]
    fn test_focus_shortcut_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.shortcuts.focus_next_pane, "Ctrl+Tab");
        assert_eq!(config.shortcuts.focus_prev_pane, "Ctrl+Shift+Tab");
    }

    #[test]
    fn test_old_config_without_focus_shortcuts_still_loads() {
        let json = r#"{
            "terminal": {"font_family": "Monospace", "font_size": 11, "scrollback_lines": 10000, "default_shell": "/bin/bash", "theme": "rmux Dark"},
            "appearance": {"theme": "rmux Dark", "sidebar_visible": true, "sidebar_width": 240},
            "shortcuts": {"new_workspace": "Ctrl+Shift+N", "close_pane": "Ctrl+W", "split_right": "Ctrl+Shift+T", "split_down": "Ctrl+Shift+D", "toggle_sidebar": "Ctrl+B", "zoom_in": "Ctrl+Plus", "zoom_out": "Ctrl+Minus", "zoom_reset": "Ctrl+0"}
        }"#;
        let recovered: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(recovered.shortcuts.focus_next_pane, "Ctrl+Tab");
        assert_eq!(recovered.shortcuts.focus_prev_pane, "Ctrl+Shift+Tab");
    }

    #[test]
    fn test_tab_key_matches() {
        use gtk4::gdk::ModifierType;
        assert!(matches_key(
            "Ctrl+Tab",
            gtk4::gdk::Key::Tab,
            ModifierType::CONTROL_MASK
        ));
        assert!(!matches_key(
            "Ctrl+Tab",
            gtk4::gdk::Key::Tab,
            ModifierType::empty()
        ));
        assert!(matches_key(
            "Ctrl+Shift+Tab",
            gtk4::gdk::Key::ISO_Left_Tab,
            ModifierType::CONTROL_MASK | ModifierType::SHIFT_MASK
        ));
    }
}
