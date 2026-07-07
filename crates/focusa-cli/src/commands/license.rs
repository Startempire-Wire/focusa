//! Focusa license CLI commands — Spec92 §5.2.
//!
//! `focusa license activate <key>`
//! `focusa license status`
//! `focusa license deactivate`
//! `focusa license doctor`
//! `focusa license check-feature <feature>`
//!
//! All commands are agent-first (Spec92 §9): they return machine-readable JSON when
//! `--json` is passed and a human-readable table otherwise. They write the local license
//! file at `~/.config/focusa/license.json` with `chmod 600`, never persisting the raw key
//! unless the operator explicitly opts in via `--persist-key`.

use crate::api_client::ApiClient;
use clap::{Args, Subcommand};
use focusa_core::license::{
    LicenseStatus, activate as core_activate, check_feature as core_check_feature,
    deactivate as core_deactivate, doctor as core_doctor, load_license_status as core_status,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct LicenseArgs {
    #[command(subcommand)]
    pub command: LicenseCmd,
}

#[derive(Subcommand, Debug)]
pub enum LicenseCmd {
    /// Activate a Focusa license key. Saves the local license state file.
    Activate(ActivateArgs),
    /// Show current license status (mode, status, features, offline-valid-until).
    Status,
    /// Deactivate the current license. The local file is removed.
    Deactivate,
    /// Run a self-check of the local license file and remote registry reachability.
    Doctor,
    /// Check whether a specific feature is enabled by the current license.
    CheckFeature(CheckFeatureArgs),
}

#[derive(Args, Debug)]
pub struct ActivateArgs {
    /// The license key (focusa_live_xxxxx or uiai_live_xxxxx).
    #[arg(value_name = "KEY")]
    pub key: String,

    /// Persist the raw key in the local file (off-spec; default is prefix only).
    #[arg(long)]
    pub persist_key: bool,

    /// Override the registry URL (default: https://install.focusa.dev).
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,
}

#[derive(Args, Debug)]
pub struct CheckFeatureArgs {
    /// Feature key (e.g. packaged_installer, public_stream).
    #[arg(value_name = "FEATURE")]
    pub feature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RegistryValidateResponse {
    #[serde(default)]
    pub valid: bool,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub commercial_use: bool,
    #[serde(default)]
    pub team_use: bool,
    #[serde(default)]
    pub client_delivery: bool,
    #[serde(default)]
    hosted_use: bool,
    #[serde(default)]
    product_embedding: bool,
    #[serde(default)]
    redistribution: bool,
    #[serde(default)]
    allowed_products: Vec<String>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    expires_at: Option<String>,
}

const DEFAULT_REGISTRY: &str = "https://wpuiai.com";
const REGISTRY_VALIDATE_PATH: &str = "/wp-json/wpuiai-ai-cloud/v1/license/validate";
const LICENSE_FILE_NAME: &str = "license.json";

fn local_license_path() -> PathBuf {
    // Spec §5.1: ~/.config/focusa/license.json
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    home.join(".config").join("focusa").join(LICENSE_FILE_NAME)
}

/// Structured errors returned by the license registry per Spec 112 §15A.2.
/// Mirrors WordPress REST error envelopes emitted by install.focusa.dev.
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
    #[error("registry rate limited; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("registry unavailable: HTTP {status}")]
    Unavailable { status: u16, detail: String },
    #[error("registry response malformed: {0}")]
    MalformedResponse(String),
    #[error("transport error: {0}")]
    Transport(String),
}

impl RegistryError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound => "focusa_license_not_found",
            Self::Invalid => "focusa_license_invalid",
            Self::Revoked => "focusa_license_revoked",
            Self::Expired(_) => "focusa_license_expired",
            Self::Malformed(_) => "focusa_license_malformed",
            Self::RateLimited { .. } => "focusa_registry_rate_limited",
            Self::Unavailable { .. } => "focusa_registry_unavailable",
            Self::MalformedResponse(_) => "focusa_registry_response_malformed",
            Self::Transport(_) => "focusa_registry_transport_error",
        }
    }

    pub fn recovery_hint(&self) -> &'static str {
        match self {
            Self::NotFound | Self::Invalid => {
                "Purchase or check key at https://wpuiai.com/buy"
            }
            Self::Revoked => "Contact https://wpuiai.com/wp-admin for reissue",
            Self::Expired(_) => "Renew at https://install.focusa.dev/renew",
            Self::Malformed(_) => {
                "Verify the key was copied correctly (no spaces or line wraps)"
            }
            Self::RateLimited { .. } => {
                "Wait 60s and retry; --eval mode avoids registry calls"
            }
            Self::Unavailable { .. } => {
                "Check https://install.focusa.dev/status; retry in 5 min"
            }
            Self::MalformedResponse(_) => {
                "File a bug at https://install.focusa.dev/help — registry schema drift"
            }
            Self::Transport(_) => "Verify network connectivity to the registry host",
        }
    }
}

