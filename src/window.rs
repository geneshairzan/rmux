//! Main application window uniting vertical workspaces sidebar and terminal split panes.

use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::config::{matches_key, AppConfig};
use crate::model::Tab;
use crate::sidebar::VerticalSidebar;
use crate::sidebar::SIDEBAR_MAX_WIDTH;
use crate::split::TabView;
use crate::theme::find_theme;
use crate::tray::TrayAction;

pub const CUSTOM_CSS: &str = r#"
/* ========================================================
   Antigravity Dark Minimalist Theme
   ======================================================== */
window {
    background-color: #131418;
}

/* Headerbar keeps the sidebar toggle and Settings; no title widget */
headerbar {
    background-color: #131418;
    border-bottom: 1px solid #1c1e24;
    min-height: 38px;
    padding: 0 12px;
}

/* Terminal Panes */
.terminal-pane {
    background-color: #0d0e12;
    border: none;
    border-radius: 0;
    margin: 0;
}

paned separator {
    background-color: #1a1c22;
    min-width: 1px;
    min-height: 1px;
    padding: 0;
}

.pane-header {
    background-color: #111216;
    border-bottom: 1px solid #1a1c22;
    border-radius: 0;
    min-height: 30px;
    padding: 4px 12px 4px 16px;
}

.terminal-pane.focused-pane .pane-header {
    background-color: #14151b;
}

.pane-title {
    font-size: 12px;
    font-weight: 500;
    font-family: monospace;
    color: #8c909e;
}

.terminal-pane.focused-pane .pane-title {
    color: #e5e7eb;
}

.pane-action-btn {
    min-width: 22px;
    min-height: 22px;
    padding: 3px;
    color: #6b7080;
    border-radius: 5px;
    opacity: 0.3;
    transition: opacity 120ms ease, color 120ms ease;
}

.terminal-pane:hover .pane-action-btn {
    opacity: 0.8;
}

.pane-action-btn:hover {
    color: #ffffff;
    background-color: #242732;
    opacity: 1;
}

/* Status dots */
.status-dot, .tab-status-dot {
    font-size: 8px;
}

.status-running {
    color: #10b981;
}

.status-attention {
    color: #f59e0b;
}

.status-exited {
    color: #ef4444;
}

.status-idle {
    color: #4b5162;
}

/* Sidebar has no header or footer: workspace list only. */
.workspace-sidebar {
    background-color: #111215;
    border-right: 1px solid #1c1e24;
}

.vertical-tabs-list {
    background: transparent;
    padding: 6px 0;
}

.tab-row {
    border-radius: 8px;
    margin: 3px 10px;
    transition: background-color 100ms ease;
    border: 1px solid transparent;
}

.tab-row:hover {
    background-color: #16171d;
}

.tab-row:selected {
    background-color: #1f2129;
    border: 1px solid #282a35;
}

.tab-title {
    font-weight: 600;
    font-size: 12.5px;
    color: #d1d5db;
}

.tab-row:selected .tab-title {
    color: #ffffff;
}

.tab-subtitle {
    font-size: 11px;
    color: #717585;
}

.tab-close-btn {
    min-width: 20px;
    min-height: 20px;
    padding: 2px;
    opacity: 0.15;
    transition: opacity 120ms ease, color 120ms ease;
}

.tab-row:hover .tab-close-btn,
.tab-row:selected .tab-close-btn {
    opacity: 0.6;
}

.tab-close-btn:hover {
    opacity: 1;
    color: #f43f5e;
}

/* Workspace right-click context menu */
.workspace-context-item {
    font-size: 12px;
    padding: 6px 12px;
    border-radius: 6px;
}

/* Sidebar footer: full-bleed top border, Settings row vertically centered */
.sidebar-footer {
    border-top: 1px solid #18191f;
    padding: 6px 8px 8px 8px;
}

.sidebar-settings-btn {
    border-radius: 8px;
    min-height: 32px;
}

.sidebar-settings-btn:hover {
    background-color: #1c1e26;
}

.sidebar-settings-row {
    padding: 6px 8px;
}

.sidebar-settings-icon {
    color: #8c909e;
    font-size: 14px;
}

