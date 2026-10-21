//! Terminal Pane widget combining header bar and VTE terminal.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

use crate::config::TerminalConfig;
use crate::vte::TerminalWidget;

#[allow(dead_code)]
pub struct Pane {
    pub id: String,
    pub container: gtk4::Box,
    pub term: Rc<RefCell<TerminalWidget>>,
    pub title_label: gtk4::Label,
    pub status_dot: gtk4::Image,
    pub is_focused: Rc<RefCell<bool>>,
}

impl Pane {
    pub fn new(
        config: &TerminalConfig,
        working_dir: String,
        command: Option<String>,
        _on_split: impl Fn(&str, &str) + 'static + Clone, // (pane_id, orientation)
        on_close: impl Fn(&str) + 'static + Clone,        // pane_id
        on_focus: impl Fn(&str) + 'static + Clone,        // pane_id
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.add_css_class("terminal-pane");
        container.set_hexpand(true);
        container.set_vexpand(true);

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        header.add_css_class("pane-header");
        header.set_margin_start(8);
        header.set_margin_end(6);
        header.set_margin_top(3);
        header.set_margin_bottom(3);

        // Status dot
        let status_dot = gtk4::Image::from_icon_name("media-record-symbolic");
        status_dot.add_css_class("status-dot");
        status_dot.add_css_class("status-running");
        header.append(&status_dot);

        // Title
        let title_label = gtk4::Label::new(Some("terminal"));
        title_label.add_css_class("pane-title");
        title_label.set_halign(gtk4::Align::Start);
        header.append(&title_label);

        // Spacer
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);

        // Close
        let btn_close = gtk4::Button::from_icon_name("window-close-symbolic");
        btn_close.add_css_class("flat");
        btn_close.add_css_class("pane-action-btn");
        btn_close.set_tooltip_text(Some("Close Pane (Ctrl+W)"));
        let id_clone = id.clone();
        btn_close.connect_clicked(move |_| {
            on_close(&id_clone);
        });
        header.append(&btn_close);

        container.append(&header);

        // VTE Terminal
        let term_widget = TerminalWidget::new(config, working_dir, command);
        container.append(term_widget.widget());

        // Terminal focus controller
        let focus_ctrl = gtk4::EventControllerFocus::new();
        let id_focus = id.clone();
        let on_f_term = on_focus.clone();
        focus_ctrl.connect_enter(move |_| {
            on_f_term(&id_focus);
        });
        term_widget.widget().add_controller(focus_ctrl);

        let term = Rc::new(RefCell::new(term_widget));
        let is_focused = Rc::new(RefCell::new(false));

        // Click gesture for focus
        let gesture = gtk4::GestureClick::new();
        let id_click = id.clone();
        let on_f_click = on_focus.clone();
        gesture.connect_pressed(move |_, _, _, _| {
            on_f_click(&id_click);
        });
        container.add_controller(gesture);

        Self {
            id,
            container,
            term,
            title_label,
            status_dot,
            is_focused,
        }
    }

    pub fn set_focused(&self, focused: bool) {
        *self.is_focused.borrow_mut() = focused;
        if focused {
            self.container.add_css_class("focused-pane");
            self.container.remove_css_class("unfocused-pane");
            self.term.borrow().widget().grab_focus();
        } else {
            self.container.remove_css_class("focused-pane");
            self.container.add_css_class("unfocused-pane");
        }
    }
}
