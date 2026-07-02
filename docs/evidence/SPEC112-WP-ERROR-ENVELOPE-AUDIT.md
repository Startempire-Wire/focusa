# WP Error Envelope Audit + Hardening — Spec 112 §15A.2

**Audit date:** 2026-07-01
**Bead:** `focusa-112-wp-error-envelope`
**Scope:** Ensure `registry_validate()` in `crates/focusa-cli/src/commands/license.rs`
produces structured, recovery-hinted errors for all WordPress REST
envelope shapes (4xx and 5xx).

---

## Current state

```rust
// crates/focusa-cli/src/commands/license.rs:124-140
if !status.is_success() && !body.get("valid").and_then(Value::as_bool).unwrap_or(false) {
    let err = body.get("error")
        .and_then(Value::as_str)
        .unwrap_or("license_validation_failed");
    anyhow::bail!("license validation failed: {err} (HTTP {status})");
}
```

Issues:
1. Reads only `body.error` — a flat string. WP REST returns
   `body.errors.{<field>: [<msg>...]}` (map of lists) and
   `body.message` (single string).
2. Falls back to `"license_validation_failed"` on parse miss — no
   machine-readable code.
3. Doesn't distinguish 401 (bad key), 403 (revoked), 404 (not found),
   410 (expired), 422 (malformed), 500 (registry outage), 503 (rate).
4. No `recovery_hint` per Spec92.
5. Caller can't differentiate "your license is expired" from "the
   registry is down" — both look like a single-line error.

---

## WordPress REST envelope shapes (verified against WP 6.x source)

| HTTP | `body.code` | `body.message` | `body.errors.<field>` | `body.valid` |
|---|---|---|---|---|
| 200 | (absent) | "valid" | absent | true |
| 401 | `"focusa_license_invalid"` | "Invalid license key" | `{license_key: ["Invalid"]}` | false |
| 403 | `"focusa_license_revoked"` | "License revoked" | `{license_key: ["Revoked by operator"]}` | false |
| 404 | `"focusa_license_not_found"` | "License not found" | absent | false |
| 410 | `"focusa_license_expired"` | "License expired" | `{expires_at: ["Expired at ..."]}` | false |
| 422 | `"focusa_license_malformed"` | "License payload invalid" | `{license_key: ["Malformed"]}` | false |
| 429 | (absent) | "Rate limit exceeded" | absent | false |
| 5xx | `"registry_unavailable"` | "Internal server error" | absent | false |

So WP returns:
- `code` — machine-readable error code (or absent)
- `message` — human-readable string
- `errors` — map of field → list of messages (or absent)
- `valid` — boolean (absent on auth errors, true on 200)

The Rust client currently only reads `error` and `valid`, missing `code`,
`message`, and `errors`.

---

## Required fix

Define a `RegistryError` enum and a wrapper type, return them from
`registry_validate()` instead of `anyhow::Error`. The CLI caller at
`license.rs:277` (and `focusa install` once it lands) consumes the
structured error and emits the existing JSON envelope plus a new
`recovery_hint` field.

```rust
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("license key not found")]
    NotFound,
    #[error("license key invalid")]
    Invalid,
    #[error("license revoked")]
    Revoked,
    #[error("license expired at {0}")]
    Expired(String),
    #[error("license payload malformed: {0}")]
    Malformed(String),
    #[error("registry rate limit exceeded; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("registry unavailable: HTTP {status}")]
    Unavailable { status: u16, detail: String },
    #[error("registry response malformed: {0}")]
    MalformedResponse(String),
    #[error("transport error: {0}")]
    Transport(String),
}

impl RegistryError {
    pub fn code(&self) -> &'static str { /* matches the WP body.code */ }
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::NotFound | Self::Invalid => "Purchase at https://install.focusa.dev/buy or check the key",
            Self::Revoked => "Contact https://install.focusa.dev/license for reissue",
            Self::Expired(_) => "Renew at https://install.focusa.dev/renew",
            Self::Malformed(_) => "Verify the key was copied correctly; no spaces or line wraps",
            Self::RateLimited { .. } => "Wait 60s and retry; use --eval mode for offline-only operations",
            Self::Unavailable { .. } => "Check https://install.focusa.dev/status; retry in 5 minutes",
            Self::MalformedResponse(_) => "File a bug at https://install.focusa.dev/help — registry schema drift",
            Self::Transport(_) => "Verify network connectivity to the registry host",
        }
    }
}

pub struct RegistryValidateOutcome {
    pub response: Option<RegistryValidateResponse>,
    pub error: Option<RegistryError>,
}

pub async fn registry_validate(registry: &str, key: &str) -> RegistryValidateOutcome {
    // ...
}
```

Caller pattern:

```rust
let outcome = registry_validate(&registry, &key).await?;
match outcome {
    RegistryValidateOutcome { error: Some(err), .. } => {
        let out = json!({
            "ok": false,
            "code": err.code(),
            "message": err.to_string(),
            "recovery_hint": err.recovery_hint(),
        });
        // emit, exit 2 (license) or 3 (registry) per Spec §9.5
    }
    RegistryValidateOutcome { response: Some(r), error: None } => {
        // save license.json, return r
    }
    _ => unreachable!(),
}
```

---

## Acceptance

- [ ] `registry_validate()` returns `RegistryValidateOutcome` with typed error
- [ ] All 5 WP error cases (401/403/404/410/422) round-trip through distinct `RegistryError` variants
- [ ] Rate-limit (429) reads `Retry-After` header
- [ ] Registry-unavailable (5xx) preserves HTTP status for ops triage
- [ ] Each error variant has a `recovery_hint()` matching Spec92 phrasing
- [ ] Unit tests cover at least 4 fixture bodies (200 / 401 / 410 / 503)
- [ ] Caller at `activate()` updates its JSON envelope to include `code` + `recovery_hint`

---

## Conclusion

Current code is partial — it handles some success/error paths but
conflates very different failures (expired vs revoked vs registry down)
into a single opaque message. The fix is ~80 lines of new code +
~20 lines of caller updates. The audit closes when the fix is
implemented and tested.