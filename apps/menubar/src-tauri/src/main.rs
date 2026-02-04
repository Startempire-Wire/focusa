//! Focusa Menubar — Tauri v2 macOS/Linux/Windows tray app.
//!
//! System tray icon with popover window.
//! Left-click toggles window. Right-click shows menu with Quit.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // macOS: hide dock icon — this is a menubar-only app
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Build right-click context menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit Focusa", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            // Tray icon
            let icon = tauri::include_image!("icons/icon.png");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(true)
                .tooltip("Focusa — Cognitive Governance")
                .menu(&menu)
                .menu_on_left_click(false) // left click = toggle window, not menu
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running focusa");
}
