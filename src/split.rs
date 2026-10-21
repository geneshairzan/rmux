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
    config: TerminalConfig,
}

impl TabView {
    pub fn new(config: &TerminalConfig, working_dir: String) -> Rc<Self> {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        let panes = Rc::new(RefCell::new(HashMap::new()));
        let focused_pane_id = Rc::new(RefCell::new(None));

        let tab_view = Rc::new(Self {
            container,
            panes,
            focused_pane_id,
            config: config.clone(),
        });

        tab_view.add_initial_pane(working_dir);
        tab_view
    }

    fn add_initial_pane(self: &Rc<Self>, working_dir: String) {
        let weak_self = Rc::downgrade(self);
        let weak_self_close = Rc::downgrade(self);
        let weak_self_focus = Rc::downgrade(self);

        let pane = Rc::new(Pane::new(
            &self.config,
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

        let new_pane = Rc::new(Pane::new(
            &self.config,
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
            paned.set_wide_handle(true);

            // Re-parent target
            if let Some(parent) = target.container.parent() {
                if let Some(box_parent) = parent.downcast_ref::<gtk4::Box>() {
                    box_parent.remove(&target.container);
                    paned.set_start_child(Some(&target.container));
                    paned.set_end_child(Some(&new_pane.container));
                    paned.set_position(400);
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
                    paned.set_position(400);
                }
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

        if let Some(first_id) = self.panes.borrow().keys().next().cloned() {
            self.focus_pane(&first_id);
        }
    }

    pub fn focus_pane(&self, pane_id: &str) {
        *self.focused_pane_id.borrow_mut() = Some(pane_id.to_string());
        for (id, p) in self.panes.borrow().iter() {
            p.set_focused(id == pane_id);
        }
    }
}
