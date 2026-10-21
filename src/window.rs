//! Main application window uniting vertical workspaces sidebar and terminal split panes.

use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::config::{matches_key, AppConfig};
use crate::model::Tab;
use crate::sidebar::VerticalSidebar;
use crate::split::TabView;
use crate::theme::find_theme;

pub const CUSTOM_CSS: &str = r#"
/* ========================================================
   Antigravity Dark Minimalist Theme
   ======================================================== */
window {
    background-color: #131418;
}

headerbar {
    background-color: #131418;
    border-bottom: 1px solid #1c1e24;
    min-height: 38px;
    padding: 0 8px;
}

/* Terminal Panes */
.terminal-pane {
    background-color: #0d0e12;
    border-radius: 6px;
    border: 1px solid #1a1c22;
    transition: border-color 150ms ease;
}

.terminal-pane.focused-pane {
    border: 1px solid #2e313d;
}

.terminal-pane.unfocused-pane {
    border: 1px solid #16171d;
}

.pane-header {
    background-color: #111216;
    border-bottom: 1px solid #1a1c22;
    border-top-left-radius: 5px;
    border-top-right-radius: 5px;
    min-height: 24px;
    padding: 2px 10px;
}

.terminal-pane.focused-pane .pane-header {
    background-color: #14151b;
}

.pane-title {
    font-size: 11px;
    font-weight: 500;
    font-family: monospace;
    color: #8c909e;
}

.terminal-pane.focused-pane .pane-title {
    color: #e5e7eb;
}

