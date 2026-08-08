//! Focusa runtime entitlement helper — Spec92 §5.5.
//!
//! Provides a central API to load the local license state, check whether a specific feature
//! is enabled by the current license, and require a feature (returning a structured error
//! when the feature is gated).
//!
//! The local license file lives at `~/.config/focusa/license.json` (Spec §5.1) and is written
//! with `chmod 600`. The file stores a SHA-256 hash of the raw key (NOT the raw key) and the
//! license identity received from the registry. The raw key is only persisted when the
//! operator explicitly passes `--persist-key` to `focusa license activate`.
//!
//! # Public API
//!
//! ```ignore
//! use focusa_core::license::{load_license_status, feature_enabled, require_feature};
//!
//! if let Ok(status) = load_license_status() {
//!     if feature_enabled("public_stream") {
//!         // enable public stream
//!     }
//! }
//!
//! require_feature("menubar_packaged_app")?; // returns Err if gated
//! ```

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use crate::entitlement_execution_guard::{
    evaluate_entitlement_execution,
    EntitlementExecutionContext,
    EntitlementExecutionDecision,
    EntitlementExecutionFailure,
    EntitlementExecutionPolicy,
};

const LICENSE_FILE: &str = "license.json";
const CONFIG_DIR: &str = ".config";
const FOCUSA_DIR: &str = "focusa";
const HASH_PREFIX_LEN: usize = 16; // Spec §5.1: store prefix only, never raw key

/// License mode per spec §5.3 (Evaluation, Operator, FoundersForge, Team, Enterprise).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseMode {
    Unactivated,
    RecoveryOnly,
    Entitled,
    OfflineGrace,
    Evaluation,
    Operator,
    FoundersForge,
    Team,
    Enterprise,
}

impl LicenseMode {
    /// Human-readable label for status output.
    pub fn label(self) -> &'static str {
        match self {
            LicenseMode::Unactivated => "Unactivated",
            LicenseMode::RecoveryOnly => "RecoveryOnly",
            LicenseMode::Entitled => "Entitled",
            LicenseMode::OfflineGrace => "OfflineGrace",
            LicenseMode::Evaluation => "Evaluation",
            LicenseMode::Operator => "Operator",
            LicenseMode::FoundersForge => "FoundersForge",
            LicenseMode::Team => "Team",
            LicenseMode::Enterprise => "Enterprise",
        }
    }
}

/// Local license file shape (Spec §5.1).
/// Always written with `chmod 600`; `key_hash` is the SHA-256 of the raw key, never the raw key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLicense {
    /// SHA-256 hash of the raw license key. Never the raw key.
    pub key_hash: String,
    /// First 16 characters of the raw key, for human display only.
    pub key_prefix: String,
    /// License identity returned by the registry (e.g. "focusa", "uiai-engine").
    pub product: String,
    /// Tier (e.g. "operator", "founders-forge").
    pub tier: String,
    /// License status returned by the registry (e.g. "active", "revoked").
    pub status: String,
    /// Whether commercial use is permitted by the current license.
    #[serde(default)]
    pub commercial_use: bool,
    /// Customer email from the registry (used for reissue, refund, security contacts).
    /// Spec §5.1: local license file shape includes customer_email.
    #[serde(default)]
    pub customer_email: String,
    /// Feature list enabled by this license (e.g. ["focusa_operator", "packaged_installer"]).
    #[serde(default)]
    pub features: Vec<String>,
    /// ISO 8601 timestamp after which the operator should re-validate with the registry.
    /// Used for the offline grace period.
    #[serde(default)]
    pub offline_valid_until: Option<String>,
    /// ISO 8601 expiration of the license itself, if any.
    #[serde(default)]
    pub expires_at: Option<String>,
    /// Whether the license was activated in evaluation mode.
    #[serde(default)]
    pub eval: bool,
    /// The registry URL used to validate this key.
    #[serde(default)]
    pub registry: String,
    /// Unix timestamp of activation.
    #[serde(default)]
    pub issued_at: u64,
}

