//! Focusa Menubar — Tauri v2 tray app.
//!
//! Left-click tray → toggle popover (positioned below tray icon).
//! Right-click tray → Quit menu.
//! Click outside → auto-hide (blur event).
//!
//! # Spec104 MBN-01: typed scope-bearing bridge messages
//!
//! All bridge messages between Mac, VPS, and Phone preserve the typed
//! ScopeContext (`project_root`, `continuity_id`, `session_id`) end-to-end.
//! The bridge does NOT mutate canonical scope state — it only forwards
//! scope metadata alongside token/nonce payloads.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

const KEYCHAIN_SERVICE: &str = "Focusa Menubar Device Token";
const BRIDGE_CALLBACK_MAX_BODY: usize = 64 * 1024;

/// Spec 172 Desktop command bridge and action registry: native/Tauri/local
/// commands resolve through the frozen desktop action registry and are
/// forwarded to the daemon core guard. The bridge never evaluates
/// entitlement and never touches storage or reducers directly
/// (docs/172-focusa-spec152-license-type-and-surface-entitlement-
/// governance-addendum.md §11.4, §12, §15).
mod spec172_desktop_bridge;

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Extract the typed Spec 172 desktop status envelope from the daemon
/// `GET /v1/license/status` payload. Presenter-safe extraction only: no
/// caller-supplied product, price, License Type, family, feature, limit, or
/// node value is accepted.
fn desktop_status_envelope(
    payload: &serde_json::Value,
) -> spec172_desktop_bridge::DesktopStatusEnvelope {
    let get_str = |key: &str| -> String {
        payload
            .get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let get_strs = |key: &str| -> Vec<String> {
        payload
            .get(key)
            .and_then(serde_json::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    let allowed_actions = payload
        .get("presenter")
        .and_then(|presenter| presenter.get("allowed_actions"))
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    spec172_desktop_bridge::DesktopStatusEnvelope {
        status: get_str("status"),
        posture: get_str("posture"),
        product: get_str("product"),
        license_type: payload
            .get("license_type")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        product_grants: get_strs("product_grants"),
        family: get_str("family"),
        allowed_actions,
    }
}

/// Spec 172 Desktop command bridge: resolve a native/Tauri/local desktop
/// action against the frozen registry and return the daemon route to call.
/// Unknown actions fail closed; value-producing actions always forward to
/// the shared core execution guard (the daemon), never to local storage or
/// reducers. The bridge never evaluates entitlement itself.
#[tauri::command]
fn focusa_desktop_route_action(action_id: String) -> Result<serde_json::Value, String> {
    let Some(resolution) = spec172_desktop_bridge::resolve_desktop_action(&action_id) else {
        return Err(format!("unknown desktop action: {action_id}"));
    };
    Ok(serde_json::json!({
        "schema": "focusa.spec172.desktop_action_resolution.v1",
        "action_id": resolution.action_id,
        "operation_id": resolution.operation_id,
        "class": resolution.class,
        "family": resolution.family,
        "method": resolution.method,
        "path": resolution.path,
        "mutation": resolution.mutation,
        "forwards_to_core_guard": resolution.forwards_to_core_guard,
        "local_storage": resolution.local_storage,
        "direct_storage": resolution.direct_storage,
    }))
}

/// Spec 172 Desktop presenter projection: render the canonical posture
/// envelope (focusa.spec172.presenter_projection.v1) from the daemon
/// `GET /v1/license/status` payload so Desktop decisions are identical to
/// CLI/API. Read-only; the bridge never mints a grant, License Type, or
/// upgrade.
#[tauri::command]
fn focusa_desktop_spec172_posture(payload: serde_json::Value) -> Result<serde_json::Value, String> {
    let envelope = desktop_status_envelope(&payload);
    let posture = spec172_desktop_bridge::project_desktop_spec172_posture(&envelope);
    // Build the canonical envelope explicitly: the bridge module is std-only
    // (no serde) so it can be compiled standalone, and the envelope keys stay
    // byte-identical to the CLI/API presenter projection.
    Ok(serde_json::json!({
        "schema": posture.schema,
        "posture": posture.posture,
        "product": posture.product,
        "license_type": posture.license_type,
        "family": posture.family,
        "denial": posture.denial,
        "retained_access": posture.retained_access.iter().copied().collect::<Vec<&'static str>>(),
        "upgrade_action": posture.upgrade_action,
        "recovery_action": posture.recovery_action,
        "grant_inferred_from_surface": posture.grant_inferred_from_surface,
        "same_node": posture.same_node,
    }))
}

/// Spec104 MBN-01 typed scope envelope for bridge messages.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
struct BridgeScope {
    project_root: String,
    continuity_id: String,
    session_id: Option<String>,
    scope_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BridgeAttachmentKey {
    root_scope: String,
    workstream: String,
    session_id: String,
    attachment_id: String,
}

impl BridgeAttachmentKey {
    fn from_nonce(nonce: &str) -> Self {
        Self {
            root_scope: "host:menubar-bridge".to_string(),
            workstream: "phone-pairing-callback".to_string(),
            session_id: "local-tauri".to_string(),
            attachment_id: nonce.to_string(),
        }
    }
}

#[derive(Default)]
struct BridgeRuntimeState {
    completions_by_attachment: Mutex<HashMap<BridgeAttachmentKey, String>>,
    listeners_by_attachment: Mutex<HashSet<BridgeAttachmentKey>>,
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
        if read >= 4
            && buffer[..read].windows(4).any(|w| {
                w == b"

"
            })
        {
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
        .position(|w| {
            w == b"

"
        })
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

fn handle_bridge_callback(
    mut stream: TcpStream,
    nonce: String,
    bridge_state: Arc<BridgeRuntimeState>,
) {
    // V2 P1.5 hardening: validate Content-Type, body shape, role/protocol,
    // and nonce binding before storing the completion payload. The Mac
    // holds the secret; the LAN bridge only forwards.
    const VALID_CT_PREFIX: &str = "application/json";
    const REQUIRED_PROTOCOL: &str = "focusa-connect-v1";
    const REQUIRED_ROLE: &str = "mac_completion_payload";

    let response = match read_http_body(&mut stream) {
        Ok((header, body))
            if header.starts_with("POST ")
                && header.contains(&format!("/focusa-phone-bridge/{nonce}"))
                && content_type_is(&header, VALID_CT_PREFIX)
                && body_bytes_are_valid_json(&body) =>
        {
            // Reject if payload doesn't carry our protocol + role.
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(v)
                    if v.get("protocol").and_then(|x| x.as_str()) == Some(REQUIRED_PROTOCOL)
                        && v.get("role").and_then(|x| x.as_str()) == Some(REQUIRED_ROLE)
                        && v.get("connect_id").and_then(|x| x.as_str()).is_some()
                        && v.get("token").and_then(|x| x.as_str()).is_some() =>
                {
                    // V2 P1 (round 2): verify path nonce == payload.mac_nonce.
                    // Without this, a same-LAN caller who learns the
                    // callback URL (which embeds a random nonce) could
                    // POST a syntactically valid mac_completion_payload
                    // even without knowing the mac_nonce. Now they have
                    // to bind both.
                    let payload_nonce = v.get("mac_nonce").and_then(|x| x.as_str());
                    if payload_nonce != Some(nonce.as_str()) {
                        "HTTP/1.1 422 Unprocessable Entity\r\nconnection: close\r\n\r\nmac_nonce mismatch"
                    } else if let Ok(mut completions) =
                        bridge_state.completions_by_attachment.lock()
                    {
                        completions.insert(BridgeAttachmentKey::from_nonce(&nonce), body);
                        "HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\nconnection: close\r\n\r\nFocusa Phone Bridge completion received. You can return to the Mac app."
                    } else {
                        "HTTP/1.1 500 Internal Server Error\r\nconnection: close\r\n\r\ncompletion store poisoned"
                    }
                }
                _ => {
                    "HTTP/1.1 422 Unprocessable Entity\r\nconnection: close\r\n\r\ninvalid completion payload"
                }
            }
        }
        _ => "HTTP/1.1 404 Not Found\r\nconnection: close\r\n\r\nNot found",
    };
    let _ = stream.write_all(response.as_bytes());
}

/// V2 P1.5: request must declare application/json (or +json variant).
fn content_type_is(header: &str, prefix: &str) -> bool {
    header
        .to_ascii_lowercase()
        .split('\r')
        .filter_map(|line| line.split_once(':').map(|(k, v)| (k.trim(), v.trim())))
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| {
            v.split(';')
                .next()
                .map(|m| m.trim().starts_with(prefix))
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// V2 P1.5: required-schema validation. Cheap shape probe so we don't
/// store arbitrary attacker-controlled blobs.
fn body_bytes_are_valid_json(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body).is_ok()
}

#[tauri::command]
fn focusa_start_bridge_callback(
    nonce: String,
    bridge_state: tauri::State<'_, Arc<BridgeRuntimeState>>,
) -> Result<Option<String>, String> {
    if nonce.trim().is_empty() {
        return Err("nonce is required".to_string());
    }
    // The LAN callback binds 0.0.0.0 and therefore triggers macOS's
    // "accept incoming network connections" prompt. Pairing already polls
    // room status, so keep that prompt-free path as the safe default.
    // Operators who need the low-latency LAN bridge can opt in explicitly.
    if std::env::var("FOCUSA_PHONE_BRIDGE_LAN_CALLBACK").as_deref() != Ok("1") {
        return Ok(None);
    }
    let attachment_key = BridgeAttachmentKey::from_nonce(&nonce);
    if let Ok(mut listeners) = bridge_state.listeners_by_attachment.lock() {
        if listeners.contains(&attachment_key) {
            return Err("callback listener already active for nonce".to_string());
        }
        listeners.insert(attachment_key.clone());
    }
    let listener =
        TcpListener::bind("0.0.0.0:0").map_err(|e| format!("callback bind failed: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("callback local addr failed: {e}"))?
        .port();
    let callback_url = format!(
        "http://{}:{}/focusa-phone-bridge/{}",
        best_local_ip(),
        port,
        nonce
    );
    std::thread::spawn({
        let nonce = nonce.clone();
        let bridge_state = Arc::clone(bridge_state.inner());
        move || {
            // V2 P1.5: bound the listener to 30s so a missed callback doesn't
            // pin an ephemeral port. After 30s without success, the
            // listener exits and frees the port.
            //
            // V2 P0 (round 2): use set_nonblocking(true) and poll the
            // deadline every iteration. The previous version called
            // set_nonblocking(false) before listener.incoming().next(),
            // which would block indefinitely on Linux/macOS until a
            // connection arrived — the deadline check at the top of the
            // loop never ran, so the 30s TTL was effectively meaningless.
            let deadline = std::time::Instant::now() + Duration::from_secs(30);
            let mut handled = false;
            // Set nonblocking so accept() returns WouldBlock instead of
            // blocking the thread when no connection is pending.
            if let Err(e) = listener.set_nonblocking(true) {
                tracing::error!(nonce = %nonce, error = %e, "V2 P0: set_nonblocking(true) failed; aborting listener");
                if let Ok(mut listeners) = bridge_state.listeners_by_attachment.lock() {
                    listeners.remove(&BridgeAttachmentKey::from_nonce(&nonce));
                }
                return;
            }
            loop {
                if std::time::Instant::now() >= deadline || handled {
                    break;
                }
                match listener.accept() {
                    Ok((stream, _peer)) => {
                        handle_bridge_callback(stream, nonce.clone(), Arc::clone(&bridge_state));
                        handled = true;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection ready; sleep briefly and check
                        // the deadline. 50ms is small enough to feel
                        // responsive while keeping the poll cheap.
                        std::thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => {
                        tracing::error!(nonce = %nonce, error = %e, "V2 P0: listener.accept() failed; aborting");
                        break;
                    }
                }
            }
            if let Ok(mut listeners) = bridge_state.listeners_by_attachment.lock() {
                listeners.remove(&BridgeAttachmentKey::from_nonce(&nonce));
            }
        }
    });
    Ok(Some(callback_url))
}

#[tauri::command]
fn focusa_take_bridge_completion(
    nonce: String,
    bridge_state: tauri::State<'_, Arc<BridgeRuntimeState>>,
) -> Result<Option<String>, String> {
    if nonce.trim().is_empty() {
        return Err("nonce is required".to_string());
    }
    bridge_state
        .completions_by_attachment
        .lock()
        .map(|mut completions| completions.remove(&BridgeAttachmentKey::from_nonce(&nonce)))
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
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

/// Result of a Bonjour / mDNS browse for `_focusa._tcp.local` services.
/// `url` is the daemon's public URL (read from the service TXT record);
/// `host` is the discovered hostname; `port` is the daemon port.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct BonjourDiscovery {
    url: String,
    host: String,
    port: u16,
}

/// Browse the LAN for `_focusa._tcp.local` services and return the first
/// reachable Focusa daemon. Resolves in <=2 seconds; returns Ok(None) if
/// no daemon is found. Used by FirstRunWizard.svelte as the Bonjour
/// discovery step (G07).
#[tauri::command]
async fn focusa_discover_via_bonjour(
    timeout_secs: Option<u64>,
) -> Result<Option<BonjourDiscovery>, String> {
    let timeout_secs = timeout_secs.unwrap_or(2);
    use mdns_sd::ServiceDaemon;
    let daemon = ServiceDaemon::new().map_err(|e| format!("mdns daemon: {e}"))?;
    let receiver = daemon
        .browse("_focusa._tcp.local")
        .map_err(|e| format!("mdns browse: {e}"))?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        // mdns-sd's recv_async returns a ServiceEvent; the daemon auto-shards
        // the channel so we don't get a Result wrapper here.
        // tokio::time::timeout produces a Result<_, Elapsed>. recv_async
        // itself returns Result<ServiceEvent, flume::RecvError>. So we
        // need double-Result matching: timeout OK + recv OK.
        if let Ok(Ok(event)) =
            tokio::time::timeout(std::time::Duration::from_millis(250), receiver.recv_async()).await
        {
            if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                let name = info.get_fullname().to_string();
                let host = info.get_hostname().to_string();
                let port = info.get_port();
                let txt: std::collections::HashMap<String, String> = info
                    .get_properties()
                    .iter()
                    .filter_map(|p| {
                        // p.val() returns Option<&[u8]>; convert to String.
                        let val = p
                            .val()
                            .and_then(|b| std::str::from_utf8(b).ok())
                            .unwrap_or("")
                            .to_string();
                        if val.is_empty() {
                            None
                        } else {
                            Some((p.key().to_string(), val))
                        }
                    })
                    .collect();
                let url = txt
                    .get("url")
                    .cloned()
                    .unwrap_or_else(|| format!("http://{}:{}", host.trim_end_matches('.'), port));
                let _ = daemon.shutdown();
                return Ok(Some(BonjourDiscovery { url, host, port }));
            }
        }
    }
    let _ = daemon.shutdown();
    Ok(None)
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(BridgeRuntimeState::default()))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            focusa_save_pairing_token,
            focusa_load_pairing_token,
            focusa_clear_pairing_token,
            focusa_start_bridge_callback,
            focusa_take_bridge_completion,
            focusa_discover_via_bonjour,
            focusa_desktop_route_action,
            focusa_desktop_spec172_posture,
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
