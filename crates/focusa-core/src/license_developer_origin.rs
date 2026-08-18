//! Developer-origin entitlement resolver (issue #307).
//!
//! Operator rule: a machine is a trusted Focusa development machine when
//! either condition is true:
//!   1. the private agent-kb-api identifies it as known; or
//!   2. Tailscale identifies it as a member of the operator tailnet.
//!
//! A trusted development machine receives `developer_full`: every feature is
//! enabled and commercial/feature gates never block development or testing.
//! The status is computed on demand (no persistence required — it survives
//! daemon restarts, reboots, and upgrades by construction) and cached with a
//! short TTL so temporary registry/tailnet outages only downgrade after the
//! cache window, per the rule's downgrade-protection intent.
//!
//! All probes are synchronous, bounded, and runtime-safe: no blocking
//! reqwest, no tokio dependency — safe to call from any thread including
//! async workers (see issue #250).

use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const DEFAULT_KB_API_URL: &str = "http://127.0.0.1:8791";
const DEFAULT_TAILNET_SUFFIX: &str = "tail9229d6.ts.net";
const DEFAULT_TTL_MS: u64 = 10 * 60 * 1000;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const READ_TIMEOUT: Duration = Duration::from_millis(1500);
const TAILSCALE_PROBE_BUDGET: Duration = Duration::from_millis(2500);
const CACHE_ENTRY_TTL_PADDING_MS: u64 = 250;

static CACHE: OnceLock<Mutex<Option<(Instant, bool)>>> = OnceLock::new();
static IN_FLIGHT: AtomicBool = AtomicBool::new(false);

fn cache() -> &'static Mutex<Option<(Instant, bool)>> {
    CACHE.get_or_init(|| Mutex::new(None))
}

fn ttl_ms() -> u64 {
    std::env::var("FOCUSA_DEV_ORIGIN_TTL_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_TTL_MS)
}

fn kb_api_url() -> String {
    std::env::var("FOCUSA_AGENT_KB_API_URL")
        .unwrap_or_else(|_| DEFAULT_KB_API_URL.to_string())
}

fn tailnet_suffix() -> String {
    std::env::var("FOCUSA_DEV_ORIGIN_TAILNET")
        .unwrap_or_else(|_| DEFAULT_TAILNET_SUFFIX.to_string())
}

/// Minimal HTTP GET over std::net — bounded, runtime-safe, no dependencies.
/// Returns the response body on 2xx.
fn http_get_json(url: &str, bearer: Option<&str>) -> Option<Value> {
    let url = url.strip_prefix("http://").unwrap_or(url);
    let (host_port, path) = url.split_once('/').unwrap_or((url, "/"));
    let host = host_port.split(':').next().unwrap_or("127.0.0.1");
    let port: u16 = host_port.split(':').nth(1).and_then(|p| p.parse().ok()).unwrap_or(80);
    let ip = if host == "localhost" {
        "127.0.0.1".parse().ok()?
    } else {
        host.parse().ok()?
    };
    let mut stream = TcpStream::connect_timeout(
        &std::net::SocketAddr::new(ip, port),
        CONNECT_TIMEOUT,
    )
    .ok()?;
    stream.set_read_timeout(Some(READ_TIMEOUT)).ok()?;
    let mut request = format!("GET /{path} HTTP/1.1\r\nHost: {host_port}\r\nAccept: application/json\r\nConnection: close\r\n");
    if let Some(token) = bearer {
        request.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                response.extend_from_slice(&buffer[..n]);
                if response.len() > 64 * 1024 {
                    break; // bounded
                }
            }
            Err(_) => break,
        }
    }
    let text = String::from_utf8_lossy(&response).to_string();
    if !text.starts_with("HTTP/1.1 2") {
        return None;
    }
    let body = text.split("\r\n\r\n").nth(1)?;
    serde_json::from_str(body.trim()).ok()
}

