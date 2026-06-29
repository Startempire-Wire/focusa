//! Bonjour / mDNS service advertisement for Focusa (focusa-ui0y v0.9.39-dev).
//!
//! The daemon advertises `_focusa._tcp.local` so the Mac menubar wizard
//! can auto-discover the VPS on the LAN without operator input (G08).
//! The TXT record carries `url=<public-pairing-url>` so the Mac can skip
//! the Tailscale round-trip when on the same LAN.
//!
//! Spec: docs/55-focusa-self-host-architecture.md §3, §5 (URL discovery).
//!
//! Implementation: uses the `mdns-sd` crate's `ServiceInfo` builder with
//! the daemon's first non-loopback IPv4 address as the host IP. The
//! service name is `focusa-daemon`. TXT record keys:
//!   - url     (e.g. https://focusa-vps.tail-net.ts.net)
//!   - version (e.g. 0.9.39-dev)
//!   - port    (e.g. 8787)

use anyhow::{Context, Result};
use std::time::Duration;

/// Result of a successful service registration.
pub struct RegisteredService {
    pub fullname: String,
    pub port: u16,
    /// Optional kept-alive handle to the underlying mDNS ServiceDaemon.
    /// Holding this prevents the advertisement from being torn down.
    pub daemon: Option<std::sync::Arc<mdns_sd::ServiceDaemon>>,
}

/// Advertise `_focusa._tcp.local` on the given port. The function returns
/// after the registration is established; the advertisement is held
/// alive in a background tokio task that runs until process shutdown.
///
/// Uses the first non-loopback IPv4 address on the host. If no such
/// address is found, falls back to 0.0.0.0 (advertisement still works
/// on link-local networks).
pub async fn advertise(service_type: &str, port: u16) -> Result<RegisteredService> {
    use mdns_sd::{ServiceDaemon, ServiceInfo};
    let host_ip = detect_first_non_loopback_ipv4().unwrap_or_else(|| "0.0.0.0".to_string());
    let url = std::env::var("FOCUSA_PUBLIC_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("http://{host_ip}:{port}"));
    let version = env!("CARGO_PKG_VERSION").to_string();

    let daemon = ServiceDaemon::new().context("create mdns daemon")?;
    let service_fullname = format!(
        "focusa-daemon.{}",
        service_type.trim_end_matches('.')
    );
    let service_type = service_type.trim_end_matches('.').to_string();

    let mut properties: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    properties.insert("url".to_string(), url.clone());
    properties.insert("version".to_string(), version.clone());
    properties.insert("port".to_string(), port.to_string());

    let service_info = ServiceInfo::new(
        &service_type,
        "focusa-daemon",
        &service_fullname,
        &host_ip,
        port,
        properties,
    )
    .context("build ServiceInfo")?
    .enable_addr_auto();

    daemon.register(service_info).context("register _focusa._tcp.local")?;

    tracing::info!(
        service_fullname = %service_fullname,
        host_ip = %host_ip,
        port = port,
        url = %url,
        version = %version,
        "Bonjour advertisement registered"
    );

    // Hold the daemon alive in a background task. If the daemon is dropped,
    // the advertisement disappears. Move the Arc INTO the spawned task
    // (and keep a clone outside so we can return it for shutdown control).
    let bg = std::sync::Arc::new(daemon);
    let bg_inside = bg.clone();
    tokio::spawn(async move {
        // Keep the ServiceDaemon alive for the life of this task. We never
        // exit because the advertisement is what makes the VPS discoverable.
        let _held = bg_inside;
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });
    Ok(RegisteredService {
        fullname: service_fullname,
        port,
        daemon: Some(bg),
    })
}

/// Browse the LAN for `_focusa._tcp.local` services. Returns the first
/// resolved service within `timeout_secs`, or None if no daemon is found.
///
/// Used by tests; production browse is done in the Tauri menubar
/// (`focusa_discover_via_bonjour` in apps/menubar/src-tauri/src/main.rs)
/// because browsers cannot do mDNS natively.
pub async fn browse(service_type: &str, timeout_secs: u64) -> Result<Option<BonjourService>> {
    use mdns_sd::ServiceDaemon;
    let daemon = ServiceDaemon::new().context("create mdns daemon")?;
    let receiver = daemon
        .browse(service_type)
        .context("browse _focusa._tcp.local")?;
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if let Ok(Ok(mdns_sd::ServiceEvent::ServiceResolved(info))) =
            tokio::time::timeout(Duration::from_millis(250), receiver.recv_async()).await
        {
            let name = info.get_fullname().to_string();
                let host = info.get_hostname().to_string();
                let port = info.get_port();
                let txt: std::collections::HashMap<String, String> = info
                    .get_properties()
                    .iter()
                    .filter_map(|p| {
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
                    .unwrap_or_else(|| {
                        format!("http://{}:{}", host.trim_end_matches('.'), port)
                    });
                let _ = daemon.shutdown();
                return Ok(Some(BonjourService {
                    url,
                    host,
                    port,
                    name,
                }));
        }
    }
    let _ = daemon.shutdown();
    Ok(None)
}

#[derive(Debug, Clone)]
pub struct BonjourService {
    pub url: String,
    pub host: String,
    pub port: u16,
    pub name: String,
}

/// Detect the first non-loopback IPv4 address on the host. Used for
/// Bonjour ServiceInfo registration. Returns None if no such address
/// is found (caller falls back to 0.0.0.0).
fn detect_first_non_loopback_ipv4() -> Option<String> {
    use std::net::{IpAddr, Ipv4Addr};
    let addrs: Vec<IpAddr> = if_addrs::get_if_addrs()
        .ok()
        .map(|ifs| ifs.into_iter().map(|i| i.ip()).collect())
        .unwrap_or_default();
    for ip in addrs {
        if let IpAddr::V4(v4) = ip {
            if !v4.is_loopback() && !v4.is_link_local() && v4 != Ipv4Addr::UNSPECIFIED {
                return Some(v4.to_string());
            }
        }
    }
    None
}
