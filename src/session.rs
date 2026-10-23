//! Session persistence: layout-only snapshot of workspaces + split panes.
//!
//! Stores `~/.config/rmux/session.json` (XDG-compliant, same dir as
//! `config.json` but a separate file so a corrupt session never wipes
//! user prefs). Layout-only: each leaf records a working directory;
//! `command` is always forced to `None` on save and ignored on load
//! (PRD §7 forbids auto-executing restored commands). No scrollback,
//! no env, no history — fresh shells on restore.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::model::LayoutNode;

pub const SESSION_VERSION: u32 = 1;
pub const MAX_TABS: usize = 20;
pub const MAX_LEAVES: usize = 16;
const MAX_TITLE_LEN: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTab {
    pub id: String,
    pub title: String,
    pub root_layout: LayoutNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub version: u32,
    pub active_tab_id: String,
    pub tabs: Vec<SavedTab>,
}

pub struct SessionStore;

impl SessionStore {
    pub fn session_path() -> PathBuf {
        let base = config_base_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("rmux");
        let _ = fs::create_dir_all(&dir);
        dir.join("session.json")
    }

    /// Load and validate the session file. Returns `None` on any
    /// failure (missing/corrupt/empty) so the caller falls back to a
    /// single default workspace without crashing.
    pub fn load() -> Option<SessionFile> {
        Self::load_from(&Self::session_path())
    }

    pub fn load_from(path: &Path) -> Option<SessionFile> {
        let content = fs::read_to_string(path).ok()?;
        if content.trim().is_empty() {
            return None;
        }
        let parsed: SessionFile = serde_json::from_str(&content).ok()?;
        Self::validate(parsed)
    }

    /// Validate + sanitize a parsed session. Enforces caps, path
    /// safety, and `command == None` everywhere.
    pub fn validate(mut file: SessionFile) -> Option<SessionFile> {
        if file.version != SESSION_VERSION {
            return None;
        }
        if file.tabs.is_empty() {
            return None;
        }
        // Cap tab count (keep first N).
        if file.tabs.len() > MAX_TABS {
            file.tabs.truncate(MAX_TABS);
        }
        let mut clean: Vec<SavedTab> = Vec::with_capacity(file.tabs.len());
        for (idx, tab) in file.tabs.into_iter().enumerate() {
            let id = if tab.id.trim().is_empty() {
                Uuid::new_v4().to_string()
            } else {
                tab.id
            };
            let title = sanitize_title(&tab.title, idx);
            let mut budget = MAX_LEAVES;
            let layout = match sanitize_node(&tab.root_layout, &mut budget) {
                Some(l) => l,
                None => continue, // oversized/degenerate tab: drop it
            };
            clean.push(SavedTab {
                id,
                title,
                root_layout: layout,
            });
        }
        if clean.is_empty() {
            return None;
        }
        // Active tab must exist; else fall back to first.
        if !clean.iter().any(|t| t.id == file.active_tab_id) {
            file.active_tab_id = clean[0].id.clone();
        }
        file.tabs = clean;
        Some(file)
    }

    /// Persist a snapshot atomically (tmp file + rename).
    pub fn save(tabs: &[SavedTab], active_tab_id: &str) {
        Self::save_to(&Self::session_path(), tabs, active_tab_id);
    }

    /// Testable save with explicit path. Returns false on IO/ser error.
    pub fn save_to(path: &Path, tabs: &[SavedTab], active_tab_id: &str) -> bool {
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return false;
            }
        }
        let mut capped: Vec<SavedTab> = tabs
            .iter()
            .take(MAX_TABS)
            .enumerate()
            .filter_map(|(idx, t)| {
                let id = if t.id.trim().is_empty() {
                    Uuid::new_v4().to_string()
                } else {
                    t.id.clone()
                };
                let title = sanitize_title(&t.title, idx);
                let mut budget = MAX_LEAVES;
                sanitize_node(&t.root_layout, &mut budget).map(|layout| SavedTab {
                    id,
                    title,
                    root_layout: layout,
                })
            })
            .collect();
        if capped.is_empty() {
            return false;
        }
        let mut active = active_tab_id.to_string();
        if !capped.iter().any(|t| t.id == active) {
            active = capped[0].id.clone();
        }
        // Keep borrow checker happy if caller passed active id of a
        // dropped tab; `capped` itself is authoritative.
        let _ = &mut capped;
        let file = SessionFile {
            version: SESSION_VERSION,
            active_tab_id: active,
            tabs: capped,
        };
        let json = match serde_json::to_string_pretty(&file) {
            Ok(j) => j,
            Err(_) => return false,
        };
        let tmp = path.with_extension("tmp");
        if fs::write(&tmp, json).is_err() {
            return false;
        }
        fs::rename(&tmp, path).is_ok()
    }
}