fn bearer_token() -> Option<String> {
    std::env::var("FOCUSA_AGENT_KB_TOKEN").ok().or_else(|| {
        std::fs::read_to_string("/etc/agent-kb/token")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

/// Probe 1: private agent-kb-api identifies this machine as known.
/// Strong signal: authenticated /v1/operator returns a valid operator
/// payload. Weak signal: the private API answers /v1/health ok on the
/// canonical machine-local port (it only exists on registered machines).
fn probe_agent_kb_known() -> bool {
    let base = kb_api_url();
    if let Some(token) = bearer_token() {
        if let Some(payload) = http_get_json(&format!("{base}/v1/operator"), Some(&token)) {
            let identified = payload
                .get("preferred_address")
                .or_else(|| payload.get("operator"))
                .and_then(Value::as_str)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
                || payload
                    .get("source")
                    .and_then(Value::as_str)
                    .map(|value| value != "error")
                    .unwrap_or(false);
            if identified {
                return true;
            }
        }
    }
    http_get_json(&format!("{base}/v1/health"), None)
        .map(|payload| {
            payload.get("status").and_then(Value::as_str) == Some("ok")
                || payload.get("ok").and_then(Value::as_bool) == Some(true)
        })
        .unwrap_or(false)
}

/// Probe 2: Tailscale identifies this device as a member of the operator
/// tailnet. The child writes to a temp file (never a pipe — a silent hang
/// must not block this thread) and is killed after a bounded polling budget.
fn probe_tailnet_member() -> bool {
    let probe_path = std::env::temp_dir().join(format!(
        "focusa-dev-origin-tailscale-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default()
    ));
    let output_file = match std::fs::File::create(&probe_path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut child: Child = match Command::new("tailscale")
        .args(["status", "--json"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(output_file))
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            let _ = std::fs::remove_file(&probe_path);
            return false;
        }
    };
    let started = Instant::now();
    let finished = loop {
        if let Ok(Some(status)) = child.try_wait() {
            break Some(status);
        }
        if started.elapsed() > TAILSCALE_PROBE_BUDGET {
            let _ = child.kill();
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let _ = child.wait();
    let raw = std::fs::read(&probe_path).unwrap_or_default();
    let _ = std::fs::remove_file(&probe_path);
    if finished.is_none() {
        return false;
    }
    let parsed: Value = match serde_json::from_slice(&raw) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let Some(self_info) = parsed.get("Self") else {
        return false;
    };
    let running = self_info
        .get("BackendState")
        .and_then(Value::as_str)
        .map(|state| state == "Running")
        .unwrap_or(false);
    let online = self_info
        .get("Online")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let expected = tailnet_suffix();
    let member = self_info
        .get("MagicDNSSuffix")
        .or_else(|| self_info.get("TailnetName"))
        .and_then(Value::as_str)
        .map(|suffix| suffix.trim_end_matches('.').eq_ignore_ascii_case(&expected))
        .unwrap_or(false);
    running && online && member
}

/// Cached developer-origin check with short TTL. Testable via
/// `developer_origin_active_with`.
pub fn developer_origin_active() -> bool {
    developer_origin_active_with(probe_agent_kb_known, probe_tailnet_member)
}

pub fn developer_origin_active_with(
    kb_known: impl Fn() -> bool,
    tailnet_member: impl Fn() -> bool,
) -> bool {
    // Re-entrancy guard: probes must never recurse into this resolver.
    if IN_FLIGHT.swap(true, Ordering::SeqCst) {
        return false;
    }
    let result = (|| {
        let ttl = ttl_ms();
        let now = Instant::now();
        {
            let lock = cache();
            if let Ok(guard) = lock.lock() {
                if let Some((cached_at, cached)) = *guard {
                    if now.duration_since(cached_at)
                        < Duration::from_millis(
                            ttl.saturating_sub(CACHE_ENTRY_TTL_PADDING_MS).max(1),
                        )
                    {
                        return cached;
                    }
                }
            }
        }
        let active = kb_known() || tailnet_member();
        if let Ok(mut guard) = cache().lock() {
            *guard = Some((Instant::now(), active));
        }
        active
    })();
    IN_FLIGHT.store(false, Ordering::SeqCst);
    result
}

/// Force the next call to re-probe (tests and diagnostics).
pub fn invalidate_developer_origin_cache() {
    if let Ok(mut guard) = cache().lock() {
        *guard = None;
    }
}

/// Diagnostic snapshot: which origin source activated, cache state, and TTL.
/// Probes both sources (no short-circuit) so operators see the full picture.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeveloperOriginReport {
    pub active: bool,
    pub agent_kb_known: bool,
    pub tailnet_member: bool,
    pub tailnet_suffix: String,
    pub kb_api_url: String,
    pub cached: bool,
    pub ttl_ms: u64,
}

pub fn developer_origin_report() -> DeveloperOriginReport {
    let ttl = ttl_ms();
    let cached = {
        let lock = cache();
        lock.lock()
            .ok()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    };
    let kb_known = probe_agent_kb_known();
    let tailnet = probe_tailnet_member();
    DeveloperOriginReport {
        active: kb_known || tailnet,
        agent_kb_known: kb_known,
        tailnet_member: tailnet,
        tailnet_suffix: tailnet_suffix(),
        kb_api_url: kb_api_url(),
        cached,
        ttl_ms: ttl,
    }
}

#[cfg(test)]
mod tests {
    static TEST_MUTEX: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    fn test_lock() -> std::sync::MutexGuard<'static, ()> { TEST_MUTEX.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap() }
    use super::*;

    #[test]
    fn either_source_activates_developer_origin() {
        let _guard = test_lock();
        invalidate_developer_origin_cache();
        assert!(developer_origin_active_with(|| true, || false));
        invalidate_developer_origin_cache();
        assert!(developer_origin_active_with(|| false, || true));
        invalidate_developer_origin_cache();
        assert!(!developer_origin_active_with(|| false, || false));
    }

    #[test]
    fn cache_serves_within_ttl_and_expires() {
        let _guard = test_lock();
        invalidate_developer_origin_cache();
        let previous_ttl = std::env::var("FOCUSA_DEV_ORIGIN_TTL_MS").ok();
        // Use TTL > padding (250ms) so effective TTL is stable and test is not flaky
        unsafe { std::env::set_var("FOCUSA_DEV_ORIGIN_TTL_MS", "1000"); }
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter = calls.clone();
        let probe = move || {
            counter.fetch_add(1, Ordering::SeqCst);
            true
        };
        assert!(developer_origin_active_with(probe, || false));
        // Within TTL (750ms effective) — should be cached, no extra call
        std::thread::sleep(Duration::from_millis(50));
        assert!(developer_origin_active_with(|| false, || false)); // cached
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        std::thread::sleep(Duration::from_millis(900));
        // Expired — must re-probe (use same counter via new closure capturing same Arc)
        let counter2 = calls.clone();
        assert!(developer_origin_active_with(move || { counter2.fetch_add(1, Ordering::SeqCst); true }, || false)); // re-probe
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        match previous_ttl {
            Some(value) => unsafe { std::env::set_var("FOCUSA_DEV_ORIGIN_TTL_MS", value) },
            None => unsafe { std::env::remove_var("FOCUSA_DEV_ORIGIN_TTL_MS") },
        }
        invalidate_developer_origin_cache();
    }

    #[test]
    fn real_kb_api_probe_resolves_against_a_local_fixture() {
        let _guard = test_lock();
        use std::net::TcpListener;
        use std::sync::atomic::AtomicU16;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        static PORT: AtomicU16 = AtomicU16::new(0);
        PORT.store(port, Ordering::SeqCst);
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0_u8; 2048];
                let _ = stream.read(&mut buf);
                let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 31\r\nConnection: close\r\n\r\n{\"status\":\"ok\",\"ok\":true}";
                let _ = stream.write_all(response.as_bytes());
            }
        });
        let previous_url = std::env::var("FOCUSA_AGENT_KB_API_URL").ok();
        unsafe {
            std::env::set_var(
                "FOCUSA_AGENT_KB_API_URL",
                format!("http://127.0.0.1:{}", PORT.load(Ordering::SeqCst)),
            );
        }
        invalidate_developer_origin_cache();
        assert!(developer_origin_active_with(probe_agent_kb_known, || false));
        let _ = handle.join();
        match previous_url {
            Some(value) => unsafe { std::env::set_var("FOCUSA_AGENT_KB_API_URL", value) },
            None => unsafe { std::env::remove_var("FOCUSA_AGENT_KB_API_URL") },
        }
        invalidate_developer_origin_cache();
    }
}
