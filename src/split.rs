//! Split container managing horizontal and vertical splits for a tab.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::config::TerminalConfig;
use crate::model::{LayoutNode, Orientation};
use crate::pane::Pane;
use crate::session::{clamp_ratio as clamp_session_ratio, resolve_working_dir};

pub struct TabView {
    pub container: gtk4::Box,
    pub panes: Rc<RefCell<HashMap<String, Rc<Pane>>>>,
    pub focused_pane_id: Rc<RefCell<Option<String>>>,
    pane_order: Rc<RefCell<Vec<String>>>,
    config: RefCell<TerminalConfig>,
    on_change: RefCell<Option<Rc<dyn Fn()>>>,
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
            on_change: RefCell::new(None),
        });

        tab_view.add_initial_pane(working_dir);
        tab_view
    }

    /// Register a callback fired on structural/focus/ratio changes so
    /// `MainWindow` can debounce-persist the session.
    pub fn set_on_change(&self, cb: Rc<dyn Fn()>) {
        *self.on_change.borrow_mut() = Some(cb);
    }

    fn notify_changed(&self) {
        if let Some(cb) = self.on_change.borrow().clone() {
            cb();
        }
    }

    /// Pure ratio clamp used by snapshot/restore (testable headless).
    pub fn clamp_ratio(ratio: f64) -> f64 {
        clamp_session_ratio(ratio)
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
            // Drag-resize should persist (debounced by MainWindow).
            {
                let weak_self = Rc::downgrade(self);
                paned.connect_position_notify(move |_| {
                    if let Some(s) = weak_self.upgrade() {
                        s.notify_changed();
                    }
                });
            }

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
        self.notify_changed();
    }

    pub fn focus_pane(&self, pane_id: &str) {
        *self.focused_pane_id.borrow_mut() = Some(pane_id.to_string());
        for (id, p) in self.panes.borrow().iter() {
            p.set_focused(id == pane_id);
        }
        self.notify_changed();
    }

    /// Walk the live GTK tree and produce a layout-only snapshot.
    /// `Box` child = leaf (cwd from the owning pane); `Paned` = split
    /// with ratio = position / allocation clamped 0.1-0.9 (0.5 fallback
    /// when unrealized). Never includes commands.
    pub fn snapshot_layout(&self) -> LayoutNode {
        let root = self.container.first_child();
        match root {
            Some(w) => self.snapshot_widget(&w).unwrap_or_default(),
            None => LayoutNode::default(),
        }
    }

    fn snapshot_widget(&self, widget: &gtk4::Widget) -> Option<LayoutNode> {
        if let Some(paned) = widget.downcast_ref::<gtk4::Paned>() {
            let gtk_orient = paned.orientation();
            let orientation = if gtk_orient == gtk4::Orientation::Horizontal {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            };
            let total = if gtk_orient == gtk4::Orientation::Horizontal {
                paned.width()
            } else {
                paned.height()
            };
            let ratio = if total > 0 {
                Self::clamp_ratio(paned.position() as f64 / total as f64)
            } else {
                0.5
            };
            let first = paned.start_child().and_then(|w| self.snapshot_widget(&w));
            let second = paned.end_child().and_then(|w| self.snapshot_widget(&w));
            match (first, second) {
                (Some(f), Some(s)) => Some(LayoutNode::Split {
                    orientation,
                    ratio,
                    first: Box::new(f),
                    second: Box::new(s),
                }),
                (Some(f), None) => Some(f),
                (None, Some(s)) => Some(s),
                (None, None) => None,
            }
        } else {
            // Leaf: find the owning pane by container identity.
            let panes = self.panes.borrow();
            let found = panes.values().find(|p| {
                p.container.upcast_ref::<gtk4::Widget>() == widget
            });
            match found {
                Some(pane) => Some(LayoutNode::Leaf {
                    pane_id: pane.id.clone(),
                    working_directory: pane.term.borrow().working_directory.clone(),
                    command: None,
                }),
                None => {
                    // Direct pane container nested in root Box (single pane):
                    // the widget itself may be the pane container.
                    None
                }
            }
        }
    }

    /// Rebuild a TabView from a saved layout, spawning one fresh shell
    /// per leaf in the validated saved cwd. Never executes `command`.
    pub fn from_layout(config: &TerminalConfig, layout: &LayoutNode) -> Rc<Self> {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        let tab_view = Rc::new(Self {
            container,
            panes: Rc::new(RefCell::new(HashMap::new())),
            focused_pane_id: Rc::new(RefCell::new(None)),
            pane_order: Rc::new(RefCell::new(Vec::new())),
            config: RefCell::new(config.clone()),
            on_change: RefCell::new(None),
        });

        let mut budget = crate::session::MAX_LEAVES;
        if let Some(widget) = tab_view.build_widget(layout, &mut budget) {
            tab_view.container.append(&widget);
        } else {
            tab_view.add_initial_pane(resolve_working_dir(""));
        }
        // Focus first pane.
        if let Some(first_id) = tab_view.pane_order.borrow().first().cloned() {
            tab_view.focus_pane_silent(&first_id);
        }
        tab_view
    }

    /// Focus without firing on_change (used during restore build).
    fn focus_pane_silent(&self, pane_id: &str) {
        *self.focused_pane_id.borrow_mut() = Some(pane_id.to_string());
        for (id, p) in self.panes.borrow().iter() {
            p.set_focused(id == pane_id);
        }
    }

    fn spawn_pane(self: &Rc<Self>, working_dir: String) -> Rc<Pane> {
        let weak_self = Rc::downgrade(self);
        let weak_self_close = Rc::downgrade(self);
        let weak_self_focus = Rc::downgrade(self);
        let cfg = self.config.borrow().clone();
        let pane = Rc::new(Pane::new(
            &cfg,
            working_dir,
            None, // never restore/execute commands (PRD §7)
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
        self.panes.borrow_mut().insert(pane.id.clone(), pane.clone());
        self.pane_order.borrow_mut().push(pane.id.clone());
        pane
    }

    fn build_widget(
        self: &Rc<Self>,
        node: &LayoutNode,
        budget: &mut usize,
    ) -> Option<gtk4::Widget> {
        match node {
            LayoutNode::Leaf {
                working_directory, ..
            } => {
                if *budget == 0 {
                    return None;
                }
                *budget -= 1;
                let dir = resolve_working_dir(working_directory);
                let pane = self.spawn_pane(dir);
                Some(pane.container.clone().upcast())
            }
            LayoutNode::Split {
                orientation,
                ratio,
                first,
                second,
            } => {
                let gtk_orient = match orientation {
                    Orientation::Vertical => gtk4::Orientation::Horizontal,
                    Orientation::Horizontal => gtk4::Orientation::Vertical,
                };
                let a = self.build_widget(first, budget);
                let b = self.build_widget(second, budget);
                match (a, b) {
                    (Some(f), Some(s)) => {
                        let paned = gtk4::Paned::new(gtk_orient);
                        paned.set_hexpand(true);
                        paned.set_vexpand(true);
                        paned.set_shrink_start_child(false);
                        paned.set_shrink_end_child(false);
                        paned.set_start_child(Some(&f));
                        paned.set_end_child(Some(&s));
                        {
                            let weak_self = Rc::downgrade(self);
                            paned.connect_position_notify(move |_| {
                                if let Some(tv) = weak_self.upgrade() {
                                    tv.notify_changed();
                                }
                            });
                        }
                        // Apply saved ratio once realized.
                        let r = Self::clamp_ratio(*ratio);
                        let paned_weak = paned.downgrade();
                        glib::idle_add_local_once(move || {
                            if let Some(paned) = paned_weak.upgrade() {
                                let total = if paned.orientation() == gtk4::Orientation::Horizontal {
                                    paned.width()
                                } else {
                                    paned.height()
                                };
                                if total > 0 {
                                    paned.set_position((total as f64 * r) as i32);
                                }
                            }
                        });
                        Some(paned.upcast())
                    }
                    (Some(f), None) => Some(f),
                    (None, Some(s)) => Some(s),
                    (None, None) => None,
                }
            }
        }
    }
    pub fn focus_next_pane(&self, direction: i32) {        let order = self.pane_order.borrow().clone();
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