.sidebar-settings-label {
    font-size: 12.5px;
    font-weight: 500;
    color: #8c909e;
}

.sidebar-settings-btn:hover .sidebar-settings-icon,
.sidebar-settings-btn:hover .sidebar-settings-label {
    color: #f0f1f5;
}

/* ========================================================
   Antigravity Settings Modal Styling
   ======================================================== */
.antigravity-modal {
    background-color: #131418;
    border: 1px solid #232530;
    border-radius: 12px;
}

.antigravity-settings-sidebar {
    background-color: #111215;
    border-right: 1px solid #1c1e24;
    padding: 20px 16px;
}

.antigravity-nav-btn {
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 13px;
    color: #8c909e;
    font-weight: 500;
    margin-bottom: 4px;
    transition: background-color 100ms ease;
}

.antigravity-nav-btn:hover {
    background-color: #191b22;
    color: #e5e7eb;
}

.antigravity-nav-btn.active-nav {
    background-color: #232530;
    color: #ffffff;
    font-weight: 600;
}

.antigravity-divider {
    background-color: #1c1e24;
    min-width: 1px;
    opacity: 1;
}

.antigravity-settings-content {
    background-color: #131418;
}

.antigravity-close-btn {
    min-width: 28px;
    min-height: 28px;
    color: #8c909e;
    border-radius: 8px;
}

.antigravity-close-btn:hover {
    color: #ffffff;
    background-color: #20222b;
}

.antigravity-content-title {
    font-size: 19px;
    font-weight: 700;
    color: #f0f1f5;
    letter-spacing: -0.2px;
}

.antigravity-content-desc {
    font-size: 12.5px;
    color: #7c808f;
    line-height: 1.5;
    margin-bottom: 10px;
}

.antigravity-card {
    background-color: #181920;
    border: 1px solid #232530;
    border-radius: 10px;
}

.antigravity-card-row {
    transition: background-color 100ms ease;
}

.antigravity-card-row:hover {
    background-color: #1d1e26;
}

.antigravity-card-sep {
    background-color: #20222c;
    min-height: 1px;
    opacity: 1;
}

.antigravity-row-title {
    font-size: 13px;
    font-weight: 600;
    color: #e5e7eb;
}

.antigravity-row-desc {
    font-size: 11.5px;
    color: #7c808f;
}

.antigravity-check-icon {
    color: #3b82f6;
    font-size: 15px;
}

.antigravity-key-badge {
    font-family: monospace;
    font-size: 11px;
    font-weight: 600;
    color: #e5e7eb;
    background-color: #20222c;
    border: 1px solid #2c2f3c;
    padding: 4px 10px;
    border-radius: 6px;
}

.antigravity-edit-btn {
    min-width: 64px;
    font-size: 11px;
    border-radius: 6px;
    border: 1px solid #2c2f3c;
    background-color: #20222b;
    color: #d1d5db;
}

.antigravity-edit-btn:hover {
    background-color: #292c38;
    color: #ffffff;
}

.antigravity-edit-btn.active-edit {
    background-color: #2563eb;
    color: #ffffff;
    border-color: #3b82f6;
}

.antigravity-reset-btn {
    font-size: 11px;
    color: #8c909e;
}

.antigravity-reset-btn:hover {
    color: #ffffff;
}

.antigravity-entry {
    background-color: #20222c;
    border: 1px solid #2c2f3c;
    border-radius: 6px;
    color: #e5e7eb;
    font-size: 12px;
    padding: 4px 8px;
}

.antigravity-spin {
    background-color: #20222c;
    border: 1px solid #2c2f3c;
    border-radius: 6px;
    color: #e5e7eb;
    font-size: 12px;
}

/* Full black scrim behind the settings modal: a solid layer at
   reduced alpha, composited over the window — never the wallpaper */
.settings-scrim {
    background-color: rgba(0, 0, 0, 0.55);
}
"#;