/// Outcome of `registry_validate`: either a parsed response or a structured error.
/// Spec 112 §15A.2 mandates we never conflate distinct failure modes.
pub(crate) struct RegistryValidateOutcome {
    pub response: Option<RegistryValidateResponse>,
    pub error: Option<RegistryError>,
}

/// POST a license key to the registry for validation. Maps every WP REST
/// envelope shape to a typed `RegistryError` so callers can emit structured
/// output with `code` + `recovery_hint` per Spec92.
pub(crate) async fn registry_validate(registry: &str, key: &str) -> RegistryValidateOutcome {
    let url = format!(
        "{}{}",
        registry.trim_end_matches('/'),
        REGISTRY_VALIDATE_PATH
    );
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return RegistryValidateOutcome {
                response: None,
                error: Some(RegistryError::Transport(format!(
                    "client build failed: {e}"
                ))),
            };
        }
    };
    let resp = match client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-License-Key", key)
        .json(&json!({ "license_key": key }))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return RegistryValidateOutcome {
                response: None,
                error: Some(RegistryError::Transport(format!("POST failed: {e}"))),
            };
        }
    };
    let status = resp.status();
    let body: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            return RegistryValidateOutcome {
                response: None,
                error: Some(RegistryError::MalformedResponse(format!("not JSON: {e}"))),
            };
        }
    };

    if !status.is_success() {
        // WordPress often returns HTTP 404 with a typed JSON envelope
        // (`{"valid":false,"error":"license_not_found",...}`). Parse that body
        // so callers see a structured `valid:false` rather than a transport error.
        if let Ok(parsed) = serde_json::from_value::<RegistryValidateResponse>(body.clone()) {
            return RegistryValidateOutcome {
                response: Some(parsed),
                error: None,
            };
        }
        return RegistryValidateOutcome {
            response: None,
            error: Some(map_wp_error_status(status.as_u16(), &body)),
        };
    }

    match serde_json::from_value::<RegistryValidateResponse>(body.clone()) {
        Ok(parsed) if parsed.valid || !body.is_null() => RegistryValidateOutcome {
            response: Some(parsed),
            error: None,
        },
        Ok(parsed) => RegistryValidateOutcome {
            response: Some(parsed),
            error: None,
        },
        Err(e) => RegistryValidateOutcome {
            response: None,
            error: Some(RegistryError::MalformedResponse(format!("schema mismatch: {e}"))),
        },
    }
}

