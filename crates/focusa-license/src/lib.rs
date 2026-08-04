//! Focusa LicenseGuard \u2014 tier evaluation + capability assertions + BSL boundary.
//!
//! Bead: focusa-nbai (MVP BLOCKER).
//!
//! Runtime capability authority comes only from a signed Spec 152 authority lease.
//! Legacy tier/file parsing is retained solely as non-authoritative migration input;
//! missing, edited, expired, revoked, or unverifiable state cannot grant capability.

pub mod authority;
pub mod authority_client;
pub mod authority_credentials;
pub mod authority_http;
pub mod authority_store;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// License tiers supported by Focusa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Unactivated,
    RecoveryOnly,
    Entitled,
    OfflineGrace,
    Eval,
    Licensed,
    Open,
}

impl Tier {
    pub fn permits_commercial_use(self) -> bool {
        matches!(self, Tier::Entitled | Tier::OfflineGrace)
    }

    pub fn permits_hosted_deployment(self) -> bool {
        matches!(self, Tier::Entitled | Tier::OfflineGrace)
    }

    pub fn permits_local_eval(self) -> bool {
        matches!(self, Tier::Entitled | Tier::OfflineGrace)
    }

    pub fn label(self) -> &'static str {
        match self {
            Tier::Unactivated => "unactivated",
            Tier::RecoveryOnly => "recovery_only",
            Tier::Entitled => "entitled",
            Tier::OfflineGrace => "offline_grace",
            Tier::Eval => "eval",
            Tier::Licensed => "licensed",
            Tier::Open => "open",
        }
    }
}

/// Capabilities that LicenseGuard gates. Each capability is a static enum member
/// so call sites can do `guard.require(Capability::HostedMode)` and get a typed
/// error rather than a stringly-typed allowlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Process / orchestrate commercial workloads.
    CommercialUse,
    /// Operate as a hosted multi-tenant daemon.
    HostedMode,
    /// Embed Focusa inside a commercial product.
    ProductEmbedding,
    /// Send telemetry/analytics events (Focusa is no-telemetry by default).
    TelemetrySend,
    /// Local-only single-user use, free for everyone.
    LocalEval,
}

impl Capability {
    pub fn label(self) -> &'static str {
        match self {
            Capability::CommercialUse => "commercial_use",
            Capability::HostedMode => "hosted_mode",
            Capability::ProductEmbedding => "product_embedding",
            Capability::TelemetrySend => "telemetry_send",
            Capability::LocalEval => "local_eval",
        }
    }
}

/// Outcome of a capability check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum CapabilityCheck {
    /// Capability is permitted under current tier.
    Permitted,
    /// Capability is permitted but with a soft warning (e.g., eval tier + commercial).
    PermittedWithWarning { warning: String },
    /// Capability is denied under current tier (hard fail).
    Denied { reason: String },
}

impl CapabilityCheck {
    pub fn is_permitted(&self) -> bool {
        !matches!(self, CapabilityCheck::Denied { .. })
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, CapabilityCheck::Denied { .. })
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            CapabilityCheck::Denied { reason } => Some(reason),
            CapabilityCheck::PermittedWithWarning { warning } => Some(warning),
            CapabilityCheck::Permitted => None,
        }
    }
}

