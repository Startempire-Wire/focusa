//! Spec 128 release/update primitives.
//!
//! This module is intentionally read-only and side-effect free.  It gives the
//! future `focusa update check/plan/apply` path a shared release manifest,
//! trust, provenance, and eligibility substrate before any auto-apply logic
//! exists.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const RELEASE_MANIFEST_SCHEMA_V1: &str = "focusa.release_manifest.v1";
pub const UPDATE_POLICY_SCHEMA_V1: &str = "focusa.update_policy.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Preview,
    Dev,
    Nightly,
}

impl ReleaseChannel {
    pub fn label(self) -> &'static str {
        match self {
            ReleaseChannel::Stable => "stable",
            ReleaseChannel::Preview => "preview",
            ReleaseChannel::Dev => "dev",
            ReleaseChannel::Nightly => "nightly",
        }
    }
}

impl std::str::FromStr for ReleaseChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "stable" => Ok(ReleaseChannel::Stable),
            "preview" => Ok(ReleaseChannel::Preview),
            "dev" => Ok(ReleaseChannel::Dev),
            "nightly" => Ok(ReleaseChannel::Nightly),
            other => Err(format!("unsupported release channel: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateMode {
    Notify,
    Prompt,
    Scheduled,
    Automatic,
    Manual,
}

impl UpdateMode {
    pub fn label(self) -> &'static str {
        match self {
            UpdateMode::Notify => "notify",
            UpdateMode::Prompt => "prompt",
            UpdateMode::Scheduled => "scheduled",
            UpdateMode::Automatic => "automatic",
            UpdateMode::Manual => "manual",
        }
    }
}