/// Map a non-success HTTP status to a typed `RegistryError`, reading the
/// `code` / `message` / `errors` fields of the WP REST envelope.
fn map_wp_error_status(status: u16, body: &Value) -> RegistryError {
    let code = body
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("");
    let message = body
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_string);
    let fields = body.get("errors").cloned().unwrap_or(Value::Null);

    let by_code = match code {
        "focusa_license_not_found" => Some(RegistryError::NotFound),
        "focusa_license_invalid" => Some(RegistryError::Invalid),
        "focusa_license_revoked" => Some(RegistryError::Revoked),
        "focusa_license_expired" => Some(RegistryError::Expired(
            body.get("expires_at")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
        )),
        "focusa_license_malformed" | "focusa_license_payload_invalid" => {
            let msg = message.clone().unwrap_or_default();
            Some(RegistryError::Malformed(msg))
        }
        _ => None,
    };
    if let Some(e) = by_code {
        return e;
    }
    match status {
        404 => RegistryError::NotFound,
        401 => RegistryError::Invalid,
        403 => RegistryError::Revoked,
        410 => RegistryError::Expired(
            body.get("expires_at")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
        ),
        422 => RegistryError::Malformed(format!("{:?}", fields)),
        429 => {
            let retry_after_secs = body
                .get("retry_after")
                .and_then(Value::as_u64)
                .unwrap_or(60);
            RegistryError::RateLimited { retry_after_secs }
        }
        s if s >= 500 => {
            let detail = message.clone().unwrap_or_default();
            RegistryError::Unavailable { status: s, detail }
        }
        _ => RegistryError::Transport(format!(
            "unexpected status {status}: {}",
            message.unwrap_or_default()
        )),
    }
}

fn print_human_activate(status: &LicenseStatus, key_prefix: &str) {
    println!("Focusa license activated.\n");
    println!("Product:        {}", status.product);
    println!("Tier:           {}", status.tier);
    println!("Mode:           {:?}", status.mode);
    println!("Status:         {}", status.status);
    println!(
        "Commercial use: {}",
        if status.commercial_use {
            "permitted under license terms"
        } else {
            "not permitted"
        }
    );
    if let Some(ref exp) = status.expires_at {
        println!("Expires at:     {}", exp);
    }
    if let Some(ref off) = status.offline_valid_until {
        println!("Offline until:  {}", off);
    }
    println!("\nKey prefix:      {}", key_prefix);
    println!("\nEnabled features:");
    if status.features.is_empty() {
        println!("  (none)");
    } else {
        for f in &status.features {
            println!("  - {}", f);
        }
    }
}

fn print_human_status(status: &LicenseStatus, license_file: &Path) {
    println!("Focusa License Status\n");
    println!("License file:   {}", license_file.display());
    println!("Mode:           {:?}", status.mode);
    println!("Tier:           {}", status.tier);
    println!("Product:        {}", status.product);
    println!("Status:         {}", status.status);
    println!(
        "Commercial use: {}",
        if status.commercial_use {
            "permitted"
        } else {
            "not permitted"
        }
    );
    if let Some(ref off) = status.offline_valid_until {
        println!("Offline valid until: {}", off);
    }
    if let Some(ref exp) = status.expires_at {
        println!("Expires at: {}", exp);
    }
    println!("\nEnabled features:");
    if status.features.is_empty() {
        println!("  (none)");
    } else {
        for f in &status.features {
            println!("  - {}", f);
        }
    }
    let disabled = [
        "team_writer_arbitration",
        "hosted_service_use",
        "client_delivery_use",
        "commercial_export",
        "official_release_bundle",
    ];
    let enabled_set: std::collections::HashSet<&str> =
        status.features.iter().map(String::as_str).collect();
    let disabled_active: Vec<&&str> = disabled
        .iter()
        .filter(|f| !enabled_set.contains(**f))
        .collect();
    if !disabled_active.is_empty() {
        println!("\nDisabled features:");
        for f in &disabled_active {
            println!("  - {}", f);
        }
    }
}

fn print_human_doctor(doctor: &focusa_core::license::DoctorReport) {
    println!("Focusa License Doctor\n");
    println!("License file: {}", doctor.license_file);
    println!();
    let status = |ok: bool| if ok { "OK" } else { "FAIL" };
    println!("  [{}] license file exists", status(doctor.file_exists));
    println!("  [{}] license file readable", status(doctor.file_readable));
    println!("  [{}] license not expired", status(doctor.not_expired));
    println!(
        "  [{}] registry reachable",
        status(doctor.registry_reachable)
    );
    println!("  [{}] features loaded", status(doctor.features_loaded));
    if doctor.eval_mode {
        println!("\n  Note: running in Evaluation mode (no commercial license)");
    }
    if !doctor.warnings.is_empty() {
        println!("\nWarnings:");
        for w in &doctor.warnings {
            println!("  - {}", w);
        }
    }
    if !doctor.failures.is_empty() {
        println!("\nFailures:");
        for f in &doctor.failures {
            println!("  - {}", f);
        }
    }
}

