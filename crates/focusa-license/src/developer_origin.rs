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
use std::io::{Read, Seek, SeekFrom, Write};
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
        .min(DEFAULT_TTL_MS)
}

fn kb_api_url() -> String {
    std::env::var("FOCUSA_AGENT_KB_API_URL").unwrap_or_else(|_| DEFAULT_KB_API_URL.to_string())
}

fn tailnet_suffix() -> &'static str {
    // An environment variable must never nominate a different licensing owner.
    DEFAULT_TAILNET_SUFFIX
}

/// Minimal HTTP GET over std::net — bounded, runtime-safe, no dependencies.
/// Returns the response body on 2xx.
fn http_get_json(url: &str, bearer: Option<&str>) -> Option<Value> {
    let url = url.strip_prefix("http://").unwrap_or(url);
    let (host_port, path) = url.split_once('/').unwrap_or((url, "/"));
    let host = host_port.split(':').next().unwrap_or("127.0.0.1");
    let port: u16 = host_port
        .split(':')
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(80);
    let ip = if host == "localhost" {
        "127.0.0.1".parse().ok()?
    } else {
        host.parse().ok()?
    };
    let mut stream =
        TcpStream::connect_timeout(&std::net::SocketAddr::new(ip, port), CONNECT_TIMEOUT).ok()?;
    stream.set_read_timeout(Some(READ_TIMEOUT)).ok()?;
    let mut request = format!(
        "GET /{path} HTTP/1.1\r\nHost: {host_port}\r\nAccept: application/json\r\nConnection: close\r\n"
    );
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
/// Uses authenticated operator metadata as a discovery signal only. A public
/// health response proves liveness, never machine identity or entitlement.
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
    false
}

/// Private anonymous output using existing standard-library primitives.
fn private_probe_file() -> std::io::Result<std::fs::File> {
    let path = std::env::temp_dir().join(format!("focusa-origin-{}", uuid::Uuid::now_v7()));
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
        options.custom_flags(FILE_FLAG_DELETE_ON_CLOSE);
    }
    let file = options.open(&path)?;
    #[cfg(unix)]
    std::fs::remove_file(&path)?;
    Ok(file)
}

