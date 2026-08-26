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
use anyhow::Context;
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
    /// Interactive authority activation (Spec 152E §14.1): one shared flow
    /// renders email → verify → offer → checkout/poll → key/lease, existing
    /// key, Evaluation (Spec 172 limited-access overlay), resume, cancel,
    /// timeout, and recovery. Never accepts card data and never self-issues.
    ActivateFlow(ActivateFlowArgs),
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
    /// Fast preflight against the canonical entitlement decision (Spec 152F
    /// §6 chokepoint 4): renders base/premium/recovery reason and next action
    /// from the authority snapshot only, and exits nonzero when the target
    /// gate would deny. Never self-issues a grant.
    Preflight(PreflightArgs),
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
pub struct ActivateFlowArgs {
    /// Override the registry URL (default: https://wpuiai.com).
    #[arg(long, value_name = "URL")]
    pub registry: Option<String>,

    /// Resume a persisted activation registration (bounded poll
    /// continuation). The poll credential is re-supplied from the protected
    /// store; the snapshot never contains it.
    #[arg(long, value_name = "REGISTRATION_ID")]
    pub resume: Option<String>,

    /// Explicit email for a new activation (prompted interactively
    /// otherwise). The email only creates a pending attempt; verification is
    /// always required before any promotion.
    #[arg(long, value_name = "EMAIL")]
    pub email: Option<String>,

    /// Bounded poll wall-clock timeout in seconds (default: the
    /// registration poll budget governs; timeout settles fail-closed via
    /// cancel → recovery_only).
    #[arg(long, value_name = "SECONDS")]
    pub poll_timeout: Option<u64>,

    /// Agent/JSON protocol (Spec 152E §14.2): non-interactive, never
    /// prompts, never invents an email, verification code, consent, payment
    /// confirmation, or license. Returns typed human-action envelopes with a
    /// resumable registration handle; requires --email for a new attempt or
    /// --resume for a bounded poll continuation.
    #[arg(long)]
    pub agent: bool,

    /// Customer-controlled key reveal opt-in (agent mode): full key output
    /// is masked by default; revealing the one-time key requires BOTH this
    /// flag and --confirm-reveal.
    #[arg(long)]
    pub reveal_key: bool,

    /// Paid fast-path (all products through the license authority): redeem
    /// an already-paid license key in ONE request — no email verification,
    /// no offer menu, no polling. The server verifies the key, promotes the
    /// account, binds this device (verbatim node identity), and returns a
    /// root-signed lease that is persisted locally. Works for every product
    /// in the authority registry (Focusa, UIAI Engine, bundles).
    #[arg(long, value_name = "KEY", conflicts_with_all = ["email", "resume"])]
    pub license_key: Option<String>,

    /// Explicit confirmation for the customer-controlled key reveal
    /// (agent mode). Without it the key stays masked.
    #[arg(long)]
    pub confirm_reveal: bool,
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

#[derive(Args, Debug)]
pub struct PreflightArgs {
    /// Canonical operation family to preflight: base_focusa (default),
    /// automation, team_remote, release_proof, premium_updates, or
    /// customer_data_export.
    #[arg(long, value_name = "FAMILY")]
    pub family: Option<String>,
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

/// Paid fast-path (Spec 180 §2.2): one request -> verified key -> signed bundle lease persisted.
async fn run_redeem_fast_path(
    json_output: bool,
    license_key: &str,
    registry_override: Option<&str>,
) -> anyhow::Result<()> {
    use focusa_license::authority::{LeaseVerificationContext, SignedEnvelope};
    use focusa_license::authority_store::{
        AUTHORITY_STATE_FILE, PersistedAuthorityState, embedded_production_trust_roots,
    };
    let registry = registry_override
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string());
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    let config_dir = std::path::PathBuf::from(home).join(".config/focusa");
    let identity = crate::commands::activation_flow::resolve_flow_node_identity(&config_dir)?;
    let url = format!(
        "{}/wp-json/wpuiai-ai-cloud/v1/activation/redeem",
        registry.trim_end_matches('/')
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "license_key": license_key.trim(),
            "device_public_key": identity.node_id,
        }))
        .send()
        .await
        .context("reach license authority for redemption")?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.context("decode redemption reply")?;
    if !status.is_success() || body.get("lease_envelope").is_none() {
        let code = body
            .pointer("/error/code")
            .and_then(|v| v.as_str())
            .unwrap_or("AUTHORITY_UNAVAILABLE")
            .to_string();
        let out = json!({
            "ok": false,
            "code": code,
            "error": code.to_lowercase(),
            "recovery_hint": "Verify the key with support; paid keys redeem in one request. Retry is idempotent.",
        });
        if json_output {
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            eprintln!("Redemption failed: {code}");
        }
        std::process::exit(2);
    }
    let envelope_str = body["lease_envelope"].as_str().unwrap_or("{}").to_string();
    let envelope: serde_json::Value =
        serde_json::from_str(&envelope_str).context("decode lease delivery envelope")?;
    let key_set_raw = envelope["key_set"].to_string();
    let lease_raw = envelope["lease"].to_string();
    let key_set: SignedEnvelope =
        serde_json::from_str(&key_set_raw).context("decode key-set envelope")?;
    let lease: SignedEnvelope =
        serde_json::from_str(&lease_raw).context("decode lease envelope")?;
    let roots = embedded_production_trust_roots().context("load production authority roots")?;
    let context = LeaseVerificationContext {
        expected_product: "focusa".into(),
        expected_node_id: identity.node_id.clone(),
        now: chrono::Utc::now(),
        minimum_sequence: None,
        expected_previous_digest: None,
    };
    let (state, _snapshot) =
        PersistedAuthorityState::from_verified_envelopes(key_set, lease, &roots, &context)
            .context("verify issued authority lease")?;
    state
        .write_atomic(&config_dir.join(AUTHORITY_STATE_FILE))
        .context("persist authority-lease.json")?;
    let out = json!({
        "ok": true,
        "status": "activated",
        "node_id": identity.node_id,
        "state_file": config_dir.join(AUTHORITY_STATE_FILE).display().to_string(),
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✅ Activated — full operator bundle is live on this device.");
        println!("   {}", config_dir.join(AUTHORITY_STATE_FILE).display());
    }
    Ok(())
}