impl LocalLicense {
    /// Construct an evaluation license (no real key, no commercial use).
    pub fn evaluation() -> Self {
        Self {
            key_hash: String::new(),
            key_prefix: String::new(),
            product: "focusa".to_string(),
            tier: "evaluation".to_string(),
            status: "active".to_string(),
            commercial_use: false,
            customer_email: String::new(),
            features: vec![],
            offline_valid_until: None,
            expires_at: None,
            eval: true,
            registry: String::new(),
            issued_at: now_unix(),
        }
    }

    /// The mode derived from this license. Evaluation if `eval` is true or features are empty.
    pub fn mode(&self) -> LicenseMode {
        if self.eval || (!self.commercial_use && self.features.is_empty()) {
            LicenseMode::Evaluation
        } else {
            match self.tier.as_str() {
                "operator" => LicenseMode::Operator,
                "founders-forge" | "founders_forge" => LicenseMode::FoundersForge,
                "team" => LicenseMode::Team,
                "enterprise" => LicenseMode::Enterprise,
                _ => LicenseMode::RecoveryOnly,
            }
        }
    }
}

/// The runtime status that `load_license_status` returns to callers. Matches the spec §5.2
/// `focusa license status` output shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    /// Canonical signed-authority projection. `None` is migration-only and must
    /// never be interpreted as an entitlement grant.
    pub authority: Option<focusa_license::EntitlementProjection>,
    /// License mode.
    pub mode: LicenseMode,
    /// Product (focusa, uiai-engine, bundle, founders-forge).
    pub product: String,
    /// Tier (operator, evaluation, ...).
    pub tier: String,
    /// Status (active, revoked, refunded, expired).
    pub status: String,
    /// Whether commercial use is permitted.
    pub commercial_use: bool,
    /// Customer email from the registry.
    pub customer_email: String,
    /// Enabled features.
    pub features: Vec<String>,
    /// ISO 8601 expiration of the license.
    pub expires_at: Option<String>,
    /// ISO 8601 offline grace period.
    pub offline_valid_until: Option<String>,
    /// License key prefix for display (never the raw key).
    pub key_prefix: String,
}

/// Structured error for `require_feature` and other license failures. Maps to spec §11
/// error JSON shape on the wire when returned to a CLI caller.
#[derive(Debug, thiserror::Error)]
pub enum LicenseError {
    #[error("feature '{0}' requires a Focusa Operator or Commercial license")]
    FeatureRequiresLicense(String),
    #[error("license file not found at {0}")]
    FileNotFound(PathBuf),
    #[error("license file unreadable: {0}")]
    FileUnreadable(String),
    #[error("license file invalid JSON: {0}")]
    FileInvalid(String),
    #[error("license expired on {0}")]
    Expired(String),
    #[error("license revoked (status={0})")]
    Revoked(String),
    #[error("registry unreachable: {0}")]
    RegistryUnreachable(String),
    #[error("evaluation mode — feature '{0}' not permitted")]
    EvaluationRestricted(String),
    #[error("base Focusa product gate not satisfied (decision={0}); one usable signed product entitlement is required for value-producing core mutations")]
    BaseProductRequired(String),
}

/// Doctor report for `focusa license doctor` per spec §5.2.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DoctorReport {
    pub license_file: String,
    pub file_exists: bool,
    pub file_readable: bool,
    pub not_expired: bool,
    pub registry_reachable: bool,
    pub features_loaded: bool,
    pub eval_mode: bool,
    pub warnings: Vec<String>,
    pub failures: Vec<String>,
}

