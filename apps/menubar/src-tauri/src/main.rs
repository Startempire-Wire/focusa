//! Focusa Menubar — Tauri v2 tray app.
//!
//! Left-click tray → toggle popover (positioned below tray icon).
//! Right-click tray → Quit menu.
//! Click outside → auto-hide (blur event).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


const KEYCHAIN_SERVICE: &str = "Focusa Menubar Device Token";

#[cfg(target_os = "macos")]
fn run_security(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("security")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run security: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[tauri::command]
fn focusa_save_pairing_token(device_id: String, token: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if device_id.trim().is_empty() || token.trim().is_empty() {
            return Err("device_id and token are required".to_string());
        }
        let _ = run_security(&[
            "delete-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            device_id.as_str(),
        ]);
        run_security(&[
            "add-generic-password",
            "-U",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            device_id.as_str(),
            "-w",
            token.as_str(),
        ])?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (device_id, token);
        Err("Focusa pairing token storage requires macOS Keychain".to_string())
    }
}

#[tauri::command]
fn focusa_load_pairing_token(device_id: String) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        if device_id.trim().is_empty() {
            return Err("device_id is required".to_string());
        }
        run_security(&[
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            device_id.as_str(),
            "-w",
        ])
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = device_id;
        Err("Focusa pairing token storage requires macOS Keychain".to_string())
    }
}

#[tauri::command]
fn focusa_clear_pairing_token(device_id: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if device_id.trim().is_empty() {
            return Err("device_id is required".to_string());
        }
        let _ = run_security(&[
            "delete-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            device_id.as_str(),
        ]);
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = device_id;
        Err("Focusa pairing token storage requires macOS Keychain".to_string())
    }
}

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .invoke_handler(tauri::generate_handler![
            focusa_save_pairing_token,
            focusa_load_pairing_token,
            focusa_clear_pairing_token,
        ])
        .setup(|app| {
            // macOS: hide dock icon — menubar-only app
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Hide window on blur (click outside = dismiss, standard menubar behavior)
            let main_window = app.get_webview_window("main").unwrap();
            let win_clone = main_window.clone();
            main_window.on_window_event(move |event| {
                if let WindowEvent::Focused(false) = event {
                    let _ = win_clone.hide();
                }
            });

            // Right-click context menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit Focusa", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            // Tray icon
            let icon = tauri::include_image!("icons/icon.png");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(true)
                .tooltip("Focusa")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // Feed tray position to positioner plugin
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

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
                                // Position window below tray icon, then show
                                use tauri_plugin_positioner::{Position, WindowExt};
                                let _ = window.move_window(Position::TrayCenter);
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
