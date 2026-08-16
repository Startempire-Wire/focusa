//! Focusa LicenseGuard — facade over the unified entitlement engine.
//!
//! #119 slice 3: tier/capability truth now lives in
//! `focusa_core::license` (the single decision point). This crate keeps its
//! public names for API compatibility and retains the local guard/record
//! helpers (resolve/persist/sha) that are installer-facing only.

pub use focusa_core::license::{
    capability_for_feature, entitlement_check, Capability, CapabilityCheck, Tier,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// LicenseGuard evaluates a Tier against Capabilities and decides permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseGuard {
    pub tier: Tier,
    pub key_hash: Option<String>,
    pub customer_email: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub bsl_change_date: DateTime<Utc>,
}

impl LicenseGuard {
    /// Construct an Eval guard (self-issued, no key, offline grace window).
    pub fn eval(duration_days: i64) -> Self {
        let now = Utc::now();
        Self {
            tier: Tier::Eval,
            key_hash: None,
            customer_email: None,
            issued_at: now,
            expires_at: Some(now + chrono::Duration::days(duration_days)),
            bsl_change_date: bsl_change_date(),
        }
    }

    /// Construct a Licensed guard (key-required).
    pub fn licensed(key_hash: String, customer_email: String) -> Self {
        let now = Utc::now();
        Self {
            tier: Tier::Licensed,
            key_hash: Some(key_hash),
            customer_email: Some(customer_email),
            issued_at: now,
            expires_at: None,
            bsl_change_date: bsl_change_date(),
        }
    }

    /// Returns true if the license has expired (only relevant for Eval).
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(e) => Utc::now() > e,
            None => false,
        }
    }

    /// Check a capability against the current tier.
    pub fn check(&self, capability: Capability) -> CapabilityCheck {
        match (self.tier, capability) {
            // Local-eval: always permitted.
            (_, Capability::LocalEval) => CapabilityCheck::Permitted,
            // Eval tier: commercial/hosted/embedding denied; commercial_use yields a warning
            // because eval can be used for evaluation on a real project but not for revenue.
            (Tier::Eval, Capability::CommercialUse) => {
                if self.is_expired() {
                    CapabilityCheck::Denied {
                        reason: format!(
                            "license expired at {} (eval grace window); renew or purchase commercial license",
                            self.expires_at
                                .map(|d| d.to_rfc3339())
                                .unwrap_or_else(|| "?".into())
                        ),
                    }
                } else {
                    CapabilityCheck::PermittedWithWarning {
                        warning:
                            "eval tier permits evaluation on a real project but not commercial use; purchase license for revenue".into(),
                    }
                }
            }
            (Tier::Eval, Capability::HostedMode) | (Tier::Eval, Capability::ProductEmbedding) => {
                CapabilityCheck::Denied {
                    reason: format!(
                        "eval tier does not permit {}; purchase commercial license",
                        capability.label()
                    ),
                }
            }
            (Tier::Eval, Capability::TelemetrySend) => CapabilityCheck::Denied {
                reason: "eval/license/open tiers: Focusa is no-telemetry by default".into(),
            },
            // Licensed tier: all capabilities permitted (subject to expiry checks elsewhere).
            (Tier::Licensed, _) => CapabilityCheck::Permitted,
            // Open tier: same as licensed for capability gating; commercial/hosted permitted.
            (Tier::Open, _) => CapabilityCheck::Permitted,
        }
    }

    /// Hard-require a capability. Returns Ok(()) when permitted (possibly with warning
    /// in caller-routed logs), Err(LicenseError::Denied{..}) when denied.
    pub fn require(&self, capability: Capability) -> Result<Option<String>, LicenseError> {
        match self.check(capability) {
            CapabilityCheck::Permitted => Ok(None),
            CapabilityCheck::PermittedWithWarning { warning } => Ok(Some(warning)),
            CapabilityCheck::Denied { reason } => Err(LicenseError::Denied {
                capability: capability.label().into(),
                tier: self.tier.label().into(),
                reason,
            }),
        }
    }
}

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LicenseError {
    #[error("license denied: tier={tier} does not permit {capability}: {reason}")]
    Denied {
        capability: String,
        tier: String,
        reason: String,
    },
}

/// BSL change date placeholder (4 years from typical release cadence).
/// Per operator rule 2026-07-08: Update when BSL change date is finalized.
fn bsl_change_date() -> DateTime<Utc> {
    // Hardcoded safe default. Real release uses release pipeline.
    chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Resolve a LicenseGuard from the operator's license state. Reads in this order:
///   1. env FOCUSA_LICENSE_KEY + LICENSE_REGISTRY (commercial validate)
///   2. ~/.config/focusa/license.json (cached license record)
///   3. ~/.focusa/license.toml (per-project override)
///   4. Self-issued eval (default)
pub fn resolve_license_guard() -> LicenseGuard {
    if let Ok(key) = std::env::var("FOCUSA_LICENSE_KEY")
        && !key.trim().is_empty()
        && let Ok(registry) = std::env::var("FOCUSA_LICENSE_REGISTRY")
    {
        let key_hash = sha256_short(&key);
        if let Ok(email) = std::env::var("FOCUSA_LICENSE_EMAIL") {
            return LicenseGuard::licensed(key_hash, email);
        }
        // Key without email \u2014 default to licensed with placeholder email from registry host.
        return LicenseGuard::licensed(key_hash, format!("owner@{registry}"));
    }

    if let Some(guard) = read_license_json() {
        return guard;
    }
    if let Some(guard) = read_license_toml() {
        return guard;
    }
    LicenseGuard::eval(7) // default 7-day offline grace
}

/// Read ~/.config/focusa/license.json and construct a guard.
fn read_license_json() -> Option<LicenseGuard> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".config/focusa/license.json");
    if !path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    Some(LicenseGuard {
        tier: parse_tier(json.get("tier")?.as_str()?)?,
        key_hash: json
            .get("key_hash")
            .and_then(|v| v.as_str())
            .map(String::from),
        customer_email: json
            .get("customer_email")
            .and_then(|v| v.as_str())
            .map(String::from),
        issued_at: parse_iso(json.get("issued_at")?.as_str()?)?,
        expires_at: json
            .get("expires_at")
            .and_then(|v| v.as_str())
            .and_then(parse_iso),
        bsl_change_date: parse_iso(json.get("bsl_change_date")?.as_str()?)
            .unwrap_or_else(bsl_change_date),
    })
}

