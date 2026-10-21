//! Ubuntu top-bar indicator (StatusNotifierItem via `ksni`).
//!
//! Closing the window hides it here instead of quitting, so terminals
//! keep running. The indicator menu owns Show, New Workspace, Settings
//! and Quit. Actions travel to the GTK main loop over a std `mpsc`
//! channel drained by a lightweight poller, because tray callbacks run
//! on the indicator runtime thread.

use std::sync::mpsc::Sender;

use ksni::blocking::TrayMethods;
use ksni::menu::{MenuItem, StandardItem};

#[derive(Debug, Clone, Copy)]
pub enum TrayAction {
    ShowWindow,
    NewWorkspace,
    OpenSettings,
    Quit,
}

#[derive(Debug)]
pub struct RmuxTray {
    actions: Sender<TrayAction>,
}

impl RmuxTray {
    pub fn spawn(actions: Sender<TrayAction>) -> Result<(), ksni::Error> {
        Self { actions }.spawn().map(|_| ())
    }
}

fn menu_entry(label: &str, sender: Sender<TrayAction>, action: TrayAction) -> MenuItem<RmuxTray> {
    let tx = sender;
    StandardItem {
        label: label.into(),
        activate: Box::new(move |_| {
            let _ = tx.send(action);
        }),
        ..Default::default()
    }
    .into()
}

impl ksni::Tray for RmuxTray {
    fn id(&self) -> String {
        "rmux".into()
    }

    fn title(&self) -> String {
        "rmux".into()
    }

    /// Themed icon installed by `make install-user` (`rmux.svg` in hicolor).
    fn icon_name(&self) -> String {
        "rmux".into()
    }

    /// Left-click restores the window.
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.actions.send(TrayAction::ShowWindow);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut new_item: MenuItem<Self> = menu_entry("New Workspace", self.actions.clone(), TrayAction::NewWorkspace);
        if let MenuItem::Standard(ref mut std) = new_item {
            std.shortcut = vec![vec!["Control".into(), "Shift".into(), "N".into()]];
        }
        vec![
            menu_entry("Show Window", self.actions.clone(), TrayAction::ShowWindow),
            new_item,
            menu_entry("Settings", self.actions.clone(), TrayAction::OpenSettings),
            MenuItem::Separator,
            menu_entry("Quit", self.actions.clone(), TrayAction::Quit),
        ]
    }
}