/// Resolve a saved cwd safely: keep it if it exists and is a dir,
/// else `$HOME` if that is a dir, else `/`. Never returns a
/// non-existent path.
pub fn resolve_working_dir(raw: &str) -> String {
    let p = Path::new(raw);
    if !raw.is_empty() {
        if let Ok(md) = fs::metadata(p) {
            if md.is_dir() {
                return raw.to_string();
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            if let Ok(md) = fs::metadata(&home) {
                if md.is_dir() {
                    return home;
                }
            }
        }
    }
    String::from("/")
}

/// Recursively sanitize a layout: validate dirs, force `command=None`,
/// enforce the leaf budget. Returns `None` if the subtree has no
/// usable leaf left.
fn sanitize_node(node: &LayoutNode, budget: &mut usize) -> Option<LayoutNode> {
    match node {
        LayoutNode::Leaf {
            working_directory,
            ..
        } => {
            if *budget == 0 {
                return None;
            }
            *budget -= 1;
            Some(LayoutNode::Leaf {
                pane_id: Uuid::new_v4().to_string(),
                working_directory: resolve_working_dir(working_directory),
                command: None,
            })
        }
        LayoutNode::Split {
            orientation,
            ratio,
            first,
            second,
        } => {
            let a = sanitize_node(first, budget);
            let b = sanitize_node(second, budget);
            match (a, b) {
                (Some(f), Some(s)) => Some(LayoutNode::Split {
                    orientation: *orientation,
                    ratio: clamp_ratio(*ratio),
                    first: Box::new(f),
                    second: Box::new(s),
                }),
                // Degenerate split (one side pruned): collapse to survivor.
                (Some(f), None) => Some(f),
                (None, Some(s)) => Some(s),
                (None, None) => None,
            }
        }
    }
}

/// Count leaves in a layout (for cap enforcement / tests).
#[allow(dead_code)]
pub fn count_leaves(node: &LayoutNode) -> usize {
    match node {
        LayoutNode::Leaf { .. } => 1,
        LayoutNode::Split { first, second, .. } => count_leaves(first) + count_leaves(second),
    }
}

pub fn clamp_ratio(r: f64) -> f64 {
    if !r.is_finite() {
        return 0.5;
    }
    r.clamp(0.1, 0.9)
}

fn sanitize_title(raw: &str, idx: usize) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return format!("Workspace {}", idx + 1);
    }
    let mut s: String = t.chars().take(MAX_TITLE_LEN).collect();
    s = s.trim().to_string();
    if s.is_empty() {
        format!("Workspace {}", idx + 1)
    } else {
        s
    }
}