/// Theme-derived chrome overrides, VSCode-style.
///
/// The terminal VTE colors come from `ColorTheme` directly, but every
/// other surface (window, sidebar, pane headers, cards, inputs,
/// separators) is styled by `CUSTOM_CSS` above. Without this override
/// those surfaces stay fixed while the terminal follows the selected
/// theme, so new tabs look detached from the preset. This keeps the
/// workbench coherent with the active theme.
pub fn chrome_override_css(theme: &crate::theme::ColorTheme) -> String {
    let c = theme.chrome();
    format!(
        r#"
window {{ background-color: {base}; }}
headerbar {{ background-color: {base}; border-bottom: 1px solid {border}; }}
.terminal-pane {{ background-color: {base}; border: none; }}
paned separator {{ background-color: {border}; min-width: 1px; min-height: 1px; padding: 0; }}
.pane-header {{ background-color: {header_bg}; border-bottom: 1px solid {border}; }}
.terminal-pane.focused-pane .pane-header {{ background-color: {card_bg}; }}
.pane-title {{ color: {muted}; }}
.terminal-pane.focused-pane .pane-title {{ color: {fg}; }}
.workspace-sidebar {{ background-color: {sidebar}; border-right: none; }}
.vertical-tabs-list {{ background: transparent; }}
.tab-row:hover {{ background-color: {hover}; }}
.tab-row:selected {{ background-color: {hover}; border: 1px solid {border_focus}; }}
.tab-title {{ color: {fg}; }}
.tab-subtitle {{ color: {muted}; }}
.sidebar-footer {{ border-top: 1px solid {border}; }}
.antigravity-modal {{ background-color: {base}; border: 1px solid {border_focus}; }}
.antigravity-settings-sidebar {{ background-color: {sidebar}; border-right: 1px solid {border}; }}
.antigravity-nav-btn.active-nav {{ background-color: {hover}; color: {fg}; }}
.antigravity-divider {{ background-color: {border}; }}
.antigravity-settings-content {{ background-color: {base}; }}
.antigravity-content-title {{ color: {fg}; }}
.antigravity-content-desc {{ color: {muted}; }}
.antigravity-card {{ background-color: {card_bg}; border: 1px solid {border}; }}
.antigravity-card-row:hover {{ background-color: {hover}; }}
.antigravity-card-sep {{ background-color: {border}; }}
.antigravity-row-title {{ color: {fg}; }}
.antigravity-row-desc {{ color: {muted}; }}
.antigravity-key-badge {{ color: {fg}; background-color: {input}; border: 1px solid {border_focus}; }}
.antigravity-edit-btn {{ border: 1px solid {border_focus}; background-color: {input}; color: {fg}; }}
.antigravity-entry {{ background-color: {input}; border: 1px solid {border_focus}; color: {fg}; }}
.antigravity-spin {{ background-color: {input}; border: 1px solid {border_focus}; color: {fg}; }}
"#,
        base = c.base,
        fg = c.foreground,
        muted = c.muted,
        sidebar = c.sidebar_bg,
        header_bg = c.header_bg,
        card_bg = c.card_bg,
        hover = c.hover_bg,
        border = c.border,
        border_focus = c.border_focus,
        input = c.input_bg,
    )
}

/// Layer theme-derived chrome colors on top of `CUSTOM_CSS`.
pub fn apply_chrome_theme(theme_name: &str) {
    let theme = crate::theme::find_theme(theme_name);
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&chrome_override_css(&theme));
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }
}

struct State {
    tabs: Vec<Tab>,
    active_tab_id: String,
    tab_views: HashMap<String, Rc<TabView>>,
    config: AppConfig,
}

pub struct MainWindow {
    window: libadwaita::ApplicationWindow,
}