.pane-action-btn {
    min-width: 20px;
    min-height: 20px;
    padding: 2px;
    color: #6b7080;
    border-radius: 4px;
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

/* Workspaces Sidebar (Antigravity Style) */
.workspace-sidebar {
    background-color: #111215;
    border-right: 1px solid #1c1e24;
}

.sidebar-header {
    border-bottom: 1px solid #18191f;
}

.sidebar-brand {
    font-weight: 700;
    font-size: 13px;
    color: #f0f1f5;
    letter-spacing: 0.8px;
}

.sidebar-icon-btn {
    min-width: 26px;
    min-height: 26px;
    padding: 3px;
    color: #8c909e;
    border-radius: 5px;
}

.sidebar-icon-btn:hover {
    color: #ffffff;
    background-color: #1e2028;
}

.vertical-tabs-list {
    background: transparent;
    padding-top: 4px;
}

.tab-row {
    border-radius: 6px;
    margin: 2px 8px;
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
    font-size: 12px;
    color: #d1d5db;
}

.tab-row:selected .tab-title {
    color: #ffffff;
}

.tab-subtitle {
    font-size: 10.5px;
    color: #717585;
}

.tab-close-btn {
    min-width: 18px;
    min-height: 18px;
    padding: 1px;
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

/* Sidebar Footer (Settings in Bottom Left) */
.sidebar-footer {
    border-top: 1px solid #18191f;
    padding-top: 6px;
}

.sidebar-sep {
    background-color: #18191f;
    margin-bottom: 4px;
}

.sidebar-settings-btn {
    border-radius: 6px;
    padding: 4px 6px;
}

.sidebar-settings-btn:hover {
    background-color: #1c1e26;
}

.sidebar-settings-icon {
    color: #8c909e;
    font-size: 14px;
}

.sidebar-settings-label {
    font-size: 12px;
    font-weight: 500;
    color: #8c909e;
}

.sidebar-settings-btn:hover .sidebar-settings-icon,
.sidebar-settings-btn:hover .sidebar-settings-label {
    color: #f0f1f5;
}

/* Header Active Workspace Title Pill */
.active-tab-header-title {
    font-size: 12px;
    font-weight: 600;
    color: #f0f1f5;
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid #232530;
    background-color: #181920;
    transition: background-color 100ms ease;
}

.active-tab-header-title:hover {
    background-color: #20222c;
    border-color: #313444;
}

/* Status Bar */
.status-bar {
    background-color: #0e0f13;
    border-top: 1px solid #17181f;
    padding: 2px 14px;
    min-height: 22px;
}

.status-bar-item {
    font-size: 10.5px;
    color: #6b7080;
    font-family: monospace;
}

/* ========================================================
   Antigravity Settings Modal Styling
   ======================================================== */
.antigravity-modal {
    background-color: #131418;
    border: 1px solid #232530;
    border-radius: 10px;
}

.antigravity-settings-sidebar {
    background-color: #111215;
    border-right: 1px solid #1c1e24;
}

.antigravity-nav-heading {
    font-size: 11px;
    font-weight: 700;
    color: #6e7282;
    letter-spacing: 0.5px;
    padding-left: 8px;
}

.antigravity-nav-btn {
    border-radius: 6px;
    padding: 7px 12px;
    font-size: 12.5px;
    color: #8c909e;
    font-weight: 500;
    margin-bottom: 2px;
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

.antigravity-nav-footer {
    font-size: 10.5px;
    color: #555866;
    padding-left: 8px;
}

.antigravity-divider {
    background-color: #1c1e24;
}

.antigravity-settings-content {
    background-color: #131418;
}

.antigravity-close-btn {
    min-width: 26px;
    min-height: 26px;
    color: #8c909e;
    border-radius: 6px;
}

.antigravity-close-btn:hover {
    color: #ffffff;
    background-color: #20222b;
}

.antigravity-content-title {
    font-size: 18px;
    font-weight: 700;
    color: #f0f1f5;
    letter-spacing: -0.2px;
}

.antigravity-content-desc {
    font-size: 12px;
    color: #7c808f;
    line-height: 1.4;
    margin-bottom: 8px;
}

.antigravity-card {
    background-color: #181920;
    border: 1px solid #232530;
    border-radius: 8px;
}

.antigravity-card-row {
    transition: background-color 100ms ease;
}

.antigravity-card-row:hover {
    background-color: #1d1e26;
}

.antigravity-card-sep {
    background-color: #20222c;
}

.antigravity-row-title {
    font-size: 13px;
    font-weight: 600;
    color: #e5e7eb;
}

.antigravity-row-desc {
    font-size: 11px;
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
    padding: 3px 9px;
    border-radius: 5px;
}

.antigravity-edit-btn {
    min-width: 64px;
    font-size: 11px;
    border-radius: 5px;
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
}

.antigravity-spin {
    background-color: #20222c;
    border: 1px solid #2c2f3c;
    border-radius: 6px;
    color: #e5e7eb;
    font-size: 12px;
}
"#;

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
    pub fn new(app: &libadwaita::Application, config: AppConfig) -> Self {
        let window = libadwaita::ApplicationWindow::new(app);
        window.set_title(Some("rmux"));
        window.set_default_size(1280, 800);

        let root_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let header_bar = libadwaita::HeaderBar::new();
        root_box.append(&header_bar);

        // Sidebar toggle button
        let btn_sidebar = gtk4::Button::from_icon_name("sidebar-show-symbolic");
        btn_sidebar.set_tooltip_text(Some("Toggle Sidebar (Ctrl+B)"));
        header_bar.pack_start(&btn_sidebar);

        // New Workspace (+) button
        let btn_new_tab = gtk4::Button::from_icon_name("list-add-symbolic");
        btn_new_tab.set_tooltip_text(Some("New Workspace (Ctrl+Shift+T)"));
        header_bar.pack_start(&btn_new_tab);

        // Center Title (Active workspace name pill, clickable to rename)
        let header_title_btn = gtk4::Button::new();
        header_title_btn.add_css_class("flat");
        let header_title = gtk4::Label::new(Some("Workspace 1"));
        header_title.add_css_class("active-tab-header-title");
        header_title_btn.set_child(Some(&header_title));
        header_title_btn.set_tooltip_text(Some("Click to rename workspace"));
        header_bar.set_title_widget(Some(&header_title_btn));

        // Main Paned
        let main_paned = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        main_paned.set_position(config.appearance.sidebar_width);
        main_paned.set_hexpand(true);
        main_paned.set_vexpand(true);

        let tab_stack = gtk4::Stack::new();
        tab_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        tab_stack.set_hexpand(true);
        tab_stack.set_vexpand(true);
        main_paned.set_end_child(Some(&tab_stack));

        root_box.append(&main_paned);

        // Bottom status bar
        let status_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        status_bar.add_css_class("status-bar");

        let status_dir_lbl = gtk4::Label::new(Some("~"));
        status_dir_lbl.add_css_class("status-bar-item");
        status_bar.append(&status_dir_lbl);

        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        status_bar.append(&spacer);

        let status_panes_lbl = gtk4::Label::new(Some("1 pane"));
        status_panes_lbl.add_css_class("status-bar-item");
        status_bar.append(&status_panes_lbl);

        root_box.append(&status_bar);
        window.set_content(Some(&root_box));

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
            let header_title = header_title.clone();
            let status_dir_lbl = status_dir_lbl.clone();

            Rc::new(move |tab_id: &str| {
                let (title, config_term) = {
                    let mut st = state.borrow_mut();
                    st.active_tab_id = tab_id.to_string();

                    let title = st
                        .tabs
                        .iter()
                        .find(|t| t.id == tab_id)
                        .map(|t| t.title.clone())
                        .unwrap_or_else(|| String::from("Workspace"));

                    let config_term = st.config.terminal.clone();

                    if !st.tab_views.contains_key(tab_id) {
                        let dir = std::env::current_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| String::from("/"));
                        let tv = TabView::new(&config_term, dir);
                        tab_stack.add_named(&tv.container, Some(tab_id));
                        st.tab_views.insert(tab_id.to_string(), tv);
                    }
                    (title, config_term)
                };

                let _ = config_term;
                header_title.set_text(&title);
                tab_stack.set_visible_child_name(tab_id);

                let home = std::env::var("HOME").unwrap_or_default();
                let mut dir_str = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();
                if !home.is_empty() && dir_str.starts_with(&home) {
                    dir_str = dir_str.replacen(&home, "~", 1);
                }
                status_dir_lbl.set_text(&dir_str);
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

        // Close all open tabs function (Ctrl+Shift+W)
        let close_all_tabs = {
            let state = state.clone();
            let tab_stack = tab_stack.clone();
            let sidebar_cell = sidebar_cell.clone();
            let switch_tab = switch_tab.clone();
            let close_tab_holder = close_tab_holder.clone();
            let rename_tab_holder = rename_tab_holder.clone();

            Rc::new(move || {
                let (new_id, tabs_snapshot) = {
                    let mut st = state.borrow_mut();
                    for tv in st.tab_views.values() {
                        tab_stack.remove(&tv.container);
                    }
                    st.tab_views.clear();

                    let fresh_tab = Tab {
                        id: uuid::Uuid::new_v4().to_string(),
                        title: String::from("Workspace 1"),
                        root_layout: crate::model::LayoutNode::default(),
                    };
                    let new_id = fresh_tab.id.clone();
                    st.tabs = vec![fresh_tab];
                    st.active_tab_id = new_id.clone();
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
            let header_title = header_title.clone();
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
                let ht_lbl = header_title.clone();
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
                        if st.active_tab_id == tid {
                            ht_lbl.set_text(trimmed);
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

        // Header title button clicked -> rename active workspace
        {
            let state = state.clone();
            let rn_holder = rename_tab_holder.clone();
            header_title_btn.connect_clicked(move |_| {
                let active = state.borrow().active_tab_id.clone();
                if let Some(cb) = rn_holder.borrow().as_ref() {
                    cb(&active);
                }
            });
        }

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

        // Open Settings Modal
        let open_settings = {
            let win_ref = window.clone();
            let state_ref = state.clone();

            Rc::new(move || {
                let cfg_cell = Rc::new(RefCell::new(state_ref.borrow().config.clone()));

                let state_theme = state_ref.clone();
                let on_theme_cb = move |theme_name: String| {
                    let theme_obj = find_theme(&theme_name);
                    let mut st = state_theme.borrow_mut();
                    st.config.appearance.theme = theme_name.clone();
                    st.config.terminal.theme = theme_name.clone();

                    for tv in st.tab_views.values() {
                        for pane in tv.panes.borrow().values() {
                            pane.term.borrow().apply_theme(&theme_obj);
                        }
                    }
                };

                let state_font = state_ref.clone();
                let on_font_cb = move |_font_fam: String, _sz: u32| {
                    let st = state_font.borrow();
                    for tv in st.tab_views.values() {
                        for pane in tv.panes.borrow().values() {
                            pane.term.borrow().apply_font();
                        }
                    }
                };

                crate::settings::show_settings_modal(
                    &win_ref,
                    cfg_cell,
                    on_theme_cb,
                    on_font_cb,
                );
            })
        };

        // Sidebar with Antigravity-styled bottom settings button
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

        // Connect sidebar toggle
        let sb_toggle = sidebar.clone();
        btn_sidebar.connect_clicked(move |_| {
            let v = sb_toggle.container.is_visible();
            sb_toggle.container.set_visible(!v);
        });

        // Connect new workspace button
        let at = add_tab.clone();
        btn_new_tab.connect_clicked(move |_| at());

        // Global Key shortcuts with Capture Phase (intercepted before terminal widget)
        let key_ctrl = gtk4::EventControllerKey::new();
        key_ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);

        let at_key = add_tab.clone();
        let close_all_tabs_key = close_all_tabs.clone();
        let close_pane_key = close_pane_action.clone();
        let sp_h_key = split_tab.clone();
        let sp_v_key = split_tab.clone();
        let sb_toggle_key = sidebar.clone();
        let state_key = state.clone();

        key_ctrl.connect_key_pressed(move |_, keyval, _, state_flags| {
            let shortcuts = state_key.borrow().config.shortcuts.clone();

            // Configurable action: Close all open tabs (default: Ctrl+Shift+W)
            if matches_key(&shortcuts.close_all_tabs, keyval, state_flags) {
                close_all_tabs_key();
                return glib::Propagation::Stop;
            }

            // Configurable action: New Workspace (default: Ctrl+Shift+T)
            if matches_key(&shortcuts.new_workspace, keyval, state_flags) {
                at_key();
                return glib::Propagation::Stop;
            }

            // Configurable action: Close Focused Pane (default: Ctrl+W)
            if matches_key(&shortcuts.close_pane, keyval, state_flags) {
                close_pane_key();
                return glib::Propagation::Stop;
            }

            // Configurable action: Split Right (default: Ctrl+Shift+R)
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
                if key == gtk4::gdk::Key::w {
                    close_all_tabs_key();
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::t {
                    at_key();
                    return glib::Propagation::Stop;
                }
                if key == gtk4::gdk::Key::r || key == gtk4::gdk::Key::e {
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