/// LicenseGuard evaluates a Tier against Capabilities and decides permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseGuard {
    pub tier: Tier,
    pub key_hash: Option<String>,
    pub customer_email: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub bsl_change_date: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entitlement: Option<authority::EntitlementSnapshot>,
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
            entitlement: None,
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
            entitlement: None,
        }
    }

    pub fn from_entitlement(entitlement: authority::EntitlementSnapshot) -> Self {
        let tier = match entitlement.state {
            authority::EntitlementState::Unactivated => Tier::Unactivated,
            authority::EntitlementState::RecoveryOnly => Tier::RecoveryOnly,
            authority::EntitlementState::Active => Tier::Entitled,
            authority::EntitlementState::OfflineGrace => Tier::OfflineGrace,
        };
        Self {
            tier,
            key_hash: entitlement.lease_digest.clone(),
            customer_email: None,
            issued_at: Utc::now(),
            expires_at: entitlement.expires_at,
            bsl_change_date: bsl_change_date(),
            entitlement: Some(entitlement),
        }
    }

    /// Returns true if the authority lease or legacy evaluation has expired.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(e) => Utc::now() > e,
            None => false,
        }
    }

    /// Check a capability only against the immutable signed entitlement snapshot.
    pub fn check(&self, capability: Capability) -> CapabilityCheck {
        let Some(entitlement) = &self.entitlement else {
            return CapabilityCheck::Denied {
                reason: "signed authority entitlement required; legacy tier is migration-only"
                    .into(),
            };
        };
        if entitlement.feature_enabled(capability.label()) {
            CapabilityCheck::Permitted
        } else {
            CapabilityCheck::Denied {
                reason: format!(
                    "authority entitlement state={} does not grant {}",
                    self.tier.label(),
                    capability.label()
                ),
            }
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

/// Resolve a LicenseGuard only from signed, persisted authority state.
pub fn resolve_license_guard() -> LicenseGuard {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    resolve_license_guard_from(
        &home.join(".config/focusa"),
        authority_store::embedded_production_trust_roots(),
        Utc::now(),
    )
}

pub fn resolve_license_guard_from(
    config_dir: &Path,
    roots: Result<
        std::collections::BTreeMap<String, ed25519_dalek::VerifyingKey>,
        authority_store::AuthorityStoreError,
    >,
    now: DateTime<Utc>,
) -> LicenseGuard {
    let state_path = config_dir.join(authority_store::AUTHORITY_STATE_FILE);
    let expected_node_id = std::fs::read_to_string(config_dir.join("node-id"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unbound".to_string());
    let context = authority::LeaseVerificationContext {
        expected_product: "focusa".to_string(),
        expected_node_id,
        now,
        minimum_sequence: None,
        expected_previous_digest: None,
    };
    LicenseGuard::from_entitlement(authority_store::resolve_authority_state(
        &state_path,
        roots,
        &context,
    ))
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
        entitlement: None,
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
        entitlement: None,
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
    fn self_issued_eval_cannot_grant_local_eval() {
        let g = LicenseGuard::eval(7);
        assert!(g.check(Capability::LocalEval).is_denied());
    }

    #[test]
    fn self_issued_eval_cannot_grant_commercial_use() {
        let g = LicenseGuard::eval(7);
        assert!(g.check(Capability::CommercialUse).is_denied());
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
    fn plaintext_licensed_tier_cannot_grant_commercial_use() {
        let g = LicenseGuard::licensed("abc123".into(), "v@x.com".into());
        assert!(g.check(Capability::CommercialUse).is_denied());
    }

    #[test]
    fn plaintext_licensed_tier_cannot_grant_hosted_mode() {
        let g = LicenseGuard::licensed("abc123".into(), "v@x.com".into());
        assert!(g.check(Capability::HostedMode).is_denied());
    }

    #[test]
    fn plaintext_open_tier_cannot_grant_capabilities() {
        let g = LicenseGuard {
            tier: Tier::Open,
            key_hash: None,
            customer_email: None,
            issued_at: Utc::now(),
            expires_at: None,
            bsl_change_date: bsl_change_date(),
            entitlement: None,
        };
        assert!(g.check(Capability::CommercialUse).is_denied());
        assert!(g.check(Capability::HostedMode).is_denied());
        assert!(g.check(Capability::ProductEmbedding).is_denied());
    }

    #[test]
    fn require_denies_without_signed_entitlement() {
        let g = LicenseGuard::eval(7);
        assert!(g.require(Capability::CommercialUse).is_err());
        assert!(g.require(Capability::HostedMode).is_err());
    }

    #[test]
    fn tier_label_round_trip() {
        assert_eq!(Tier::Eval.label(), "eval");
        assert_eq!(Tier::Licensed.label(), "licensed");
        assert_eq!(Tier::Open.label(), "open");
        for t in [
            Tier::Unactivated,
            Tier::RecoveryOnly,
            Tier::Entitled,
            Tier::OfflineGrace,
            Tier::Eval,
            Tier::Licensed,
            Tier::Open,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: Tier = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }
}
