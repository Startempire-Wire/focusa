//! `focusa upgrade` — evaluator-driven upgrade path for stale daemon/version drift.
//!
//! The real upgrade delegates to `focusa install`, which owns atomic stash,
//! rollback, checksum, service rendering, and license preservation behavior.
//! Optional latest lookup shells out to `gh release view` when requested.

use crate::commands::install::{Channel, InstallArgs, InstallTarget, ShellFamily};
use clap::Args;
use serde_json::{Value, json};
use std::process::Command;

#[derive(Args, Debug)]
pub struct UpgradeArgs {
    /// Release channel to upgrade to.
    #[arg(long, value_name = "CHANNEL", default_value = "stable")]
    pub channel: Channel,

    /// Print current vs latest version and the install plan without swapping binaries.
    #[arg(long)]
    pub dry_run: bool,

    /// Query GitHub release metadata with `gh release view` when available.
    #[arg(long)]
    pub check_github: bool,

    /// Optional override for the GitHub owner/repo.
    #[arg(long, value_name = "OWNER/REPO")]
    pub github_repo: Option<String>,

    /// Persist PATH addition during delegated install.
    #[arg(long)]
    pub persist_path: bool,

    /// Skip persisting PATH addition during delegated install.
    #[arg(long, conflicts_with = "persist_path")]
    pub no_persist_path: bool,
}

pub async fn run(json_output: bool, args: UpgradeArgs) -> anyhow::Result<()> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let latest_version =
        latest_version(args.channel, args.github_repo.as_deref(), args.check_github);
    let plan = json!({
        "ok": true,
        "status": if args.dry_run { "dry_run" } else { "planned" },
        "command": "focusa upgrade",
        "channel": format!("{:?}", args.channel).to_lowercase(),
        "current_version": current_version,
        "latest_version": latest_version,
        "atomicity": "delegates_to_focusa_install_atomic_stash_and_rollback",
        "license_preserved": true,
        "recovery_hint": "If upgrade fails, focusa install rollback restores the stashed install; run focusa recover --dry-run and focusa doctor --scope host.",
        "next_command": "focusa install --target=auto --channel=<channel>",
        "evidence_ref": "crates/focusa-cli/src/commands/upgrade.rs",
    });

    if args.dry_run {
        print_upgrade(&plan, json_output)?;
        return Ok(());
    }

    let install_args = InstallArgs {
        target: InstallTarget::Auto,
        channel: args.channel,
        dry_run: false,
        preflight: false,
        no_animation: false,
        quiet: false,
        assume_yes: false,
        license_key: None,
        eval: false,
        persist_path: args.persist_path,
        no_persist_path: args.no_persist_path,
        on_shell: ShellFamily::Auto,
        json: json_output,
        github_repo: args.github_repo,
    };

    if let Err(error) = crate::commands::install::run(install_args).await {
        let blocked = json!({
            "ok": false,
            "status": "blocked",
            "failure_class": "upgrade_failed",
            "current_version": env!("CARGO_PKG_VERSION"),
            "latest_version": plan["latest_version"],
            "license_preserved": true,
            "recovery_hint": format!("Upgrade failed: {error}. Installer rollback should preserve the prior install; run focusa recover --dry-run, focusa doctor --scope host, then retry focusa upgrade --dry-run."),
            "next_tools": ["focusa recover --dry-run", "focusa doctor --scope host", "focusa install --dry-run"],
            "evidence_ref": "crates/focusa-cli/src/commands/upgrade.rs",
        });
        print_upgrade(&blocked, json_output)?;
        anyhow::bail!("focusa upgrade failed; see recovery_hint above");
    }

    let completed = json!({
        "ok": true,
        "status": "completed",
        "current_version_before": plan["current_version"],
        "latest_version": plan["latest_version"],
        "license_preserved": true,
        "recovery_hint": "Run focusa doctor --scope host and focusa recover --dry-run if post-upgrade daemon state looks stale.",
        "evidence_ref": "crates/focusa-cli/src/commands/upgrade.rs",
    });
    print_upgrade(&completed, json_output)?;
    Ok(())
}

fn latest_version(channel: Channel, repo: Option<&str>, check_github: bool) -> Value {
    if let Ok(version) = std::env::var("FOCUSA_LATEST_VERSION") {
        return json!({"source":"FOCUSA_LATEST_VERSION", "value": version});
    }
    if check_github {
        let repo = repo.unwrap_or("Startempire-Wire/focusa");
        if let Ok(output) = Command::new("gh")
            .args([
                "release", "view", "--repo", repo, "--json", "tagName", "-q", ".tagName",
            ])
            .output()
            && output.status.success()
        {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !tag.is_empty() {
                return json!({"source":"gh_release_view", "value": tag});
            }
        }
    }
    json!({
        "source": "not_queried",
        "value": "unknown",
        "channel": format!("{:?}", channel).to_lowercase(),
        "hint": "set FOCUSA_LATEST_VERSION or pass --check-github",
    })
}

fn print_upgrade(envelope: &Value, json_output: bool) -> anyhow::Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(envelope)?);
    } else {
        println!("focusa upgrade");
        println!(
            "  status: {}",
            envelope["status"].as_str().unwrap_or("unknown")
        );
        println!(
            "  current_version: {}",
            envelope["current_version"]
                .as_str()
                .or_else(|| envelope["current_version_before"].as_str())
                .unwrap_or("unknown")
        );
        println!("  latest_version: {}", envelope["latest_version"]);
        println!(
            "  license_preserved: {}",
            envelope["license_preserved"].as_bool().unwrap_or(false)
        );
        println!(
            "  recovery_hint: {}",
            envelope["recovery_hint"].as_str().unwrap_or("unknown")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_version_has_non_network_fallback() {
        let latest = latest_version(Channel::Stable, None, false);
        assert_eq!(latest["source"], "not_queried");
    }
}