/// Read ~/.focusa/license.toml (per-project override) and construct a guard.
fn read_license_toml() -> Option<LicenseGuard> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".focusa/license.toml");
    if !path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    let table: toml::Value = toml::from_str(&raw).ok()?;
    Some(LicenseGuard {
        tier: parse_tier(table.get("tier")?.as_str()?)?,
        key_hash: table
            .get("key_hash")
            .and_then(|v| v.as_str())
            .map(String::from),
        customer_email: table
            .get("customer_email")
            .and_then(|v| v.as_str())
            .map(String::from),
        issued_at: parse_iso(table.get("issued_at")?.as_str()?)?,
        expires_at: table
            .get("expires_at")
            .and_then(|v| v.as_str())
            .and_then(parse_iso),
        bsl_change_date: parse_iso(table.get("bsl_change_date")?.as_str()?)
            .unwrap_or_else(bsl_change_date),
    })
}

fn parse_tier(s: &str) -> Option<Tier> {
    match s.trim().to_ascii_lowercase().as_str() {
        "eval" => Some(Tier::Eval),
        "licensed" => Some(Tier::Licensed),
        "open" => Some(Tier::Open),
        _ => None,
    }
}

fn parse_iso(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Short SHA256 fingerprint (first 16 hex chars) of a license key, for logs.
pub fn sha256_short(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    let v = h.finish();
    format!("{:016x}", v)
}

/// Write a license record to ~/.config/focusa/license.json (used by installer).
pub fn persist_eval_license(home: &Path) -> std::io::Result<LicenseGuard> {
    let dir = home.join(".config/focusa");
    std::fs::create_dir_all(&dir)?;
    let guard = LicenseGuard::eval(7);
    let json = serde_json::to_string_pretty(&guard).map_err(std::io::Error::other)?;
    std::fs::write(dir.join("license.json"), json)?;
    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_tier_permits_local_eval() {
        let g = LicenseGuard::eval(7);
        assert_eq!(g.check(Capability::LocalEval), CapabilityCheck::Permitted);
    }

    #[test]
    fn eval_tier_warns_on_commercial_use_when_fresh() {
        let g = LicenseGuard::eval(7);
        let c = g.check(Capability::CommercialUse);
        assert!(matches!(c, CapabilityCheck::PermittedWithWarning { .. }));
    }

    #[test]
    fn eval_tier_denies_hosted_mode() {
        let g = LicenseGuard::eval(7);
        let c = g.check(Capability::HostedMode);
        assert!(c.is_denied());
    }

    #[test]
    fn eval_tier_denies_product_embedding() {
        let g = LicenseGuard::eval(7);
        let c = g.check(Capability::ProductEmbedding);
        assert!(c.is_denied());
    }

    #[test]
    fn licensed_tier_permits_commercial_use() {
        let g = LicenseGuard::licensed("abc123".into(), "v@x.com".into());
        assert_eq!(
            g.check(Capability::CommercialUse),
            CapabilityCheck::Permitted
        );
    }

    #[test]
    fn licensed_tier_permits_hosted_mode() {
        let g = LicenseGuard::licensed("abc123".into(), "v@x.com".into());
        assert_eq!(g.check(Capability::HostedMode), CapabilityCheck::Permitted);
    }

    #[test]
    fn open_tier_permits_everything() {
        let g = LicenseGuard {
            tier: Tier::Open,
            key_hash: None,
            customer_email: None,
            issued_at: Utc::now(),
            expires_at: None,
            bsl_change_date: bsl_change_date(),
        };
        assert_eq!(
            g.check(Capability::CommercialUse),
            CapabilityCheck::Permitted
        );
        assert_eq!(g.check(Capability::HostedMode), CapabilityCheck::Permitted);
        assert_eq!(
            g.check(Capability::ProductEmbedding),
            CapabilityCheck::Permitted
        );
    }

    #[test]
    fn require_returns_warning_or_denied() {
        let g = LicenseGuard::eval(7);
        // CommercialUse should warn.
        let r = g.require(Capability::CommercialUse);
        assert!(r.is_ok());
        // HostedMode should deny.
        let r = g.require(Capability::HostedMode);
        assert!(r.is_err());
    }

    #[test]
    fn tier_label_round_trip() {
        assert_eq!(Tier::Eval.label(), "eval");
        assert_eq!(Tier::Licensed.label(), "licensed");
        assert_eq!(Tier::Open.label(), "open");
        for t in [Tier::Eval, Tier::Licensed, Tier::Open] {
            let json = serde_json::to_string(&t).unwrap();
            let back: Tier = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }
}