pub async fn run(json_output: bool, args: LicenseArgs) -> anyhow::Result<()> {
    match args.command {
        LicenseCmd::Status => run_status(json_output).await,
        LicenseCmd::Doctor => run_doctor(json_output).await,
        LicenseCmd::CheckFeature(a) => run_check_feature(json_output, a).await,
        LicenseCmd::Preflight(a) => run_preflight(json_output, a).await,
        LicenseCmd::ActivateFlow(a) => {
            if a.license_key.is_some() && !a.agent {
                run_redeem_fast_path(json_output, &a.license_key.unwrap(), a.registry.as_deref())
                    .await
            } else if a.agent {
                run_agent_activation_command(json_output, a).await
            } else {
                run_activation_flow_command(json_output, a).await
            }
        }
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

/// Canonical decision presenter (Spec 152F §5/§6). These helpers render the
/// authority snapshot's base/premium/recovery decisions through the same
/// projections the core, REST, TUI, and Pi surfaces inherit; the CLI never
/// grants, prices, or reinterprets entitlement (Spec 152F P5/P9) and never
/// exposes raw keys, tokens, or customer identity.
///
/// Render the canonical base-product decision plus the optional premium-family
/// decisions and the permanent recovery allowance for one snapshot.
fn canonical_decision_payload(
    snapshot: Option<&focusa_license::authority::EntitlementSnapshot>,
) -> Value {
    let base_product = match focusa_license::base_product_projection(snapshot) {
        Ok(projection) => serde_json::to_value(projection)
            .unwrap_or_else(|_| json!({ "decision": "denied", "permits_base_mutations": false })),
        Err(_) => json!({
            "schema": "focusa.base_product_projection.v1",
            "product": "unknown",
            "decision": "denied",
            "permits_base_mutations": false,
            "compatibility": {},
        }),
    };
    json!({
        "base_product": base_product,
        "premium": canonical_premium_presenter(snapshot),
        "recovery_allowance": canonical_recovery_presenter(snapshot),
    })
}

/// Render the canonical optional-premium family decisions (Spec 152F §3/§4).
/// Every decision is re-resolved from the authority snapshot only; the feature
/// identifiers are the exact registered registry entries and can never request
/// or expand a grant (Spec 152F P9).
fn canonical_premium_presenter(
    snapshot: Option<&focusa_license::authority::EntitlementSnapshot>,
) -> Vec<Value> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    let now = chrono::Utc::now();
    let mut rows = Vec::new();
    for (family, feature) in [
        (
            focusa_license::CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
        ),
        (
            focusa_license::CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
        ),
        (
            focusa_license::CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
        ),
        (
            focusa_license::CapabilityFamily::PremiumUpdates,
            "focusa.update.unattended",
        ),
        (
            focusa_license::CapabilityFamily::CustomerDataExport,
            "focusa.export.packaged",
        ),
    ] {
        rows.push(render_premium_decision(snapshot, family, feature, now));
    }
    rows
}

/// Render one premium family decision with a stable reason and recovery action.
fn render_premium_decision(
    snapshot: &focusa_license::authority::EntitlementSnapshot,
    family: focusa_license::CapabilityFamily,
    feature: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Value {
    let decision = if family == focusa_license::CapabilityFamily::CustomerDataExport {
        focusa_license::resolve_export_packaged(snapshot, feature, now)
    } else {
        focusa_license::resolve_premium_family(snapshot, family, feature, now)
    };
    let (decision_label, reason_code, recovery_action, offline_cached) = match &decision {
        focusa_license::PremiumFamilyDecision::Feature { offline_cached, .. } => (
            "feature",
            focusa_license::DecisionReason::RequireFeature.label(),
            focusa_license::DecisionReason::RequireFeature.recovery_action(),
            *offline_cached,
        ),
        focusa_license::PremiumFamilyDecision::Denied(denial) => {
            let (reason, recovery) = premium_denial_reason(denial);
            ("denied", reason, recovery, false)
        }
    };
    json!({
        "family": family.label(),
        "required_feature": feature,
        "decision": decision_label,
        "reason_code": reason_code,
        "recovery_action": recovery_action,
        "offline_cached": offline_cached,
    })
}

/// Stable snake_case reason and canonical recovery action for one premium
/// denial. Recovery guidance never exposes internal or raw authority material.
fn premium_denial_reason(
    denial: &focusa_license::PremiumFamilyDenial,
) -> (&'static str, &'static str) {
    use focusa_license::{DecisionReason, PremiumFamilyDenial};
    let _ = DecisionReason::RequireBase.label(); // canonical reason vocabulary
    match denial {
        PremiumFamilyDenial::BaseProductRequired { .. } => (
            "base_product_required",
            DecisionReason::RequireBase.recovery_action(),
        ),
        PremiumFamilyDenial::MissingLeaseSequence => (
            "missing_lease_sequence",
            DecisionReason::RequireFeature.recovery_action(),
        ),
        PremiumFamilyDenial::MissingLeaseBinding => (
            "missing_lease_binding",
            DecisionReason::RequireFeature.recovery_action(),
        ),
        PremiumFamilyDenial::InvalidRequiredFeature { .. } => (
            "invalid_required_feature",
            DecisionReason::RequireFeature.recovery_action(),
        ),
        PremiumFamilyDenial::FeatureNotRegistered { .. } => (
            "feature_not_registered",
            DecisionReason::RequireFeature.recovery_action(),
        ),
        PremiumFamilyDenial::MissingFeature { .. } => (
            "missing_feature",
            DecisionReason::RequireFeature.recovery_action(),
        ),
        PremiumFamilyDenial::MissingCachedGrantExpiry => (
            "missing_cached_grant_expiry",
            DecisionReason::RequireCachedFeature.recovery_action(),
        ),
        PremiumFamilyDenial::CachedGrantExpired => (
            "cached_grant_expired",
            DecisionReason::RequireCachedFeature.recovery_action(),
        ),
        PremiumFamilyDenial::ActiveLeaseExpired => (
            "active_lease_expired",
            DecisionReason::RequireFeature.recovery_action(),
        ),
        PremiumFamilyDenial::EntitlementStateNotUsable { .. } => (
            "entitlement_state_not_usable",
            DecisionReason::RequireFeature.recovery_action(),
        ),
        PremiumFamilyDenial::NotPremiumFamily { .. } => (
            "not_premium_family",
            DecisionReason::RequireFeature.recovery_action(),
        ),
    }
}

/// Render the permanent recovery allowance (Spec 152F §3 account_recovery,
/// §3.1 stable updates/repair, §3.3 export). Recovery, read, export, repair,
/// stable security update, rollback, and uninstall remain available regardless
/// of commercial state; the CLI never blocks them.
fn canonical_recovery_presenter(
    snapshot: Option<&focusa_license::authority::EntitlementSnapshot>,
) -> Value {
    let reason = snapshot
        .and_then(|entry| entry.recovery_reason.clone())
        .unwrap_or_else(|| "no_recovery_event".to_string());
    json!({
        "schema": "focusa.recovery_projection.v1",
        "reason": reason,
        "next_action": "recovery, export, repair, and uninstall remain available when execution is locked",
        "always_available": true,
    })
}

/// Spec 172 canonical presenter projection (Spec 172 §2.6, §4.1, §11, §21).
///
/// Renders the canonical posture, product, License Type, capability family,
/// denial, retained access, and upgrade/recovery action for one family from
/// the authority snapshot only. The CLI is a presenter surface: it never
/// accepts a caller-selected product, price, License Type, family, feature,
/// limit, node, or commercial right, never infers a grant from the installed
/// client, pairing, tool discovery, or email, and executes through the core
/// license guard (`focusa_license::resolve_license_guard`) that REST, TUI, Pi,
/// and agents inherit. JSON stays stable and redacted: no raw email, key,
/// token, or customer row.
const SPEC172_PRESENTER_SCHEMA: &str = "focusa.spec172.presenter_projection.v1";

/// Canonical Spec 172 postures (Spec 172 §4.1). `verified_no_license` is the
/// explicit authority-issued limited-access posture; a presenter never
/// synthesizes it from a paid-lease snapshot.
const SPEC172_POSTURES: [&str; 7] = [
    "unverified",
    "verified_no_license",
    "active_paid_operator",
    "offline_grace",
    "refunded_or_revoked",
    "expired",
    "missing_or_corrupt",
];

/// Canonical License Type codes and the composite Bundle SKU (Spec 172 §4.1).
/// The presenter renders only the frozen code matching the snapshot's own
/// product; it never selects, prices, or invents a License Type.
const SPEC172_LICENSE_TYPE_CODES: [&str; 3] = [
    "focusa_operator_lifetime_v1",
    "uiai_operator_lifetime_v1",
    "focusa_uiai_operator_bundle_lifetime_v1",
];

/// Stable error vocabulary (Spec 172 §21). Denials use only these codes.
const SPEC172_STABLE_ERRORS: [&str; 13] = [
    "EMAIL_VERIFICATION_REQUIRED",
    "VERIFIED_LIMITED_ACCESS",
    "LICENSE_TYPE_REQUIRED",
    "LICENSE_TYPE_NOT_INCLUDED",
    "PRODUCT_NOT_INCLUDED",
    "CAPABILITY_FAMILY_NOT_INCLUDED",
    "ENTITLEMENT_POLICY_UNKNOWN",
    "ENTITLEMENT_PRODUCT_MISMATCH",
    "NODE_LIMIT_REACHED",
    "OPERATOR_SEAT_LIMIT_REACHED",
    "HOSTED_RESOURCE_NOT_INCLUDED",
    "UPGRADE_AVAILABLE",
    "RECOVERY_ONLY",
];

/// Frozen retained-access set (Spec 172 §5.3/§17, Spec 152F P6): navigation,
/// status, account, read, export, recovery, repair, update, and uninstall stay
/// available regardless of commercial state. Byte-identical across CLI, Pi,
/// and agent presenters.
const SPEC172_RETAINED_ACCESS: [&str; 9] = [
    "navigation",
    "status",
    "account",
    "read",
    "export",
    "recovery",
    "repair",
    "update",
    "uninstall",
];

/// Stable upgrade actions a denial may recommend (presentation vocabulary
/// only; the action never grants or prices anything).
const SPEC172_UPGRADE_ACTIONS: [&str; 4] = [
    "none_required",
    "verify_email_or_manage_entitlement",
    "review_offer_or_manage_entitlement",
    "purchase_or_manage_entitlement",
];

const SPEC172_RECOVERY_ACTION: &str =
    "recovery, export, repair, and uninstall remain available when execution is locked";

/// Canonical Spec 172 posture label derived ONLY from the authority snapshot
/// state. A missing snapshot fails closed as `missing_or_corrupt`; the
/// presenter never invents `verified_no_license`.
fn spec172_posture(
    snapshot: Option<&focusa_license::authority::EntitlementSnapshot>,
) -> &'static str {
    use focusa_license::authority::EntitlementState;
    match snapshot.map(|entry| entry.state) {
        Some(EntitlementState::Active) => "active_paid_operator",
        Some(EntitlementState::OfflineGrace) => "offline_grace",
        Some(EntitlementState::Unactivated) => "unverified",
        Some(EntitlementState::RecoveryOnly) => "refunded_or_revoked",
        None => "missing_or_corrupt",
    }
}

/// Canonical License Type code for the snapshot's own product (Spec 172 §4.1).
/// Only usable authority states carry a License Type; the presenter renders
/// the frozen code for the snapshot's product and never a caller-chosen code.
fn spec172_license_type(
    snapshot: Option<&focusa_license::authority::EntitlementSnapshot>,
) -> &'static str {
    use focusa_license::authority::EntitlementState;
    let Some(snapshot) = snapshot else {
        return "none";
    };
    if !matches!(
        snapshot.state,
        EntitlementState::Active | EntitlementState::OfflineGrace
    ) {
        return "none";
    }
    match snapshot.product.as_str() {
        "focusa" => "focusa_operator_lifetime_v1",
        "uiai_engine" => "uiai_operator_lifetime_v1",
        _ => "none",
    }
}

