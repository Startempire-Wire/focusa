//! Bonjour / mDNS service advertisement for Focusa (focusa-ui0y v0.9.35-dev).
//!
//! The daemon advertises `_focusa._tcp.local` so the Mac menubar wizard can
//! auto-discover the VPS on the LAN without operator input (G08). The TXT
//! record carries `url=<public-pairing-url>` so the Mac can skip the
//! Tailscale round-trip when on the same LAN.
//!
//! Spec: docs/55-focusa-self-host-architecture.md §3, §5 (URL discovery).
//!
//! NOTE: mdns-sd 0.11's ServiceInfo API takes more required fields than we
//! can populate in a portable daemon (hostname, IP addrs). For v0.9.35-dev
//! the Bonjour path is a stub: the module compiles and exposes the right
//! surface, but the actual advertisement relies on the macOS-side
//! daemon_lan_announce helper (apps/menubar Tauri command) plus the
//! `_focusa._tcp.local` service that the operator can install manually
//! via `dns-sd -R "Focusa" _focusa._tcp local 8787` on macOS, or via the
//! `focusa pairing transport-setup` helper that writes a hostfile entry.
//! A full mdns-sd 0.11 integration is queued for v0.9.36.

use anyhow::Result;
use std::time::Duration;

/// Stub: holds the daemon alive to keep any platform-specific advertisement
/// alive (in a future version). On macOS the Tauri-side menubar also uses
/// the same mdns-sd crate to advertise; for the daemon we accept that
/// Bonjour registration on Linux/macOS-from-CLI may be a no-op until a
/// later version wires ServiceInfo with the right hostname/IP discovery.
pub async fn advertise(service_type: &str, port: u16) -> Result<()> {
    tracing::info!(
        service_type = %service_type,
        port = port,
        "Bonjour advertisement stub (see bonjour.rs note); Tailscale/Bonjour auto-discovery on the Mac side is implemented in FirstRunWizard.svelte"
    );
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

/// Stub browse. The Mac-side browse uses the Tauri command
/// `focusa_discover_via_bonjour` (apps/menubar/src-tauri/src/main.rs).
pub async fn browse(_service_type: &str, _timeout_secs: u64) -> Result<Option<BonjourService>> {
    Ok(None)
}

#[derive(Debug, Clone)]
pub struct BonjourService {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub name: String,
}