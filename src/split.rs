//! Split container managing horizontal and vertical splits for a tab.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::config::TerminalConfig;
use crate::pane::Pane;

pub struct TabView {
    pub container: gtk4::Box,
    pub panes: Rc<RefCell<HashMap<String, Rc<Pane>>>>,
    pub focused_pane_id: Rc<RefCell<Option<String>>>,
    pane_order: Rc<RefCell<Vec<String>>>,
    config: RefCell<TerminalConfig>,
}

impl TabView {
    pub fn new(config: &TerminalConfig, working_dir: String) -> Rc<Self> {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        let panes = Rc::new(RefCell::new(HashMap::new()));
        let focused_pane_id = Rc::new(RefCell::new(None));
        let pane_order = Rc::new(RefCell::new(Vec::new()));

        let tab_view = Rc::new(Self {
            container,
            panes,
            focused_pane_id,
            pane_order,
            config: RefCell::new(config.clone()),
        });

        tab_view.add_initial_pane(working_dir);
        tab_view
    }

    pub fn set_theme(&self, theme_name: &str, theme_obj: &crate::theme::ColorTheme) {
        self.config.borrow_mut().theme = theme_name.to_string();
        for pane in self.panes.borrow().values() {
            pane.term.borrow_mut().set_theme_name(theme_name);
            pane.term.borrow().apply_theme(theme_obj);
        }
    }

    pub fn set_font(&self, family: &str, size: u32) {
        {
            let mut cfg = self.config.borrow_mut();
            cfg.font_family = family.to_string();
            cfg.font_size = size;
        }
        for pane in self.panes.borrow().values() {
            pane.term.borrow_mut().set_font_config(family, size);
        }
    }

    fn add_initial_pane(self: &Rc<Self>, working_dir: String) {
        let weak_self = Rc::downgrade(self);
        let weak_self_close = Rc::downgrade(self);
        let weak_self_focus = Rc::downgrade(self);
        let cfg = self.config.borrow().clone();

        let pane = Rc::new(Pane::new(
            &cfg,
            working_dir,
            None,
            move |target_id, orientation| {
                if let Some(s) = weak_self.upgrade() {
                    s.split_pane(target_id, orientation);
                }
            },
            move |target_id| {
                if let Some(s) = weak_self_close.upgrade() {
                    s.close_pane(target_id);
                }
            },
            move |target_id| {
                if let Some(s) = weak_self_focus.upgrade() {
                    s.focus_pane(target_id);
                }
            },
        ));

        *self.focused_pane_id.borrow_mut() = Some(pane.id.clone());
        self.panes.borrow_mut().insert(pane.id.clone(), pane.clone());
        self.pane_order.borrow_mut().push(pane.id.clone());
        self.container.append(&pane.container);
        pane.set_focused(true);
    }

    pub fn split_pane(self: &Rc<Self>, target_id: &str, orientation: &str) {
        let working_dir = {
            let panes = self.panes.borrow();
            panes
                .get(target_id)
                .map(|p| p.term.borrow().working_directory.clone())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().to_string_lossy().to_string())
        };

        let weak_self = Rc::downgrade(self);
        let weak_self_close = Rc::downgrade(self);
        let weak_self_focus = Rc::downgrade(self);
        let cfg = self.config.borrow().clone();

        let new_pane = Rc::new(Pane::new(
            &cfg,
            working_dir,
            None,
            move |t_id, orient| {
                if let Some(s) = weak_self.upgrade() {
                    s.split_pane(t_id, orient);
                }
            },
            move |t_id| {
                if let Some(s) = weak_self_close.upgrade() {
                    s.close_pane(t_id);
                }
            },
            move |t_id| {
                if let Some(s) = weak_self_focus.upgrade() {
                    s.focus_pane(t_id);
                }
            },
        ));

        let new_id = new_pane.id.clone();
        self.panes.borrow_mut().insert(new_id.clone(), new_pane.clone());
        {
            let mut order = self.pane_order.borrow_mut();
            if let Some(pos) = order.iter().position(|id| id == target_id) {
                order.insert(pos + 1, new_id.clone());
            } else {
                order.push(new_id.clone());
            }
        }