/// Require the canonical base Focusa product gate for value-producing core
/// mutations (Spec 152F P3). One usable signed product entitlement for product
/// `focusa` gates the base; the legacy `focusa.core.mission` / `focusa.core.workpoint` /
/// `focusa.core.evidence` identifiers are compatibility/projection claims, never
/// separately purchased features.
pub fn require_base_product() -> Result<focusa_license::BaseProductProjection, LicenseError> {
    let guard = focusa_license::resolve_license_guard();
    let policy = EntitlementExecutionPolicy::new(
        "focusa.core.mutation.base_focusa",
        focusa_license::OperationClass::ValueMutation,
        focusa_license::CapabilityFamily::BaseFocusa,
        None,
        None,
        focusa_license::RecoveryAllowance::None,
    );
    if let Err(error) = evaluate_entitlement_execution(
        &guard,
        &policy,
        EntitlementExecutionContext::default(),
    ) {
        return Err(LicenseError::BaseProductRequired(error.code));
    }
    let projection = focusa_license::base_product_projection(guard.entitlement.as_ref())
        .map_err(|_| LicenseError::BaseProductRequired("snapshot_missing".to_string()))?;
    if projection.permits_base_mutations {
        Ok(projection)
    } else {
        Err(LicenseError::BaseProductRequired(projection.decision))
    }
}

/// Path to the local license file. Resolves to `~/.config/focusa/license.json`.
pub fn license_file_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    home.join(CONFIG_DIR).join(FOCUSA_DIR).join(LICENSE_FILE)
}

/// Load the one canonical signed authority entitlement projection.
pub fn load_license_status() -> anyhow::Result<LicenseStatus> {
    let guard = focusa_license::resolve_license_guard();
    let entitlement = guard.entitlement.as_ref();
    let authority = focusa_license::entitlement_projection(entitlement)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let mode = match entitlement.map(|snapshot| snapshot.state) {
        Some(focusa_license::authority::EntitlementState::Active) => LicenseMode::Entitled,
        Some(focusa_license::authority::EntitlementState::OfflineGrace) => {
            LicenseMode::OfflineGrace
        }
        Some(focusa_license::authority::EntitlementState::Unactivated) => LicenseMode::Unactivated,
        Some(focusa_license::authority::EntitlementState::RecoveryOnly) | None => {
            LicenseMode::RecoveryOnly
        }
    };
    let features = entitlement
        .map(|snapshot| {
            snapshot
                .features
                .iter()
                .filter(|(_, enabled)| **enabled)
                .map(|(feature, _)| feature.clone())
                .collect()
        })
        .unwrap_or_default();
    Ok(LicenseStatus {
        authority: Some(authority),
        mode,
        product: entitlement
            .map(|snapshot| snapshot.product.clone())
            .unwrap_or_else(|| "focusa".to_string()),
        tier: guard.tier.label().to_string(),
        status: guard.tier.label().to_string(),
        commercial_use: matches!(
            guard.check(focusa_license::Capability::CommercialUse),
            focusa_license::CapabilityCheck::Permitted
        ),
        customer_email: String::new(),
        features,
        expires_at: entitlement
            .and_then(|snapshot| snapshot.expires_at)
            .map(|value| value.to_rfc3339()),
        offline_valid_until: entitlement
            .and_then(|snapshot| snapshot.offline_grace_until)
            .map(|value| value.to_rfc3339()),
        key_prefix: entitlement
            .and_then(|snapshot| snapshot.lease_digest.as_deref())
            .map(|digest| digest.chars().take(HASH_PREFIX_LEN).collect())
            .unwrap_or_default(),
    })
}

fn status_from_local(local: &LocalLicense) -> LicenseStatus {
    LicenseStatus {
        authority: None,
        mode: local.mode(),
        product: local.product.clone(),
        tier: local.tier.clone(),
        status: local.status.clone(),
        commercial_use: local.commercial_use,
        customer_email: local.customer_email.clone(),
        features: local.features.clone(),
        expires_at: local.expires_at.clone(),
        offline_valid_until: local.offline_valid_until.clone(),
        key_prefix: local.key_prefix.clone(),
    }
}

/// Read the raw `LocalLicense` from disk. Returns Evaluation if no file.
pub fn load_local_license() -> anyhow::Result<LocalLicense> {
    let path = license_file_path();
    if !path.exists() {
        return Ok(LocalLicense::evaluation());
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("read license file {}: {e}", path.display()))?;
    let local: LocalLicense = serde_json::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("parse license file {}: {e}", path.display()))?;
    Ok(local)
}

