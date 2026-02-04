//! Focusa Menubar — Tauri v2 desktop app.
//!
//! System tray with ambient cognitive awareness.
//! Click tray icon → toggle popover window.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    image::Image,
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Create tray icon programmatically (gives us the click handler).
            let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;

            let _tray = TrayIconBuilder::new("focusa-tray")
                .icon(icon)
                .icon_as_template(true)
                .tooltip("Focusa")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
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
        .expect("error while running focusa menubar");
}