fn print_license_gate_matrix(matrix: &[Value], missing_gates: &[Value], recovery_hint: &str) {
    println!("\nLicense gate matrix:");
    for row in matrix {
        println!(
            "  - {} -> {} ({})",
            row["command"].as_str().unwrap_or("unknown"),
            row["required_gate"].as_str().unwrap_or("unknown"),
            row["gate_status"].as_str().unwrap_or("unknown")
        );
    }
    if missing_gates.is_empty() {
        println!("\nMissing gates: none");
    } else {
        println!("\nMissing gates:");
        for row in missing_gates {
            println!("  - {}", row["command"].as_str().unwrap_or("unknown"));
        }
    }
    println!("Recovery hint: {recovery_hint}");
}

pub async fn run(json_output: bool, args: LicenseArgs) -> anyhow::Result<()> {
    match args.command {
        LicenseCmd::Activate(a) => run_activate(json_output, a).await,
        LicenseCmd::Status => run_status(json_output).await,
        LicenseCmd::Deactivate => run_deactivate(json_output).await,
        LicenseCmd::Doctor => run_doctor(json_output).await,
        LicenseCmd::CheckFeature(a) => run_check_feature(json_output, a).await,
    }
}

async fn run_activate(json_output: bool, args: ActivateArgs) -> anyhow::Result<()> {
    let registry = args
        .registry
        .clone()
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string());
    let key = args.key.trim().to_string();
    if key.is_empty() {
        anyhow::bail!("license key is empty");
    }
    let prefix: String = key.chars().take(16).collect();

    // Spec §5.2: POST key to license validation endpoint, then save local file.
    let outcome = registry_validate(&registry, &key).await;
    let resp = match outcome {
        RegistryValidateOutcome {
            response: Some(r),
            error: None,
        } => r,
        RegistryValidateOutcome {
            response: None,
            error: Some(err),
        } => {
            let code = err.code();
            let message = err.to_string();
            let recovery = err.recovery_hint();
            let out = json!({
                "ok": false,
                "code": code,
                "error": code,
                "message": message,
                "recovery_hint": recovery,
            });
            if json_output {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                eprintln!("{message}\nrecovery_hint: {recovery}");
            }
            std::process::exit(2);
        }
        _ => unreachable!("RegistryValidateOutcome must set exactly one branch"),
    };
    if !resp.valid {
        let purchase = format!("{}/buy", DEFAULT_REGISTRY.trim_end_matches('/'));
        let license_url = format!("{}/license", DEFAULT_REGISTRY.trim_end_matches('/'));
        let out = json!({
            "ok": false,
            "code": "focusa_license_not_valid",
            "error": "license_not_valid",
            "recovery_hint": format!("Purchase at {} or check {}.", purchase, license_url),
        });
        if json_output {
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            eprintln!(
                "License not valid. Purchase at {} or check {}.",
                purchase, license_url
            );
        }
        std::process::exit(2);
    }

    // Persist via focusa-core (chmod 600, default prefix-only, opt-in raw key).
    let status = core_activate(&key, &registry, args.persist_key).await?;
    let human_key_prefix = prefix;
    if json_output {
        let out = json!({
            "ok": true,
            "tier": status.tier,
            "mode": format!("{:?}", status.mode),
            "commercial_use": status.commercial_use,
            "features": status.features,
            "expires_at": status.expires_at,
            "offline_valid_until": status.offline_valid_until,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_human_activate(&status, &human_key_prefix);
    }
    Ok(())
}

async fn run_status(json_output: bool) -> anyhow::Result<()> {
    let license_file = local_license_path();
    let status = core_status()?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        print_human_status(&status, &license_file);
    }
    Ok(())
}

async fn run_deactivate(json_output: bool) -> anyhow::Result<()> {
    let license_file = local_license_path();
    core_deactivate(&license_file)?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "deactivated": true,
            }))?
        );
    } else {
        println!("Focusa license deactivated.");
        println!("License file removed: {}", license_file.display());
        println!("Next focusa command will run in Evaluation mode.");
    }
    Ok(())
}