/// Check whether a specific feature is enabled by the current license.
/// Returns `true` if enabled, `false` if not (or if in evaluation mode).
pub fn feature_enabled(feature: &str) -> bool {
    match load_license_status() {
        Ok(status) if status.commercial_use && !status.features.is_empty() => {
            status.features.iter().any(|f| f == feature)
        }
        _ => false,
    }
}

/// Require a feature — returns `Ok(())` if enabled, `Err(LicenseError::FeatureRequiresLicense)`
/// if gated. Use this in feature paths per spec §5.5.
pub fn require_feature(feature: &str) -> Result<(), LicenseError> {
    if feature_enabled(feature) {
        return Ok(());
    }
    let mode = load_license_status().ok().map(|s| s.mode);
    match mode {
        Some(LicenseMode::Evaluation) => {
            Err(LicenseError::EvaluationRestricted(feature.to_string()))
        }
        _ => Err(LicenseError::FeatureRequiresLicense(feature.to_string())),
    }
}

/// Require the release-proof premium family for advanced governed release
/// orchestration and proof operations (Spec 152F §3, §4, §6).
///
/// Safe release status reads remain available through the ReadProjection
/// family; only mutation-class release orchestration and proof operations
/// require the `focusa.release.proof` feature grant.
pub fn require_release_proof() -> Result<(), LicenseError> {
    let guard = focusa_license::resolve_license_guard();
    let policy = EntitlementExecutionPolicy::new(
        "focusa.release.proof.orchestrate",
        focusa_license::OperationClass::ValueMutation,
        focusa_license::CapabilityFamily::ReleaseProof,
        Some("focusa.release.proof"),
        Some("release_proof_runs"),
        focusa_license::RecoveryAllowance::None,
    );
    match evaluate_entitlement_execution(
        &guard,
        &policy,
        EntitlementExecutionContext::default(),
    ) {
        Ok(_decision) => Ok(()),
        Err(failure) => Err(LicenseError::FeatureRequiresLicense(failure.code)),
    }
}

/// Require the export-packaged premium feature for value-added hosted
/// packaging, transformation, and report formats (Spec 152F §3.3, §8).
///
/// Basic customer-data export (JSONL, Parquet, silent-session retention
/// export) is always available through the CustomerDataExport recovery
/// allowance. This function gates only the optional `focusa.export.packaged`
/// additive premium feature. It does not require the base product gate
/// because basic export always works.
pub fn require_export_packaged() -> Result<(), LicenseError> {
    let guard = focusa_license::resolve_license_guard();
    let snapshot = guard
        .entitlement
        .as_ref()
        .ok_or_else(|| LicenseError::FeatureRequiresLicense(
            "focusa.export.packaged".to_string()
        ))?;
    match focusa_license::resolve_export_packaged(
        snapshot,
        "focusa.export.packaged",
        chrono::Utc::now(),
    ) {
        focusa_license::PremiumFamilyDecision::Feature { .. } => Ok(()),
        focusa_license::PremiumFamilyDecision::Denied(denial) => {
            Err(LicenseError::FeatureRequiresLicense(format!("{denial:?}")))
        }
    }
}

