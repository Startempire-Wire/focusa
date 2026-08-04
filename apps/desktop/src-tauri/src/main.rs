//! Focusa Desktop — primary native application shell.
//!
//! The 5% shell owns presentation only. It exposes no domain mutation command
//! and does not cache canonical cognitive state.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Focusa Desktop");
}
