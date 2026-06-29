//! Route-scoped mutation rate limiting middleware.
//!
//! This is a local-first fixed-window guard for OWASP API4/CWE-400 posture.
//! It is intentionally simple: route + method + caller identity get a bounded
//! number of mutation requests per window, with env overrides for tests/deploys.

use axum::extract::Request;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct RateBucket {
    window_start: Instant,
    count: u32,
}

static MUTATION_BUCKETS: LazyLock<Mutex<HashMap<String, RateBucket>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn mutation_rate_limit_per_window() -> u32 {
    env_u32("FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW", 120)
}

fn mutation_rate_limit_window() -> Duration {
    Duration::from_millis(env_u64("FOCUSA_API_MUTATION_RATE_LIMIT_WINDOW_MS", 1_000).max(100))
}

fn mutation_rate_limit_enabled() -> bool {
    mutation_rate_limit_per_window() > 0
}

fn is_mutation_request(method: &Method, path: &str) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) && path != "/v1/health"
}

/// V2 P1.4: pre-auth pairing routes get a stricter cap so an
/// unauthenticated attacker cannot burn the global mutation bucket.
fn is_preauth_pairing_mutation(method: &Method, path: &str) -> bool {
    if !is_mutation_request(method, path) {
        return false;
    }
    // POST/DELETE/PUT to room create, room mac-offer, room join,
    // room approve, room firstrun — all pre-auth V2 Bridge Room paths.
    path.starts_with("/v1/connect/room/")
        || path.starts_with("/v1/device/pair/start")
        || path.starts_with("/v1/device/pair/complete")
        || path.starts_with("/v1/device/pair/status")
}

fn hash_value(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn caller_key(headers: &HeaderMap) -> String {
    if let Some(token_hash) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(hash_value)
    {
        return format!("bearer:{token_hash:016x}");
    }
    if let Some(forwarded) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("xff:{}", hash_value(forwarded));
    }
    if let Some(real_ip) = headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("ip:{}", hash_value(real_ip));
    }
    "local-anon".to_string()
}

fn rate_key(method: &Method, path: &str, headers: &HeaderMap) -> String {
    format!("{}:{}:{}", caller_key(headers), method.as_str(), path)
}

fn prune_stale_buckets(buckets: &mut HashMap<String, RateBucket>, now: Instant, window: Duration) {
    if buckets.len() <= 4096 {
        return;
    }
    let max_age = window.saturating_mul(4);
    buckets.retain(|_, bucket| now.duration_since(bucket.window_start) <= max_age);
}

fn request_is_limited(method: &Method, path: &str, headers: &HeaderMap) -> bool {
    if !mutation_rate_limit_enabled() || !is_mutation_request(method, path) {
        return false;
    }

    // V2 P1.4: pre-auth pairing routes get a tighter cap. An attacker
    // without a Bearer can't burn the global bucket, but they shouldn't
    // be able to brute-force room codes either.
    let limit = if is_preauth_pairing_mutation(method, path) {
        env_u32("FOCUSA_PREAUTH_PAIRING_RATE_LIMIT_PER_WINDOW", 20)
    } else {
        mutation_rate_limit_per_window()
    };
    let window = mutation_rate_limit_window();
    let now = Instant::now();
    let key = rate_key(method, path, headers);
    let Ok(mut buckets) = MUTATION_BUCKETS.lock() else {
        return false;
    };
    prune_stale_buckets(&mut buckets, now, window);
    let bucket = buckets.entry(key).or_insert(RateBucket {
        window_start: now,
        count: 0,
    });
    if now.duration_since(bucket.window_start) >= window {
        bucket.window_start = now;
        bucket.count = 0;
    }
    bucket.count = bucket.count.saturating_add(1);
    bucket.count > limit
}

pub async fn mutation_rate_limit_layer(req: Request, next: Next) -> Result<Response, StatusCode> {
    if request_is_limited(req.method(), req.uri().path(), req.headers()) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_detection_skips_read_methods() {
        assert!(!is_mutation_request(&Method::GET, "/v1/project/identity"));
        assert!(!is_mutation_request(&Method::HEAD, "/v1/health"));
        assert!(is_mutation_request(
            &Method::POST,
            "/v1/workpoint/checkpoint"
        ));
    }

    #[test]
    fn caller_key_hashes_bearer_token_without_returning_secret() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer secret-token".parse().unwrap());
        let key = caller_key(&headers);
        assert!(key.starts_with("bearer:"));
        assert!(!key.contains("secret-token"));
    }
}
