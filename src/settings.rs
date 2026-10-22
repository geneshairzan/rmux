//! Sleek Antigravity-styled 2-column modal settings dialog for rmux.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::AppConfig;
use crate::theme::get_available_themes;

pub fn show_settings_modal(
    parent: &libadwaita::ApplicationWindow,
    config_state: Rc<RefCell<AppConfig>>,
    on_theme_applied: impl Fn(String) + 'static + Clone,
    on_font_applied: impl Fn(String, u32) + 'static + Clone,
) -> gtk4::Window {
    let dialog = gtk4::Window::new();
    dialog.set_title(Some("Settings"));
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_default_size(780, 560);
    dialog.add_css_class("antigravity-modal");

    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    // ========================================================
    // LEFT COLUMN: Navigation Sidebar (Antigravity Style)
    // ========================================================
    let left_col = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    left_col.add_css_class("antigravity-settings-sidebar");
    left_col.set_size_request(210, -1);

    let stack = gtk4::Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    let nav_items = [
        ("themes", "Appearance"),
        ("shortcuts", "Shortcuts"),
        ("terminal", "Terminal & Shell"),
        ("about", "About rmux"),
    ];

    let nav_buttons: Rc<RefCell<Vec<(String, gtk4::Button)>>> = Rc::new(RefCell::new(Vec::new()));

    for (page_id, label_text) in nav_items {
        let btn = gtk4::Button::new();
        let nav_lbl = gtk4::Label::new(Some(label_text));
        nav_lbl.set_halign(gtk4::Align::Start);
        nav_lbl.set_hexpand(true);
        btn.set_child(Some(&nav_lbl));
        btn.add_css_class("flat");
        btn.add_css_class("antigravity-nav-btn");
        btn.set_halign(gtk4::Align::Fill);

        if page_id == "themes" {
            btn.add_css_class("active-nav");
        }

        let p_id = page_id.to_string();
        let stack_clone = stack.clone();
        let btns_clone = nav_buttons.clone();

        btn.connect_clicked(move |_| {
            stack_clone.set_visible_child_name(&p_id);
            for (id, b) in btns_clone.borrow().iter() {
                if id == &p_id {
                    b.add_css_class("active-nav");
                } else {
                    b.remove_css_class("active-nav");
                }
            }
        });

        left_col.append(&btn);
        nav_buttons.borrow_mut().push((page_id.to_string(), btn));
    }

    main_box.append(&left_col);

    // Vertical Divider
    let divider = gtk4::Separator::new(gtk4::Orientation::Vertical);
    divider.add_css_class("antigravity-divider");
    main_box.append(&divider);

    // ========================================================
    // RIGHT COLUMN: Content Panel (Antigravity Style)
    // ========================================================
    let right_col = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    right_col.set_hexpand(true);
    right_col.set_vexpand(true);
    right_col.set_margin_top(12);
    right_col.add_css_class("antigravity-settings-content");

    // Content starts directly — the native window decoration owns the
    // single close button, so no in-content close row is rendered.
    // ----------------------------------------------------
    // PAGE 1: APPEARANCE & THEMES
    // ----------------------------------------------------
    let page_themes = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    page_themes.set_margin_start(28);
    page_themes.set_margin_end(28);
    page_themes.set_margin_top(8);
    page_themes.set_margin_bottom(24);

    let th_title = gtk4::Label::new(Some("Appearance"));
    th_title.add_css_class("antigravity-content-title");
    th_title.set_halign(gtk4::Align::Start);
    page_themes.append(&th_title);

    let th_desc = gtk4::Label::new(Some("Select your preferred color theme. Changes take effect across all active terminals immediately."));
    th_desc.add_css_class("antigravity-content-desc");
    th_desc.set_halign(gtk4::Align::Start);
    th_desc.set_wrap(true);
    page_themes.append(&th_desc);

    let themes_card = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    themes_card.add_css_class("antigravity-card");

    let all_themes = get_available_themes();
    let current_theme = config_state.borrow().appearance.theme.clone();
    let theme_check_widgets: Rc<RefCell<Vec<(String, gtk4::Widget)>>> = Rc::new(RefCell::new(Vec::new()));

    for (i, theme) in all_themes.iter().enumerate() {
        let t_name = theme.name.clone();
        let item_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        item_box.add_css_class("antigravity-card-row");
        item_box.set_margin_start(16);
        item_box.set_margin_end(16);
        item_box.set_margin_top(12);
        item_box.set_margin_bottom(12);

        let lbl = gtk4::Label::new(Some(&theme.name));
        lbl.add_css_class("antigravity-row-title");
        lbl.set_halign(gtk4::Align::Start);
        lbl.set_hexpand(true);
        item_box.append(&lbl);

        // Color preview dots
        let dots_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let sample_colors = [
            &theme.background,
            &theme.foreground,
            &theme.palette[1],
            &theme.palette[2],
            &theme.palette[4],
            &theme.palette[5],
        ];

        for (j, col) in sample_colors.iter().enumerate() {
            let dot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            dot.set_size_request(14, 14);
            let css = format!("border-radius: 7px; background-color: {}; border: 1px solid rgba(255,255,255,0.12);", col);
            let prov = gtk4::CssProvider::new();
            let class_name = format!("theme-dot-{}-{}", i, j);
            prov.load_from_string(&format!(".{} {{ {} }}", class_name, css));
            dot.add_css_class(&class_name);
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(&display, &prov, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
            }
            dots_box.append(&dot);
        }
        item_box.append(&dots_box);

        // Active checkmark
        let check = gtk4::Image::from_icon_name("object-select-symbolic");
        check.add_css_class("antigravity-check-icon");
        if theme.name.eq_ignore_ascii_case(&current_theme) {
            check.set_opacity(1.0);
        } else {
            check.set_opacity(0.0);
        }
        item_box.append(&check);
        theme_check_widgets.borrow_mut().push((t_name.clone(), check.upcast()));

        // Click on row to activate theme
        let gesture = gtk4::GestureClick::new();
        let cfg_clone = config_state.clone();
        let on_th = on_theme_applied.clone();
        let t_click = t_name.clone();
        let checks_ref = theme_check_widgets.clone();

        gesture.connect_pressed(move |_, _, _, _| {
            {
                let mut cfg = cfg_clone.borrow_mut();
                cfg.appearance.theme = t_click.clone();
                cfg.terminal.theme = t_click.clone();
                cfg.save();
            }
            on_th(t_click.clone());

            for (name, w) in checks_ref.borrow().iter() {
                if name.eq_ignore_ascii_case(&t_click) {
                    w.set_opacity(1.0);
                } else {
                    w.set_opacity(0.0);
                }
            }
        });
        item_box.add_controller(gesture);

        themes_card.append(&item_box);

        if i < all_themes.len() - 1 {
            let row_sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            row_sep.add_css_class("antigravity-card-sep");
            themes_card.append(&row_sep);
        }
    }

    let th_scroll = gtk4::ScrolledWindow::new();
    th_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    th_scroll.set_hexpand(true);
    th_scroll.set_vexpand(true);
    th_scroll.set_child(Some(&themes_card));
    page_themes.append(&th_scroll);

    stack.add_named(&page_themes, Some("themes"));

    // ----------------------------------------------------
    // PAGE 2: KEYBOARD SHORTCUTS
    // ----------------------------------------------------
    let page_shortcuts = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    page_shortcuts.set_margin_start(28);
    page_shortcuts.set_margin_end(28);
    page_shortcuts.set_margin_top(8);
    page_shortcuts.set_margin_bottom(24);

    let sc_title = gtk4::Label::new(Some("Keyboard Shortcuts"));
    sc_title.add_css_class("antigravity-content-title");
    sc_title.set_halign(gtk4::Align::Start);
    page_shortcuts.append(&sc_title);

    let sc_desc = gtk4::Label::new(Some("Customize your workflow keybindings. Click Edit on any row and press the desired combination."));
    sc_desc.add_css_class("antigravity-content-desc");
    sc_desc.set_halign(gtk4::Align::Start);
    sc_desc.set_wrap(true);
    page_shortcuts.append(&sc_desc);

    let sc_card = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sc_card.add_css_class("antigravity-card");

    let actions: [(&str, &str, fn(&AppConfig) -> String, fn(&mut AppConfig, String)); 9] = [
        ("New Workspace", "Create and switch to a fresh workspace", |c| c.shortcuts.new_workspace.clone(), |c, s| c.shortcuts.new_workspace = s),
        ("Close All Open Tabs", "Close all open workspaces and reset", |c| c.shortcuts.close_all_tabs.clone(), |c, s| c.shortcuts.close_all_tabs = s),
        ("Close Focused Pane", "Close active terminal pane", |c| c.shortcuts.close_pane.clone(), |c, s| c.shortcuts.close_pane = s),
        ("Split Right", "Split current pane side-by-side", |c| c.shortcuts.split_right.clone(), |c, s| c.shortcuts.split_right = s),
        ("Split Down", "Split current pane top-and-bottom", |c| c.shortcuts.split_down.clone(), |c, s| c.shortcuts.split_down = s),
        ("Toggle Sidebar", "Show or hide the workspaces sidebar", |c| c.shortcuts.toggle_sidebar.clone(), |c, s| c.shortcuts.toggle_sidebar = s),
        ("Zoom In", "Increase terminal font size", |c| c.shortcuts.zoom_in.clone(), |c, s| c.shortcuts.zoom_in = s),
        ("Zoom Out", "Decrease terminal font size", |c| c.shortcuts.zoom_out.clone(), |c, s| c.shortcuts.zoom_out = s),
        ("Reset Zoom", "Reset terminal font size to default", |c| c.shortcuts.zoom_reset.clone(), |c, s| c.shortcuts.zoom_reset = s),
    ];

    let currently_editing: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
    let badge_labels: Rc<RefCell<Vec<gtk4::Label>>> = Rc::new(RefCell::new(Vec::new()));
    let edit_buttons: Rc<RefCell<Vec<gtk4::Button>>> = Rc::new(RefCell::new(Vec::new()));

    for (idx, (title, desc, getter, _)) in actions.iter().enumerate() {
        let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row_box.add_css_class("antigravity-card-row");
        row_box.set_margin_start(16);
        row_box.set_margin_end(16);
        row_box.set_margin_top(10);
        row_box.set_margin_bottom(10);

        let text_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        text_vbox.set_hexpand(true);

        let t_lbl = gtk4::Label::new(Some(title));
        t_lbl.add_css_class("antigravity-row-title");
        t_lbl.set_halign(gtk4::Align::Start);
        text_vbox.append(&t_lbl);

        let d_lbl = gtk4::Label::new(Some(desc));
        d_lbl.add_css_class("antigravity-row-desc");
        d_lbl.set_halign(gtk4::Align::Start);
        text_vbox.append(&d_lbl);

        row_box.append(&text_vbox);

        let cur_key = getter(&config_state.borrow());
        let badge = gtk4::Label::new(Some(&cur_key));
        badge.add_css_class("antigravity-key-badge");
        row_box.append(&badge);
        badge_labels.borrow_mut().push(badge.clone());

        let btn_edit = gtk4::Button::with_label("Edit");
        btn_edit.add_css_class("flat");
        btn_edit.add_css_class("antigravity-edit-btn");

        let ed_cell = currently_editing.clone();
        let ed_btns = edit_buttons.clone();
        let b_idx = idx;

        btn_edit.connect_clicked(move |b| {
            let mut ed = ed_cell.borrow_mut();
            if *ed == Some(b_idx) {
                *ed = None;
                b.set_label("Edit");
                b.remove_css_class("active-edit");
            } else {
                if let Some(prev) = *ed {
                    if let Some(prev_btn) = ed_btns.borrow().get(prev) {
                        prev_btn.set_label("Edit");
                        prev_btn.remove_css_class("active-edit");
                    }
                }
                *ed = Some(b_idx);
                b.set_label("Press keys...");
                b.add_css_class("active-edit");
            }
        });

        edit_buttons.borrow_mut().push(btn_edit.clone());
        row_box.append(&btn_edit);

        sc_card.append(&row_box);

        if idx < actions.len() - 1 {
            let row_sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            row_sep.add_css_class("antigravity-card-sep");
            sc_card.append(&row_sep);
        }
    }

    let sc_scroll = gtk4::ScrolledWindow::new();
    sc_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    sc_scroll.set_hexpand(true);
    sc_scroll.set_vexpand(true);
    sc_scroll.set_child(Some(&sc_card));
    page_shortcuts.append(&sc_scroll);

    let sc_footer = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    sc_footer.set_halign(gtk4::Align::End);
    let btn_reset = gtk4::Button::with_label("Reset to Defaults");
    btn_reset.add_css_class("flat");
    btn_reset.add_css_class("antigravity-reset-btn");

    let cfg_res = config_state.clone();
    let badges_res = badge_labels.clone();
    btn_reset.connect_clicked(move |_| {
        {
            let mut cfg = cfg_res.borrow_mut();
            cfg.shortcuts = crate::config::ShortcutsConfig::default();
            cfg.save();
        }
        let cfg = cfg_res.borrow();
        for (i, (_, _, getter, _)) in actions.iter().enumerate() {
            if let Some(badge) = badges_res.borrow().get(i) {
                badge.set_text(&getter(&cfg));
            }
        }
    });
    sc_footer.append(&btn_reset);
    page_shortcuts.append(&sc_footer);

    stack.add_named(&page_shortcuts, Some("shortcuts"));

    // Key capturing controller for shortcut editing
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);

    let ed_listen = currently_editing.clone();
    let cfg_listen = config_state.clone();
    let badges_listen = badge_labels.clone();
    let btns_listen = edit_buttons.clone();

    key_controller.connect_key_pressed(move |_, keyval, _, state_flags| {
        let editing_opt = *ed_listen.borrow();
        if let Some(target_idx) = editing_opt {
            let key = keyval.to_lower();
            if key == gtk4::gdk::Key::Control_L
                || key == gtk4::gdk::Key::Control_R
                || key == gtk4::gdk::Key::Shift_L
                || key == gtk4::gdk::Key::Shift_R
                || key == gtk4::gdk::Key::Alt_L
                || key == gtk4::gdk::Key::Alt_R
                || key == gtk4::gdk::Key::Super_L
                || key == gtk4::gdk::Key::Super_R
            {
                return glib::Propagation::Proceed;
            }

            let mut parts = Vec::new();
            if state_flags.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                parts.push("Ctrl");
            }
            if state_flags.contains(gtk4::gdk::ModifierType::ALT_MASK) {
                parts.push("Alt");
            }
            if state_flags.contains(gtk4::gdk::ModifierType::SHIFT_MASK) {
                parts.push("Shift");
            }

            let key_name = match key {
                gtk4::gdk::Key::plus | gtk4::gdk::Key::equal => "Plus".to_string(),
                gtk4::gdk::Key::minus => "Minus".to_string(),
                gtk4::gdk::Key::_0 => "0".to_string(),
                gtk4::gdk::Key::Return => "Enter".to_string(),
                gtk4::gdk::Key::Escape => {
                    *ed_listen.borrow_mut() = None;
                    if let Some(btn) = btns_listen.borrow().get(target_idx) {
                        btn.set_label("Edit");
                        btn.remove_css_class("active-edit");
                    }
                    return glib::Propagation::Stop;
                }
                _ => {
                    if let Some(ch) = keyval.to_unicode() {
                        ch.to_ascii_uppercase().to_string()
                    } else {
                        return glib::Propagation::Proceed;
                    }
                }
            };

            parts.push(&key_name);
            let shortcut_str = parts.join("+");

            {
                let mut cfg = cfg_listen.borrow_mut();
                let (_, _, _, setter) = actions[target_idx];
                setter(&mut cfg, shortcut_str.clone());
                cfg.save();
            }

            if let Some(badge) = badges_listen.borrow().get(target_idx) {
                badge.set_text(&shortcut_str);
            }

            if let Some(btn) = btns_listen.borrow().get(target_idx) {
                btn.set_label("Edit");
                btn.remove_css_class("active-edit");
            }
            *ed_listen.borrow_mut() = None;

            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });
    dialog.add_controller(key_controller);

    // ----------------------------------------------------
    // PAGE 3: TERMINAL & SHELL
    // ----------------------------------------------------
    let page_terminal = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    page_terminal.set_margin_start(28);
    page_terminal.set_margin_end(28);
    page_terminal.set_margin_top(8);
    page_terminal.set_margin_bottom(24);

    let tm_title = gtk4::Label::new(Some("Terminal & Shell"));
    tm_title.add_css_class("antigravity-content-title");
    tm_title.set_halign(gtk4::Align::Start);
    page_terminal.append(&tm_title);

    let tm_desc = gtk4::Label::new(Some("Configure default shell environment, font family, and typography metrics."));
    tm_desc.add_css_class("antigravity-content-desc");
    tm_desc.set_halign(gtk4::Align::Start);
    tm_desc.set_wrap(true);
    page_terminal.append(&tm_desc);

    let term_card = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    term_card.add_css_class("antigravity-card");

    // Font Family row
    let font_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    font_box.add_css_class("antigravity-card-row");
    font_box.set_margin_start(16);
    font_box.set_margin_end(16);
    font_box.set_margin_top(12);
    font_box.set_margin_bottom(12);

    let font_text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    font_text.set_hexpand(true);
    let f_lbl = gtk4::Label::new(Some("Font Family"));
    f_lbl.add_css_class("antigravity-row-title");
    f_lbl.set_halign(gtk4::Align::Start);
    font_text.append(&f_lbl);
    let f_sub = gtk4::Label::new(Some("Monospace typeface for code and terminal sessions"));
    f_sub.add_css_class("antigravity-row-desc");
    f_sub.set_halign(gtk4::Align::Start);
    font_text.append(&f_sub);
    font_box.append(&font_text);

    let font_entry = gtk4::Entry::new();
    font_entry.add_css_class("antigravity-entry");
    font_entry.set_text(&config_state.borrow().terminal.font_family);
    font_entry.set_width_chars(16);
    let cfg_font = config_state.clone();
    let on_font = on_font_applied.clone();
    font_entry.connect_changed(move |e| {
        let text = e.text().to_string();
        if !text.trim().is_empty() {
            let mut cfg = cfg_font.borrow_mut();
            cfg.terminal.font_family = text.clone();
            cfg.save();
            on_font(text, cfg.terminal.font_size);
        }
    });
    font_box.append(&font_entry);
    term_card.append(&font_box);

    let sep1 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    sep1.add_css_class("antigravity-card-sep");
    term_card.append(&sep1);

    // Font Size row
    let size_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    size_box.add_css_class("antigravity-card-row");
    size_box.set_margin_start(16);
    size_box.set_margin_end(16);
    size_box.set_margin_top(12);
    size_box.set_margin_bottom(12);

    let size_text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    size_text.set_hexpand(true);
    let s_lbl = gtk4::Label::new(Some("Font Size"));
    s_lbl.add_css_class("antigravity-row-title");
    s_lbl.set_halign(gtk4::Align::Start);
    size_text.append(&s_lbl);
    let s_sub = gtk4::Label::new(Some("Font size in points (Ctrl+Plus / Ctrl+Minus to adjust quickly)"));
    s_sub.add_css_class("antigravity-row-desc");
    s_sub.set_halign(gtk4::Align::Start);
    size_text.append(&s_sub);
    size_box.append(&size_text);

    let spin_btn = gtk4::SpinButton::with_range(8.0, 32.0, 1.0);
    spin_btn.add_css_class("antigravity-spin");
    spin_btn.set_value(config_state.borrow().terminal.font_size as f64);
    let cfg_size = config_state.clone();
    let on_font_sz = on_font_applied.clone();
    spin_btn.connect_value_changed(move |sb| {
        let val = sb.value() as u32;
        let mut cfg = cfg_size.borrow_mut();
        cfg.terminal.font_size = val;
        cfg.save();
        on_font_sz(cfg.terminal.font_family.clone(), val);
    });
    size_box.append(&spin_btn);
    term_card.append(&size_box);

    let sep2 = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    sep2.add_css_class("antigravity-card-sep");
    term_card.append(&sep2);

    // Default Shell row
    let shell_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    shell_box.add_css_class("antigravity-card-row");
    shell_box.set_margin_start(16);
    shell_box.set_margin_end(16);
    shell_box.set_margin_top(12);
    shell_box.set_margin_bottom(12);

    let shell_text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    shell_text.set_hexpand(true);
    let sh_lbl = gtk4::Label::new(Some("Default Shell"));
    sh_lbl.add_css_class("antigravity-row-title");
    sh_lbl.set_halign(gtk4::Align::Start);
    shell_text.append(&sh_lbl);
    let sh_sub = gtk4::Label::new(Some("Path to interactive shell executable (e.g. /bin/bash, /bin/zsh)"));
    sh_sub.add_css_class("antigravity-row-desc");
    sh_sub.set_halign(gtk4::Align::Start);
    shell_text.append(&sh_sub);
    shell_box.append(&shell_text);

    let shell_entry = gtk4::Entry::new();
    shell_entry.add_css_class("antigravity-entry");
    shell_entry.set_text(&config_state.borrow().terminal.default_shell);
    shell_entry.set_width_chars(16);
    let cfg_sh = config_state.clone();
    shell_entry.connect_changed(move |e| {
        let text = e.text().to_string();
        if !text.trim().is_empty() {
            let mut cfg = cfg_sh.borrow_mut();
            cfg.terminal.default_shell = text;
            cfg.save();
        }
    });
    shell_box.append(&shell_entry);
    term_card.append(&shell_box);

    page_terminal.append(&term_card);
    stack.add_named(&page_terminal, Some("terminal"));

    // ----------------------------------------------------
    // PAGE 4: ABOUT
    // ----------------------------------------------------
    let page_about = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    page_about.set_margin_start(28);
    page_about.set_margin_end(28);
    page_about.set_margin_top(8);
    page_about.set_margin_bottom(24);

    let ab_title = gtk4::Label::new(Some("About rmux"));
    ab_title.add_css_class("antigravity-content-title");
    ab_title.set_halign(gtk4::Align::Start);
    page_about.append(&ab_title);

    let ab_card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    ab_card.add_css_class("antigravity-card");
    ab_card.set_margin_top(8);

    let ab_inner = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    ab_inner.set_margin_start(20);
    ab_inner.set_margin_end(20);
    ab_inner.set_margin_top(16);
    ab_inner.set_margin_bottom(16);

    let app_name_lbl = gtk4::Label::new(Some("rmux — Fast, Simple, Modern Terminal for Linux"));
    app_name_lbl.add_css_class("antigravity-row-title");
    app_name_lbl.set_halign(gtk4::Align::Start);
    ab_inner.append(&app_name_lbl);

    let app_version_lbl = gtk4::Label::new(Some(concat!("Version ", env!("CARGO_PKG_VERSION"))));
    app_version_lbl.add_css_class("antigravity-row-desc");
    app_version_lbl.set_halign(gtk4::Align::Start);
    ab_inner.append(&app_version_lbl);

    let app_desc_lbl = gtk4::Label::new(Some("A fast, simple, modern terminal for Linux. Native workspaces, split panes, and live themes. Pure Rust, GTK4."));
    app_desc_lbl.add_css_class("antigravity-row-desc");
    app_desc_lbl.set_halign(gtk4::Align::Start);
    app_desc_lbl.set_wrap(true);
    ab_inner.append(&app_desc_lbl);

    // On-demand updates: check against the latest GitHub release, then
    // optionally download and install. Nothing runs on startup; every
    // step is user-initiated.
    let upd_status_lbl = gtk4::Label::new(None);
    upd_status_lbl.add_css_class("antigravity-row-desc");
    upd_status_lbl.set_halign(gtk4::Align::Start);
    upd_status_lbl.set_wrap(true);
    ab_inner.append(&upd_status_lbl);

    let upd_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    upd_row.set_halign(gtk4::Align::Start);

    let btn_check = gtk4::Button::with_label("Check for Updates");
    btn_check.add_css_class("flat");
    btn_check.add_css_class("antigravity-edit-btn");
    upd_row.append(&btn_check);

    let btn_install = gtk4::Button::with_label("Download & Install");
    btn_install.add_css_class("suggested-action");
    btn_install.set_visible(false);
    upd_row.append(&btn_install);
    ab_inner.append(&upd_row);

    let pending_asset: Rc<RefCell<Option<(String, String)>>> = Rc::new(RefCell::new(None));

    let status_check = upd_status_lbl.clone();
    let install_check = btn_install.clone();
    let pending_check = pending_asset.clone();
    btn_check.connect_clicked(move |b| {
        b.set_sensitive(false);
        status_check.set_text("Checking for updates…");
        install_check.set_visible(false);
        pending_check.borrow_mut().take();

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(crate::updater::check_for_updates());
        });
        let status_poll = status_check.clone();
        let btn_poll = b.clone();
        let install_poll = install_check.clone();
        let pending_poll = pending_check.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            match rx.try_recv() {
                Ok(crate::updater::UpdateOutcome::UpToDate(latest)) => {
                    status_poll.set_text(&format!(
                        "You are up to date (version {}, latest {}).",
                        env!("CARGO_PKG_VERSION"),
                        latest
                    ));
                    btn_poll.set_sensitive(true);
                    glib::ControlFlow::Break
                }
                Ok(crate::updater::UpdateOutcome::Available(info)) => {
                    match &info.deb_asset_url {
                        Some(asset_url) => {
                            status_poll.set_text(&format!(
                                "Version {} is available (you have {}). Press Download & Install, or get it from {}",
                                info.version,
                                env!("CARGO_PKG_VERSION"),
                                info.page_url
                            ));
                            *pending_poll.borrow_mut() =
                                Some((info.version.clone(), asset_url.clone()));
                            install_poll.set_visible(true);
                        }
                        None => {
                            status_poll.set_text(&format!(
                                "Version {} is available (you have {}). No installer is attached to that release — see {}",
                                info.version,
                                env!("CARGO_PKG_VERSION"),
                                info.page_url
                            ));
                        }
                    }
                    btn_poll.set_sensitive(true);
                    glib::ControlFlow::Break
                }
                Ok(crate::updater::UpdateOutcome::Failed(reason)) => {
                    status_poll.set_text(&format!(
                        "Could not check for updates: {reason}. Check your connection and try again."
                    ));
                    btn_poll.set_sensitive(true);
                    glib::ControlFlow::Break
                }
                Err(_) => glib::ControlFlow::Continue,
            }
        });
    });

    let status_install = upd_status_lbl.clone();
    let check_install = btn_check.clone();
    let pending_install = pending_asset.clone();
    btn_install.connect_clicked(move |b| {
        let selected = pending_install.borrow().clone();
        let Some((version, asset_url)) = selected else {
            status_install.set_text("No update selected. Press Check for Updates first.");
            return;
        };
        b.set_sensitive(false);
        check_install.set_sensitive(false);
        status_install.set_text(&format!("Downloading version {version}…"));

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = crate::updater::download_deb(&asset_url, &version)
                .and_then(|path| crate::updater::install_deb(&path).map(|_| path));
            let _ = tx.send(result);
        });
        let status_poll = status_install.clone();
        let btn_poll = b.clone();
        let check_poll = check_install.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            match rx.try_recv() {
                Ok(Ok(path)) => {
                    status_poll.set_text(&format!(
                        "Version installed from {}. Restart rmux to run it (close the window, then reopen from the top-bar indicator or the dock).",
                        path.display()
                    ));
                    btn_poll.set_visible(false);
                    check_poll.set_sensitive(true);
                    glib::ControlFlow::Break
                }
                Ok(Err(reason)) => {
                    status_poll.set_text(&format!("Update failed: {reason}."));
                    btn_poll.set_sensitive(true);
                    check_poll.set_sensitive(true);
                    glib::ControlFlow::Break
                }
                Err(_) => glib::ControlFlow::Continue,
            }
        });
    });

    ab_card.append(&ab_inner);
    page_about.append(&ab_card);
    stack.add_named(&page_about, Some("about"));

    right_col.append(&stack);
    main_box.append(&right_col);

    dialog.set_child(Some(&main_box));

    // The caller owns a full-window black scrim behind this modal, so the
    // parent is never faded to wallpaper transparency. Focus stays here:
    // the dialog is modal and transient for the parent window.
    dialog.present();
    dialog
}