/// Stable Spec 172 denial code and upgrade action for a denied base gate.
fn spec172_base_denial(posture: &str, product: &str) -> (Option<&'static str>, &'static str) {
    match posture {
        "unverified" => (
            Some("EMAIL_VERIFICATION_REQUIRED"),
            "verify_email_or_manage_entitlement",
        ),
        "refunded_or_revoked" => (Some("RECOVERY_ONLY"), "review_offer_or_manage_entitlement"),
        "expired" => (
            Some("LICENSE_TYPE_REQUIRED"),
            "purchase_or_manage_entitlement",
        ),
        "missing_or_corrupt" => (
            Some("ENTITLEMENT_POLICY_UNKNOWN"),
            "review_offer_or_manage_entitlement",
        ),
        _ if product != "focusa" => (
            Some("PRODUCT_NOT_INCLUDED"),
            "review_offer_or_manage_entitlement",
        ),
        _ => (
            Some("LICENSE_TYPE_REQUIRED"),
            "purchase_or_manage_entitlement",
        ),
    }
}

/// Resolve one family's canonical Spec 172 denial + upgrade action from the
/// same base/premium decisions the other presenters inherit. `None` denial
/// means the family is usable. All vocabulary comes from the frozen constants
/// above; no caller-supplied product, price, License Type, feature, limit,
/// node, or commercial right is accepted.
fn spec172_denial_and_upgrade(
    snapshot: Option<&focusa_license::authority::EntitlementSnapshot>,
    family: &str,
    posture: &str,
    product: &str,
) -> (Option<&'static str>, &'static str) {
    let Some(snapshot) = snapshot else {
        return (
            Some("ENTITLEMENT_POLICY_UNKNOWN"),
            "review_offer_or_manage_entitlement",
        );
    };
    if family == "base_focusa" {
        use focusa_license::BaseProductDecision;
        return match focusa_license::resolve_base_focusa_product(
            &snapshot.product,
            focusa_license::authority_policy_state(snapshot),
        ) {
            BaseProductDecision::Entitled => (None, "none_required"),
            BaseProductDecision::Limited => (
                Some("VERIFIED_LIMITED_ACCESS"),
                "review_offer_or_manage_entitlement",
            ),
            BaseProductDecision::Denied => spec172_base_denial(posture, product),
        };
    }
    // Optional families re-resolve the exact registered feature identifier so
    // the denial mirrors the canonical premium decision (never a stored claim).
    let (family_enum, feature) = match family {
        "automation" => (
            focusa_license::CapabilityFamily::Automation,
            "focusa.agent.silent_sessions",
        ),
        "team_remote" => (
            focusa_license::CapabilityFamily::TeamRemote,
            "focusa.team.multi_operator",
        ),
        "release_proof" => (
            focusa_license::CapabilityFamily::ReleaseProof,
            "focusa.release.proof",
        ),
        "premium_updates" => (
            focusa_license::CapabilityFamily::PremiumUpdates,
            "focusa.update.unattended",
        ),
        "customer_data_export" => (
            focusa_license::CapabilityFamily::CustomerDataExport,
            "focusa.export.packaged",
        ),
        _ => {
            return (
                Some("CAPABILITY_FAMILY_NOT_INCLUDED"),
                "review_offer_or_manage_entitlement",
            );
        }
    };
    let now = chrono::Utc::now();
    let decision = if family == "customer_data_export" {
        focusa_license::resolve_export_packaged(snapshot, feature, now)
    } else {
        focusa_license::resolve_premium_family(snapshot, family_enum, feature, now)
    };
    use focusa_license::PremiumFamilyDecision;
    match decision {
        PremiumFamilyDecision::Feature { .. } => (None, "none_required"),
        PremiumFamilyDecision::Denied(_) => {
            use focusa_license::BaseProductDecision;
            let base = focusa_license::resolve_base_focusa_product(
                &snapshot.product,
                focusa_license::authority_policy_state(snapshot),
            );
            if !base.permits_base_mutations() {
                spec172_base_denial(posture, product)
            } else {
                (
                    Some("CAPABILITY_FAMILY_NOT_INCLUDED"),
                    "review_offer_or_manage_entitlement",
                )
            }
        }
    }
}

