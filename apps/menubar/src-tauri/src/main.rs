//! Focusa Menubar — Tauri v2 desktop app.
//!
//! System tray with ambient cognitive awareness.
//! Read-only, non-invasive, calm.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running focusa menubar");
}
