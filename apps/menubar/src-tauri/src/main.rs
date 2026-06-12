//! Focusa Menubar — Tauri v2 tray app.
//!
//! Left-click tray → toggle popover (positioned below tray icon).
//! Right-click tray → Quit menu.
//! Click outside → auto-hide (blur event).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


const KEYCHAIN_SERVICE: &str = "Focusa Menubar Device Token";
const BRIDGE_CALLBACK_MAX_BODY: usize = 64 * 1024;

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

static BRIDGE_COMPLETIONS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static BRIDGE_LISTENERS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn bridge_completions() -> &'static Mutex<HashMap<String, String>> {
    BRIDGE_COMPLETIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn bridge_listeners() -> &'static Mutex<HashSet<String>> {
    BRIDGE_LISTENERS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn best_local_ip() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            let _ = socket.connect("8.8.8.8:80");
            socket.local_addr()
        })
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn read_http_body(stream: &mut TcpStream) -> Result<(String, String), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("callback read timeout setup failed: {e}"))?;
    let mut buffer = vec![0_u8; 8192];
    let mut read = 0_usize;
    loop {
        let n = stream
            .read(&mut buffer[read..])
            .map_err(|e| format!("callback read failed: {e}"))?;
        if n == 0 {
            break;
        }
        read += n;
        if read >= 4 && buffer[..read].windows(4).any(|w| w == b"

") {
            break;
        }
        if read == buffer.len() {
            buffer.resize(buffer.len() * 2, 0);
            if buffer.len() > BRIDGE_CALLBACK_MAX_BODY {
                return Err("callback headers too large".to_string());
            }
        }
    }
    let header_end = buffer[..read]
        .windows(4)
        .position(|w| w == b"

")
        .ok_or_else(|| "callback missing HTTP header terminator".to_string())?
        + 4;
    let header = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let content_length = header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0)
        .min(BRIDGE_CALLBACK_MAX_BODY);
    let mut body = buffer[header_end..read].to_vec();
    while body.len() < content_length {
        let mut chunk = vec![0_u8; content_length - body.len()];
        let n = stream
            .read(&mut chunk)
            .map_err(|e| format!("callback body read failed: {e}"))?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    Ok((header, String::from_utf8_lossy(&body).to_string()))
}

fn handle_bridge_callback(mut stream: TcpStream, nonce: String) {
    let response = match read_http_body(&mut stream) {
        Ok((header, body)) if header.starts_with("POST ") && header.contains(&format!("/focusa-phone-bridge/{nonce}")) => {
            if let Ok(mut completions) = bridge_completions().lock() {
                completions.insert(nonce, body);
            }
            "HTTP/1.1 200 OK\r\ncontent-type: text/html\r\naccess-control-allow-origin: *\r\nconnection: close\r\n\r\nFocusa Phone Bridge completion received. You can return to the Mac app."
        }
        _ => "HTTP/1.1 404 Not Found\r\naccess-control-allow-origin: *\r\nconnection: close\r\n\r\nNot found",
    };
    let _ = stream.write_all(response.as_bytes());
}

#[tauri::command]
fn focusa_start_bridge_callback(nonce: String) -> Result<String, String> {
    if nonce.trim().is_empty() {
        return Err("nonce is required".to_string());
    }
    if let Ok(mut listeners) = bridge_listeners().lock() {
        if listeners.contains(&nonce) {
            return Err("callback listener already active for nonce".to_string());
        }
        listeners.insert(nonce.clone());
    }
    let listener = TcpListener::bind("0.0.0.0:0").map_err(|e| format!("callback bind failed: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("callback local addr failed: {e}"))?
        .port();
    let callback_url = format!("http://{}:{}/focusa-phone-bridge/{}", best_local_ip(), port, nonce);
    std::thread::spawn({
        let nonce = nonce.clone();
        move || {
            for stream in listener.incoming().take(1).flatten() {
                handle_bridge_callback(stream, nonce.clone());
            }
            if let Ok(mut listeners) = bridge_listeners().lock() {
                listeners.remove(&nonce);
            }
        }
    });
    Ok(callback_url)
}

#[tauri::command]
fn focusa_take_bridge_completion(nonce: String) -> Result<Option<String>, String> {
    if nonce.trim().is_empty() {
        return Err("nonce is required".to_string());
    }
    bridge_completions()
        .lock()
        .map(|mut completions| completions.remove(&nonce))
        .map_err(|_| "bridge completion store poisoned".to_string())
}


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
            focusa_start_bridge_callback,
            focusa_take_bridge_completion,
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