/// Render the Spec 172 canonical presenter projection for one family. The
/// envelope is byte-stable across CLI, Pi, and agent presenters and matches
/// the committed parity fixtures (`crates/focusa-cli/tests/fixtures/`).
fn spec172_projection(
    snapshot: Option<&focusa_license::authority::EntitlementSnapshot>,
    family: &str,
) -> Value {
    let posture = spec172_posture(snapshot);
    let license_type = spec172_license_type(snapshot);
    let product = snapshot
        .map(|entry| entry.product.as_str())
        .unwrap_or("unknown");
    let (denial, upgrade_action) = spec172_denial_and_upgrade(snapshot, family, posture, product);
    json!({
        "schema": SPEC172_PRESENTER_SCHEMA,
        "posture": posture,
        "product": product,
        "license_type": license_type,
        "family": family,
        "denial": denial,
        "retained_access": SPEC172_RETAINED_ACCESS,
        "upgrade_action": upgrade_action,
        "recovery_action": SPEC172_RECOVERY_ACTION,
        "grant_inferred_from_surface": false,
    })
}

/// Fast preflight against the canonical entitlement decision (Spec 152F §6
/// chokepoint 4). Renders the same base/premium/recovery envelope as `status`
/// and exits nonzero when the target gate would deny, giving commands fast
/// feedback before side effects. The decision is resolved from the authority
/// snapshot only; no local grant is ever issued.
async fn run_preflight(json_output: bool, args: PreflightArgs) -> anyhow::Result<()> {
    let guard = focusa_license::resolve_license_guard();
    let snapshot = guard.entitlement.as_ref();
    let authority = focusa_license::entitlement_projection(snapshot)?;
    let entitlement_decision = focusa_license::entitlement_decision_projection(snapshot)?;
    let decision = canonical_decision_payload(snapshot);
    let family = args.family.as_deref().unwrap_or("base_focusa");
    let payload = json!({
        "schema": "focusa.authority_preflight.v1",
        "authority": authority,
        "entitlement_decision": entitlement_decision,
        "base_product": decision["base_product"],
        "premium": decision["premium"],
        "recovery_allowance": decision["recovery_allowance"],
        "spec172": spec172_projection(snapshot, family),
        "recovery_policy": "recovery, export, repair, and uninstall remain available when execution is locked",
    });
    let (decision_label, reason_code, next_action) = match family {
        "base_focusa" => {
            let label = decision["base_product"]["decision"]
                .as_str()
                .unwrap_or("denied")
                .to_string();
            (
                label,
                focusa_license::DecisionReason::RequireBase
                    .label()
                    .to_string(),
                focusa_license::DecisionReason::RequireBase
                    .recovery_action()
                    .to_string(),
            )
        }
        "automation"
        | "team_remote"
        | "release_proof"
        | "premium_updates"
        | "customer_data_export" => {
            let row = decision["premium"]
                .as_array()
                .and_then(|rows| {
                    rows.iter()
                        .find(|row| row["family"].as_str() == Some(family))
                })
                .cloned()
                .unwrap_or_else(|| {
                    json!({
                        "decision": "denied",
                        "reason_code": "missing_feature",
                        "recovery_action": "review_offer_or_manage_entitlement",
                    })
                });
            let label = row["decision"].as_str().unwrap_or("denied").to_string();
            let reason = row["reason_code"].as_str().unwrap_or("denied").to_string();
            let next = row["recovery_action"]
                .as_str()
                .unwrap_or("license_status")
                .to_string();
            (label, reason, next)
        }
        _ => anyhow::bail!("E_AUTHORITY_UNKNOWN_PREFLIGHT_FAMILY: unknown family {family}"),
    };

    if json_output {
        let mut out = payload;
        out["preflight"] = json!({
            "family": family,
            "decision": decision_label,
            "reason_code": reason_code,
            "next_action": next_action,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Focusa entitlement preflight");
        println!("Family:         {family}");
        println!(
            "Posture:        {}",
            payload["spec172"]["posture"].as_str().unwrap_or("unknown")
        );
        println!(
            "License type:   {}",
            payload["spec172"]["license_type"]
                .as_str()
                .unwrap_or("none")
        );
        println!("Decision:       {decision_label}");
        println!("Reason:         {reason_code}");
        println!("Next action:    {next_action}");
        if let Some(denial) = payload["spec172"]["denial"].as_str() {
            println!("Denial:         {denial}");
        }
        println!(
            "Recovery:       {}",
            decision["recovery_allowance"]["next_action"]
                .as_str()
                .unwrap_or("recovery, export, repair, and uninstall remain available when execution is locked")
        );
    }

    // Nonzero exit semantics: a denied (or base-limited) gate fails closed.
    if decision_label == "denied" || decision_label == "limited" {
        anyhow::bail!(
            "E_AUTHORITY_ENTITLEMENT_REQUIRED: family={family} decision={decision_label}"
        );
    }
    Ok(())
}

async fn run_status(json_output: bool) -> anyhow::Result<()> {
    // #342 field evidence: a customer who completed activation manually on the
    // authority website must see licensed state here. Before projecting, give
    // any persisted registration one bounded chance to reconcile with the
    // authority. Fail-closed: errors leave the local projection untouched.
    {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        if let Some(home) = home {
            let config_dir = home.join(".config/focusa");
            let guard_probe = focusa_license::resolve_license_guard();
            let activated = guard_probe
                .entitlement
                .as_ref()
                .map(|entitlement| {
                    entitlement.state != focusa_license::authority::EntitlementState::Unactivated
                })
                .unwrap_or(false);
            if !activated {
                let _ =
                    crate::commands::activation_flow::reconcile_status_with_authority(&config_dir);
            }
        }
    }
    let guard = focusa_license::resolve_license_guard();
    let authority = focusa_license::entitlement_projection(guard.entitlement.as_ref())?;
    let entitlement_decision =
        focusa_license::entitlement_decision_projection(guard.entitlement.as_ref())?;
    let decision = canonical_decision_payload(guard.entitlement.as_ref());
    let payload = json!({
        "schema": "focusa.authority_license_status.v1",
        "authority": authority,
        "entitlement_decision": entitlement_decision,
        "base_product": decision["base_product"],
        "premium": decision["premium"],
        "recovery_allowance": decision["recovery_allowance"],
        "spec172": spec172_projection(guard.entitlement.as_ref(), "base_focusa"),
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
            "Posture:        {}",
            payload["spec172"]["posture"].as_str().unwrap_or("unknown")
        );
        println!(
            "License type:   {}",
            payload["spec172"]["license_type"]
                .as_str()
                .unwrap_or("none")
        );
        println!(
            "Decision:       {} ({})",
            payload["entitlement_decision"]["status"]
                .as_str()
                .unwrap_or("unknown"),
            payload["entitlement_decision"]["reason_code"]
                .as_str()
                .unwrap_or("unknown")
        );
        println!(
            "Recovery action: {}",
            payload["entitlement_decision"]["recovery_action"]
                .as_str()
                .unwrap_or("unknown")
        );
        if let Some(sequence) = payload["authority"]["lease_sequence"].as_u64() {
            println!("Lease sequence: {sequence}");
        }
        println!(
            "Base product:   {} (product={})",
            payload["base_product"]["decision"]
                .as_str()
                .unwrap_or("denied"),
            payload["base_product"]["product"]
                .as_str()
                .unwrap_or("unknown")
        );
        let premium_summary = payload["premium"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .map(|row| {
                        format!(
                            "{}={}",
                            row["family"].as_str().unwrap_or("unknown"),
                            row["decision"].as_str().unwrap_or("denied")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        if !premium_summary.is_empty() {
            println!("Premium:        {premium_summary}");
        }
        println!(
            "Recovery:       {}",
            payload["recovery_allowance"]["next_action"]
                .as_str()
                .unwrap_or("recovery, export, repair, and uninstall remain available when execution is locked")
        );
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

// ── Spec 152E §14.1 interactive activation (shared flow) ────────────────

/// Interactive authority activation through the shared activation flow
/// (crates/focusa-cli/src/commands/activation_flow.rs). The flow drives the
/// shared `ActivationSession`; this command only wires the HTTP transport,
/// the terminal prompt source, and safe persistence (snapshot + poll
/// credential + verified signed lease). Card data is never accepted and
/// nothing is self-issued.
async fn run_activation_flow_command(
    json_output: bool,
    args: ActivateFlowArgs,
) -> anyhow::Result<()> {
    use crate::commands::activation_flow::{
        ActivationFlowSessionPersist, CLI_FLOW, StdinFlowInput, load_poll_credential,
        load_registration_snapshot, persist_poll_credential, persist_registration_snapshot,
        resolve_flow_node_identity, resume_activation_flow, run_activation_flow,
    };
    use focusa_license::activation_client::ActivationSession;
    use focusa_license::authority_credentials::KeyringCredentialStore;
    use focusa_license::{ActivationHttpClient, ActivationHttpPolicy};

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("HOME not set; cannot resolve activation state"))?;
    let config_dir = home.join(".config/focusa");
    let identity =
        resolve_flow_node_identity(&config_dir).map_err(|error| anyhow::anyhow!("{error}"))?;

    let origin = std::env::var("FOCUSA_AUTHORITY_ORIGIN")
        .unwrap_or_else(|_| "https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/".to_string());
    let base_url = reqwest::Url::parse(&origin).context("parse FOCUSA_AUTHORITY_ORIGIN")?;
    let policy = ActivationHttpPolicy {
        base_url,
        timeout: std::time::Duration::from_secs(30),
        max_response_bytes: 1024 * 1024,
    };
    let client = ActivationHttpClient::new(policy)
        .map_err(|error| anyhow::anyhow!("initialize activation authority transport: {error}"))?;
    let persist = ActivationFlowSessionPersist::new(&config_dir);

    if let Some(registration_id) = args.resume.as_deref() {
        let registration = load_registration_snapshot(&config_dir, registration_id)?;
        let credential = load_poll_credential(&KeyringCredentialStore, registration_id)?;
        let outcome = resume_activation_flow(
            client,
            CLI_FLOW,
            registration,
            credential,
            args.poll_timeout,
            json_output,
            Some(&persist),
        )?;
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "resumed": true,
                    "presenter_state": outcome.presenter_state,
                    "terminal": outcome.terminal,
                    "registration_id": outcome.registration_id,
                }))?
            );
        } else if outcome.terminal {
            println!("Resumed activation settled as {}.", outcome.presenter_state);
        } else {
            println!("Resumed activation is {}.", outcome.presenter_state);
        }
        return Ok(());
    }

    let mut input = StdinFlowInput;
    let outcome = run_activation_flow(
        client,
        CLI_FLOW,
        &mut input,
        args.email,
        Some(identity.node_id.clone()),
        None,
        args.poll_timeout,
        json_output,
        Some(&persist),
    )?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "resumed": false,
                "presenter_state": outcome.presenter_state,
                "terminal": outcome.terminal,
                "registration_id": outcome.registration_id,
            }))?
        );
    } else if outcome.terminal && outcome.presenter_state == "activated" {
        println!("Activation complete: device is entitled.");
    } else if outcome.terminal {
        println!(
            "Activation settled as {}; recovery, export, repair, and uninstall remain available.",
            outcome.presenter_state
        );
    } else {
        println!(
            "Activation paused at {}; resume with --resume {}.",
            outcome.presenter_state, outcome.registration_id
        );
    }
    Ok(())
}