/// Activate a license: validate with the registry, write the local file. Spec §5.2.
/// Async because the registry call uses reqwest which needs a tokio runtime.
pub async fn activate(
    key: &str,
    registry: &str,
    persist_key: bool,
) -> anyhow::Result<LicenseStatus> {
    let key = key.trim();
    if key.is_empty() {
        anyhow::bail!("license key is empty");
    }
    let prefix: String = key.chars().take(HASH_PREFIX_LEN).collect();
    let key_hash = sha256_hex(key.as_bytes());

    // POST key to /wp-json/wpuiai-ai-cloud/v1/license/validate
    let resp = validate_with_registry(registry, key).await?;
    if !resp.valid {
        anyhow::bail!(
            "license not valid: error={} product={}",
            resp.error.unwrap_or_default(),
            resp.product.unwrap_or_default()
        );
    }
    let product = resp.product.clone().unwrap_or_default();
    let tier = resp.tier.clone().unwrap_or_default();
    let status = resp.status.clone().unwrap_or_else(|| "active".into());
    let commercial_use = resp.commercial_use.unwrap_or(false);
    let features = resp.features.clone().unwrap_or_default();
    let expires_at = resp.expires_at.clone();
    let customer_email = resp.customer_email.clone().unwrap_or_default();

    // Compute offline_valid_until: 7 days from now
    let offline_valid_until = iso860i_in_days(7);

    let local = LocalLicense {
        key_hash,
        key_prefix: prefix,
        product: product.clone(),
        tier: tier.clone(),
        status,
        commercial_use,
        customer_email,
        features: features.clone(),
        offline_valid_until: Some(offline_valid_until),
        expires_at,
        eval: false,
        registry: registry.to_string(),
        issued_at: now_unix(),
    };
    write_local_license(&local, persist_key.then_some(key))?;
    Ok(status_from_local(&local))
}

/// Deactivate: remove the local license file. Spec §5.2.
pub fn deactivate(license_file: &Path) -> anyhow::Result<()> {
    if license_file.exists() {
        std::fs::remove_file(license_file)
            .map_err(|e| anyhow::anyhow!("remove license file {}: {e}", license_file.display()))?;
    }
    Ok(())
}

/// Doctor: self-check of the local license state. Spec §5.2.
pub async fn doctor(license_file: &Path) -> anyhow::Result<DoctorReport> {
    let mut report = DoctorReport {
        license_file: license_file.display().to_string(),
        ..Default::default()
    };
    if !license_file.exists() {
        report
            .warnings
            .push("no license file (running in Evaluation mode)".to_string());
        report.eval_mode = true;
        return Ok(report);
    }
    report.file_exists = true;
    let raw = match std::fs::read_to_string(license_file) {
        Ok(s) => {
            report.file_readable = true;
            s
        }
        Err(e) => {
            report.failures.push(format!("unreadable: {e}"));
            return Ok(report);
        }
    };
    let local: LocalLicense = match serde_json::from_str(&raw) {
        Ok(l) => l,
        Err(e) => {
            report.failures.push(format!("invalid JSON: {e}"));
            return Ok(report);
        }
    };
    report.features_loaded = !local.features.is_empty();
    report.eval_mode = local.eval || (!local.commercial_use && local.features.is_empty());
    if let Some(ref exp) = local.expires_at {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(exp) {
            if dt < chrono::Utc::now() {
                report.not_expired = false;
                report.failures.push(format!("expired on {exp}"));
            } else {
                report.not_expired = true;
            }
        }
    } else {
        report.not_expired = true; // no expiry set
    }
    if local.status == "revoked" || local.status == "refunded" {
        report.failures.push(format!("status={}", local.status));
    }
    // Try registry reachability (best-effort, non-blocking)
    if let Ok(ok) = registry_ping_blocking(&local.registry).await {
        report.registry_reachable = ok;
        if !ok {
            report
                .warnings
                .push("registry unreachable — using local state".to_string());
        }
    } else {
        report
            .warnings
            .push("registry ping failed — using local state".to_string());
    }
    Ok(report)
}

