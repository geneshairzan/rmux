//! Data models for workspaces, vertical tabs, and nested split layouts.

use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LayoutNode {
    #[serde(rename = "leaf")]
    Leaf {
        pane_id: String,
        working_directory: String,
        command: Option<String>,
    },
    #[serde(rename = "split")]
    Split {
        orientation: Orientation,
        ratio: f64,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

impl Default for LayoutNode {
    fn default() -> Self {
        LayoutNode::Leaf {
            pane_id: Uuid::new_v4().to_string(),
            working_directory: env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| String::from("/")),
            command: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: String,
    pub title: String,
    pub root_layout: LayoutNode,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: String::from("Workspace 1"),
            root_layout: LayoutNode::default(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    pub starting_directory: String,
}

impl Default for Workspace {
    fn default() -> Self {
        let dir = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from("/"));
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::from("Main"),
            tabs: vec![Tab::default()],
            active_tab_index: 0,
            starting_directory: dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::default();
        assert_eq!(tab.title, "Workspace 1");
        assert!(!tab.id.is_empty());
    }

    #[test]
    fn test_workspace_serialization() {
        let ws = Workspace::default();
        let json = serde_json::to_string(&ws).unwrap();
        let recovered: Workspace = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.name, "Main");
        assert_eq!(recovered.tabs.len(), 1);
    }
}