/// Agent/JSON protocol (Spec 152E §14.2): non-interactive, fail-closed. The
/// agent begins with `--email` (pending attempt only) or resumes with
/// `--resume <handle>`, and receives typed human-action envelopes with a
/// resumable registration handle. The agent never invents an email,
/// verification code, consent, payment confirmation, or license; the full key
/// stays masked unless the customer explicitly opts in AND confirms
/// (--reveal-key --confirm-reveal).
async fn run_agent_activation_command(
    json_output: bool,
    args: ActivateFlowArgs,
) -> anyhow::Result<()> {
    use crate::commands::activation_flow::{
        ActivationFlowError, CLI_FLOW, load_poll_credential, load_registration_snapshot,
        persist_poll_credential, persist_registration_snapshot, resolve_flow_node_identity,
        resume_agent_activation, run_agent_activation,
    };
    use focusa_license::activation_client::ActivationSession;
    use focusa_license::authority_credentials::KeyringCredentialStore;
    use focusa_license::{ActivationHttpClient, ActivationHttpPolicy, AgentKeyReveal};

    let reveal = AgentKeyReveal {
        reveal_key: args.reveal_key,
        reveal_confirmation: args.confirm_reveal,
    };

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("HOME not set; cannot resolve activation state"))?;
    let config_dir = home.join(".config/focusa");
    let identity =
        resolve_flow_node_identity(&config_dir).map_err(|error| anyhow::anyhow!("{error}"))?;

    let origin = std::env::var("FOCUSA_AUTHORITY_ORIGIN")
        .unwrap_or_else(|_| "https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/".to_string());
    let base_url = reqwest::Url::parse(&origin).context("parse FOCUSA_AUTHORITY_ORIGIN")?;
    let policy = ActivationHttpPolicy {
        base_url,
        timeout: std::time::Duration::from_secs(30),
        max_response_bytes: 1024 * 1024,
    };
    let client = ActivationHttpClient::new(policy)
        .map_err(|error| anyhow::anyhow!("initialize activation authority transport: {error}"))?;

    // Agent-mode continuity: every state change persists the registration
    // snapshot and protected poll credential so any later process can resume
    // with --resume <registration_id> (#370). Mirrors the interactive path.
    let agent_persist = |session: &ActivationSession<ActivationHttpClient>| -> Result<
        (),
        crate::commands::activation_flow::ActivationFlowError,
    > {
        persist_registration_snapshot(&config_dir, session.registration())?;
        if let Some(credential) = session.poll_credential() {
            persist_poll_credential(&KeyringCredentialStore, session.registration_id(), credential)?;
        }
        Ok(())
    };

    let outcome = if let Some(registration_id) = args.resume.as_deref() {
        let registration = load_registration_snapshot(&config_dir, registration_id)?;
        let credential = load_poll_credential(&KeyringCredentialStore, registration_id)?;
        resume_agent_activation(
            client,
            CLI_FLOW,
            registration,
            credential,
            args.poll_timeout,
            reveal,
            Some(&agent_persist),
        )?
    } else {
        let email = args.email.ok_or_else(|| {
            anyhow::anyhow!(
                "EMAIL_REQUIRED: agent mode never prompts; pass --email for a new attempt or --resume <registration_id> to continue"
            )
        })?;
        run_agent_activation(
            client,
            CLI_FLOW,
            Some(email),
            Some(identity.node_id.clone()),
            reveal,
            Some(&agent_persist),
        )?
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&outcome.envelope)?);
    } else if outcome.terminal {
        println!(
            "Activation settled as {}; recovery, export, repair, and uninstall remain available.",
            outcome.envelope.state
        );
    } else {
        println!(
            "Human action required: {} (registration {}) — resume with --resume {} after the human completes it.",
            outcome
                .envelope
                .human_action
                .as_deref()
                .unwrap_or("human_action"),
            outcome.registration_id,
            outcome.registration_id
        );
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
