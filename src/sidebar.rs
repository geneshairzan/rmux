//! Vertical workspaces sidebar for rmux (workspace list plus Settings footer).

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::model::Tab;

/// Minimum sidebar width in pixels. The split container and the window
/// both clamp to this so labels never crop at narrow sizes.
pub const SIDEBAR_MIN_WIDTH: i32 = 230;

/// Maximum sidebar width in pixels. The divider cannot drag past this,
/// so the sidebar never swallows the terminal area.
pub const SIDEBAR_MAX_WIDTH: i32 = 420;

pub struct VerticalSidebar {
    pub container: gtk4::Box,
    pub list_box: gtk4::ListBox,
    row_map: Rc<RefCell<HashMap<String, gtk4::ListBoxRow>>>,
    suppress_signals: Rc<Cell<bool>>,
    on_new: Rc<dyn Fn()>,
}

impl VerticalSidebar {
    pub fn new(
        on_tab_selected: impl Fn(String) + 'static + Clone,
        on_new_tab: impl Fn() + 'static + Clone,
        on_settings: impl Fn() + 'static,
    ) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.add_css_class("workspace-sidebar");
        container.set_size_request(SIDEBAR_MIN_WIDTH, -1);

        // Scrolled List of Workspaces (header and footer removed: creation
        // happens via background right-click or the Ctrl+Shift+N shortcut)
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scroll.set_hexpand(true);
        scroll.set_vexpand(true);

        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        list_box.add_css_class("navigation-sidebar");
        list_box.add_css_class("vertical-tabs-list");

        let row_map: Rc<RefCell<HashMap<String, gtk4::ListBoxRow>>> = Rc::new(RefCell::new(HashMap::new()));
        let row_map_clone = row_map.clone();
        let suppress_signals = Rc::new(Cell::new(false));
        let suppress_clone = suppress_signals.clone();

        list_box.connect_row_selected(move |_, row| {
            if suppress_clone.get() {
                return;
            }
            if let Some(r) = row {
                let target_id = {
                    let map = row_map_clone.borrow();
                    map.iter()
                        .find(|(_, item_row)| *item_row == r)
                        .map(|(id, _)| id.clone())
                };
                if let Some(id) = target_id {
                    on_tab_selected(id);
                }
            }
        });

        scroll.set_child(Some(&list_box));
        container.append(&scroll);

        // Bottom-left Settings footer (icon plus label, vertically centered).
        let footer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        footer.add_css_class("sidebar-footer");

        let btn_settings = gtk4::Button::new();
        btn_settings.add_css_class("flat");
        btn_settings.add_css_class("sidebar-settings-btn");
        btn_settings.set_tooltip_text(Some("Settings & Preferences"));

        let s_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        s_box.add_css_class("sidebar-settings-row");
        s_box.set_valign(gtk4::Align::Center);

        let icon_settings = gtk4::Image::from_icon_name("emblem-system-symbolic");
        icon_settings.add_css_class("sidebar-settings-icon");
        icon_settings.set_valign(gtk4::Align::Center);
        s_box.append(&icon_settings);

        let lbl_settings = gtk4::Label::new(Some("Settings"));
        lbl_settings.add_css_class("sidebar-settings-label");
        lbl_settings.set_halign(gtk4::Align::Start);
        lbl_settings.set_hexpand(true);
        lbl_settings.set_valign(gtk4::Align::Center);
        s_box.append(&lbl_settings);

        btn_settings.set_child(Some(&s_box));
        btn_settings.connect_clicked(move |_| {
            on_settings();
        });
        footer.append(&btn_settings);
        container.append(&footer);

        // Background right-click on empty sidebar area -> New Workspace.
        // Attached to the container so clicks below the last row still work.
        {
            let on_new_bg = on_new_tab.clone();
            let container_ref = container.clone();
            let gesture_bg = gtk4::GestureClick::new();
            gesture_bg.set_button(3);
            gesture_bg.connect_pressed(move |_, _, x, y| {
                let popover = gtk4::Popover::new();
                let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
                menu_box.set_margin_start(6);
                menu_box.set_margin_end(6);
                menu_box.set_margin_top(6);
                menu_box.set_margin_bottom(6);

                let btn_new = gtk4::Button::new();
                btn_new.add_css_class("flat");
                btn_new.add_css_class("workspace-context-item");
                let new_lbl = gtk4::Label::new(Some("New Workspace"));
                new_lbl.set_halign(gtk4::Align::Start);
                new_lbl.set_hexpand(true);
                btn_new.set_child(Some(&new_lbl));
                let cb = on_new_bg.clone();
                let p = popover.clone();
                btn_new.connect_clicked(move |_| {
                    p.popdown();
                    cb();
                });
                menu_box.append(&btn_new);

                popover.set_child(Some(&menu_box));
                popover.set_parent(&container_ref);
                let rect = gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                popover.set_pointing_to(Some(&rect));
                popover.popup();
            });
            container.add_controller(gesture_bg);
        }