async fn run_doctor(json_output: bool) -> anyhow::Result<()> {
    let license_file = local_license_path();
    let report = core_doctor(&license_file).await?;
    let license_gate_matrix = license_gate_matrix();
    let missing_gates = missing_license_gates(&license_gate_matrix);
    let recovery_hint = "Missing gates block MVP/commercial release: wire side-effect commands through focusa_core::license::require_feature or the install registry validation path, then rerun focusa license doctor.";
    if json_output {
        let out = json!({
            "license_doctor": report,
            "license_gate_matrix": license_gate_matrix,
            "missing_gates": missing_gates,
            "recovery_hint": recovery_hint,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_human_doctor(&report);
        print_license_gate_matrix(&license_gate_matrix, &missing_gates, recovery_hint);
    }
    Ok(())
}

fn license_gate_matrix() -> Vec<Value> {
    vec![
        json!({"command":"focusa install", "side_effect":"install_or_replace_binaries_and_service", "required_gate":"registry_validate_or_eval_mode", "gate_status":"gated", "evidence":"crates/focusa-cli/src/commands/install.rs:phase_license"}),
        json!({"command":"focusa upgrade", "side_effect":"atomic_binary_swap", "required_gate":"delegates_to_focusa_install_license_gate", "gate_status":"gated", "evidence":"crates/focusa-cli/src/commands/upgrade.rs"}),
        json!({"command":"focusa release prove", "side_effect":"official_release_bundle_proof", "required_gate":"official_release_bundle", "gate_status":"gated", "evidence":"crates/focusa-cli/src/commands/release.rs:require_feature"}),
        json!({"command":"focusa export", "side_effect":"commercial_export_artifact", "required_gate":"commercial_export", "gate_status":"gated", "evidence":"crates/focusa-cli/src/commands/export.rs:require_feature"}),
        json!({"command":"focusa binary", "side_effect":"packaged_installer_generation", "required_gate":"packaged_installer", "gate_status":"gated", "evidence":"crates/focusa-cli/src/commands/binary.rs:require_feature"}),
        json!({"command":"focusa device pair-qr", "side_effect":"qr_pwa_device_handoff", "required_gate":"qr_pwa_handoff", "gate_status":"gated", "evidence":"crates/focusa-cli/src/commands/device_pairing.rs:require_feature"}),
        json!({"command":"focusa license activate/deactivate", "side_effect":"local_license_state_admin", "required_gate":"not_required_license_administration", "gate_status":"not_required", "evidence":"crates/focusa-cli/src/commands/license.rs"}),
    ]
}

fn missing_license_gates(matrix: &[Value]) -> Vec<Value> {
    matrix
        .iter()
        .filter(|row| row.get("gate_status").and_then(Value::as_str) == Some("missing"))
        .cloned()
        .collect()
}

async fn run_check_feature(json_output: bool, args: CheckFeatureArgs) -> anyhow::Result<()> {
    let license_file = local_license_path();
    let feature = args.feature.as_str();
    // Spec §5.2: returns JSON with enabled + reason, or 402-equivalent error JSON
    let result = core_check_feature(&license_file, feature);
    match result {
        Ok(reason) => {
            let out = json!({
                "feature": feature,
                "enabled": true,
                "reason": reason,
            });
            if json_output {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("feature={} enabled=true reason={}", feature, reason);
            }
        }
        Err(err) => {
            let purchase = "https://focusa.dev";
            let docs_url = "https://wpuiai.com/wp-admin";
            let out = json!({
                "error": "license_required",
                "feature": feature,
                "message": err.to_string(),
                "purchase_url": purchase,
                "docs_url": docs_url,
            });
            if json_output {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                eprintln!("feature={} license_required", feature);
                eprintln!("reason: {}", err);
                eprintln!("purchase: {}", purchase);
            }
            std::process::exit(2);
        }
    }
    Ok(())
}

// Avoid unused import warnings when ApiClient is not used in this module directly.
#[allow(dead_code)]
fn _suppress_unused(_: &ApiClient) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_error_codes_are_stable() {
        // WP code values are part of the wire contract; lock them down.
        assert_eq!(RegistryError::NotFound.code(), "focusa_license_not_found");
        assert_eq!(RegistryError::Invalid.code(), "focusa_license_invalid");
        assert_eq!(RegistryError::Revoked.code(), "focusa_license_revoked");
        let _ = RegistryError::Expired("2026-01-01T00:00:00Z".to_string());
        assert_eq!(
            RegistryError::Malformed("bad".to_string()).code(),
            "focusa_license_malformed"
        );
        assert_eq!(
            RegistryError::RateLimited {
                retry_after_secs: 60
            }
            .code(),
            "focusa_registry_rate_limited"
        );
        assert_eq!(
            RegistryError::Unavailable {
                status: 503,
                detail: "down".to_string()
            }
            .code(),
            "focusa_registry_unavailable"
        );
    }

    #[test]
    fn registry_error_recovery_hints_are_actionable() {
        // Every variant must produce a non-empty hint that mentions a URL,
        // a retry, or a remediation — never blank.
        let variants: Vec<RegistryError> = vec![
            RegistryError::NotFound,
            RegistryError::Invalid,
            RegistryError::Revoked,
            RegistryError::Expired("2026-01-01T00:00:00Z".to_string()),
            RegistryError::Malformed("bad".to_string()),
            RegistryError::RateLimited {
                retry_after_secs: 60,
            },
            RegistryError::Unavailable {
                status: 503,
                detail: "down".to_string(),
            },
            RegistryError::MalformedResponse("not json".to_string()),
            RegistryError::Transport("connect refused".to_string()),
        ];
        for v in &variants {
            let hint = v.recovery_hint();
            assert!(!hint.is_empty(), "empty hint for {:?}", v.code());
            assert!(
                hint.contains("https://install.focusa.dev")
                    || hint.contains("https://wpuiai.com")
                    || hint.contains("60s")
                    || hint.contains("verify")
                    || hint.contains("Wait")
                    || hint.contains("Verify"),
                "hint {:?} not actionable: {hint}",
                v.code()
            );
        }
    }

    #[test]
    fn wp_envelope_status_to_error() {
        // 404 → NotFound
        let body = serde_json::json!({"code": "focusa_license_not_found", "message": "missing"});
        assert!(matches!(map_wp_error_status(404, &body), RegistryError::NotFound));

        // 410 with expires_at → Expired
        let body = serde_json::json!({
            "code": "focusa_license_expired",
            "expires_at": "2026-01-01T00:00:00Z"
        });
        match map_wp_error_status(410, &body) {
            RegistryError::Expired(d) => assert_eq!(d, "2026-01-01T00:00:00Z"),
            other => panic!("expected Expired, got {other:?}"),
        }

        // 429 with retry_after
        let body = serde_json::json!({"retry_after": 120});
        match map_wp_error_status(429, &body) {
            RegistryError::RateLimited { retry_after_secs } => {
                assert_eq!(retry_after_secs, 120)
            }
            other => panic!("expected RateLimited, got {other:?}"),
        }

        // 503 → Unavailable with detail
        let body = serde_json::json!({"message": "registry offline"});
        match map_wp_error_status(503, &body) {
            RegistryError::Unavailable { status, detail } => {
                assert_eq!(status, 503);
                assert_eq!(detail, "registry offline");
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }

        // 401 → Invalid (no code match, falls through to status)
        let body = serde_json::json!({"code": "", "message": "auth required"});
        assert!(matches!(map_wp_error_status(401, &body), RegistryError::Invalid));

        // 403 → Revoked (no code match, falls through to status)
        let body = serde_json::json!({"code": "", "message": "revoked"});
        assert!(matches!(map_wp_error_status(403, &body), RegistryError::Revoked));

        // 422 → Malformed
        let body = serde_json::json!({"errors": {"license_key": ["bad"]}});
        assert!(matches!(map_wp_error_status(422, &body), RegistryError::Malformed(_)));
    }
}
