//! Color themes for rmux terminal emulator.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorTheme {
    pub name: String,
    pub foreground: String,
    pub background: String,
    pub palette: Vec<String>,
}

pub fn get_available_themes() -> Vec<ColorTheme> {
    vec![
        ColorTheme {
            name: "rmux Dark".to_string(),
            foreground: "#e4e4e7".to_string(),
            background: "#09090b".to_string(),
            palette: vec![
                "#18181b".into(), "#ef4444".into(), "#22c55e".into(), "#eab308".into(),
                "#3b82f6".into(), "#a855f7".into(), "#06b6d4".into(), "#f4f4f5".into(),
                "#71717a".into(), "#f87171".into(), "#4ade80".into(), "#facc15".into(),
                "#60a5fa".into(), "#c084fc".into(), "#38bdf8".into(), "#ffffff".into(),
            ],
        },
        ColorTheme {
            name: "Catppuccin Mocha".to_string(),
            foreground: "#cdd6f4".to_string(),
            background: "#1e1e2e".to_string(),
            palette: vec![
                "#45475a".into(), "#f38ba8".into(), "#a6e3a1".into(), "#f9e2af".into(),
                "#89b4fa".into(), "#f5c2e7".into(), "#94e2d5".into(), "#bac2de".into(),
                "#585b70".into(), "#f38ba8".into(), "#a6e3a1".into(), "#f9e2af".into(),
                "#89b4fa".into(), "#f5c2e7".into(), "#94e2d5".into(), "#a6adc8".into(),
            ],
        },
        ColorTheme {
            name: "Tokyo Night".to_string(),
            foreground: "#c0caf5".to_string(),
            background: "#1a1b26".to_string(),
            palette: vec![
                "#15161e".into(), "#f7768e".into(), "#9ece6a".into(), "#e0af68".into(),
                "#7aa2f7".into(), "#bb9af7".into(), "#7dcfff".into(), "#a9b1d6".into(),
                "#414868".into(), "#f7768e".into(), "#9ece6a".into(), "#e0af68".into(),
                "#7aa2f7".into(), "#bb9af7".into(), "#7dcfff".into(), "#c0caf5".into(),
            ],
        },
        ColorTheme {
            name: "Dracula".to_string(),
            foreground: "#f8f8f2".to_string(),
            background: "#282a36".to_string(),
            palette: vec![
                "#21222c".into(), "#ff5555".into(), "#50fa7b".into(), "#f1fa8c".into(),
                "#bd93f9".into(), "#ff79c6".into(), "#8be9fd".into(), "#f8f8f2".into(),
                "#6272a4".into(), "#ff6e6e".into(), "#69ff94".into(), "#ffffa5".into(),
                "#d6acff".into(), "#ff92df".into(), "#a4ffff".into(), "#ffffff".into(),
            ],
        },
        ColorTheme {
            name: "Nord".to_string(),
            foreground: "#d8dee9".to_string(),
            background: "#2e3440".to_string(),
            palette: vec![
                "#3b4252".into(), "#bf616a".into(), "#a3be8c".into(), "#ebcb8b".into(),
                "#81a1c1".into(), "#b48ead".into(), "#88c0d0".into(), "#e5e9f0".into(),
                "#4c566a".into(), "#bf616a".into(), "#a3be8c".into(), "#ebcb8b".into(),
                "#81a1c1".into(), "#b48ead".into(), "#8fbcbb".into(), "#eceff4".into(),
            ],
        },
        ColorTheme {
            name: "Gruvbox Dark".to_string(),
            foreground: "#ebdbb2".to_string(),
            background: "#282828".to_string(),
            palette: vec![
                "#282828".into(), "#cc241d".into(), "#98971a".into(), "#d79921".into(),
                "#458588".into(), "#b16286".into(), "#689d6a".into(), "#a89984".into(),
                "#928374".into(), "#fb4934".into(), "#b8bb26".into(), "#fabd2f".into(),
                "#83a598".into(), "#d3869b".into(), "#8ec07c".into(), "#ebdbb2".into(),
            ],
        },
        ColorTheme {
            name: "One Dark".to_string(),
            foreground: "#abb2bf".to_string(),
            background: "#21252b".to_string(),
            palette: vec![
                "#1e2127".into(), "#e06c75".into(), "#98c379".into(), "#d19a66".into(),
                "#61afef".into(), "#c678dd".into(), "#56b6c2".into(), "#abb2bf".into(),
                "#5c6370".into(), "#e06c75".into(), "#98c379".into(), "#d19a66".into(),
                "#61afef".into(), "#c678dd".into(), "#56b6c2".into(), "#ffffff".into(),
            ],
        },
    ]
}

pub fn find_theme(name: &str) -> ColorTheme {
    get_available_themes()
        .into_iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| get_available_themes()[0].clone())
}

/// UI chrome colors derived from a terminal theme, VSCode-style:
/// workbench (sidebar, header, cards, borders) stays coherent with
/// the terminal background instead of using fixed hardcoded grays.
#[derive(Debug, Clone)]
pub struct UiChrome {
    pub base: String,
    pub foreground: String,
    pub muted: String,
    pub sidebar_bg: String,
    pub header_bg: String,
    pub card_bg: String,
    pub hover_bg: String,
    pub border: String,
    pub border_focus: String,
    pub input_bg: String,
}

fn parse_hex(hex: &str) -> (f32, f32, f32) {
    let h = hex.trim_start_matches('#');
    if h.len() < 6 {
        return (0.0, 0.0, 0.0);
    }
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0) as f32 / 255.0;
    (r, g, b)
}

fn to_hex(r: f32, g: f32, b: f32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8
    )
}

fn mix(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

impl ColorTheme {
    pub fn chrome(&self) -> UiChrome {
        let bg = parse_hex(&self.background);
        let fg = parse_hex(&self.foreground);
        let white = (1.0, 1.0, 1.0);
        let black = (0.0, 0.0, 0.0);
        let rgb = |c: (f32, f32, f32)| to_hex(c.0, c.1, c.2);

        UiChrome {
            base: self.background.clone(),
            foreground: self.foreground.clone(),
            muted: rgb(mix(fg, bg, 0.45)),
            sidebar_bg: rgb(mix(bg, black, 0.28)),
            header_bg: rgb(mix(bg, white, 0.035)),
            card_bg: rgb(mix(bg, white, 0.045)),
            hover_bg: rgb(mix(bg, white, 0.075)),
            border: rgb(mix(bg, white, 0.11)),
            border_focus: rgb(mix(bg, white, 0.20)),
            input_bg: rgb(mix(bg, white, 0.07)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_themes_list() {
        let themes = get_available_themes();
        assert!(themes.len() >= 5);
        let found = find_theme("Catppuccin Mocha");
        assert_eq!(found.name, "Catppuccin Mocha");
    }
}