/// Check a feature: returns a string reason when enabled, or `Err` when gated.
/// Used by `focusa license check-feature <name>` per spec §5.2.
pub fn check_feature(license_file: &Path, feature: &str) -> Result<String, LicenseError> {
    let _ = license_file;
    let status =
        load_license_status().map_err(|_| LicenseError::FileNotFound(license_file_path()))?;
    if status.commercial_use && status.features.iter().any(|f| f == feature) {
        // Provide a coarse reason label
        let reason = match status.mode {
            LicenseMode::Unactivated => "unactivated",
            LicenseMode::RecoveryOnly => "recovery_only",
            LicenseMode::Entitled => "signed_authority_lease",
            LicenseMode::OfflineGrace => "signed_authority_offline_grace",
            LicenseMode::Evaluation => "evaluation",
            LicenseMode::Operator => "operator_license",
            LicenseMode::FoundersForge => "founders_forge_license",
            LicenseMode::Team => "team_license",
            LicenseMode::Enterprise => "enterprise_license",
        };
        return Ok(reason.to_string());
    }
    if matches!(
        status.mode,
        LicenseMode::Unactivated | LicenseMode::RecoveryOnly | LicenseMode::Evaluation
    ) {
        return Err(LicenseError::EvaluationRestricted(feature.to_string()));
    }
    Err(LicenseError::FeatureRequiresLicense(feature.to_string()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

fn write_local_license(local: &LocalLicense, raw_key: Option<&str>) -> anyhow::Result<()> {
    let path = license_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("create license dir {}: {e}", parent.display()))?;
    }
    let serialized = if let Some(k) = raw_key {
        let mut with_key = serde_json::to_value(local)?;
        if let Some(obj) = with_key.as_object_mut() {
            obj.insert(
                "raw_key".to_string(),
                serde_json::Value::String(k.to_string()),
            );
        }
        serde_json::to_string_pretty(&with_key)?
    } else {
        serde_json::to_string_pretty(local)?
    };
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serialized.as_bytes())
        .map_err(|e| anyhow::anyhow!("write license tmp {}: {e}", tmp.display()))?;
    // chmod 600 on the tmp file before atomic rename
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&tmp, perms)
            .map_err(|e| anyhow::anyhow!("chmod 600 license file: {e}"))?;
    }
    std::fs::rename(&tmp, &path)
        .map_err(|e| anyhow::anyhow!("rename license tmp to {}: {e}", path.display()))?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    hex::encode(out)
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn iso860i_in_days(days: i64) -> String {
    let now = chrono::Utc::now();
    let later = now + chrono::Duration::days(days);
    later.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[derive(Debug, Deserialize)]
struct RegistryValidateResponse {
    #[serde(default)]
    valid: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    product: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    commercial_use: Option<bool>,
    #[serde(default)]
    customer_email: Option<String>,
    #[serde(default)]
    features: Option<Vec<String>>,
    #[serde(default)]
    expires_at: Option<String>,
}

fn validate_with_registry(
    registry: &str,
    key: &str,
) -> impl std::future::Future<Output = anyhow::Result<RegistryValidateResponse>> {
    async fn inner(registry: &str, key: &str) -> anyhow::Result<RegistryValidateResponse> {
        let url = format!(
            "{}/wp-json/wpuiai-ai-cloud/v1/license/validate",
            registry.trim_end_matches('/')
        );
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| anyhow::anyhow!("client build: {e}"))?;
        let body = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-License-Key", key)
            .json(&serde_json::json!({ "license_key": key }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("registry POST: {e}"))?;
        let status = body.status();
        if !status.is_success() {
            let text = body.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("registry HTTP {}: {}", status, text));
        }
        let parsed: RegistryValidateResponse = body
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("registry JSON: {e}"))?;
        Ok(parsed)
    }
    inner(registry, key)
}

