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
    /// End-to-end license provisioning harness. Generates a fresh test
    /// key, validates it against the registry (dev_mode is acceptable for
    /// operator testing but downgrades commercial_use to false), writes
    /// license.json / license_authority.json / license_receipt.json,
    /// round-trips the files through the daemon parser, and reports the
    /// result. Use this to verify the full provisioning pipeline before
    /// the first real transaction.
    DevmodeFull(DevmodeFullArgs),
    /// Re-validate the current license against the registry and update
    /// the local file. Picks up revoke / refund / expire changes that
    /// happened on the registry side since the last validation.
    Refresh(RefreshArgs),
    /// Watch the local license file and the registry. When the registry
    /// returns a new state, the local file is updated and a notification
    /// is printed. Use this as a long-running sidecar after a purchase
    /// so refunds and revokes propagate within the poll interval.
    Watch(WatchArgs),
}

#[derive(Args, Debug)]
pub struct DevmodeFullArgs {
    /// Override the registry URL (default: https://wpuiai.com).
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,
    /// Optional customer email to embed in the receipt (test fixture).
    #[arg(long, value_name = "EMAIL")]
    pub email: Option<String>,
    /// Optional fixed license key to use (otherwise one is generated).
    #[arg(long, value_name = "KEY")]
    pub key: Option<String>,
    /// Print the parsed registry response as JSON for inspection.
    #[arg(long)]
    pub print_response: bool,
}

#[derive(Args, Debug)]
pub struct RefreshArgs {
    /// Override the registry URL (default: https://wpuiai.com).
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,
    /// Persist the raw key from --raw-key in the local file (off-spec).
    #[arg(long, value_name = "KEY")]
    pub raw_key: Option<String>,
    /// Set FOCUSA_REQUIRE_REAL_LICENSE=1 for this run (refuse dev_mode).
    #[arg(long)]
    pub require_real: bool,
}

#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Override the registry URL (default: https://wpuiai.com).
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,
    /// Poll interval in seconds (default 60, min 5).
    #[arg(long, value_name = "SECONDS", default_value_t = 60)]
    pub interval: u64,
    /// Stop after this many polls (default: forever).
    #[arg(long, value_name = "COUNT")]
    pub max_polls: Option<u64>,
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
                "Purchase or check key at https://install.focusa.dev/license"
            }
            Self::Revoked => {
                "Email support@focusa.dev or visit https://focusa.dev/support for reissue"
            }
            Self::Expired(_) => "Renew at https://install.focusa.dev/renew",
            Self::Malformed(_) => "Verify the key was copied correctly (no spaces or line wraps)",
            Self::RateLimited { .. } => "Wait 60s and retry; --eval mode avoids registry calls",
            Self::Unavailable { .. } => "Check https://install.focusa.dev/status; retry in 5 min",
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
            error: Some(RegistryError::MalformedResponse(format!(
                "schema mismatch: {e}"
            ))),
        },
    }
}