impl MainWindow {
    pub fn new(
        app: &libadwaita::Application,
        config: AppConfig,
        tray_rx: std::sync::mpsc::Receiver<TrayAction>,
    ) -> Self {
        let window = libadwaita::ApplicationWindow::new(app);
        window.set_title(None);
        window.set_default_size(1280, 800);
        apply_chrome_theme(&config.appearance.theme);

        let root_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let header_bar = libadwaita::HeaderBar::new();
        // No centered title widget — the red-marked "rmux" label comes from
        // the default HeaderBar title, which mirrors the window title.
        header_bar.set_title_widget(Some(&gtk4::Box::new(gtk4::Orientation::Horizontal, 0)));
        root_box.append(&header_bar);

        // Sidebar toggle button
        let btn_sidebar = gtk4::Button::from_icon_name("sidebar-show-symbolic");
        btn_sidebar.set_tooltip_text(Some("Toggle Sidebar (Ctrl+B)"));
        header_bar.pack_start(&btn_sidebar);

        // Main Paned
        let main_paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        main_paned.set_position(config.appearance.sidebar_width);
        // Never squeeze the sidebar below its minimum width: without
        // this the divider (or a narrow window) crops workspace labels.
        // Clamp the other way too: the divider cannot drag the sidebar
        // past its maximum, so it never swallows the terminal area.
        main_paned.set_shrink_start_child(false);
        main_paned.set_shrink_end_child(false);
        {
            let guard = Rc::new(Cell::new(false));
            main_paned.connect_position_notify(move |p| {
                if guard.get() {
                    return;
                }
                if p.position() > SIDEBAR_MAX_WIDTH {
                    guard.set(true);
                    p.set_position(SIDEBAR_MAX_WIDTH);
                    guard.set(false);
                }
            });
        }
        main_paned.set_hexpand(true);
        main_paned.set_vexpand(true);

        let tab_stack = gtk4::Stack::new();
        tab_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        tab_stack.set_hexpand(true);
        tab_stack.set_vexpand(true);
        main_paned.set_end_child(Some(&tab_stack));

        root_box.append(&main_paned);

        // Overlay hosts a full-window black scrim behind modal dialogs.
        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&root_box));
        window.set_content(Some(&overlay));

        // Shared quit flag: tray Quit sets this so the window really
        // closes instead of hiding again.
        let confirmed_quit = Rc::new(Cell::new(false));

        // Close (X) hides straight to the top-bar indicator. No prompt:
        // terminals keep running, quit happens from the indicator menu.
        {
            let win_ref = window.clone();
            let quit_flag = confirmed_quit.clone();
            window.connect_close_request(move |_| {
                if quit_flag.get() {
                    return glib::Propagation::Proceed;
                }
                win_ref.set_visible(false);
                glib::Propagation::Stop
            });
        }

        let initial_tab = Tab::default();
        let initial_id = initial_tab.id.clone();

        let state = Rc::new(RefCell::new(State {
            tabs: vec![initial_tab],
            active_tab_id: initial_id.clone(),
            tab_views: HashMap::new(),
            config: config.clone(),
        }));

        // Functions for manipulating tabs
        let switch_tab = {
            let state = state.clone();
            let tab_stack = tab_stack.clone();

            Rc::new(move |tab_id: &str| {
                let config_term = {
                    let mut st = state.borrow_mut();
                    st.active_tab_id = tab_id.to_string();

                    let config_term = st.config.terminal.clone();

                    if !st.tab_views.contains_key(tab_id) {
                        let dir = std::env::current_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| String::from("/"));
                        let tv = TabView::new(&config_term, dir);
                        tab_stack.add_named(&tv.container, Some(tab_id));
                        st.tab_views.insert(tab_id.to_string(), tv);
                    }
                    config_term
                };

                let _ = config_term;
                tab_stack.set_visible_child_name(tab_id);
            })
        };

        // Create sidebar holder and callbacks
        let sidebar_cell: Rc<RefCell<Option<Rc<VerticalSidebar>>>> = Rc::new(RefCell::new(None));
        let close_tab_holder: Rc<RefCell<Option<Rc<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let rename_tab_holder: Rc<RefCell<Option<Rc<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        let close_tab = {
            let state = state.clone();
            let tab_stack = tab_stack.clone();
            let sidebar_cell = sidebar_cell.clone();
            let switch_tab = switch_tab.clone();
            let close_tab_holder = close_tab_holder.clone();
            let rename_tab_holder = rename_tab_holder.clone();

            Rc::new(move |tab_id: &str| {
                let mut next_id_opt = None;
                let mut tabs_snapshot = Vec::new();

                {
                    let mut st = state.borrow_mut();
                    if st.tabs.len() <= 1 {
                        return;
                    }
                    if let Some(idx) = st.tabs.iter().position(|t| t.id == tab_id) {
                        st.tabs.remove(idx);
                        let next_id = if idx < st.tabs.len() {
                            st.tabs[idx].id.clone()
                        } else {
                            st.tabs[st.tabs.len() - 1].id.clone()
                        };

                        if let Some(tv) = st.tab_views.remove(tab_id) {
                            tab_stack.remove(&tv.container);
                        }

                        tabs_snapshot = st.tabs.clone();
                        next_id_opt = Some(next_id);
                    }
                }

                if let Some(next_id) = next_id_opt {
                    if let Some(sb) = sidebar_cell.borrow().as_ref() {
                        let ct_cb = close_tab_holder.clone();
                        let rn_cb = rename_tab_holder.clone();
                        sb.set_tabs(
                            &tabs_snapshot,
                            &next_id,
                            move |id| {
                                if let Some(cb) = ct_cb.borrow().as_ref() {
                                    cb(&id);
                                }
                            },
                            move |id| {
                                if let Some(cb) = rn_cb.borrow().as_ref() {
                                    cb(&id);
                                }
                            },
                        );
                    }
                    switch_tab(&next_id);
                }
            })
        };

        *close_tab_holder.borrow_mut() = Some(close_tab.clone());

        let add_tab = {
            let state = state.clone();
            let sidebar_cell = sidebar_cell.clone();
            let switch_tab = switch_tab.clone();
            let close_tab_holder = close_tab_holder.clone();
            let rename_tab_holder = rename_tab_holder.clone();

            Rc::new(move || {
                let (new_id, tabs_snapshot) = {
                    let mut st = state.borrow_mut();
                    let count = st.tabs.len() + 1;
                    let new_tab = Tab {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: format!("Workspace {}", count),
                        root_layout: crate::model::LayoutNode::default(),
                    };
                    let new_id = new_tab.id.clone();
                    st.tabs.push(new_tab);
                    (new_id, st.tabs.clone())
                };

                if let Some(sb) = sidebar_cell.borrow().as_ref() {
                    let ct_cb = close_tab_holder.clone();
                    let rn_cb = rename_tab_holder.clone();
                    sb.set_tabs(
                        &tabs_snapshot,
                        &new_id,
                        move |id| {
                            if let Some(cb) = ct_cb.borrow().as_ref() {
                                cb(&id);
                            }
                        },
                        move |id| {
                            if let Some(cb) = rn_cb.borrow().as_ref() {
                                cb(&id);
                            }
                        },
                    );
                }

                switch_tab(&new_id);
            })
        };

        // Rename Workspace dialog
        let rename_tab = {
            let state = state.clone();
            let window_clone = window.clone();
            let sidebar_cell = sidebar_cell.clone();
            let close_tab_holder = close_tab_holder.clone();
            let rename_tab_holder = rename_tab_holder.clone();

            Rc::new(move |target_tab_id: &str| {
                let current_title = {
                    let st = state.borrow();
                    st.tabs
                        .iter()
                        .find(|t| t.id == target_tab_id)
                        .map(|t| t.title.clone())
                        .unwrap_or_else(|| String::from("Workspace"))
                };

                let dialog = gtk4::Window::new();
                dialog.set_title(Some("Rename Workspace"));
                dialog.set_transient_for(Some(&window_clone));
                dialog.set_modal(true);
                dialog.set_default_size(360, 140);
                dialog.set_resizable(false);

                let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
                vbox.set_margin_start(16);
                vbox.set_margin_end(16);
                vbox.set_margin_top(16);
                vbox.set_margin_bottom(16);

                let lbl = gtk4::Label::new(Some("Enter new name for workspace:"));
                lbl.set_halign(gtk4::Align::Start);
                vbox.append(&lbl);

                let entry = gtk4::Entry::new();
                entry.set_text(&current_title);
                entry.select_region(0, -1);
                vbox.append(&entry);

                let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                btn_box.set_halign(gtk4::Align::End);

                let btn_cancel = gtk4::Button::with_label("Cancel");
                let d_cancel = dialog.clone();
                btn_cancel.connect_clicked(move |_| {
                    d_cancel.close();
                });
                btn_box.append(&btn_cancel);

                let btn_save = gtk4::Button::with_label("Rename");
                btn_save.add_css_class("suggested-action");

                let state_save = state.clone();
                let tid = target_tab_id.to_string();
                let sb_cell = sidebar_cell.clone();
                let ct_h = close_tab_holder.clone();
                let rn_h = rename_tab_holder.clone();
                let d_save = dialog.clone();

                let do_rename = move |new_name: String| {
                    let trimmed = new_name.trim();
                    if trimmed.is_empty() {
                        return;
                    }
                    let (tabs_snapshot, active_id) = {
                        let mut st = state_save.borrow_mut();
                        if let Some(tab) = st.tabs.iter_mut().find(|t| t.id == tid) {
                            tab.title = trimmed.to_string();
                        }
                        (st.tabs.clone(), st.active_tab_id.clone())
                    };

                    if let Some(sb) = sb_cell.borrow().as_ref() {
                        let ct_cb = ct_h.clone();
                        let rn_cb = rn_h.clone();
                        sb.set_tabs(
                            &tabs_snapshot,
                            &active_id,
                            move |id| {
                                if let Some(cb) = ct_cb.borrow().as_ref() {
                                    cb(&id);
                                }
                            },
                            move |id| {
                                if let Some(cb) = rn_cb.borrow().as_ref() {
                                    cb(&id);
                                }
                            },
                        );
                    }
                    d_save.close();
                };

                let do_r_clone = do_rename.clone();
                let entry_clone = entry.clone();
                btn_save.connect_clicked(move |_| {
                    do_r_clone(entry_clone.text().to_string());
                });
                btn_box.append(&btn_save);

                entry.connect_activate(move |e| {
                    do_rename(e.text().to_string());
                });

                vbox.append(&btn_box);
                dialog.set_child(Some(&vbox));
                dialog.present();
            })
        };

        *rename_tab_holder.borrow_mut() = Some(rename_tab.clone());

        let split_tab = {
            let state = state.clone();
            Rc::new(move |orientation: &str| {
                let tv_opt = {
                    let st = state.borrow();
                    st.tab_views.get(&st.active_tab_id).cloned()
                };
                if let Some(tv) = tv_opt {
                    let target = tv
                        .focused_pane_id
                        .borrow()
                        .clone()
                        .or_else(|| tv.panes.borrow().keys().next().cloned());
                    if let Some(focused) = target {
                        tv.split_pane(&focused, orientation);
                    }
                }
            })
        };

        // Close focused pane (or workspace if 1 pane)
        let close_pane_action = {
            let state = state.clone();
            let close_tab = close_tab.clone();
            Rc::new(move || {
                let (active_id, tv_opt) = {
                    let st = state.borrow();
                    (st.active_tab_id.clone(), st.tab_views.get(&st.active_tab_id).cloned())
                };
                if let Some(tv) = tv_opt {
                    if tv.panes.borrow().len() > 1 {
                        let target = tv
                            .focused_pane_id
                            .borrow()
                            .clone()
                            .or_else(|| tv.panes.borrow().keys().next().cloned());
                        if let Some(t) = target {
                            tv.close_pane(&t);
                            return;
                        }
                    }
                }
                close_tab(&active_id);
            })
        };

        // Open Settings Modal over a full-window black scrim (a real layer
        // behind the modal, never wallpaper transparency).
        let open_settings = {
            let win_ref = window.clone();
            let state_ref = state.clone();
            let overlay_ref = overlay.clone();

            Rc::new(move || {
                let cfg_cell = Rc::new(RefCell::new(state_ref.borrow().config.clone()));

                let state_theme = state_ref.clone();
                let on_theme_cb = move |theme_name: String| {
                    let theme_obj = find_theme(&theme_name);
                    apply_chrome_theme(&theme_name);
                    let mut st = state_theme.borrow_mut();
                    st.config.appearance.theme = theme_name.clone();
                    st.config.terminal.theme = theme_name.clone();

                    for tv in st.tab_views.values() {
                        tv.set_theme(&theme_name, &theme_obj);
                    }
                };

                let state_font = state_ref.clone();
                let on_font_cb = move |font_fam: String, sz: u32| {
                    let mut st = state_font.borrow_mut();
                    st.config.terminal.font_family = font_fam.clone();
                    st.config.terminal.font_size = sz;
                    for tv in st.tab_views.values() {
                        tv.set_font(&font_fam, sz);
                    }
                };

                let scrim = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                scrim.add_css_class("settings-scrim");
                scrim.set_hexpand(true);
                scrim.set_vexpand(true);
                overlay_ref.add_overlay(&scrim);

                let dialog = crate::settings::show_settings_modal(
                    &win_ref,
                    cfg_cell,
                    on_theme_cb,
                    on_font_cb,
                );
                let overlay_rm = overlay_ref.clone();
                dialog.connect_close_request(move |_| {
                    overlay_rm.remove_overlay(&scrim);
                    glib::Propagation::Proceed
                });
            })
        };

        // Sidebar holds the workspace list plus a bottom-left Settings footer.
        let sidebar = {
            let switch_tab = switch_tab.clone();
            let add_tab = add_tab.clone();
            let open_settings = open_settings.clone();

            Rc::new(VerticalSidebar::new(
                move |id| switch_tab(&id),
                move || add_tab(),
                move || open_settings(),
            ))
        };

        *sidebar_cell.borrow_mut() = Some(sidebar.clone());
        main_paned.set_start_child(Some(&sidebar.container));

        // Top-bar indicator actions drain here on the GTK loop. Quit
        // bypasses the hide-to-tray close dialog via the shared
        // confirmed-quit flag.
        {
            let tray_rx = Rc::new(RefCell::new(tray_rx));
            let win_tray = window.clone();
            let add_tray = add_tab.clone();
            let settings_tray = open_settings.clone();
            let quit_tray = confirmed_quit.clone();
            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                let rx = tray_rx.borrow();
                while let Ok(action) = rx.try_recv() {
                    match action {
                        TrayAction::ShowWindow => {
                            win_tray.set_visible(true);
                            win_tray.present();
                        }
                        TrayAction::NewWorkspace => {
                            win_tray.set_visible(true);
                            win_tray.present();
                            add_tray();
                        }
                        TrayAction::OpenSettings => {
                            win_tray.set_visible(true);
                            win_tray.present();
                            settings_tray();
                        }
                        TrayAction::Quit => {
                            quit_tray.set(true);
                            win_tray.close();
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        }

        // Connect sidebar toggle
        let sb_toggle = sidebar.clone();
        btn_sidebar.connect_clicked(move |_| {
            let v = sb_toggle.container.is_visible();
            sb_toggle.container.set_visible(!v);
        });

        // Global Key shortcuts with Capture Phase (intercepted before terminal widget)
        let key_ctrl = gtk4::EventControllerKey::new();
        key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);

        let at_key = add_tab.clone();
        let close_pane_key = close_pane_action.clone();
        let sp_h_key = split_tab.clone();
        let sp_v_key = split_tab.clone();
        let sb_toggle_key = sidebar.clone();
        let state_key = state.clone();

        key_ctrl.connect_key_pressed(move |_, keyval, _, state_flags| {
            let shortcuts = state_key.borrow().config.shortcuts.clone();

            // Configurable action: New Workspace (default: Ctrl+Shift+N)
            if matches_key(&shortcuts.new_workspace, keyval, state_flags) {
                at_key();
                return glib::Propagation::Stop;
            }

            // Configurable action: Close Focused Pane (default: Ctrl+W)
            if matches_key(&shortcuts.close_pane, keyval, state_flags) {
                close_pane_key();
                return glib::Propagation::Stop;
            }

            // Configurable action: Split Right (default: Ctrl+Shift+T)
            if matches_key(&shortcuts.split_right, keyval, state_flags) {
                sp_h_key("vertical");
                return glib::Propagation::Stop;
            }

            // Configurable action: Split Down (default: Ctrl+Shift+D)
            if matches_key(&shortcuts.split_down, keyval, state_flags) {
                sp_v_key("horizontal");
                return glib::Propagation::Stop;
            }

            // Configurable action: Toggle Sidebar (default: Ctrl+B)
            if matches_key(&shortcuts.toggle_sidebar, keyval, state_flags) {
                let v = sb_toggle_key.container.is_visible();
                sb_toggle_key.container.set_visible(!v);
                return glib::Propagation::Stop;
            }

            // Configurable action: Zoom In (default: Ctrl+Plus)
            if matches_key(&shortcuts.zoom_in, keyval, state_flags) {
                let st = state_key.borrow();
                if let Some(tv) = st.tab_views.get(&st.active_tab_id) {
                    for pane in tv.panes.borrow().values() {
                        pane.term.borrow_mut().zoom_in();
                    }
                }
                return glib::Propagation::Stop;
            }

            // Configurable action: Zoom Out (default: Ctrl+Minus)
            if matches_key(&shortcuts.zoom_out, keyval, state_flags) {
                let st = state_key.borrow();
                if let Some(tv) = st.tab_views.get(&st.active_tab_id) {
                    for pane in tv.panes.borrow().values() {
                        pane.term.borrow_mut().zoom_out();
                    }
                }
                return glib::Propagation::Stop;
            }

            // Configurable action: Reset Zoom (default: Ctrl+0)
            if matches_key(&shortcuts.zoom_reset, keyval, state_flags) {
                let st = state_key.borrow();
                if let Some(tv) = st.tab_views.get(&st.active_tab_id) {
                    for pane in tv.panes.borrow().values() {
                        pane.term.borrow_mut().zoom_reset();
                    }
                }
                return glib::Propagation::Stop;
            }

            // Direct fallbacks for standard developer shortcuts
            let ctrl = state_flags.contains(gtk4::gdk::ModifierType::CONTROL_MASK);
            let shift = state_flags.contains(gtk4::gdk::ModifierType::SHIFT_MASK);

            if ctrl && shift {
                let key = keyval.to_lower();
                if key == gtk4::gdk::Key::c {
                    let st = state_key.borrow();
                    if let Some(tv) = st.tab_views.get(&st.active_tab_id) {
                        let target = tv
                            .focused_pane_id
                            .borrow()
                            .clone()
                            .or_else(|| tv.panes.borrow().keys().next().cloned());
                        if let Some(t) = target {
                            if let Some(pane) = tv.panes.borrow().get(&t) {
                                pane.term.borrow().copy();
                            }
                        }
                    }
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::v {
                    let st = state_key.borrow();
                    if let Some(tv) = st.tab_views.get(&st.active_tab_id) {
                        let target = tv
                            .focused_pane_id
                            .borrow()
                            .clone()
                            .or_else(|| tv.panes.borrow().keys().next().cloned());
                        if let Some(t) = target {
                            if let Some(pane) = tv.panes.borrow().get(&t) {
                                pane.term.borrow().paste();
                            }
                        }
                    }
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::n {
                    at_key();
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::t {
                    sp_h_key("vertical");
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::d || key == gtk4::gdk::Key::o {
                    sp_v_key("horizontal");
                    return glib::Propagation::Stop;
                }
            } else if ctrl && !shift {
                let key = keyval.to_lower();
                if key == gtk4::gdk::Key::w {
                    close_pane_key();
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::b {
                    let v = sb_toggle_key.container.is_visible();
                    sb_toggle_key.container.set_visible(!v);
                    return glib::Propagation::Stop;
                }
            }

            glib::Propagation::Proceed
        });
        window.add_controller(key_ctrl);

        // Populate initial tabs and switch without holding any borrows
        let (initial_tabs, active_id) = {
            let st = state.borrow();
            (st.tabs.clone(), st.active_tab_id.clone())
        };

        let ct_cb = close_tab_holder.clone();
        let rn_cb = rename_tab_holder.clone();
        sidebar.set_tabs(
            &initial_tabs,
            &active_id,
            move |id| {
                if let Some(cb) = ct_cb.borrow().as_ref() {
                    cb(&id);
                }
            },
            move |id| {
                if let Some(cb) = rn_cb.borrow().as_ref() {
                    cb(&id);
                }
            },
        );
        switch_tab(&initial_id);

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