impl std::str::FromStr for UpdateMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "notify" => Ok(UpdateMode::Notify),
            "prompt" => Ok(UpdateMode::Prompt),
            "scheduled" => Ok(UpdateMode::Scheduled),
            "automatic" => Ok(UpdateMode::Automatic),
            "manual" => Ok(UpdateMode::Manual),
            other => Err(format!("unsupported update mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePolicyParts {
    pub cli: bool,
    pub daemon: bool,
    pub tui: bool,
    pub pi_extension: bool,
    pub menubar: bool,
    pub installer: bool,
}

impl UpdatePolicyParts {
    pub fn local_server_parts(enabled: bool) -> Self {
        Self {
            cli: enabled,
            daemon: enabled,
            tui: enabled,
            pi_extension: enabled,
            menubar: false,
            installer: false,
        }
    }

    pub fn all_surfaces(enabled: bool) -> Self {
        Self {
            cli: enabled,
            daemon: enabled,
            tui: enabled,
            pi_extension: enabled,
            menubar: enabled,
            installer: enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicy {
    pub schema: String,
    pub enabled: bool,
    pub channel: ReleaseChannel,
    pub mode: UpdateMode,
    pub license_level: String,
    #[serde(default)]
    pub dev_mode_override: bool,
    pub parts: UpdatePolicyParts,
    pub maintenance_window: String,
    pub require_ci_success: bool,
    pub require_release_success: bool,
    pub require_deploy_success_for_daemon_hosts: bool,
    pub require_checksums: bool,
    pub require_signatures: bool,
    pub rollback: bool,
    pub notify_before_restart: bool,
    pub auto_apply_allowed: bool,
    pub auto_apply_blocked_until: Vec<String>,
}

impl UpdatePolicy {
    pub fn default_for_license(
        license_level: impl Into<String>,
        features: &[String],
        dev_override: bool,
    ) -> Self {
        let license_level = license_level.into();
        let has = |feature: &str| features.iter().any(|f| f == feature);
        let is_dev_mode = dev_override
            || license_level == "dev_mode"
            || (has("developer_channel") && has("ota_auto_update"));
        let is_evaluation = license_level == "evaluation" || license_level == "eval";
        if is_dev_mode {
            Self {
                schema: UPDATE_POLICY_SCHEMA_V1.into(),
                enabled: true,
                channel: ReleaseChannel::Dev,
                mode: UpdateMode::Automatic,
                license_level: "dev_mode".into(),
                dev_mode_override: dev_override,
                parts: UpdatePolicyParts::all_surfaces(true),
                maintenance_window: "always".into(),
                require_ci_success: true,
                require_release_success: true,
                require_deploy_success_for_daemon_hosts: true,
                require_checksums: true,
                require_signatures: true,
                rollback: true,
                notify_before_restart: false,
                auto_apply_allowed: true,
                auto_apply_blocked_until: vec![],
            }
        } else if is_evaluation {
            Self {
                schema: UPDATE_POLICY_SCHEMA_V1.into(),
                enabled: true,
                channel: ReleaseChannel::Stable,
                mode: UpdateMode::Notify,
                license_level: "evaluation".into(),
                dev_mode_override: false,
                parts: UpdatePolicyParts::local_server_parts(false),
                maintenance_window: "manual".into(),
                require_ci_success: true,
                require_release_success: true,
                require_deploy_success_for_daemon_hosts: true,
                require_checksums: true,
                require_signatures: true,
                rollback: true,
                notify_before_restart: true,
                auto_apply_allowed: false,
                auto_apply_blocked_until: vec!["license_disallows_unattended_apply".into()],
            }
        } else {
            Self {
                schema: UPDATE_POLICY_SCHEMA_V1.into(),
                enabled: true,
                channel: ReleaseChannel::Stable,
                mode: UpdateMode::Prompt,
                license_level,
                dev_mode_override: false,
                parts: UpdatePolicyParts::local_server_parts(true),
                maintenance_window: "prompt".into(),
                require_ci_success: true,
                require_release_success: true,
                require_deploy_success_for_daemon_hosts: true,
                require_checksums: true,
                require_signatures: true,
                rollback: true,
                notify_before_restart: true,
                auto_apply_allowed: false,
                auto_apply_blocked_until: vec![
                    "explicit_policy_opt_in".into(),
                    "update_locking".into(),
                    "rollback_apply".into(),
                ],
            }
        }
    }

    /// Spec 152F.04.04: Split stable security maintenance from premium updates.
    ///
    /// Returns `None` when the policy uses the stable channel with a manual mode
    /// (notify, prompt, manual) — these are always-available stable security updates
    /// and repair paths. Returns `Some(feature)` when the policy requires a premium
    /// update feature:
    /// - `focusa.install.channel.preview` for the Preview channel
    /// - `focusa.install.channel.nightly` for the Nightly channel
    /// - `focusa.update.unattended` for Automatic or Scheduled modes
    ///
    /// Dev mode is a dev-override and does not require premium features.
    pub fn premium_update_required(&self) -> Option<&'static str> {
        let dev_mode = self.dev_mode_override || self.license_level == "dev_mode";
        if dev_mode {
            return None;
        }
        match self.channel {
            ReleaseChannel::Preview => return Some("focusa.install.channel.preview"),
            ReleaseChannel::Nightly => return Some("focusa.install.channel.nightly"),
            _ => {}
        }
        match self.mode {
            UpdateMode::Automatic | UpdateMode::Scheduled => {
                return Some("focusa.update.unattended");
            }
            _ => {}
        }
        None
    }

    /// Returns true when the policy is safe stable security maintenance —
    /// stable channel + manual mode — that remains available even when
    /// commercial entitlement is blocked, expired, refunded, or revoked.
    pub fn is_stable_security_maintenance(&self) -> bool {
        self.premium_update_required().is_none()
            && matches!(self.channel, ReleaseChannel::Stable)
            && matches!(
                self.mode,
                UpdateMode::Notify | UpdateMode::Prompt | UpdateMode::Manual
            )
    }

    pub fn refresh_auto_apply_authority(&mut self, features: &[String], dev_override: bool) {
        let has = |feature: &str| features.iter().any(|value| value == feature);
        let dev_mode = dev_override
            || self.dev_mode_override
            || self.license_level == "dev_mode"
            || (has("developer_channel") && has("ota_auto_update"));
        let unattended_entitled = dev_mode || has("ota_auto_update") || has("ota_scheduled");
        let automatic_mode = matches!(self.mode, UpdateMode::Automatic | UpdateMode::Scheduled);
        let any_part = self.parts.cli
            || self.parts.daemon
            || self.parts.tui
            || self.parts.pi_extension
            || self.parts.menubar
            || self.parts.installer;
        self.auto_apply_allowed = self.enabled && automatic_mode && unattended_entitled && any_part;
        self.auto_apply_blocked_until.clear();
        if !self.enabled {
            self.auto_apply_blocked_until.push("policy_disabled".into());
        }
        if !automatic_mode {
            self.auto_apply_blocked_until
                .push("policy_mode_not_automatic_or_scheduled".into());
        }
        if !unattended_entitled {
            self.auto_apply_blocked_until
                .push("license_disallows_unattended_apply".into());
        }
        if !any_part {
            self.auto_apply_blocked_until
                .push("no_update_parts_enabled".into());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub schema: String,
    pub tag: String,
    pub commit: String,
    pub channel: ReleaseChannel,
    #[serde(default)]
    pub publication_status: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub yanked: bool,
    #[serde(default)]
    pub revoked: bool,
    #[serde(default)]
    pub superseded_by: Option<String>,
    #[serde(default)]
    pub gates: ReleaseGates,
    #[serde(default)]
    pub compatibility_canary: Option<CompatibilityCanaryAuthorization>,
    pub trust: ReleaseTrust,
    #[serde(default)]
    pub provenance: Option<ReleaseProvenance>,
    #[serde(default)]
    pub compatibility: Option<ReleaseCompatibility>,
    pub assets: BTreeMap<String, ReleaseAsset>,
    #[serde(default)]
    pub requires_license_features: Vec<String>,
    #[serde(default)]
    pub dev_mode_features: Vec<String>,
    #[serde(default)]
    pub rollback_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityCanaryAuthorization {
    pub schema: String,
    pub status: String,
    pub environment: String,
    pub allowed_install_scope: String,
    pub required_previous_tag: String,
    #[serde(default)]
    pub required_sequence: Vec<String>,
    #[serde(default)]
    pub production_apply_authorized: bool,
    #[serde(default)]
    pub system_install_authorized: bool,
    #[serde(default)]
    pub service_mutation_authorized: bool,
    #[serde(default)]
    pub automatic_apply_authorized: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseGates {
    #[serde(default)]
    pub ci_success: Option<bool>,
    #[serde(default)]
    pub release_success: Option<bool>,
    #[serde(default)]
    pub deploy_success: Option<bool>,
    #[serde(default)]
    pub smoke_success: Option<bool>,
    #[serde(default)]
    pub installer_proof_success: Option<bool>,
    #[serde(default)]
    pub ci_run_url: Option<String>,
    #[serde(default)]
    pub release_run_url: Option<String>,
    #[serde(default)]
    pub deploy_run_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseTrust {
    /// Accepted initial values are `ed25519` and `cosign_keyless`.
    pub signing_algorithm: String,
    pub key_id: String,
    pub public_key_fingerprint: String,
    #[serde(default)]
    pub valid_from: Option<String>,
    #[serde(default)]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseProvenance {
    pub builder: String,
    pub workflow: String,
    pub run_url: String,
    pub artifact_digest: String,
    #[serde(default)]
    pub slsa_attestation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseCompatibility {
    #[serde(default)]
    pub min_installed_version: Option<String>,
    #[serde(default)]
    pub daemon_api_contract: Option<String>,
    #[serde(default)]
    pub pi_tool_contract: Option<String>,
    #[serde(default)]
    pub data_schema: Option<String>,
    #[serde(default)]
    pub requires_migration: bool,
    #[serde(default)]
    pub downgrade_supported: bool,
    #[serde(default)]
    pub requires_restart: Vec<String>,
    #[serde(default)]
    pub incompatible_if_features_missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub platform: String,
    pub name: String,
    pub sha256: String,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    pub signature: AssetSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSignature {
    pub algorithm: String,
    pub key_id: String,
    pub signature: String,
    #[serde(default)]
    pub certificate_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetDigestVerification {
    pub valid: bool,
    pub expected_sha256: String,
    pub actual_sha256: String,
    pub expected_size_bytes: Option<u64>,
    pub actual_size_bytes: u64,
    #[serde(default)]
    pub failures: Vec<String>,
}

/// Verify staged bytes against the signed manifest metadata before any install
/// or service action. Signature verification is a separate mandatory gate.
pub fn verify_release_asset_bytes(asset: &ReleaseAsset, bytes: &[u8]) -> AssetDigestVerification {
    let actual_sha256 = format!("{:x}", Sha256::digest(bytes));
    let expected_sha256 = asset.sha256.trim().to_ascii_lowercase();
    let actual_size_bytes = bytes.len() as u64;
    let mut failures = Vec::new();
    if expected_sha256.len() != 64 || !expected_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        failures.push("manifest_sha256_invalid".to_string());
    } else if expected_sha256 != actual_sha256 {
        failures.push("asset_sha256_mismatch".to_string());
    }
    if asset
        .size_bytes
        .is_some_and(|expected| expected != actual_size_bytes)
    {
        failures.push("asset_size_mismatch".to_string());
    }
    AssetDigestVerification {
        valid: failures.is_empty(),
        expected_sha256,
        actual_sha256,
        expected_size_bytes: asset.size_bytes,
        actual_size_bytes,
        failures,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSignatureVerification {
    pub valid: bool,
    pub key_id: String,
    pub algorithm: String,
    #[serde(default)]
    pub failures: Vec<String>,
}

/// Cryptographically verify staged bytes before install. Cosign-keyless uses a
/// separate identity/Rekor verifier and is rejected by this Ed25519 path.
pub fn verify_release_asset_signature(
    asset: &ReleaseAsset,
    bytes: &[u8],
    trusted_keys: &[TrustedReleaseKey],
) -> AssetSignatureVerification {
    let signature = &asset.signature;
    let mut failures = Vec::new();
    if !signature.algorithm.eq_ignore_ascii_case("ed25519") {
        failures.push("unsupported_signature_algorithm".to_string());
    }
    let Some(trusted) = trusted_keys
        .iter()
        .find(|key| key.key_id == signature.key_id)
    else {
        failures.push("trusted_key_missing".to_string());
        return AssetSignatureVerification {
            valid: false,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
            failures,
        };
    };
    if trusted.revoked_at.is_some() {
        failures.push("trusted_key_revoked".to_string());
    }
    if !trusted.signing_algorithm.eq_ignore_ascii_case("ed25519")
        || !trusted
            .signing_algorithm
            .eq_ignore_ascii_case(&signature.algorithm)
    {
        failures.push("trusted_key_algorithm_mismatch".to_string());
    }
    let public_key = trusted
        .public_key_base64
        .as_deref()
        .and_then(|encoded| BASE64.decode(encoded).ok())
        .and_then(|decoded| <[u8; 32]>::try_from(decoded).ok());
    let Some(public_key) = public_key else {
        failures.push("trusted_public_key_missing_or_invalid".to_string());
        return AssetSignatureVerification {
            valid: false,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
            failures,
        };
    };
    let fingerprint = format!("{:x}", Sha256::digest(public_key));
    if !trusted
        .public_key_fingerprint
        .eq_ignore_ascii_case(&fingerprint)
    {
        failures.push("trusted_key_fingerprint_mismatch".to_string());
    }
    let parsed_signature = BASE64
        .decode(signature.signature.trim())
        .ok()
        .and_then(|decoded| Signature::from_slice(&decoded).ok());
    let Some(parsed_signature) = parsed_signature else {
        failures.push("asset_signature_decode_failed".to_string());
        return AssetSignatureVerification {
            valid: false,
            key_id: signature.key_id.clone(),
            algorithm: signature.algorithm.clone(),
            failures,
        };
    };
    match VerifyingKey::from_bytes(&public_key) {
        Ok(key) if key.verify_strict(bytes, &parsed_signature).is_ok() => {}
        Ok(_) => failures.push("asset_signature_verification_failed".to_string()),
        Err(_) => failures.push("trusted_public_key_invalid".to_string()),
    }
    AssetSignatureVerification {
        valid: failures.is_empty(),
        key_id: signature.key_id.clone(),
        algorithm: signature.algorithm.clone(),
        failures,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedReleaseKey {
    pub key_id: String,
    pub public_key_fingerprint: String,
    pub signing_algorithm: String,
    /// Base64 raw Ed25519 public key; required for mutation/apply verification.
    #[serde(default)]
    pub public_key_base64: Option<String>,
    #[serde(default)]
    pub revoked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEligibilityOptions {
    pub channel: ReleaseChannel,
    pub platform: String,
    #[serde(default = "default_true")]
    pub require_ci_success: bool,
    #[serde(default = "default_true")]
    pub require_release_success: bool,
    #[serde(default)]
    pub require_deploy_success_for_daemon_hosts: bool,
    #[serde(default)]
    pub require_smoke_success: bool,
    #[serde(default)]
    pub require_installer_proof_success: bool,
    #[serde(default)]
    pub trusted_keys: Vec<TrustedReleaseKey>,
}

impl ReleaseEligibilityOptions {
    pub fn dev(platform: impl Into<String>, trusted_keys: Vec<TrustedReleaseKey>) -> Self {
        Self {
            channel: ReleaseChannel::Dev,
            platform: platform.into(),
            require_ci_success: true,
            require_release_success: true,
            require_deploy_success_for_daemon_hosts: false,
            require_smoke_success: false,
            require_installer_proof_success: false,
            trusted_keys,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEligibilityReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub eligible: bool,
    pub tag: String,
    pub channel: String,
    pub platform: String,
    pub matched_assets: Vec<String>,
    pub errors: Vec<ReleaseEligibilityFinding>,
    pub warnings: Vec<ReleaseEligibilityFinding>,
    pub auto_apply_allowed: bool,
    pub cryptographic_verification_required_for_apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseEligibilityFinding {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl ReleaseEligibilityFinding {
    fn error(code: impl Into<String>, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: Some(path.into()),
        }
    }

    fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self::error(code, message, path)
    }
}

pub fn parse_release_manifest_json(input: &str) -> Result<ReleaseManifest, serde_json::Error> {
    serde_json::from_str(input)
}

pub fn evaluate_release_manifest(
    manifest: &ReleaseManifest,
    options: &ReleaseEligibilityOptions,
) -> ReleaseEligibilityReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if manifest.schema != RELEASE_MANIFEST_SCHEMA_V1 {
        errors.push(ReleaseEligibilityFinding::error(
            "unsupported_manifest_schema",
            format!(
                "manifest schema must be {RELEASE_MANIFEST_SCHEMA_V1}, got {}",
                manifest.schema
            ),
            "/schema",
        ));
    }
    if !manifest.tag.starts_with('v') {
        errors.push(ReleaseEligibilityFinding::error(
            "invalid_tag",
            "release tag must start with v",
            "/tag",
        ));
    }
    if manifest.commit.len() < 7 {
        errors.push(ReleaseEligibilityFinding::error(
            "invalid_commit",
            "commit must contain at least a short git sha",
            "/commit",
        ));
    }
    if manifest.channel != options.channel {
        errors.push(ReleaseEligibilityFinding::error(
            "channel_mismatch",
            format!(
                "manifest channel {} does not match requested channel {}",
                manifest.channel.label(),
                options.channel.label()
            ),
            "/channel",
        ));
    }
    if manifest.yanked {
        errors.push(ReleaseEligibilityFinding::error(
            "release_yanked",
            "yanked releases are ineligible for update",
            "/yanked",
        ));
    }
    if manifest.revoked {
        errors.push(ReleaseEligibilityFinding::error(
            "release_revoked",
            "revoked releases are ineligible for update",
            "/revoked",
        ));
    }
    if manifest.assets.is_empty() {
        errors.push(ReleaseEligibilityFinding::error(
            "missing_assets",
            "release manifest must include at least one asset",
            "/assets",
        ));
    }

    require_gate(
        options.require_ci_success,
        manifest.gates.ci_success,
        "ci_success_required",
        "/gates/ci_success",
        &mut errors,
    );
    require_gate(
        options.require_release_success,
        manifest.gates.release_success,
        "release_success_required",
        "/gates/release_success",
        &mut errors,
    );
    require_gate(
        options.require_deploy_success_for_daemon_hosts,
        manifest.gates.deploy_success,
        "deploy_success_required",
        "/gates/deploy_success",
        &mut errors,
    );
    require_gate(
        options.require_smoke_success,
        manifest.gates.smoke_success,
        "smoke_success_required",
        "/gates/smoke_success",
        &mut errors,
    );
    require_gate(
        options.require_installer_proof_success,
        manifest.gates.installer_proof_success,
        "installer_proof_success_required",
        "/gates/installer_proof_success",
        &mut errors,
    );

    validate_trust(manifest, options, &mut errors, &mut warnings);

    let mut matched_assets = Vec::new();
    for (kind, asset) in &manifest.assets {
        let path = format!("/assets/{kind}");
        validate_asset(kind, asset, manifest, &mut errors);
        if asset.platform == options.platform || asset.platform == "all" {
            matched_assets.push(kind.clone());
        } else if asset.platform.trim().is_empty() {
            errors.push(ReleaseEligibilityFinding::error(
                "asset_platform_missing",
                format!("asset {kind} must declare platform"),
                format!("{path}/platform"),
            ));
        }
    }
    if matched_assets.is_empty() {
        errors.push(ReleaseEligibilityFinding::error(
            "unsupported_platform",
            format!("manifest has no asset for platform {}", options.platform),
            "/assets",
        ));
    }

    if manifest.provenance.is_none() {
        errors.push(ReleaseEligibilityFinding::error(
            "missing_provenance",
            "release manifest must include builder/workflow/run/digest provenance",
            "/provenance",
        ));
    }
    if !manifest.rollback_supported {
        warnings.push(ReleaseEligibilityFinding::warning(
            "rollback_not_declared",
            "release does not declare rollback support; apply must remain guarded",
            "/rollback_supported",
        ));
    }

    let eligible = errors.is_empty();
    ReleaseEligibilityReport {
        schema: "focusa.release_eligibility_report.v1",
        status: if eligible { "eligible" } else { "ineligible" },
        eligible,
        tag: manifest.tag.clone(),
        channel: manifest.channel.label().to_string(),
        platform: options.platform.clone(),
        matched_assets,
        errors,
        warnings,
        // Spec 128 forbids auto-apply until later policy/locking/rollback gates
        // exist. This primitive only answers release eligibility.
        auto_apply_allowed: false,
        cryptographic_verification_required_for_apply: true,
    }
}

fn require_gate(
    required: bool,
    actual: Option<bool>,
    code: &'static str,
    path: &'static str,
    errors: &mut Vec<ReleaseEligibilityFinding>,
) {
    if required && actual != Some(true) {
        errors.push(ReleaseEligibilityFinding::error(
            code,
            format!("required release gate {path} is not true"),
            path,
        ));
    }
}

fn validate_trust(
    manifest: &ReleaseManifest,
    options: &ReleaseEligibilityOptions,
    errors: &mut Vec<ReleaseEligibilityFinding>,
    warnings: &mut Vec<ReleaseEligibilityFinding>,
) {
    let algorithm = manifest.trust.signing_algorithm.as_str();
    if !matches!(algorithm, "ed25519" | "cosign_keyless") {
        errors.push(ReleaseEligibilityFinding::error(
            "unsupported_signing_algorithm",
            format!("unsupported signing algorithm {algorithm}"),
            "/trust/signing_algorithm",
        ));
    }
    if manifest.trust.revoked_at.is_some() {
        errors.push(ReleaseEligibilityFinding::error(
            "manifest_key_revoked",
            "manifest signing key is revoked",
            "/trust/revoked_at",
        ));
    }
    if options.trusted_keys.is_empty() {
        errors.push(ReleaseEligibilityFinding::error(
            "trust_root_missing",
            "release eligibility requires a configured trusted key",
            "/trust",
        ));
        return;
    }
    let matched = options.trusted_keys.iter().find(|key| {
        key.key_id == manifest.trust.key_id
            && key.public_key_fingerprint == manifest.trust.public_key_fingerprint
            && key.signing_algorithm == manifest.trust.signing_algorithm
    });
    match matched {
        Some(key) if key.revoked_at.is_some() => errors.push(ReleaseEligibilityFinding::error(
            "trusted_key_revoked",
            "configured trusted key is revoked",
            "/trust/key_id",
        )),
        Some(_) => {}
        None => errors.push(ReleaseEligibilityFinding::error(
            "untrusted_signing_key",
            "manifest signing key is not in the trusted key set",
            "/trust/key_id",
        )),
    }
    if manifest.trust.valid_until.is_none() {
        warnings.push(ReleaseEligibilityFinding::warning(
            "trust_key_without_expiry",
            "trusted key has no valid_until metadata; key rotation should be explicit",
            "/trust/valid_until",
        ));
    }
}

fn validate_asset(
    kind: &str,
    asset: &ReleaseAsset,
    manifest: &ReleaseManifest,
    errors: &mut Vec<ReleaseEligibilityFinding>,
) {
    let path = format!("/assets/{kind}");
    if asset.name.trim().is_empty() {
        errors.push(ReleaseEligibilityFinding::error(
            "asset_name_missing",
            format!("asset {kind} must have a name"),
            format!("{path}/name"),
        ));
    }
    if !is_sha256_hex(&asset.sha256) {
        errors.push(ReleaseEligibilityFinding::error(
            "asset_sha256_invalid",
            format!("asset {kind} sha256 must be 64 lowercase/uppercase hex chars"),
            format!("{path}/sha256"),
        ));
    }
    if asset.size_bytes == Some(0) {
        errors.push(ReleaseEligibilityFinding::error(
            "asset_size_zero",
            format!("asset {kind} size_bytes must be greater than zero"),
            format!("{path}/size_bytes"),
        ));
    }
    if asset.signature.signature.trim().is_empty() {
        errors.push(ReleaseEligibilityFinding::error(
            "asset_signature_missing",
            format!("asset {kind} must include a signature"),
            format!("{path}/signature/signature"),
        ));
    }
    if asset.signature.key_id != manifest.trust.key_id {
        errors.push(ReleaseEligibilityFinding::error(
            "asset_signature_key_mismatch",
            format!("asset {kind} signature key must match manifest trust key"),
            format!("{path}/signature/key_id"),
        ));
    }
    if asset.signature.algorithm != manifest.trust.signing_algorithm {
        errors.push(ReleaseEligibilityFinding::error(
            "asset_signature_algorithm_mismatch",
            format!("asset {kind} signature algorithm must match manifest trust algorithm"),
            format!("{path}/signature/algorithm"),
        ));
    }
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn trusted_key() -> TrustedReleaseKey {
        TrustedReleaseKey {
            key_id: "focusa-dev-2026".into(),
            public_key_fingerprint: "SHA256:focusadev".into(),
            signing_algorithm: "ed25519".into(),
            public_key_base64: None,
            revoked_at: None,
        }
    }

    fn asset(platform: &str) -> ReleaseAsset {
        ReleaseAsset {
            platform: platform.into(),
            name: format!("focusa-v0.9.80-dev-{platform}"),
            sha256: "a".repeat(64),
            size_bytes: Some(123),
            url: Some(
                "https://github.com/Startempire-Wire/focusa/releases/download/v0.9.80-dev/focusa"
                    .into(),
            ),
            signature: AssetSignature {
                algorithm: "ed25519".into(),
                key_id: "focusa-dev-2026".into(),
                signature: "base64-signature".into(),
                certificate_sha256: None,
            },
        }
    }

    fn manifest() -> ReleaseManifest {
        ReleaseManifest {
            schema: RELEASE_MANIFEST_SCHEMA_V1.into(),
            tag: "v0.9.80-dev".into(),
            commit: "8fa6452d".into(),
            channel: ReleaseChannel::Dev,
            publication_status: Some("published".into()),
            published_at: Some("2026-07-10T00:00:00Z".into()),
            yanked: false,
            revoked: false,
            superseded_by: None,
            gates: ReleaseGates {
                ci_success: Some(true),
                release_success: Some(true),
                deploy_success: Some(true),
                smoke_success: Some(true),
                installer_proof_success: Some(true),
                ci_run_url: Some(
                    "https://github.com/Startempire-Wire/focusa/actions/runs/1".into(),
                ),
                release_run_url: Some(
                    "https://github.com/Startempire-Wire/focusa/actions/runs/2".into(),
                ),
                deploy_run_url: Some(
                    "https://github.com/Startempire-Wire/focusa/actions/runs/3".into(),
                ),
            },
            compatibility_canary: None,
            trust: ReleaseTrust {
                signing_algorithm: "ed25519".into(),
                key_id: "focusa-dev-2026".into(),
                public_key_fingerprint: "SHA256:focusadev".into(),
                valid_from: Some("2026-01-01T00:00:00Z".into()),
                valid_until: Some("2027-01-01T00:00:00Z".into()),
                revoked_at: None,
            },
            provenance: Some(ReleaseProvenance {
                builder: "github-actions".into(),
                workflow: "release.yml".into(),
                run_url: "https://github.com/Startempire-Wire/focusa/actions/runs/2".into(),
                artifact_digest: "sha256:artifact".into(),
                slsa_attestation: None,
            }),
            compatibility: Some(ReleaseCompatibility {
                min_installed_version: Some("0.9.74-dev".into()),
                daemon_api_contract: Some("focusa.api.v1".into()),
                pi_tool_contract: Some("focusa.pi-tools.v1".into()),
                data_schema: Some("focusa.data.v1".into()),
                requires_migration: false,
                downgrade_supported: false,
                requires_restart: vec!["daemon".into()],
                incompatible_if_features_missing: vec!["packaged_installer".into()],
            }),
            assets: BTreeMap::from([("focusa".into(), asset("x86_64-unknown-linux-gnu"))]),
            requires_license_features: vec!["packaged_installer".into()],
            dev_mode_features: vec!["ota_auto_update".into(), "developer_channel".into()],
            rollback_supported: true,
        }
    }

    fn options() -> ReleaseEligibilityOptions {
        ReleaseEligibilityOptions::dev("x86_64-unknown-linux-gnu", vec![trusted_key()])
    }

    fn codes(report: &ReleaseEligibilityReport) -> Vec<&str> {
        report.errors.iter().map(|f| f.code.as_str()).collect()
    }

    #[test]
    fn dev_mode_policy_defaults_to_automatic_dev_for_all_surfaces() {
        let policy = UpdatePolicy::default_for_license(
            "dev_mode",
            &["developer_channel".into(), "ota_auto_update".into()],
            false,
        );
        assert!(policy.enabled);
        assert_eq!(policy.channel, ReleaseChannel::Dev);
        assert_eq!(policy.mode, UpdateMode::Automatic);
        assert!(policy.parts.cli);
        assert!(policy.parts.daemon);
        assert!(policy.parts.tui);
        assert!(policy.parts.pi_extension);
        assert!(policy.parts.menubar);
        assert!(policy.parts.installer);
        assert!(policy.auto_apply_allowed);
        assert!(policy.auto_apply_blocked_until.is_empty());
    }

    #[test]
    fn evaluation_policy_defaults_to_notify_only() {
        let policy = UpdatePolicy::default_for_license("evaluation", &[], false);
        assert!(policy.enabled);
        assert_eq!(policy.channel, ReleaseChannel::Stable);
        assert_eq!(policy.mode, UpdateMode::Notify);
        assert!(!policy.parts.cli);
        assert!(!policy.auto_apply_allowed);
        assert!(
            policy
                .auto_apply_blocked_until
                .contains(&"license_disallows_unattended_apply".to_string())
        );
    }

    #[test]
    fn paid_policy_defaults_to_prompt_not_automatic() {
        let policy = UpdatePolicy::default_for_license(
            "operator",
            &["packaged_installer".into(), "ota_apply_manual".into()],
            false,
        );
        assert!(policy.enabled);
        assert_eq!(policy.channel, ReleaseChannel::Stable);
        assert_eq!(policy.mode, UpdateMode::Prompt);
        assert!(policy.parts.cli);
        assert!(!policy.auto_apply_allowed);
    }

    #[test]
    fn valid_dev_manifest_is_eligible_but_not_auto_apply_allowed() {
        let report = evaluate_release_manifest(&manifest(), &options());
        assert!(report.eligible, "{report:?}");
        assert_eq!(report.status, "eligible");
        assert!(
            !report.auto_apply_allowed,
            "Spec128 gates auto-apply until policy/lock/rollback exist"
        );
        assert!(report.cryptographic_verification_required_for_apply);
        assert_eq!(report.matched_assets, vec!["focusa"]);
    }

    #[test]
    fn yanked_release_is_ineligible() {
        let mut m = manifest();
        m.yanked = true;
        let report = evaluate_release_manifest(&m, &options());
        assert!(!report.eligible);
        assert!(codes(&report).contains(&"release_yanked"));
    }

    #[test]
    fn missing_asset_signature_is_ineligible() {
        let mut m = manifest();
        m.assets
            .get_mut("focusa")
            .unwrap()
            .signature
            .signature
            .clear();
        let report = evaluate_release_manifest(&m, &options());
        assert!(!report.eligible);
        assert!(codes(&report).contains(&"asset_signature_missing"));
    }

    #[test]
    fn revoked_trusted_key_is_ineligible() {
        let mut key = trusted_key();
        key.revoked_at = Some("2026-07-10T00:00:00Z".into());
        let report = evaluate_release_manifest(
            &manifest(),
            &ReleaseEligibilityOptions::dev("x86_64-unknown-linux-gnu", vec![key]),
        );
        assert!(!report.eligible);
        assert!(codes(&report).contains(&"trusted_key_revoked"));
    }

    #[test]
    fn channel_mismatch_is_ineligible() {
        let mut opts = options();
        opts.channel = ReleaseChannel::Stable;
        let report = evaluate_release_manifest(&manifest(), &opts);
        assert!(!report.eligible);
        assert!(codes(&report).contains(&"channel_mismatch"));
    }

    #[test]
    fn unsupported_platform_is_ineligible() {
        let report = evaluate_release_manifest(
            &manifest(),
            &ReleaseEligibilityOptions::dev("aarch64-apple-darwin", vec![trusted_key()]),
        );
        assert!(!report.eligible);
        assert!(codes(&report).contains(&"unsupported_platform"));
    }

    #[test]
    fn missing_required_gate_is_ineligible() {
        let mut m = manifest();
        m.gates.ci_success = Some(false);
        let report = evaluate_release_manifest(&m, &options());
        assert!(!report.eligible);
        assert!(codes(&report).contains(&"ci_success_required"));
    }

    #[test]
    fn staged_asset_digest_and_size_must_match_manifest() {
        let bytes = b"verified focusa release asset";
        let mut release_asset = asset("x86_64-unknown-linux-gnu");
        release_asset.sha256 = format!("{:x}", Sha256::digest(bytes));
        release_asset.size_bytes = Some(bytes.len() as u64);
        let verification = verify_release_asset_bytes(&release_asset, bytes);
        assert!(verification.valid, "{verification:?}");
        assert!(verification.failures.is_empty());
    }

    #[test]
    fn staged_asset_digest_mismatch_blocks_install() {
        let mut release_asset = asset("x86_64-unknown-linux-gnu");
        release_asset.sha256 = "0".repeat(64);
        release_asset.size_bytes = Some(999);
        let verification = verify_release_asset_bytes(&release_asset, b"tampered");
        assert!(!verification.valid);
        assert!(
            verification
                .failures
                .contains(&"asset_sha256_mismatch".to_string())
        );
        assert!(
            verification
                .failures
                .contains(&"asset_size_mismatch".to_string())
        );
    }

    #[test]
    fn ed25519_signature_verifies_staged_asset_bytes() {
        let bytes = b"signed focusa release asset";
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let signature = signing_key.sign(bytes);
        let mut release_asset = asset("x86_64-unknown-linux-gnu");
        release_asset.signature.signature = BASE64.encode(signature.to_bytes());
        let trusted = TrustedReleaseKey {
            key_id: release_asset.signature.key_id.clone(),
            public_key_fingerprint: format!("{:x}", Sha256::digest(verifying_key.as_bytes())),
            signing_algorithm: "ed25519".into(),
            public_key_base64: Some(BASE64.encode(verifying_key.as_bytes())),
            revoked_at: None,
        };
        let verification = verify_release_asset_signature(&release_asset, bytes, &[trusted]);
        assert!(verification.valid, "{verification:?}");
    }

    #[test]
    fn tampered_or_revoked_ed25519_signature_blocks_install() {
        let bytes = b"signed focusa release asset";
        let signing_key = SigningKey::from_bytes(&[9_u8; 32]);
        let verifying_key = signing_key.verifying_key();
        let mut release_asset = asset("x86_64-unknown-linux-gnu");
        release_asset.signature.signature = BASE64.encode(signing_key.sign(bytes).to_bytes());
        let trusted = TrustedReleaseKey {
            key_id: release_asset.signature.key_id.clone(),
            public_key_fingerprint: format!("{:x}", Sha256::digest(verifying_key.as_bytes())),
            signing_algorithm: "ed25519".into(),
            public_key_base64: Some(BASE64.encode(verifying_key.as_bytes())),
            revoked_at: Some("2026-07-12T00:00:00Z".into()),
        };
        let verification = verify_release_asset_signature(&release_asset, b"tampered", &[trusted]);
        assert!(!verification.valid);
        assert!(
            verification
                .failures
                .contains(&"trusted_key_revoked".to_string())
        );
        assert!(
            verification
                .failures
                .contains(&"asset_signature_verification_failed".to_string())
        );
    }
}

#[cfg(test)]
mod spec152f_update_entitlement {
    use super::*;

    /// Helper: build a policy with the given channel and mode.
    fn policy(channel: ReleaseChannel, mode: UpdateMode) -> UpdatePolicy {
        let mut p = UpdatePolicy::default_for_license("operator", &[], false);
        p.channel = channel;
        p.mode = mode;
        p
    }

    fn dev_policy() -> UpdatePolicy {
        UpdatePolicy::default_for_license(
            "dev_mode",
            &["developer_channel".into(), "ota_auto_update".into()],
            false,
        )
    }

    // ── Stable security maintenance is always available ──────────────────

    #[test]
    fn stable_notify_is_maintenance_not_premium() {
        let p = policy(ReleaseChannel::Stable, UpdateMode::Notify);
        assert!(p.is_stable_security_maintenance());
        assert_eq!(p.premium_update_required(), None);
    }

    #[test]
    fn stable_prompt_is_maintenance_not_premium() {
        let p = policy(ReleaseChannel::Stable, UpdateMode::Prompt);
        assert!(p.is_stable_security_maintenance());
        assert_eq!(p.premium_update_required(), None);
    }

    #[test]
    fn stable_manual_is_maintenance_not_premium() {
        let p = policy(ReleaseChannel::Stable, UpdateMode::Manual);
        assert!(p.is_stable_security_maintenance());
        assert_eq!(p.premium_update_required(), None);
    }

    // ── Premium channels require premium features ────────────────────────

    #[test]
    fn preview_channel_requires_premium_feature() {
        let p = policy(ReleaseChannel::Preview, UpdateMode::Prompt);
        assert!(!p.is_stable_security_maintenance());
        assert_eq!(
            p.premium_update_required(),
            Some("focusa.install.channel.preview")
        );
    }

    #[test]
    fn nightly_channel_requires_premium_feature() {
        let p = policy(ReleaseChannel::Nightly, UpdateMode::Manual);
        assert!(!p.is_stable_security_maintenance());
        assert_eq!(
            p.premium_update_required(),
            Some("focusa.install.channel.nightly")
        );
    }

    // ── Unattended modes require premium features ────────────────────────

    #[test]
    fn automatic_mode_requires_premium_feature() {
        let p = policy(ReleaseChannel::Stable, UpdateMode::Automatic);
        assert!(!p.is_stable_security_maintenance());
        assert_eq!(
            p.premium_update_required(),
            Some("focusa.update.unattended")
        );
    }

    #[test]
    fn scheduled_mode_requires_premium_feature() {
        let p = policy(ReleaseChannel::Stable, UpdateMode::Scheduled);
        assert!(!p.is_stable_security_maintenance());
        assert_eq!(
            p.premium_update_required(),
            Some("focusa.update.unattended")
        );
    }

    // ── Dev mode overrides premium requirements ──────────────────────────

    #[test]
    fn dev_mode_does_not_require_premium_features() {
        let p = dev_policy();
        assert_eq!(p.premium_update_required(), None);
        assert!(!p.is_stable_security_maintenance());
    }

    #[test]
    fn dev_mode_override_relaxes_premium_requirements() {
        let mut p = policy(ReleaseChannel::Preview, UpdateMode::Automatic);
        p.dev_mode_override = true;
        assert_eq!(p.premium_update_required(), None);
    }

    // ── Default policies are stable maintenance ──────────────────────────

    #[test]
    fn default_paid_policy_is_stable_maintenance() {
        let p = UpdatePolicy::default_for_license("operator", &[], false);
        assert!(p.is_stable_security_maintenance());
        assert_eq!(p.premium_update_required(), None);
    }

    #[test]
    fn default_evaluation_policy_is_stable_maintenance() {
        let p = UpdatePolicy::default_for_license("evaluation", &[], false);
        assert!(p.is_stable_security_maintenance());
        assert_eq!(p.premium_update_required(), None);
    }

    // ── Premium update feature IDs are canonical ─────────────────────────

    #[test]
    fn premium_update_features_match_policy_classification() {
        let p = policy(ReleaseChannel::Preview, UpdateMode::Prompt);
        assert_eq!(
            p.premium_update_required(),
            Some("focusa.install.channel.preview")
        );
        let p = policy(ReleaseChannel::Nightly, UpdateMode::Prompt);
        assert_eq!(
            p.premium_update_required(),
            Some("focusa.install.channel.nightly")
        );
        let p = policy(ReleaseChannel::Stable, UpdateMode::Automatic);
        assert_eq!(
            p.premium_update_required(),
            Some("focusa.update.unattended")
        );
    }

    // ── Stable security maintenance survives blocked states ──────────────

    #[test]
    fn stable_maintenance_policy_never_requires_entitlement() {
        // Stable + notify/prompt/manual policies must remain usable even
        // when the commercial entitlement state is expired, refunded,
        // revoked, or missing. The premium_update_required() method
        // returns None, and the route maps to RecoveryAllowance::StableSecurityUpdate.
        let modes = [UpdateMode::Notify, UpdateMode::Prompt, UpdateMode::Manual];
        for mode in modes {
            let p = policy(ReleaseChannel::Stable, mode);
            assert!(
                p.is_stable_security_maintenance(),
                "stable/{:?} must be maintenance",
                mode
            );
            assert_eq!(
                p.premium_update_required(),
                None,
                "stable/{:?} must not require premium features",
                mode
            );
        }
    }

    #[test]
    fn premium_channel_or_mode_policies_require_entitlement() {
        // Preview/nightly channels and automatic/scheduled modes require
        // premium features. In blocked states these are denied.
        let premium_cases: Vec<(&str, UpdatePolicy)> = vec![
            (
                "preview",
                policy(ReleaseChannel::Preview, UpdateMode::Prompt),
            ),
            (
                "nightly",
                policy(ReleaseChannel::Nightly, UpdateMode::Manual),
            ),
            (
                "automatic",
                policy(ReleaseChannel::Stable, UpdateMode::Automatic),
            ),
            (
                "scheduled",
                policy(ReleaseChannel::Stable, UpdateMode::Scheduled),
            ),
        ];
        for (label, p) in premium_cases {
            assert!(
                p.premium_update_required().is_some(),
                "{label} must require premium features"
            );
            assert!(
                !p.is_stable_security_maintenance(),
                "{label} must not be classified as stable maintenance"
            );
        }
    }

    // ── Default policies produce correct channel and mode ────────────────

    #[test]
    fn default_policies_produce_expected_update_policy() {
        let dev = UpdatePolicy::default_for_license(
            "dev_mode",
            &["developer_channel".into(), "ota_auto_update".into()],
            false,
        );
        assert_eq!(dev.channel, ReleaseChannel::Dev);
        assert_eq!(dev.mode, UpdateMode::Automatic);
        assert!(dev.auto_apply_allowed);

        let eval = UpdatePolicy::default_for_license("evaluation", &[], false);
        assert_eq!(eval.channel, ReleaseChannel::Stable);
        assert_eq!(eval.mode, UpdateMode::Notify);
        assert!(!eval.auto_apply_allowed);

        let paid = UpdatePolicy::default_for_license("operator", &[], false);
        assert_eq!(paid.channel, ReleaseChannel::Stable);
        assert_eq!(paid.mode, UpdateMode::Prompt);
        assert!(!paid.auto_apply_allowed);
    }

    // ── Channel and mode parsing round-trips ─────────────────────────────

    #[test]
    fn channel_and_mode_round_trip() {
        for (label, channel) in [
            ("stable", ReleaseChannel::Stable),
            ("preview", ReleaseChannel::Preview),
            ("dev", ReleaseChannel::Dev),
            ("nightly", ReleaseChannel::Nightly),
        ] {
            let parsed: ReleaseChannel = label.parse().unwrap();
            assert_eq!(parsed, channel);
            assert_eq!(parsed.label(), label);
        }
        for (label, mode) in [
            ("notify", UpdateMode::Notify),
            ("prompt", UpdateMode::Prompt),
            ("scheduled", UpdateMode::Scheduled),
            ("automatic", UpdateMode::Automatic),
            ("manual", UpdateMode::Manual),
        ] {
            let parsed: UpdateMode = label.parse().unwrap();
            assert_eq!(parsed, mode);
            assert_eq!(parsed.label(), label);
        }
    }
}