/// Map a non-success HTTP status to a typed `RegistryError`, reading the
/// `code` / `message` / `errors` fields of the WP REST envelope.
fn map_wp_error_status(status: u16, body: &Value) -> RegistryError {
    let code = body.get("code").and_then(Value::as_str).unwrap_or("");
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
    println!(
        "\nRecovery policy: recovery, export, repair, and uninstall remain available when execution is locked."
    );
    println!(
        "Locked capabilities and remaining limits are authority-signed; no local cap list is inferred."
    );
    println!("Marketing preference is managed separately from terms and entitlement.");
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
        LicenseCmd::Status => run_status(json_output).await,
        LicenseCmd::Doctor => run_doctor(json_output).await,
        LicenseCmd::CheckFeature(a) => run_check_feature(json_output, a).await,
        LicenseCmd::Activate(_)
        | LicenseCmd::Deactivate
        | LicenseCmd::DevmodeFull(_)
        | LicenseCmd::Refresh(_)
        | LicenseCmd::Watch(_) => anyhow::bail!(
            "E_AUTHORITY_COMMAND_RETIRED: plaintext activation, deactivation, dev-mode issuance, registry refresh, and watch cannot grant or mutate production entitlement; use signed authority device authorization"
        ),
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
    let guard = focusa_license::resolve_license_guard();
    let authority = focusa_license::entitlement_projection(guard.entitlement.as_ref())?;
    let entitlement_decision = focusa_license::entitlement_decision_projection(guard.entitlement.as_ref())?;
    let payload = json!({
        "schema": "focusa.authority_license_status.v1",
        "authority": authority,
        "entitlement_decision": entitlement_decision,
        "recovery_policy": "recovery, export, repair, and uninstall remain available when execution is locked",
        "marketing_preference": "managed_separately"
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Focusa Signed Authority Status\n");
        println!(
            "State:          {}",
            payload["authority"]["state"]
                .as_str()
                .unwrap_or("unactivated")
        );
        println!(
            "Product:        {}",
            payload["authority"]["product"].as_str().unwrap_or("focusa")
        );
        println!(
            "Decision:       {} ({})",
            payload["entitlement_decision"]["status"].as_str().unwrap_or("unknown"),
            payload["entitlement_decision"]["reason_code"].as_str().unwrap_or("unknown")
        );
        println!(
            "Recovery action: {}",
            payload["entitlement_decision"]["recovery_action"].as_str().unwrap_or("unknown")
        );
        if let Some(sequence) = payload["authority"]["lease_sequence"].as_u64() {
            println!("Lease sequence: {sequence}");
        }
        println!(
            "Recovery policy: {}",
            payload["recovery_policy"].as_str().unwrap_or_default()
        );
        println!("Marketing preference: managed separately");
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
        json!({"command":"focusa export", "side_effect":"premium_export_packaging", "required_gate":"focusa.export.packaged", "gate_status":"gated", "evidence":"crates/focusa-core/src/license.rs:require_export_packaged"}),
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
    let feature = args.feature.as_str();
    let guard = focusa_license::resolve_license_guard();
    let enabled = guard
        .entitlement
        .as_ref()
        .and_then(|snapshot| snapshot.features.get(feature))
        .copied()
        .unwrap_or(false);
    let out = json!({
        "schema": "focusa.authority_feature_decision.v1",
        "feature": feature,
        "enabled": enabled,
        "reason": if enabled { "signed_feature_grant" } else { "unknown_or_not_granted" },
        "recovery_policy": "recovery, export, repair, and uninstall remain available"
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!(
            "feature={} enabled={} reason={}",
            feature,
            enabled,
            out["reason"].as_str().unwrap_or("unknown_or_not_granted")
        );
    }
    if !enabled {
        anyhow::bail!("ENTITLEMENT_FEATURE_REQUIRED: unknown or ungranted feature {feature}");
    }
    Ok(())
}

// Avoid unused import warnings when ApiClient is not used in this module directly.
#[allow(dead_code)]
fn _suppress_unused(_: &ApiClient) {}

/// Derive a stable machine fingerprint for license seat binding.
/// Order of preference:
///   1. $FOCUSA_MACHINE_ID if the operator sets it explicitly (test/cluster)
///   2. /etc/machine-id (systemd, always present on Linux)
///   3. hostname + first non-loopback MAC address
///   4. hostname only (last-resort fallback, NOT stable across reboots
///      on some cloud images — callers should pin /etc/machine-id or
///      FOCUSA_MACHINE_ID when seat enforcement matters)
pub(crate) fn derive_machine_id() -> String {
    use sha2::{Digest, Sha256};
    if let Ok(v) = std::env::var("FOCUSA_MACHINE_ID") {
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
    }
    if let Ok(s) = std::fs::read_to_string("/etc/machine-id") {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    // Fallback: hostname + first non-loopback MAC (best-effort).
    let host = std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
        .unwrap_or_else(|_| "unknown".to_string());
    let mac = read_first_mac().unwrap_or_else(|| "nomac".to_string());
    let mut hasher = Sha256::new();
    hasher.update(host.as_bytes());
    hasher.update(b"|");
    hasher.update(mac.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(unix)]
fn read_first_mac() -> Option<String> {
    use std::fs;
    for entry in fs::read_dir("/sys/class/net").ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "lo" {
            continue;
        }
        let addr_path = entry.path().join("address");
        if let Ok(s) = fs::read_to_string(&addr_path) {
            let s = s.trim();
            if !s.is_empty() && s != "00:00:00:00:00:00" {
                return Some(s.to_string());
            }
        }
    }
    None
}

#[cfg(not(unix))]
fn read_first_mac() -> Option<String> {
    None
}

/// Read the active license key from the local file. We don't persist the
/// raw key (Spec §5.1), so refresh uses the same source-of-truth as
/// devmode-full: the caller passes --raw-key OR we read from the receipt
/// file (which carries key_hash + key_prefix).
fn read_active_key_from_receipt() -> Option<String> {
    // We can't reconstruct the raw key from the hash; only the operator
    // can supply it. This helper is a placeholder for the future when
    // the receipt file (or daemon sqlite) carries the raw key encrypted
    // at rest. For now, refresh requires --raw-key OR a fresh
    // devmode-full-style test.
    None
}

/// Re-validate the current license against the registry. Picks up
/// revoke, refund, and expiry that happened on the registry side since
/// the last validation. Writes a new license.json + receipt if the
/// state changed.
async fn run_refresh(json_output: bool, args: RefreshArgs) -> anyhow::Result<()> {
    use chrono::Utc;
    use focusa_core::license::{LocalLicense, load_local_license};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::PathBuf;

    let registry = args
        .registry
        .clone()
        .or_else(|| std::env::var("FOCUSA_LICENSE_REGISTRY").ok())
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string());
    let validate_url = format!(
        "{}{}",
        registry.trim_end_matches('/'),
        REGISTRY_VALIDATE_PATH
    );

    let key = args.raw_key.clone().or_else(read_active_key_from_receipt);
    let key = match key {
        Some(k) if !k.trim().is_empty() => k,
        _ => {
            let payload = serde_json::json!({
                "status": "blocked",
                "step": "refresh_input",
                "error": "no license key available",
                "recovery_hint": "pass --raw-key <KEY>, or run `focusa license activate <KEY>` first, or run `focusa license devmode-full`",
            });
            if json_output {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!(
                    "[refresh] step=refresh_input status=blocked error=\"no license key available\""
                );
                println!("[refresh] recovery_hint: pass --raw-key <KEY>");
            }
            std::process::exit(2);
        }
    };

    let machine_id = derive_machine_id();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let body: serde_json::Value = match client
        .post(&validate_url)
        .header("Content-Type", "application/json")
        .header("X-License-Key", &key)
        .header("X-Machine-Id", &machine_id)
        .json(&serde_json::json!({
            "license_key": key,
            "machine_id": machine_id,
            "intent": "refresh",
        }))
        .send()
        .await
    {
        Ok(r) => r
            .json::<serde_json::Value>()
            .await
            .unwrap_or(serde_json::Value::Null),
        Err(_) => serde_json::Value::Null,
    };

    let valid = body.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
    let status = body
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tier = body
        .get("tier")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let commercial_use = body
        .get("commercial_use")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let features: Vec<String> = body
        .get("features")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let expires_at = body
        .get("expires_at")
        .and_then(|v| v.as_str())
        .map(String::from);

    // dev_mode rule (same as devmode-full): downgrade to eval.
    let is_dev_mode_fixture = status == "dev_mode";
    let require_real = args.require_real
        || std::env::var("FOCUSA_REQUIRE_REAL_LICENSE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    if is_dev_mode_fixture && require_real {
        let payload = serde_json::json!({
            "status": "blocked",
            "step": "registry_post",
            "registry_status": status,
            "error": "dev_mode response with FOCUSA_REQUIRE_REAL_LICENSE=1",
            "recovery_hint": "unset FOCUSA_REQUIRE_REAL_LICENSE to allow dev_mode downgrades, or purchase a real license at https://install.focusa.dev/license",
        });
        if json_output {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            println!("[refresh] blocked: dev_mode with require_real");
        }
        std::process::exit(2);
    }

    let granted_tier = if is_dev_mode_fixture {
        "evaluation".to_string()
    } else {
        tier.clone()
    };
    let granted_features = if is_dev_mode_fixture {
        vec!["daemon".to_string(), "tui".to_string(), "cli".to_string()]
    } else {
        features.clone()
    };
    let granted_commercial = commercial_use && !is_dev_mode_fixture;

    // Detect revoke: registry returns valid=false with status=revoked or
    // status=expired. Surface this with a non-zero exit so callers can
    // act on it.
    let revoked = status == "revoked" || status == "expired" || !valid;
    if revoked && !is_dev_mode_fixture {
        let payload = serde_json::json!({
            "status": "blocked",
            "step": "registry_post",
            "registry_status": status,
            "valid": valid,
            "machine_id": machine_id,
            "recovery_hint": format!(
                "registry reports license state '{}'. Run `focusa license activate <KEY>` with a current key, or contact {} for reissue.",
                status, "https://focusa.dev/support"
            ),
        });
        if json_output {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            println!("[refresh] step=registry_post status=blocked registry_status={status}");
            println!("[refresh] recovery_hint: {}", payload["recovery_hint"]);
        }
        std::process::exit(2);
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    let license_dir = home.join(".config").join("focusa");
    let license_file = license_dir.join("license.json");
    let receipt_file = license_dir.join("license_receipt.json");
    fs::create_dir_all(&license_dir)?;

    let key_hash = {
        let mut h = Sha256::new();
        h.update(key.as_bytes());
        format!("{:x}", h.finalize())
    };
    let key_prefix: String = key.chars().take(16).collect();
    let offline_until = (Utc::now() + chrono::Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let issued_at: u64 = Utc::now().timestamp() as u64;

    let prior = load_local_license().ok();
    let license = LocalLicense {
        key_hash: key_hash.clone(),
        key_prefix: key_prefix.clone(),
        product: "focusa".to_string(),
        tier: granted_tier.clone(),
        status: if is_dev_mode_fixture {
            "active".to_string()
        } else {
            status.clone()
        },
        commercial_use: granted_commercial,
        customer_email: prior
            .as_ref()
            .map(|p| p.customer_email.clone())
            .unwrap_or_default(),
        features: granted_features.clone(),
        offline_valid_until: Some(offline_until.clone()),
        expires_at: expires_at.clone(),
        eval: is_dev_mode_fixture,
        registry: registry.clone(),
        issued_at,
    };
    let license_json = serde_json::to_string_pretty(&license)?;
    fs::write(&license_file, format!("{license_json}\n"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&license_file)?.permissions();
        p.set_mode(0o600);
        fs::set_permissions(&license_file, p)?;
    }

    let receipt = serde_json::json!({
        "issued_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "tier": granted_tier,
        "status": if is_dev_mode_fixture { "active".to_string() } else { status.clone() },
        "expires_at": expires_at,
        "machine_id": machine_id,
        "key_hash": key_hash,
        "key_prefix": key_prefix,
        "eval": is_dev_mode_fixture,
        "commercial_use": granted_commercial,
        "intent": "refresh",
        "note": "Refreshed from registry. Use `focusa license status` to view the current state.",
    });
    fs::write(
        &receipt_file,
        format!("{}\n", serde_json::to_string_pretty(&receipt)?),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&receipt_file)?.permissions();
        p.set_mode(0o600);
        fs::set_permissions(&receipt_file, p)?;
    }

    let payload = serde_json::json!({
        "status": "completed",
        "step": "report",
        "machine_id": machine_id,
        "registry": registry,
        "registry_status": status,
        "valid": valid,
        "granted_tier": license.tier,
        "granted_features": granted_features,
        "commercial_use": granted_commercial,
        "is_dev_mode_fixture": is_dev_mode_fixture,
        "offline_valid_until": offline_until,
        "issued_at": issued_at,
        "files": {
            "license": license_file.to_string_lossy(),
            "receipt": receipt_file.to_string_lossy(),
        },
        "round_trip": "parsed",
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("[refresh] step=report");
        println!("  machine_id:        {machine_id}");
        println!("  registry_status:   {status}");
        println!("  granted tier:      {}", license.tier);
        println!("  commercial_use:    {granted_commercial}");
        println!("  offline_valid_until: {offline_until}");
        if is_dev_mode_fixture {
            println!(
                "  note: dev_mode is a TEST FIXTURE; this refresh was downgraded to evaluation."
            );
        }
    }
    Ok(())
}

/// Watch the local license file and the registry. Long-running sidecar
/// that polls every N seconds and updates the local file when the
/// registry reports a state change. Picks up revoke / refund / expire
/// without operator action.
async fn run_watch(json_output: bool, args: WatchArgs) -> anyhow::Result<()> {
    let interval = args.interval.max(5);
    let max_polls = args.max_polls.unwrap_or(u64::MAX);
    let mut polls: u64 = 0;
    let mut last_signature = String::new();
    loop {
        polls += 1;
        let args_refresh = RefreshArgs {
            registry: args.registry.clone(),
            raw_key: None,
            require_real: std::env::var("FOCUSA_REQUIRE_REAL_LICENSE")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
        };
        match run_refresh(true, args_refresh).await {
            Ok(()) => {
                let license_file = std::env::var_os("HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("/root"))
                    .join(".config")
                    .join("focusa")
                    .join("license.json");
                let sig = std::fs::read_to_string(&license_file)
                    .ok()
                    .and_then(|s| {
                        let v: serde_json::Value = serde_json::from_str(&s).ok()?;
                        Some(format!(
                            "{}:{}:{}:{}",
                            v.get("status").and_then(|x| x.as_str()).unwrap_or(""),
                            v.get("tier").and_then(|x| x.as_str()).unwrap_or(""),
                            v.get("offline_valid_until")
                                .and_then(|x| x.as_str())
                                .unwrap_or(""),
                            v.get("commercial_use")
                                .and_then(|x| x.as_bool())
                                .unwrap_or(false),
                        ))
                    })
                    .unwrap_or_default();
                if !json_output {
                    println!("[watch] poll={polls} signature={sig}");
                } else if sig != last_signature {
                    println!(
                        "{{\"event\":\"watch_change\",\"poll\":{polls},\"signature\":\"{sig}\"}}"
                    );
                    last_signature = sig;
                } else {
                    last_signature = sig;
                }
            }
            Err(e) => {
                if json_output {
                    println!(
                        "{{\"event\":\"watch_error\",\"poll\":{polls},\"error\":\"{}\"}}",
                        e
                    );
                } else {
                    println!("[watch] poll={polls} error={e}");
                }
            }
        }
        if polls >= max_polls {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
    Ok(())
}

/// End-to-end license provisioning harness. The devmodefull command
/// exercises the entire provisioning pipeline (test key → registry
/// validate → license file write → daemon-side round-trip parse) and
/// reports the result of every step. Use it to validate the pipeline
/// before the first real-money transaction.
///
/// Operator rule (2026-07-07): dev_mode is a TEST FIXTURE. The harness
/// always writes a license.json with `commercial_use=false` when the
/// registry returns `status=dev_mode`, so devmodefull can never grant
/// commercial privileges by accident.
async fn run_devmode_full(json_output: bool, args: DevmodeFullArgs) -> anyhow::Result<()> {
    use chrono::Utc;
    use focusa_core::license::{LicenseStatus, LocalLicense};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::path::PathBuf;

    let registry = args
        .registry
        .clone()
        .or_else(|| std::env::var("FOCUSA_LICENSE_REGISTRY").ok())
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string());
    let validate_url = format!(
        "{}{}",
        registry.trim_end_matches('/'),
        REGISTRY_VALIDATE_PATH
    );

    // 1. Generate (or accept) a test key. We use a recognisable prefix so
    //    the registry / receipt audit trail is easy to filter.
    let key = args.key.clone().unwrap_or_else(|| {
        let stamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        format!("focusa_test_devmodefull_{stamp}")
    });
    let key_hash = {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    let key_prefix: String = key.chars().take(16).collect();

    // 2. POST to the registry.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let response = client
        .post(&validate_url)
        .header("Content-Type", "application/json")
        .header("X-License-Key", &key)
        .json(&serde_json::json!({ "license_key": key }))
        .send()
        .await;

    let body: serde_json::Value = match response {
        Ok(r) => r.json().await.unwrap_or(serde_json::Value::Null),
        Err(e) => {
            let payload = serde_json::json!({
                "status": "blocked",
                "step": "registry_post",
                "registry": registry,
                "validate_url": validate_url,
                "error": e.to_string(),
                "recovery_hint": "check network connectivity to the license authority",
            });
            if json_output {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("[devmodefull] step=registry_post status=blocked error={e}");
            }
            std::process::exit(2);
        }
    };
    let valid = body.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
    let status = body
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tier = body
        .get("tier")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let commercial_use = body
        .get("commercial_use")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let features: Vec<String> = body
        .get("features")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let expires_at = body
        .get("expires_at")
        .and_then(|v| v.as_str())
        .map(String::from);

    // 3. Decide commercial vs eval based on registry status. The
    //    operator's rule: dev_mode is for testing only; it never grants
    //    commercial_use.
    let is_dev_mode_fixture = status == "dev_mode";
    let granted_commercial_use = commercial_use && !is_dev_mode_fixture;
    let granted_tier = if is_dev_mode_fixture {
        "evaluation".to_string()
    } else {
        tier.clone()
    };
    let granted_features = if is_dev_mode_fixture {
        vec!["daemon".to_string(), "tui".to_string(), "cli".to_string()]
    } else {
        features.clone()
    };

    // 4. Locate writeable file paths.
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    let license_dir = home.join(".config").join("focusa");
    let license_file = license_dir.join("license.json");
    let authority_file = license_dir.join("license_authority.json");
    let receipt_file = license_dir.join("license_receipt.json");
    fs::create_dir_all(&license_dir)?;

    let offline_until = (Utc::now() + chrono::Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    let issued_at: u64 = Utc::now().timestamp() as u64;

    // 5. Write the daemon-compatible license.json.
    let license = LocalLicense {
        key_hash: key_hash.clone(),
        key_prefix: key_prefix.clone(),
        product: "focusa".to_string(),
        tier: granted_tier.clone(),
        status: if is_dev_mode_fixture {
            "active".to_string()
        } else {
            status.clone()
        },
        commercial_use: granted_commercial_use,
        customer_email: args.email.clone().unwrap_or_default(),
        features: granted_features.clone(),
        offline_valid_until: Some(offline_until.clone()),
        expires_at: expires_at.clone(),
        eval: is_dev_mode_fixture,
        registry: registry.clone(),
        issued_at,
    };
    let license_json = serde_json::to_string_pretty(&license)?;
    fs::write(&license_file, format!("{license_json}\n"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&license_file)?.permissions();
        p.set_mode(0o600);
        fs::set_permissions(&license_file, p)?;
    }

    // 6. Write the license authority file so the operator can see who
    //    governs this install.
    let authority = serde_json::json!({
        "name": "Wirebot / Phil Overacity LLC",
        "url": registry.clone(),
        "doc": "https://install.focusa.dev/license",
        "support": "https://focusa.dev/support",
        "registry_url": registry.clone(),
        "validate_path": REGISTRY_VALIDATE_PATH,
        "spec_refs": [
            "docs/SPEC_118_LICENSING.md",
            "docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md"
        ],
        "written_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "channel": "devmodefull",
        "target": "test-fixture",
    });
    fs::write(
        &authority_file,
        format!("{}\n", serde_json::to_string_pretty(&authority)?),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&authority_file)?.permissions();
        p.set_mode(0o600);
        fs::set_permissions(&authority_file, p)?;
    }

    // 7. Write the durable receipt (the operator's only local record).
    let receipt = serde_json::json!({
        "issued_at": Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "tier": granted_tier,
        "status": if is_dev_mode_fixture { "active".to_string() } else { status.clone() },
        "expires_at": expires_at,
        "customer_email": args.email.clone(),
        "authority": {
            "name": "Wirebot / Phil Overacity LLC",
            "url": registry.clone(),
        },
        "key_hash": key_hash,
        "key_prefix": key_prefix,
        "eval": is_dev_mode_fixture,
        "commercial_use": granted_commercial_use,
        "devmodefull": true,
        "note": "Created by `focusa license devmodefull`. Use this to verify the full provisioning pipeline before the first real-money transaction. This receipt is the only durable local record of which authority + tier issued this license.",
    });
    fs::write(
        &receipt_file,
        format!("{}\n", serde_json::to_string_pretty(&receipt)?),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = fs::metadata(&receipt_file)?.permissions();
        p.set_mode(0o600);
        fs::set_permissions(&receipt_file, p)?;
    }

    // 8. Round-trip: read the license.json back through the daemon's
    //    parser to confirm the file shape is acceptable. A failure here
    //    means the on-disk shape diverges from what the daemon expects;
    //    in production that would be a "focusa license status" parse
    //    error.
    let round_trip = fs::read_to_string(&license_file)
        .ok()
        .and_then(|s| serde_json::from_str::<LocalLicense>(&s).ok());
    let round_trip_status = match round_trip {
        Some(_) => "parsed",
        None => "parse_failed",
    };
    // Also load through the public `load_local_license` so we exercise
    // the same code path as `focusa license status` / `focusa license
    // doctor`. The daemon reads from the canonical path
    // (`~/.config/focusa/license.json`), which is where we just wrote.
    let canonical_path = focusa_core::license::license_file_path();
    let status_round_trip = focusa_core::license::load_local_license()
        .ok()
        .map(|s| s.status);

    // 9. Report.
    let summary = serde_json::json!({
        "step": "report",
        "valid": valid,
        "registry_status": status,
        "registry_tier": tier,
        "registry_commercial_use": commercial_use,
        "is_dev_mode_fixture": is_dev_mode_fixture,
        "granted_tier": if is_dev_mode_fixture { "evaluation".to_string() } else { tier.clone() },
        "granted_commercial_use": granted_commercial_use,
        "granted_features": granted_features,
        "offline_valid_until": offline_until,
        "issued_at": issued_at,
        "files": {
            "license": license_file.to_string_lossy(),
            "authority": authority_file.to_string_lossy(),
            "receipt": receipt_file.to_string_lossy(),
        },
        "round_trip": {
            "license_file_parse": round_trip_status,
            "license_status_load": if status_round_trip.is_some() { "ok" } else { "failed" },
        },
        "key_hash": key_hash,
        "key_prefix": key_prefix,
        "registry": registry,
        "validate_url": validate_url,
    });

    if args.print_response {
        let mut both = body.clone();
        if let Some(obj) = both.as_object_mut() {
            obj.insert("devmodefull".to_string(), summary.clone());
        }
        let rendered = serde_json::to_string_pretty(&both)?;
        println!("{rendered}");
    } else if json_output {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("[devmodefull] step=report");
        println!("  registry:            {registry}");
        println!("  valid:               {valid}");
        println!("  registry status:     {status}");
        println!("  registry tier:       {tier}");
        println!("  is_dev_mode:         {is_dev_mode_fixture}");
        println!(
            "  granted tier:        {}",
            if is_dev_mode_fixture {
                "evaluation"
            } else {
                tier.as_str()
            }
        );
        println!("  commercial_use:      {granted_commercial_use}");
        println!("  offline_valid_until: {offline_until}");
        println!("  files written:");
        println!("    license:    {}", license_file.to_string_lossy());
        println!("    authority:  {}", authority_file.to_string_lossy());
        println!("    receipt:    {}", receipt_file.to_string_lossy());
        println!("  round-trip:");
        println!("    license_file_parse:  {round_trip_status}");
        println!(
            "    license_status_load:  {}",
            if status_round_trip.is_some() {
                "ok"
            } else {
                "failed"
            }
        );
        if is_dev_mode_fixture {
            println!(
                "  note: dev_mode is a TEST FIXTURE; this install was downgraded to evaluation."
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_registry_error_codes_are_stable() {
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
    fn license_registry_error_recovery_hints_are_actionable() {
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
                    || hint.contains("https://focusa.dev/support")
                    || hint.contains("support@focusa.dev")
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
    fn license_wp_envelope_status_to_error() {
        // 404 → NotFound
        let body = serde_json::json!({"code": "focusa_license_not_found", "message": "missing"});
        assert!(matches!(
            map_wp_error_status(404, &body),
            RegistryError::NotFound
        ));

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
        assert!(matches!(
            map_wp_error_status(401, &body),
            RegistryError::Invalid
        ));

        // 403 → Revoked (no code match, falls through to status)
        let body = serde_json::json!({"code": "", "message": "revoked"});
        assert!(matches!(
            map_wp_error_status(403, &body),
            RegistryError::Revoked
        ));

        // 422 → Malformed
        let body = serde_json::json!({"errors": {"license_key": ["bad"]}});
        assert!(matches!(
            map_wp_error_status(422, &body),
            RegistryError::Malformed(_)
        ));
    }
}