        Self {
            container,
            list_box,
            row_map,
            suppress_signals,
            on_new: Rc::new(on_new_tab),
        }
    }

    pub fn set_tabs(
        &self,
        tabs: &[Tab],
        active_id: &str,
        on_close: impl Fn(String) + 'static + Clone,
        on_rename: impl Fn(String) + 'static + Clone,
    ) {
        self.suppress_signals.set(true);

        // Clear existing rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        self.row_map.borrow_mut().clear();

        for tab in tabs {
            let row = gtk4::ListBoxRow::new();
            row.add_css_class("tab-row");

            let box_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
            box_row.set_margin_start(10);
            box_row.set_margin_end(8);
            box_row.set_margin_top(7);
            box_row.set_margin_bottom(7);

            // Status dot
            let dot = gtk4::Image::from_icon_name("media-record-symbolic");
            dot.add_css_class("tab-status-dot");
            dot.add_css_class("status-running");
            box_row.append(&dot);

            // Text VBox
            let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
            vbox.set_hexpand(true);

            let title_lbl = gtk4::Label::new(Some(&tab.title));
            title_lbl.add_css_class("tab-title");
            title_lbl.set_halign(gtk4::Align::Start);
            title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            vbox.append(&title_lbl);

            let home = std::env::var("HOME").unwrap_or_default();
            let mut dir_str = std::env::current_dir().unwrap_or_default().to_string_lossy().to_string();
            if !home.is_empty() && dir_str.starts_with(&home) {
                dir_str = dir_str.replacen(&home, "~", 1);
            }
            let sub_lbl = gtk4::Label::new(Some(&dir_str));
            sub_lbl.add_css_class("tab-subtitle");
            sub_lbl.set_halign(gtk4::Align::Start);
            sub_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            vbox.append(&sub_lbl);

            box_row.append(&vbox);

            // Close button
            let btn_close = gtk4::Button::from_icon_name("window-close-symbolic");
            btn_close.add_css_class("flat");
            btn_close.add_css_class("circular");
            btn_close.add_css_class("tab-close-btn");
            btn_close.set_tooltip_text(Some("Close Workspace (Ctrl+W)"));
            let tab_id = tab.id.clone();
            let on_c = on_close.clone();
            btn_close.connect_clicked(move |_| {
                on_c(tab_id.clone());
            });
            box_row.append(&btn_close);

            row.set_child(Some(&box_row));

            // Double click gesture on row to rename workspace
            let gesture_click = gtk4::GestureClick::new();
            gesture_click.set_button(1);
            let tab_id_click = tab.id.clone();
            let on_ren_click = on_rename.clone();
            gesture_click.connect_pressed(move |_, n_press, _x, _y| {
                if n_press == 2 {
                    on_ren_click(tab_id_click.clone());
                }
            });
            row.add_controller(gesture_click);

            // Right-click context menu gesture (New / Rename / Close)
            let gesture_right = gtk4::GestureClick::new();
            gesture_right.set_button(3);
            let tab_id_ctx = tab.id.clone();
            let on_ren_ctx = on_rename.clone();
            let on_cls_ctx = on_close.clone();
            let on_new_ctx = self.on_new.clone();
            let row_ref = row.clone();
            gesture_right.connect_pressed(move |_, _, x, y| {
                let popover = gtk4::Popover::new();
                let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
                menu_box.set_margin_start(6);
                menu_box.set_margin_end(6);
                menu_box.set_margin_top(6);
                menu_box.set_margin_bottom(6);

                let btn_new = gtk4::Button::new();
                btn_new.add_css_class("flat");
                btn_new.add_css_class("workspace-context-item");
                let new_lbl = gtk4::Label::new(Some("New Workspace"));
                new_lbl.set_halign(gtk4::Align::Start);
                new_lbl.set_hexpand(true);
                btn_new.set_child(Some(&new_lbl));
                let n_cb = on_new_ctx.clone();
                let p0 = popover.clone();
                btn_new.connect_clicked(move |_| {
                    p0.popdown();
                    n_cb();
                });
                menu_box.append(&btn_new);

                let btn_ren = gtk4::Button::new();
                btn_ren.add_css_class("flat");
                btn_ren.add_css_class("workspace-context-item");
                let ren_lbl = gtk4::Label::new(Some("Rename Workspace..."));
                ren_lbl.set_halign(gtk4::Align::Start);
                ren_lbl.set_hexpand(true);
                btn_ren.set_child(Some(&ren_lbl));
                let r_id = tab_id_ctx.clone();
                let r_cb = on_ren_ctx.clone();
                let p1 = popover.clone();
                btn_ren.connect_clicked(move |_| {
                    p1.popdown();
                    r_cb(r_id.clone());
                });
                menu_box.append(&btn_ren);

                let btn_cls = gtk4::Button::new();
                btn_cls.add_css_class("flat");
                btn_cls.add_css_class("workspace-context-item");
                let cls_lbl = gtk4::Label::new(Some("Close Workspace"));
                cls_lbl.set_halign(gtk4::Align::Start);
                cls_lbl.set_hexpand(true);
                btn_cls.set_child(Some(&cls_lbl));
                let c_id = tab_id_ctx.clone();
                let c_cb = on_cls_ctx.clone();
                let p2 = popover.clone();
                btn_cls.connect_clicked(move |_| {
                    p2.popdown();
                    c_cb(c_id.clone());
                });
                menu_box.append(&btn_cls);

                popover.set_child(Some(&menu_box));
                popover.set_parent(&row_ref);
                let rect = gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                popover.set_pointing_to(Some(&rect));
                popover.popup();
            });
            row.add_controller(gesture_right);

            self.list_box.append(&row);
            self.row_map.borrow_mut().insert(tab.id.clone(), row.clone());

            if tab.id == active_id {
                self.list_box.select_row(Some(&row));
            }
        }

        self.suppress_signals.set(false);
    }
}