async fn registry_ping_blocking(registry: &str) -> anyhow::Result<bool> {
    if registry.is_empty() {
        return Ok(false);
    }
    let url = format!(
        "{}/wp-json/wpuiai-ai-cloud/v1/license/status?license_key=focusa_live_probe",
        registry.trim_end_matches('/')
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| anyhow::anyhow!("client build: {e}"))?;
    let result = client.get(&url).send().await;
    match result {
        Ok(r) => {
            let s = r.status();
            // 200, 400, 404 all prove the registry is up
            Ok(s.is_success() || s.as_u16() == 400 || s.as_u16() == 404)
        }
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_mode_is_evaluation() {
        let local = LocalLicense::evaluation();
        assert_eq!(local.mode(), LicenseMode::Evaluation);
    }

    #[test]
    fn operator_tier_maps_to_operator_mode() {
        let mut local = LocalLicense::evaluation();
        local.tier = "operator".to_string();
        local.commercial_use = true;
        local.features = vec!["packaged_installer".to_string()];
        local.eval = false;
        assert_eq!(local.mode(), LicenseMode::Operator);
    }

    #[test]
    fn forge_tier_maps_to_forge_mode() {
        let mut local = LocalLicense::evaluation();
        local.tier = "founders-forge".to_string();
        local.commercial_use = true;
        local.features = vec!["packaged_installer".to_string()];
        local.eval = false;
        assert_eq!(local.mode(), LicenseMode::FoundersForge);
    }

    #[test]
    fn team_tier_maps_to_team_mode() {
        let mut local = LocalLicense::evaluation();
        local.tier = "team".to_string();
        local.commercial_use = true;
        local.features = vec!["packaged_installer".to_string()];
        local.eval = false;
        assert_eq!(local.mode(), LicenseMode::Team);
    }

    #[test]
    fn enterprise_tier_maps_to_enterprise_mode() {
        let mut local = LocalLicense::evaluation();
        local.tier = "enterprise".to_string();
        local.commercial_use = true;
        local.features = vec!["packaged_installer".to_string()];
        local.eval = false;
        assert_eq!(local.mode(), LicenseMode::Enterprise);
    }

    #[test]
    fn forge_tier_accepts_underscore_alias() {
        let mut local = LocalLicense::evaluation();
        local.tier = "founders_forge".to_string();
        local.commercial_use = true;
        local.features = vec!["packaged_installer".to_string()];
        local.eval = false;
        assert_eq!(local.mode(), LicenseMode::FoundersForge);
    }

    #[test]
    fn unknown_tier_fails_closed_to_recovery_only() {
        let mut local = LocalLicense::evaluation();
        local.tier = "future-tier-shape".to_string();
        local.commercial_use = true;
        local.features = vec!["packaged_installer".to_string()];
        local.eval = false;
        // Unknown legacy tier strings are migration-only and fail closed.
        assert_eq!(local.mode(), LicenseMode::RecoveryOnly);
    }

    #[test]
    fn key_hash_is_sha256_and_prefix_stored() {
        let key = "focusa_live_abc123";
        let hash = sha256_hex(key.as_bytes());
        assert_eq!(hash.len(), 64);
        let prefix: String = key.chars().take(16).collect();
        assert_eq!(prefix, "focusa_live_abc1");
    }

    #[test]
    fn license_base_product_gate_requires_one_signed_entitlement() {
        use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-core-001");
        snapshot.state = EntitlementState::Active;
        let guard = focusa_license::LicenseGuard::from_entitlement(snapshot);
        let projection =
            focusa_license::base_product_projection(guard.entitlement.as_ref()).expect("projection");
        assert_eq!(projection.product, "focusa");
        assert_eq!(projection.decision, "entitled");
        assert!(projection.permits_base_mutations);
        // Legacy core identifiers resolve as base-product claims, not separate purchases.
        assert_eq!(projection.compatibility.get("focusa.core.mission"), Some(&true));
        assert_eq!(projection.compatibility.get("focusa.core.workpoint"), Some(&true));
        assert_eq!(projection.compatibility.get("focusa.core.evidence"), Some(&true));
    }

    #[test]
    fn license_base_product_gate_fails_closed_without_signed_entitlement() {
        // Self-issued Evaluation carries no signed entitlement snapshot and must
        // never satisfy the base product gate.
        let guard = focusa_license::LicenseGuard::eval(7);
        assert!(guard.entitlement.is_none());
        assert!(focusa_license::base_product_projection(guard.entitlement.as_ref()).is_err());

        // Offline Grace remains a usable base product posture.
        use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-core-002");
        snapshot.state = EntitlementState::OfflineGrace;
        let guard = focusa_license::LicenseGuard::from_entitlement(snapshot);
        let projection =
            focusa_license::base_product_projection(guard.entitlement.as_ref()).expect("projection");
        assert!(projection.permits_base_mutations);
    }
}