        // Find the target pane container and replace it with a Paned containing target and new
        let target_pane = self.panes.borrow().get(target_id).cloned();
        if let Some(target) = target_pane {
            let gtk_orient = if orientation == "vertical" {
                gtk4::Orientation::Horizontal
            } else {
                gtk4::Orientation::Vertical
            };

            let paned = gtk4::Paned::new(gtk_orient);
            paned.set_hexpand(true);
            paned.set_vexpand(true);
            paned.set_shrink_start_child(false);
            paned.set_shrink_end_child(false);

            // Even split: each child gets half of the focused pane's
            // current size, so 1 pane -> 50/50, then 50/25/25, and so on.
            let half_now = if orientation == "vertical" {
                target.container.width() / 2
            } else {
                target.container.height() / 2
            };
            // Re-parent target
            if let Some(parent) = target.container.parent() {
                if let Some(box_parent) = parent.downcast_ref::<gtk4::Box>() {
                    box_parent.remove(&target.container);
                    paned.set_start_child(Some(&target.container));
                    paned.set_end_child(Some(&new_pane.container));
                    if half_now > 0 {
                        paned.set_position(half_now);
                    }
                    box_parent.append(&paned);
                } else if let Some(paned_parent) = parent.downcast_ref::<gtk4::Paned>() {
                    let is_start = paned_parent.start_child().as_ref() == Some(&target.container.clone().upcast());
                    if is_start {
                        paned_parent.set_start_child(None::<&gtk4::Widget>);
                        paned_parent.set_start_child(Some(&paned));
                    } else {
                        paned_parent.set_end_child(None::<&gtk4::Widget>);
                        paned_parent.set_end_child(Some(&paned));
                    }
                    paned.set_start_child(Some(&target.container));
                    paned.set_end_child(Some(&new_pane.container));
                    if half_now > 0 {
                        paned.set_position(half_now);
                    }
                }
            }

            // If the target had no allocation yet (unrealized), center
            // once layout runs so the split still lands at 50/50.
            if half_now <= 0 {
                let paned_weak = paned.downgrade();
                let orient_owned = orientation.to_string();
                glib::idle_add_local_once(move || {
                    if let Some(paned) = paned_weak.upgrade() {
                        let half = if orient_owned == "vertical" {
                            paned.width() / 2
                        } else {
                            paned.height() / 2
                        };
                        if half > 0 {
                            paned.set_position(half);
                        }
                    }
                });
            }

            self.focus_pane(&new_id);
        }
    }

    pub fn close_pane(self: &Rc<Self>, target_id: &str) {
        if self.panes.borrow().len() <= 1 {
            return;
        }

        let pane = self.panes.borrow_mut().remove(target_id);
        if let Some(p) = pane {
            if let Some(parent) = p.container.parent() {
                if let Some(paned) = parent.downcast_ref::<gtk4::Paned>() {
                    let sibling = if paned.start_child().as_ref() == Some(&p.container.clone().upcast()) {
                        paned.end_child()
                    } else {
                        paned.start_child()
                    };

                    if let Some(sib) = sibling {
                        if let Some(grandparent) = paned.parent() {
                            // Unparent sib from paned before reparenting to grandparent
                            paned.set_start_child(None::<&gtk4::Widget>);
                            paned.set_end_child(None::<&gtk4::Widget>);

                            if let Some(gp_box) = grandparent.downcast_ref::<gtk4::Box>() {
                                gp_box.remove(paned);
                                gp_box.append(&sib);
                            } else if let Some(gp_paned) = grandparent.downcast_ref::<gtk4::Paned>() {
                                if gp_paned.start_child().as_ref() == Some(&paned.clone().upcast()) {
                                    gp_paned.set_start_child(Some(&sib));
                                } else {
                                    gp_paned.set_end_child(Some(&sib));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(first_id) = self.pane_order.borrow().first().cloned() {
            if self.panes.borrow().contains_key(&first_id) {
                self.focus_pane(&first_id);
            } else if let Some(any_id) = self.panes.borrow().keys().next().cloned() {
                self.focus_pane(&any_id);
            }
        }
        self.pane_order
            .borrow_mut()
            .retain(|id| id != target_id);
    }

    pub fn focus_pane(&self, pane_id: &str) {
        *self.focused_pane_id.borrow_mut() = Some(pane_id.to_string());
        for (id, p) in self.panes.borrow().iter() {
            p.set_focused(id == pane_id);
        }
    }

    pub fn focus_next_pane(&self, direction: i32) {
        let order = self.pane_order.borrow().clone();
        if order.len() <= 1 {
            return;
        }
        let current = self.focused_pane_id.borrow().clone().unwrap_or_default();
        let idx = order.iter().position(|id| *id == current).unwrap_or(0);
        let len = order.len() as i32;
        let next_idx = (idx as i32 + direction).rem_euclid(len) as usize;
        if let Some(next_id) = order.get(next_idx) {
            self.focus_pane(next_id);
        }
    }
}