/// Probe 2: Tailscale identifies this device as a member of the operator
/// tailnet. The child writes to a temp file (never a pipe — a silent hang
/// must not block this thread) and is killed after a bounded polling budget.
fn probe_tailnet_member() -> bool {
    // Keep the same private file handle throughout the probe: no predictable
    // path that another process can replace with forged membership output.
    let mut output_file = match private_probe_file() {
        Ok(file) => file,
        Err(_) => return false,
    };
    let child_output = match output_file.try_clone() {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut child: Child = match Command::new("tailscale")
        .args(["status", "--json", "--peers=false"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(child_output))
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
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
    let mut raw = Vec::new();
    if output_file.seek(SeekFrom::Start(0)).is_err()
        || output_file
            .take(2 * 1024 * 1024)
            .read_to_end(&mut raw)
            .is_err()
    {
        return false;
    }
    if !finished.is_some_and(|status| status.success()) {
        return false;
    }
    let parsed: Value = match serde_json::from_slice(&raw) {
        Ok(value) => value,
        Err(_) => return false,
    };
    tailnet_status_matches(&parsed, tailnet_suffix())
}

fn tailnet_status_matches(parsed: &Value, expected: &str) -> bool {
    let Some(self_info) = parsed.get("Self") else {
        return false;
    };
    let running = parsed
        .get("BackendState")
        .and_then(Value::as_str)
        .map(|state| state == "Running")
        .unwrap_or(false);
    let online = self_info
        .get("Online")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let identified = self_info
        .get("ID")
        .and_then(Value::as_str)
        .is_some_and(|id| !id.trim().is_empty());
    let member = parsed
        .get("CurrentTailnet")
        .and_then(|tailnet| tailnet.get("MagicDNSSuffix"))
        .and_then(Value::as_str)
        .map(|suffix| suffix.trim_end_matches('.').eq_ignore_ascii_case(expected))
        .unwrap_or(false);
    running && online && identified && member
}

/// Cached developer-origin check with short TTL. Testable via
/// `developer_origin_active_with`.
pub fn developer_origin_active() -> bool {
    // Operator metadata is discovery, not machine identification. Until the KB
    // provides verified machine evidence, use the existing tailnet identity proof.
    developer_origin_active_with(|| false, probe_tailnet_member)
}

fn developer_origin_active_with(
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
    pub agent_kb_discovery_available: bool,
    pub tailnet_member: bool,
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
        active: tailnet,
        agent_kb_known: false,
        agent_kb_discovery_available: kb_known,
        tailnet_member: tailnet,
        cached,
        ttl_ms: ttl,
    }
}

#[cfg(test)]
mod tests {
    static TEST_MUTEX: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_MUTEX
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap()
    }
    use super::*;

    #[test]
    fn environment_cannot_choose_owner_or_extend_origin_cache() {
        let _guard = test_lock();
        let previous_owner = std::env::var("FOCUSA_DEV_ORIGIN_TAILNET").ok();
        let previous_ttl = std::env::var("FOCUSA_DEV_ORIGIN_TTL_MS").ok();
        unsafe {
            std::env::set_var("FOCUSA_DEV_ORIGIN_TAILNET", "unrelated.example.ts.net");
            std::env::set_var("FOCUSA_DEV_ORIGIN_TTL_MS", u64::MAX.to_string());
        }
        let owner = tailnet_suffix();
        let ttl = ttl_ms();
        for (key, previous) in [
            ("FOCUSA_DEV_ORIGIN_TAILNET", previous_owner),
            ("FOCUSA_DEV_ORIGIN_TTL_MS", previous_ttl),
        ] {
            match previous {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
        assert_eq!(owner, DEFAULT_TAILNET_SUFFIX);
        assert_eq!(ttl, DEFAULT_TTL_MS);
    }

    #[test]
    fn tailnet_probe_uses_native_top_level_identity_fields() {
        let valid = serde_json::json!({
            "BackendState": "Running",
            "CurrentTailnet": {"MagicDNSSuffix": "developer.example.ts.net"},
            "Self": {"ID": "fixture-node", "Online": true}
        });
        assert!(tailnet_status_matches(&valid, "developer.example.ts.net"));
        assert!(!tailnet_status_matches(&valid, "other.example.ts.net"));
        for (pointer, value) in [
            ("/BackendState", serde_json::json!("NeedsLogin")),
            ("/Self/Online", serde_json::json!(false)),
            ("/Self/ID", serde_json::json!("")),
            ("/CurrentTailnet", serde_json::Value::Null),
        ] {
            let mut invalid = valid.clone();
            *invalid.pointer_mut(pointer).unwrap() = value;
            assert!(!tailnet_status_matches(
                &invalid,
                "developer.example.ts.net"
            ));
        }
        let wrong_shape = serde_json::json!({
            "Self": {"ID": "fixture-node", "Online": true,
                "BackendState": "Running", "MagicDNSSuffix": "developer.example.ts.net"}
        });
        assert!(!tailnet_status_matches(
            &wrong_shape,
            "developer.example.ts.net"
        ));
        assert!(!tailnet_status_matches(
            &serde_json::Value::Null,
            "developer.example.ts.net"
        ));
    }

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
        unsafe {
            std::env::set_var("FOCUSA_DEV_ORIGIN_TTL_MS", "1000");
        }
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
        assert!(developer_origin_active_with(
            move || {
                counter2.fetch_add(1, Ordering::SeqCst);
                true
            },
            || false
        )); // re-probe
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        match previous_ttl {
            Some(value) => unsafe { std::env::set_var("FOCUSA_DEV_ORIGIN_TTL_MS", value) },
            None => unsafe { std::env::remove_var("FOCUSA_DEV_ORIGIN_TTL_MS") },
        }
        invalidate_developer_origin_cache();
    }

    #[test]
    fn health_response_does_not_establish_known_machine() {
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
                // Liveness-only JSON must not become an identity grant, even
                // when returned from an authenticated request.
                let body = r#"{"status":"ok","ok":true}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        let previous_url = std::env::var("FOCUSA_AGENT_KB_API_URL").ok();
        let previous_token = std::env::var("FOCUSA_AGENT_KB_TOKEN").ok();
        unsafe {
            std::env::set_var("FOCUSA_AGENT_KB_TOKEN", "fixture-only-not-a-credential");
            std::env::set_var(
                "FOCUSA_AGENT_KB_API_URL",
                format!("http://127.0.0.1:{}", PORT.load(Ordering::SeqCst)),
            );
        }
        invalidate_developer_origin_cache();
        assert!(!developer_origin_active_with(probe_agent_kb_known, || {
            false
        }));
        handle.join().unwrap();
        match previous_token {
            Some(value) => unsafe { std::env::set_var("FOCUSA_AGENT_KB_TOKEN", value) },
            None => unsafe { std::env::remove_var("FOCUSA_AGENT_KB_TOKEN") },
        }
        match previous_url {
            Some(value) => unsafe { std::env::set_var("FOCUSA_AGENT_KB_API_URL", value) },
            None => unsafe { std::env::remove_var("FOCUSA_AGENT_KB_API_URL") },
        }
        invalidate_developer_origin_cache();
    }
}
