//! Safe Rust wrapper around vte-sys / libvte-2.91-gtk4.

use glib::translate::{from_glib_none, ToGlibPtr};
use gtk4::prelude::*;
use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::ptr;
use std::rc::Rc;
use vte_sys::VteTerminal;

use crate::config::TerminalConfig;

#[allow(dead_code)]
pub struct TerminalWidget {
    raw: *mut VteTerminal,
    widget: gtk4::Widget,
    config: TerminalConfig,
    current_font_size: u32,
    pub working_directory: String,
    /// Heap-allocated pid slot shared with the async spawn callback.
    /// Must be heap-stable: the callback fires after `new()` returns
    /// (and `Self` has been moved), so a pointer to a struct field
    /// would dangle. `Rc::into_raw` / `from_raw` pairs the ownership.
    pub child_pid: Rc<Cell<Option<i32>>>,
}

/// Resolve the live cwd of a child pid, falling back gracefully.
///
/// Pure helper extracted so it can be unit-tested headless (GTK-free).
/// Never panics: any failure (exited process, permission error,
/// non-directory target) returns `fallback`.
pub fn resolve_live_dir(pid_opt: Option<i32>, fallback: &str) -> String {
    if let Some(pid) = pid_opt {
        if pid > 0 {
            let link = format!("/proc/{}/cwd", pid);
            if let Ok(path) = std::fs::read_link(&link) {
                if path.exists() && path.is_dir() {
                    return path.to_string_lossy().to_string();
                }
            }
        }
    }
    fallback.to_string()
}

#[allow(dead_code)]
impl TerminalWidget {
    pub fn new(config: &TerminalConfig, working_dir: String, command: Option<String>) -> Self {
        let raw = unsafe { vte_sys::vte_terminal_new() };
        let widget: gtk4::Widget = unsafe { from_glib_none(raw as *mut gtk4::ffi::GtkWidget) };

        widget.set_hexpand(true);
        widget.set_vexpand(true);

        let mut term = Self {
            raw,
            widget,
            config: config.clone(),
            current_font_size: config.font_size,
            working_directory: working_dir,
            child_pid: Rc::new(Cell::new(None)),
        };

        term.apply_colors();
        term.apply_font();
        term.apply_properties();
        term.spawn_shell(command);

        term
    }

    pub fn widget(&self) -> &gtk4::Widget {
        &self.widget
    }

    /// Live cwd of the shell in this pane: `/proc/<child_pid>/cwd`
    /// when available, else the spawn-time `working_directory`.
    /// Never panics and never blocks the UI (single `read_link`).
    pub fn current_directory(&self) -> String {
        resolve_live_dir(self.child_pid.get(), &self.working_directory)
    }

    pub fn raw(&self) -> *mut VteTerminal {
        self.raw
    }

    fn apply_properties(&self) {
        unsafe {
            vte_sys::vte_terminal_set_scrollback_lines(self.raw, self.config.scrollback_lines);
            vte_sys::vte_terminal_set_mouse_autohide(self.raw, glib::ffi::GTRUE);
        }
    }

    pub fn apply_colors(&self) {
        let theme = crate::theme::find_theme(&self.config.theme);
        self.apply_theme(&theme);
    }

    pub fn apply_theme(&self, theme: &crate::theme::ColorTheme) {
        let fg = gtk4::gdk::RGBA::parse(&theme.foreground).unwrap_or_else(|_| gtk4::gdk::RGBA::parse("#e4e4e7").unwrap());
        let bg = gtk4::gdk::RGBA::parse(&theme.background).unwrap_or_else(|_| gtk4::gdk::RGBA::parse("#09090b").unwrap());

        let palette: Vec<gtk4::gdk::RGBA> = theme
            .palette
            .iter()
            .map(|h| gtk4::gdk::RGBA::parse(h).unwrap_or_else(|_| gtk4::gdk::RGBA::parse("#ffffff").unwrap()))
            .collect();

        let raw_palette: Vec<gtk4::gdk::ffi::GdkRGBA> =
            palette.iter().map(|c| unsafe { *c.to_glib_none().0 }).collect();

        unsafe {
            vte_sys::vte_terminal_set_colors(
                self.raw,
                fg.to_glib_none().0,
                bg.to_glib_none().0,
                raw_palette.as_ptr(),
                raw_palette.len(),
            );
        }
    }

    pub fn set_theme_name(&mut self, theme_name: &str) {
        self.config.theme = theme_name.to_string();
    }

    pub fn set_font_config(&mut self, family: &str, size: u32) {
        self.config.font_family = family.to_string();
        self.config.font_size = size;
        self.current_font_size = size;
        self.apply_font();
    }

    pub fn apply_font(&self) {
        let font_str = format!("{} {}", self.config.font_family, self.current_font_size);
        let font_desc = gtk4::pango::FontDescription::from_string(&font_str);
        unsafe {
            vte_sys::vte_terminal_set_font(self.raw, font_desc.to_glib_none().0);
        }
    }

    pub fn zoom_in(&mut self) {
        if self.current_font_size < 36 {
            self.current_font_size += 1;
            self.apply_font();
        }
    }

    pub fn zoom_out(&mut self) {
        if self.current_font_size > 6 {
            self.current_font_size -= 1;
            self.apply_font();
        }
    }

    pub fn zoom_reset(&mut self) {
        self.current_font_size = self.config.font_size;
        self.apply_font();
    }

