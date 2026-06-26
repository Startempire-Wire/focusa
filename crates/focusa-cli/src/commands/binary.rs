//! Binary provenance and compatibility preflight commands.

use clap::{Args, Subcommand};
use focusa_core::license::require_feature;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum BinaryCmd {
    /// Inspect a Focusa binary path for provenance/compatibility hints.
    Inspect(BinaryInspectArgs),
    /// Preflight a binary install before replacing a target path.
    PreflightInstall(BinaryPreflightInstallArgs),
}

#[derive(Args)]
pub struct BinaryInspectArgs {
    /// Binary path to inspect.
    pub path: PathBuf,
}

#[derive(Args)]
pub struct BinaryPreflightInstallArgs {
    /// Asset path to install.
    #[arg(long)]
    pub asset: PathBuf,
    /// Target binary path that would be overwritten.
    #[arg(long)]
    pub target: PathBuf,
    /// Install/environment role.
    #[arg(long, default_value = "unknown")]
    pub install_role: String,
    /// Asset source type: release_asset, github_release_asset, local_repo_build.
    #[arg(long, default_value = "unknown")]
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct BinaryInspection {
    pub schema: &'static str,
    pub binary: String,
    pub path: String,
    pub exists: bool,
    pub version: Option<String>,
    pub source_type: &'static str,
    pub host_glibc: String,
    pub required_glibc: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BinaryPreflightVerdict {
    Allow,
    Block,
    AskOperator,
}

#[derive(Debug, Serialize)]
pub struct BinaryPreflightEnvelope {
    pub schema: &'static str,
    pub verdict: BinaryPreflightVerdict,
    pub asset: BinaryInspection,
    pub target: BinaryInspection,
    pub conflicts: Vec<BinaryPreflightConflict>,
    pub safe_alternative: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BinaryPreflightConflict {
    pub class: &'static str,
    pub why: String,
}

pub async fn run(cmd: BinaryCmd, json_mode: bool) -> anyhow::Result<()> {
    // Spec §5.4 + §5.5: PreflightInstall is a packaged-installer-gated feature.
    if matches!(cmd, BinaryCmd::PreflightInstall(_))
        && let Err(e) = require_feature("packaged_installer") {
            anyhow::bail!("{e}");
        }
    match cmd {
        BinaryCmd::Inspect(args) => {
            let inspection = inspect_binary(&args.path);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&inspection)?);
            } else {
                println!("binary: {}", inspection.path);
                println!("exists: {}", inspection.exists);
                println!(
                    "version: {}",
                    inspection.version.as_deref().unwrap_or("unknown")
                );
                println!("host_glibc: {}", inspection.host_glibc);
                println!(
                    "required_glibc: {}",
                    inspection.required_glibc.as_deref().unwrap_or("unknown")
                );
            }
        }
        BinaryCmd::PreflightInstall(args) => {
            let envelope = preflight_install(args);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("verdict: {:?}", envelope.verdict);
                for conflict in &envelope.conflicts {
                    println!("conflict: {} — {}", conflict.class, conflict.why);
                }
                if let Some(action) = &envelope.safe_alternative {
                    println!("safe_alternative: {action}");
                }
            }
        }
    }
    Ok(())
}

pub fn preflight_install(args: BinaryPreflightInstallArgs) -> BinaryPreflightEnvelope {
    let asset = inspect_binary(&args.asset);
    let target = inspect_binary(&args.target);
    let mut conflicts = Vec::new();
    let mut verdict = BinaryPreflightVerdict::Allow;
    let mut safe_alternative = None;

    let source = args.source.trim().to_ascii_lowercase();
    let install_role = args.install_role.trim().to_ascii_lowercase();
    let is_release_asset = source == "release_asset" || source == "github_release_asset";

    if install_role == "live_build_host" && is_release_asset {
        verdict = BinaryPreflightVerdict::Block;
        conflicts.push(BinaryPreflightConflict {
            class: "release_asset_blocked_by_environment_contract",
            why: "live_build_host policy requires local repo build as the repair source"
                .to_string(),
        });
        safe_alternative = Some(
            "build from the verified local repo and install paired CLI/daemon outputs".to_string(),
        );
    }

    if let (Some(required), Some(host)) = (
        asset
            .required_glibc
            .as_deref()
            .and_then(parse_glibc_version),
        parse_glibc_version(&asset.host_glibc),
    ) && required > host
    {
        verdict = BinaryPreflightVerdict::Block;
        conflicts.push(BinaryPreflightConflict {
            class: "glibc_incompatible_asset",
            why: format!(
                "asset requires GLIBC_{}.{} but host provides {}",
                required.0, required.1, asset.host_glibc
            ),
        });
        safe_alternative.get_or_insert_with(|| "build from source on this host".to_string());
    }

    if install_role == "unknown" && is_release_asset && verdict == BinaryPreflightVerdict::Allow {
        verdict = BinaryPreflightVerdict::AskOperator;
        conflicts.push(BinaryPreflightConflict {
            class: "environment_role_unknown_for_binary_install",
            why: "release asset install requires a verified environment contract".to_string(),
        });
        safe_alternative =
            Some("run focusa env contract show/init before binary replacement".to_string());
    }

    BinaryPreflightEnvelope {
        schema: "focusa.binary_preflight.v1",
        verdict,
        asset,
        target,
        conflicts,
        safe_alternative,
    }
}

pub fn inspect_binary(path: &PathBuf) -> BinaryInspection {
    let exists = path.exists();
    BinaryInspection {
        schema: "focusa.binary_provenance.v1",
        binary: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string(),
        path: path.display().to_string(),
        exists,
        version: if exists { binary_version(path) } else { None },
        source_type: "unknown",
        host_glibc: detect_glibc(),
        required_glibc: if exists { required_glibc(path) } else { None },
    }
}

fn binary_version(path: &PathBuf) -> Option<String> {
    let output = std::process::Command::new(path)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn required_glibc(path: &PathBuf) -> Option<String> {
    let output = std::process::Command::new("strings")
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout
        .split(|ch: char| ch.is_whitespace() || ch == '\0')
        .filter_map(|token| token.strip_prefix("GLIBC_"))
        .filter_map(|version| {
            parse_glibc_version(version).map(|parsed| (parsed, version.to_string()))
        })
        .max_by_key(|(parsed, _)| *parsed)
        .map(|(_, version)| version)
}

fn detect_glibc() -> String {
    std::process::Command::new("getconf")
        .arg("GNU_LIBC_VERSION")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().replace("glibc ", ""))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn parse_glibc_version(value: &str) -> Option<(u32, u32)> {
    let clean = value
        .trim()
        .trim_start_matches("GLIBC_")
        .trim_start_matches("glibc ");
    let mut parts = clean.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    Some((major, minor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_glibc_versions() {
        assert_eq!(parse_glibc_version("GLIBC_2.39"), Some((2, 39)));
        assert_eq!(parse_glibc_version("glibc 2.28"), Some((2, 28)));
    }
}
