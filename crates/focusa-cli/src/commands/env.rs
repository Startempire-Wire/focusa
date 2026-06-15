//! Env export and environment contract CLI.

use crate::api_client::ApiClient;
use anyhow::Context;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_ENVIRONMENT_CONTRACT_PATH: &str = "/etc/focusa/environment-contract.json";

#[derive(Subcommand)]
pub enum EnvCmd {
    /// Print env exports for shell integration.
    Shell,
    /// Machine-readable install/environment contract.
    #[command(subcommand)]
    Contract(EnvContractCmd),
}

#[derive(Subcommand)]
pub enum EnvContractCmd {
    /// Show the environment contract.
    Show(EnvContractShowArgs),
    /// Initialize or overwrite the environment contract.
    Init(EnvContractInitArgs),
}

#[derive(Args)]
pub struct EnvContractShowArgs {
    /// Contract path. Defaults to /etc/focusa/environment-contract.json.
    #[arg(long)]
    path: Option<PathBuf>,
}

#[derive(Args)]
pub struct EnvContractInitArgs {
    /// Contract path. Defaults to /etc/focusa/environment-contract.json.
    #[arg(long)]
    path: Option<PathBuf>,
    /// Install role: live_build_host, consumer_install, dev_worktree, unknown.
    #[arg(long)]
    role: String,
    /// Focusa project root.
    #[arg(long)]
    project_root: String,
    /// Expected owner user.
    #[arg(long)]
    owner: String,
    /// Machine kind: vps, mac, local, container, unknown.
    #[arg(long, default_value = "unknown")]
    machine_kind: String,
    /// Preferred binary source.
    #[arg(long, default_value = "local_repo_build")]
    preferred_source: String,
    /// Allow release asset installation on this host.
    #[arg(long, default_value_t = false)]
    release_asset_install_allowed: bool,
    /// Pairing state: never_paired, paired, unknown.
    #[arg(long, default_value = "unknown")]
    pairing_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContract {
    pub schema: &'static str,
    pub install_role: String,
    pub machine_kind: String,
    pub project_root: String,
    pub owner: String,
    pub binary_policy: BinaryPolicy,
    pub pairing_state: String,
    pub host: HostFacts,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryPolicy {
    pub preferred_source: String,
    pub release_asset_install_allowed: bool,
    pub local_build_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostFacts {
    pub os: String,
    pub arch: String,
    pub glibc: String,
}

#[derive(Debug, Serialize)]
struct MissingContract<'a> {
    schema: &'static str,
    status: &'static str,
    path: &'a Path,
    recommended_action: &'static str,
}

pub async fn run(cmd: EnvCmd, json: bool) -> anyhow::Result<()> {
    match cmd {
        EnvCmd::Contract(contract_cmd) => run_contract(contract_cmd, json).await,
        EnvCmd::Shell => {
            let api = ApiClient::new();
            let resp = api.get("/v1/env").await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
                return Ok(());
            }

            let exports = [
                ("MESSAGES_BASE_URL", resp["messages_base_url"].as_str()),
                ("KIMI_BASE_URL", resp["kimi_base_url"].as_str()),
                (
                    "KIMI_MESSAGES_BASE_URL",
                    resp["kimi_messages_base_url"].as_str(),
                ),
                ("OPENAI_BASE_URL", resp["openai_base_url"].as_str()),
            ];
            for (key, val) in exports {
                if let Some(v) = val {
                    println!("export {}=\"{}\"", key, v);
                }
            }
            Ok(())
        }
    }
}

async fn run_contract(cmd: EnvContractCmd, json: bool) -> anyhow::Result<()> {
    match cmd {
        EnvContractCmd::Show(args) => {
            let path = contract_path(args.path);
            if !path.exists() {
                let missing = MissingContract {
                    schema: "focusa.environment_contract.v1",
                    status: "missing",
                    path: &path,
                    recommended_action: "run focusa env contract init after verifying this host install role",
                };
                if json {
                    println!("{}", serde_json::to_string_pretty(&missing)?);
                } else {
                    println!("environment contract missing: {}", path.display());
                    println!("recommended_action: {}", missing.recommended_action);
                }
                return Ok(());
            }

            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let value: serde_json::Value = serde_json::from_str(&content)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&value)?);
            } else {
                println!("environment contract: {}", path.display());
                println!(
                    "install_role: {}",
                    value["install_role"].as_str().unwrap_or("unknown")
                );
                println!(
                    "project_root: {}",
                    value["project_root"].as_str().unwrap_or("unknown")
                );
                println!("owner: {}", value["owner"].as_str().unwrap_or("unknown"));
            }
        }
        EnvContractCmd::Init(args) => {
            let path = contract_path(args.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            let now = chrono::Utc::now().to_rfc3339();
            let contract = EnvironmentContract {
                schema: "focusa.environment_contract.v1",
                install_role: args.role.clone(),
                machine_kind: args.machine_kind,
                project_root: args.project_root,
                owner: args.owner,
                binary_policy: BinaryPolicy {
                    preferred_source: args.preferred_source,
                    release_asset_install_allowed: args.release_asset_install_allowed,
                    local_build_required: args.role == "live_build_host",
                },
                pairing_state: args.pairing_state,
                host: HostFacts {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    glibc: detect_glibc(),
                },
                created_at: now.clone(),
                updated_at: now,
            };
            let rendered = serde_json::to_string_pretty(&contract)?;
            std::fs::write(&path, format!("{rendered}\n"))
                .with_context(|| format!("failed to write {}", path.display()))?;
            if json {
                println!("{rendered}");
            } else {
                println!("environment contract written: {}", path.display());
            }
        }
    }
    Ok(())
}

fn contract_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| PathBuf::from(DEFAULT_ENVIRONMENT_CONTRACT_PATH))
}

fn detect_glibc() -> String {
    std::process::Command::new("getconf")
        .arg("GNU_LIBC_VERSION")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