fn config_base_dir() -> Option<PathBuf> {
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
    use crate::model::Orientation;

    fn leaf(dir: &str) -> LayoutNode {
        LayoutNode::Leaf {
            pane_id: Uuid::new_v4().to_string(),
            working_directory: dir.to_string(),
            command: None,
        }
    }

    fn tmp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("rmux_session_test_{}_{}", std::process::id(), name));
        let _ = fs::create_dir_all(&p);
        p.join("session.json")
    }

    fn sample_tabs() -> Vec<SavedTab> {
        vec![
            SavedTab {
                id: "tab-1".into(),
                title: "Workspace 1".into(),
                root_layout: leaf("/tmp"),
            },
            SavedTab {
                id: "tab-2".into(),
                title: "Workspace 2".into(),
                root_layout: LayoutNode::Split {
                    orientation: Orientation::Vertical,
                    ratio: 0.5,
                    first: Box::new(leaf("/tmp")),
                    second: Box::new(leaf("/tmp")),
                },
            },
        ]
    }

    #[test]
    fn test_round_trip() {
        let path = tmp_path("roundtrip");
        let tabs = sample_tabs();
        assert!(SessionStore::save_to(&path, &tabs, "tab-2"));
        let loaded = SessionStore::load_from(&path).expect("should load");
        assert_eq!(loaded.version, SESSION_VERSION);
        assert_eq!(loaded.tabs.len(), 2);
        assert_eq!(loaded.active_tab_id, "tab-2");
        assert_eq!(loaded.tabs[0].title, "Workspace 1");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_corrupt_falls_back_to_none() {
        let path = tmp_path("corrupt");
        fs::write(&path, "{ not valid json!!!").unwrap();
        assert!(SessionStore::load_from(&path).is_none());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_missing_file_returns_none() {
        let path = tmp_path("missing");
        let _ = fs::remove_file(&path.join("nope.json"));
        assert!(SessionStore::load_from(&path.join("nope.json")).is_none());
    }

    #[test]
    fn test_empty_tabs_returns_none() {
        let path = tmp_path("empty");
        let file = SessionFile {
            version: SESSION_VERSION,
            active_tab_id: "x".into(),
            tabs: vec![],
        };
        fs::write(&path, serde_json::to_string(&file).unwrap()).unwrap();
        assert!(SessionStore::load_from(&path).is_none());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_command_never_restored() {
        let path = tmp_path("cmd");
        // Hand-craft JSON with a command payload (simulating tampering).
        let raw = r#"{"version":1,"active_tab_id":"t1","tabs":[{"id":"t1","title":"W1","root_layout":{"type":"leaf","pane_id":"p1","working_directory":"/tmp","command":"rm -rf ~"}}]}"#;
        fs::write(&path, raw).unwrap();
        let loaded = SessionStore::load_from(&path).expect("loads");
        match &loaded.tabs[0].root_layout {
            LayoutNode::Leaf { command, .. } => assert!(command.is_none()),
            _ => panic!("expected leaf"),
        }
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_tab_cap_enforced() {
        let path = tmp_path("tabcap");
        let tabs: Vec<SavedTab> = (0..30)
            .map(|i| SavedTab {
                id: format!("t{}", i),
                title: format!("W{}", i),
                root_layout: leaf("/tmp"),
            })
            .collect();
        assert!(SessionStore::save_to(&path, &tabs, "t0"));
        let loaded = SessionStore::load_from(&path).unwrap();
        assert_eq!(loaded.tabs.len(), MAX_TABS);
        let _ = fs::remove_file(&path);
    }

    fn deep_chain(n: usize) -> LayoutNode {
        let mut node = leaf("/tmp");
        for _ in 1..n {
            node = LayoutNode::Split {
                orientation: Orientation::Vertical,
                ratio: 0.5,
                first: Box::new(node),
                second: Box::new(leaf("/tmp")),
            };
        }
        node
    }

    #[test]
    fn test_leaf_cap_enforced() {
        // 25 leaves chained; sanitize must prune to MAX_LEAVES.
        let node = deep_chain(25);
        assert_eq!(count_leaves(&node), 25);
        let mut budget = MAX_LEAVES;
        let clean = sanitize_node(&node, &mut budget).unwrap();
        assert!(count_leaves(&clean) <= MAX_LEAVES);
    }

    #[test]
    fn test_bad_cwd_falls_back() {
        let got = resolve_working_dir("/nonexistent_rmux_dir_xyz_123");
        assert_ne!(got, "/nonexistent_rmux_dir_xyz_123");
        // Must be an existing dir.
        assert!(fs::metadata(&got).map(|m| m.is_dir()).unwrap_or(false));
    }

    #[test]
    fn test_bad_version_rejected() {
        let path = tmp_path("version");
        let raw = r#"{"version":999,"active_tab_id":"t1","tabs":[{"id":"t1","title":"W1","root_layout":{"type":"leaf","pane_id":"p1","working_directory":"/tmp","command":null}}]}"#;
        fs::write(&path, raw).unwrap();
        assert!(SessionStore::load_from(&path).is_none());
        let _ = fs::remove_file(&path);
    }
}
