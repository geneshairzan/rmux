//! Safe Rust wrapper around vte-sys / libvte-2.91-gtk4.

use glib::translate::{from_glib_none, ToGlibPtr};
use gtk4::prelude::*;
use std::ffi::{CStr, CString};
use std::ptr;
use vte_sys::VteTerminal;

use crate::config::TerminalConfig;

#[allow(dead_code)]
pub struct TerminalWidget {
    raw: *mut VteTerminal,
    widget: gtk4::Widget,
    config: TerminalConfig,
    current_font_size: u32,
    pub working_directory: String,
    pub child_pid: Option<i32>,
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
            child_pid: None,
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

        unsafe extern "C" fn on_spawn_cb(
            _terminal: *mut VteTerminal,
            pid: glib::ffi::GPid,
            _error: *mut glib::ffi::GError,
            user_data: glib::ffi::gpointer,
        ) {
            if !user_data.is_null() {
                let pid_ptr = user_data as *mut Option<i32>;
                *pid_ptr = Some(pid);
            }
        }

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
                &mut self.child_pid as *mut _ as glib::ffi::gpointer,
            );
        }
    }
}
