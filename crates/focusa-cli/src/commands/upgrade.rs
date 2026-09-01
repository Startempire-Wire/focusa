//! `focusa upgrade` — exact-release upgrade path for stale daemon/version drift.
//!
//! Upgrade resolves one immutable release tag, binds that tag into every
//! delegated installer download, and preserves the authoritative system path
//! when the running CLI came from `/usr/local/bin`.

use crate::commands::install::{
    Channel, InstallArgs, InstallTarget, ShellFamily, validate_release_tag,
};
use anyhow::{Context, anyhow};
use clap::Args;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Args, Debug)]
pub struct UpgradeArgs {
    /// Release channel to upgrade to.
    #[arg(long, value_name = "CHANNEL", default_value = "stable")]
    pub channel: Channel,

    /// Print current vs resolved version and the install plan without swapping binaries.
    #[arg(long)]
    pub dry_run: bool,

    /// Retained compatibility flag; stable upgrades always resolve canonical GitHub Latest.
    #[arg(long)]
    pub check_github: bool,

    /// Optional override for the GitHub owner/repo.
    #[arg(long, value_name = "OWNER/REPO")]
    pub github_repo: Option<String>,

    /// Commercial license key when no reusable local license record exists.
    #[arg(long, value_name = "KEY", conflicts_with = "eval")]
    pub license_key: Option<String>,

    /// Upgrade an evaluation installation.
    #[arg(long, conflicts_with = "license_key")]
    pub eval: bool,

    /// Skip systemd user unit or launchd registration during delegated install.
    #[arg(long)]
    pub no_service: bool,

    /// Persist PATH addition during delegated install.
    #[arg(long)]
    pub persist_path: bool,

    /// Skip persisting PATH addition during delegated install.
    #[arg(long, conflicts_with = "persist_path")]
    pub no_persist_path: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedUpgradeRelease {
    tag: String,
    source: &'static str,
}

pub async fn run(json_output: bool, args: UpgradeArgs) -> anyhow::Result<()> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let repo = args
        .github_repo
        .as_deref()
        .unwrap_or("Startempire-Wire/focusa");
    let resolved = resolve_upgrade_release(args.channel, repo).await?;
    let current_exe = std::env::current_exe().context("resolve current Focusa executable")?;
    let invoked_as = std::env::args_os().next().map(std::path::PathBuf::from);
    let system_install = executable_uses_system_surface(&current_exe)
        || invoked_as
            .as_deref()
            .is_some_and(executable_uses_system_surface)
        || system_link_targets_executable(&current_exe, Path::new("/usr/local/bin/focusa"));
    let latest_version = json!({"source": resolved.source, "value": resolved.tag.clone()});
    let plan = json!({
        "ok": true,
        "status": if args.dry_run { "dry_run" } else { "planned" },
        "command": "focusa upgrade",
        "channel": format!("{:?}", args.channel).to_lowercase(),
        "current_version": current_version,
        "latest_version": latest_version,
        "resolved_release_tag": resolved.tag.clone(),
        "system_install": system_install,
        "authoritative_surface": if system_install { "/usr/local/bin" } else { "$HOME/.local/bin" },
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
        install_dependencies: false,
        assume_yes: false,
        license_key: args.license_key.clone(),
        eval: args.eval,
        accept_license: false,
        no_service: args.no_service,
        reuse_existing_license: args.license_key.is_none() && !args.eval,
        suppress_completion_output: true,
        release_tag_override: Some(resolved.tag.clone()),
        system_install,
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
            "resolved_release_tag": plan["resolved_release_tag"],
            "system_install": system_install,
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
        "resolved_release_tag": plan["resolved_release_tag"],
        "system_install": system_install,
        "license_preserved": true,
        "recovery_hint": "Run focusa doctor --scope host and focusa recover --dry-run if post-upgrade daemon state looks stale.",
        "evidence_ref": "crates/focusa-cli/src/commands/upgrade.rs",
    });
    print_upgrade(&completed, json_output)?;
    Ok(())
}

