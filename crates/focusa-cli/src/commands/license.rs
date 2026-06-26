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
struct RegistryValidateResponse {
    #[serde(default)]
    valid: bool,
    #[serde(default)]
    product: String,
    #[serde(default)]
    tier: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    commercial_use: bool,
    #[serde(default)]
    team_use: bool,
    #[serde(default)]
    client_delivery: bool,
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

const DEFAULT_REGISTRY: &str = "https://install.focusa.dev";
const REGISTRY_VALIDATE_PATH: &str = "/wp-json/wpuiai-ai-cloud/v1/license/validate";
const LICENSE_FILE_NAME: &str = "license.json";

fn local_license_path() -> PathBuf {
    // Spec §5.1: ~/.config/focusa/license.json
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    home.join(".config").join("focusa").join(LICENSE_FILE_NAME)
}

/// POST a license key to the registry for validation, return parsed response.
async fn registry_validate(registry: &str, key: &str) -> anyhow::Result<RegistryValidateResponse> {
    let url = format!(
        "{}{}",
        registry.trim_end_matches('/'),
        REGISTRY_VALIDATE_PATH
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| anyhow::anyhow!("registry client build failed: {e}"))?;
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-License-Key", key)
        .json(&json!({ "license_key": key }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("registry POST failed: {e}"))?;
    let status = resp.status();
    let body: Value = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("registry response not JSON: {e}"))?;
    if !status.is_success() && !body.get("valid").and_then(Value::as_bool).unwrap_or(false) {
        // Registry returned an error envelope (e.g. 404 / license_not_found).
        let err = body
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("license_validation_failed");
        anyhow::bail!("license validation failed: {err} (HTTP {status})");
    }
    Ok(serde_json::from_value(body)?)
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
        std::process::exit(1);
    }
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
    let resp = registry_validate(&registry, &key).await?;
    if !resp.valid {
        let purchase = format!("{}/buy", DEFAULT_REGISTRY.trim_end_matches('/'));
        let license_url = format!("{}/license", DEFAULT_REGISTRY.trim_end_matches('/'));
        let out = json!({
            "ok": false,
            "error": "license_not_valid",
            "purchase_url": purchase,
            "license_url": license_url,
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
    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human_doctor(&report);
    }
    Ok(())
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
            let docs_url = "https://install.focusa.dev/license";
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