    pub fn copy(&self) {
        unsafe {
            vte_sys::vte_terminal_copy_clipboard_format(self.raw, vte_sys::VTE_FORMAT_TEXT);
        }
    }

    pub fn paste(&self) {
        unsafe {
            vte_sys::vte_terminal_paste_clipboard(self.raw);
        }
    }

    /// Write raw bytes to the child process, as if typed.
    pub fn feed_child(&self, bytes: &[u8]) {
        unsafe {
            vte_sys::vte_terminal_feed_child(self.raw, bytes.as_ptr(), bytes.len() as isize);
        }
    }

    pub fn window_title(&self) -> Option<String> {
        unsafe {
            let ptr = vte_sys::vte_terminal_get_window_title(self.raw);
            if !ptr.is_null() {
                Some(CStr::from_ptr(ptr).to_string_lossy().to_string())
            } else {
                None
            }
        }
    }

    pub fn spawn_shell(&mut self, command: Option<String>) {
        let shell = if !self.config.default_shell.is_empty() {
            self.config.default_shell.clone()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"))
        };

        let cwd_c = CString::new(self.working_directory.as_str()).unwrap_or_default();
        let shell_c = CString::new(shell.as_str()).unwrap_or_default();

        let mut argv_c: Vec<CString> = Vec::new();
        if let Some(cmd) = command {
            argv_c.push(shell_c);
            argv_c.push(CString::new("-c").unwrap());
            argv_c.push(CString::new(cmd).unwrap());
        } else {
            argv_c.push(shell_c);
        }

        let mut argv_ptrs: Vec<*mut std::os::raw::c_char> =
            argv_c.iter().map(|s| s.as_ptr() as *mut _).collect();
        argv_ptrs.push(ptr::null_mut());

        // The callback fires on the GTK main loop after `new()` has
        // returned and `Self` has been moved into its `Rc`, so the pid
        // slot must live on the heap. `into_raw` adds one ref that the
        // callback reclaims with `from_raw` (it fires exactly once).
        unsafe extern "C" fn on_spawn_cb(
            _terminal: *mut VteTerminal,
            pid: glib::ffi::GPid,
            _error: *mut glib::ffi::GError,
            user_data: glib::ffi::gpointer,
        ) {
            if user_data.is_null() {
                return;
            }
            let cell = user_data as *const Cell<Option<i32>>;
            if pid > 0 {
                (*cell).set(Some(pid));
            }
            let _ = Rc::from_raw(cell);
        }

        let pid_sink = Rc::into_raw(self.child_pid.clone()) as glib::ffi::gpointer;

        unsafe {
            vte_sys::vte_terminal_spawn_async(
                self.raw,
                vte_sys::VTE_PTY_DEFAULT,
                cwd_c.as_ptr(),
                argv_ptrs.as_mut_ptr(),
                ptr::null_mut(),
                glib::ffi::G_SPAWN_DEFAULT,
                None,
                ptr::null_mut(),
                None,
                -1,
                ptr::null_mut(),
                Some(on_spawn_cb),
                pid_sink,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_live_dir;

    #[test]
    fn none_pid_returns_fallback() {
        assert_eq!(resolve_live_dir(None, "/home/user"), "/home/user");
    }

    #[test]
    fn invalid_pid_returns_fallback() {
        assert_eq!(resolve_live_dir(Some(0), "/fallback"), "/fallback");
        assert_eq!(resolve_live_dir(Some(-5), "/fallback"), "/fallback");
        // Non-existent pid: read_link fails -> fallback.
        assert_eq!(resolve_live_dir(Some(i32::MAX), "/fallback"), "/fallback");
    }

    #[test]
    fn own_process_pid_returns_live_dir() {
        let pid = std::process::id() as i32;
        let got = resolve_live_dir(Some(pid), "/fallback");
        let path = std::path::Path::new(&got);
        assert!(path.exists() && path.is_dir(), "got: {}", got);
    }

    /// Regression test: the async spawn callback must record the child
    /// pid so `current_directory()` tracks `cd` (previously the callback
    /// wrote to a moved stack address, pid stayed `None`, and splits
    /// always reopened in the spawn-time dir). Needs a display; run
    /// under `xvfb-run` headless.
    #[test]
    fn spawn_captures_pid_and_tracks_live_cwd() {
        let _ = gtk4::init();
        if gtk4::gdk::Display::default().is_none() {
            eprintln!("SKIP: no display available");
            return;
        }
        let mut cfg = super::super::config::TerminalConfig::default();
        cfg.default_shell = String::from("/bin/bash");
        // `cd /` first: live cwd must differ from the spawn dir (/tmp).
        let term =
            super::TerminalWidget::new(&cfg, String::from("/tmp"), Some(String::from("cd / && sleep 10")));
        let ctx = glib::MainContext::default();
        for _ in 0..500 {
            if term.child_pid.get().is_some() {
                break;
            }
            ctx.iteration(false);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let pid = term
            .child_pid
            .get()
            .expect("spawn callback must record child pid");
        assert!(pid > 0, "pid: {pid}");
        let mut live = String::new();
        for _ in 0..200 {
            ctx.iteration(false);
            live = term.current_directory();
            if live == "/" {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        assert_eq!(live, "/", "shell did cd / but live cwd is {live}");
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .output();
    }
}