async fn resolve_upgrade_release(
    channel: Channel,
    repo: &str,
) -> anyhow::Result<ResolvedUpgradeRelease> {
    if let Ok(tag) = std::env::var("FOCUSA_RELEASE_TAG") {
        let tag = tag.trim().to_string();
        if !tag.is_empty() {
            validate_release_tag(channel, &tag)?;
            return Ok(ResolvedUpgradeRelease {
                tag,
                source: "FOCUSA_RELEASE_TAG",
            });
        }
    }
    if channel != Channel::Stable {
        let suffix = match channel {
            Channel::Preview => "preview",
            Channel::Nightly => "nightly",
            Channel::Stable => unreachable!(),
        };
        let tag = format!("v{}-{suffix}", env!("CARGO_PKG_VERSION"));
        validate_release_tag(channel, &tag)?;
        return Ok(ResolvedUpgradeRelease {
            tag,
            source: "compiled_channel_version",
        });
    }
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let body = reqwest::Client::builder()
        .user_agent("focusa-upgrade/latest-resolver")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .context("build GitHub latest resolver")?
        .get(url)
        .send()
        .await
        .context("resolve canonical GitHub Latest release")?
        .error_for_status()
        .context("canonical GitHub Latest release returned failure")?
        .json::<Value>()
        .await
        .context("decode canonical GitHub Latest release")?;
    let tag = parse_latest_release_tag(channel, &body)?;
    Ok(ResolvedUpgradeRelease {
        tag,
        source: "github_releases_latest_api",
    })
}

fn parse_latest_release_tag(channel: Channel, body: &Value) -> anyhow::Result<String> {
    let tag = body
        .get("tag_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .ok_or_else(|| anyhow!("canonical GitHub Latest response has no tag_name"))?
        .to_string();
    validate_release_tag(channel, &tag)?;
    if body.get("draft").and_then(Value::as_bool).unwrap_or(true)
        || body
            .get("prerelease")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    {
        anyhow::bail!("canonical GitHub Latest release is draft or prerelease");
    }
    Ok(tag)
}

fn executable_uses_system_surface(path: &Path) -> bool {
    path.parent() == Some(Path::new("/usr/local/bin"))
}

fn system_link_targets_executable(executable: &Path, system_link: &Path) -> bool {
    let Ok(target) = std::fs::read_link(system_link) else {
        return false;
    };
    let target = if target.is_absolute() {
        target
    } else {
        system_link
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .join(target)
    };
    match (target.canonicalize(), executable.canonicalize()) {
        (Ok(target), Ok(executable)) => target == executable,
        _ => false,
    }
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
    fn latest_release_requires_stable_published_tag() {
        let body = json!({"tag_name":"v0.9.187", "draft":false, "prerelease":false});
        assert_eq!(
            parse_latest_release_tag(Channel::Stable, &body).unwrap(),
            "v0.9.187"
        );
        for bad in [
            json!({"tag_name":"v0.9.187-dev", "draft":false, "prerelease":false}),
            json!({"tag_name":"v0.9.187", "draft":true, "prerelease":false}),
            json!({"tag_name":"", "draft":false, "prerelease":false}),
        ] {
            assert!(parse_latest_release_tag(Channel::Stable, &bad).is_err());
        }
    }

    #[test]
    fn authoritative_system_surface_is_exact() {
        assert!(executable_uses_system_surface(Path::new(
            "/usr/local/bin/focusa"
        )));
        assert!(!executable_uses_system_surface(Path::new(
            "/root/.local/bin/focusa"
        )));
        assert!(!executable_uses_system_surface(Path::new("/tmp/focusa")));
    }

    #[cfg(unix)]
    #[test]
    fn promoted_system_link_preserves_upgrade_authority() {
        let fixture = std::env::temp_dir().join(format!(
            "focusa-upgrade-system-link-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&fixture).unwrap();
        let executable = fixture.join("focusa-real");
        let link = fixture.join("focusa");
        std::fs::write(&executable, b"binary").unwrap();
        std::os::unix::fs::symlink(&executable, &link).unwrap();
        assert!(system_link_targets_executable(&executable, &link));
        std::fs::remove_dir_all(fixture).unwrap();
    }
}
