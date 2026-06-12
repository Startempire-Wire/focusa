//! Action authority / mutation preflight commands.
//!
//! This is the first concrete enforcement surface for the Phone Bridge
//! context-authority incident: preserved context must become an allow/block/ask
//! verdict before risky mutations.

use clap::{Args, Subcommand};
use serde::Serialize;

#[derive(Subcommand)]
pub enum ActionCmd {
    /// Preflight a proposed action against current environment/ask authority.
    Preflight(ActionPreflightArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ActionPreflightArgs {
    /// Current operator ask or immediate task.
    #[arg(long)]
    pub current_ask: String,

    /// Proposed action kind, e.g. binary_replace, daemon_restart, pairing_start.
    #[arg(long)]
    pub kind: String,

    /// Proposed action target, e.g. /usr/local/bin/focusa.
    #[arg(long)]
    pub target: Option<String>,

    /// Proposed action source, e.g. github_release_asset, local_repo_build.
    #[arg(long)]
    pub source: Option<String>,

    /// Environment/install role, e.g. live_build_host, consumer_install.
    #[arg(long, default_value = "unknown")]
    pub install_role: String,

    /// Project root used as authority boundary.
    #[arg(long)]
    pub project_root: Option<String>,

    /// Repository version/tag/head, when known.
    #[arg(long)]
    pub repo_version: Option<String>,

    /// Installed CLI version, when known.
    #[arg(long)]
    pub cli_version: Option<String>,

    /// Running daemon version, when known.
    #[arg(long)]
    pub daemon_version: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreflightVerdict {
    Allow,
    Block,
    AskOperator,
}

#[derive(Debug, Serialize)]
pub struct ActionPreflightEnvelope {
    pub schema: &'static str,
    pub verdict: PreflightVerdict,
    pub risk_class: &'static str,
    pub current_ask: String,
    pub project_root: Option<String>,
    pub environment_role: String,
    pub proposed_action: ProposedAction,
    pub conflicts: Vec<PreflightConflict>,
    pub safe_alternative: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProposedAction {
    pub kind: String,
    pub target: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PreflightConflict {
    pub class: &'static str,
    pub why: &'static str,
}

pub async fn run(cmd: ActionCmd, json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        ActionCmd::Preflight(args) => {
            let envelope = evaluate_preflight(args);
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("verdict: {:?}", envelope.verdict);
                for conflict in &envelope.conflicts {
                    println!("conflict: {} — {}", conflict.class, conflict.why);
                }
                if let Some(safe_alternative) = &envelope.safe_alternative {
                    println!("safe_alternative: {safe_alternative}");
                }
            }
        }
    }
    Ok(())
}

pub fn evaluate_preflight(args: ActionPreflightArgs) -> ActionPreflightEnvelope {
    let kind = args.kind.trim().to_ascii_lowercase();
    let source = args.source.as_deref().unwrap_or("").trim().to_ascii_lowercase();
    let install_role = args.install_role.trim().to_ascii_lowercase();
    let current_ask = args.current_ask.trim().to_ascii_lowercase();

    let mut conflicts = Vec::new();
    let mut safe_alternative = None;
    let mut verdict = PreflightVerdict::Allow;

    let is_release_binary_replace = kind == "binary_replace"
        && (source == "github_release_asset" || source == "release_asset")
        && args
            .target
            .as_deref()
            .map(|target| target.ends_with("/focusa") || target.ends_with("/focusa-daemon"))
            .unwrap_or(false);

    if install_role == "live_build_host" && is_release_binary_replace {
        verdict = PreflightVerdict::Block;
        conflicts.push(PreflightConflict {
            class: "consumer_install_path_conflicts_with_live_build_host",
            why: "This host is the live Focusa build host; release assets are not the repair source.",
        });
        safe_alternative = Some("Build from the verified local Focusa repo and restart the daemon as the project owner.".to_string());
    }

    if current_ask.contains("pair") && is_release_binary_replace {
        verdict = PreflightVerdict::Block;
        conflicts.push(PreflightConflict {
            class: "task_substitution_detected",
            why: "The current ask is pairing initiation, but the proposed action is binary installation/replacement.",
        });
        safe_alternative.get_or_insert_with(|| {
            "Inspect existing runtime, repair from local repo if needed, then run focusa pair.".to_string()
        });
    }

    if install_role == "unknown" && is_release_binary_replace && verdict == PreflightVerdict::Allow {
        verdict = PreflightVerdict::AskOperator;
        conflicts.push(PreflightConflict {
            class: "environment_role_unknown_for_risky_mutation",
            why: "Binary replacement requires a verified install role before mutation.",
        });
        safe_alternative = Some("Verify environment contract before replacing Focusa binaries.".to_string());
    }

    ActionPreflightEnvelope {
        schema: "focusa.operational_context_gate.v1",
        verdict,
        risk_class: if kind == "binary_replace" { "high" } else { "medium" },
        current_ask: args.current_ask,
        project_root: args.project_root,
        environment_role: args.install_role,
        proposed_action: ProposedAction {
            kind: args.kind,
            target: args.target,
            source: args.source,
        },
        conflicts,
        safe_alternative,
        evidence_refs: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_release_asset_install_on_live_build_host_during_pairing() {
        let envelope = evaluate_preflight(ActionPreflightArgs {
            current_ask: "initiate Phone Bridge pairing".to_string(),
            kind: "binary_replace".to_string(),
            target: Some("/usr/local/bin/focusa".to_string()),
            source: Some("github_release_asset".to_string()),
            install_role: "live_build_host".to_string(),
            project_root: Some("/home/wirebot/focusa".to_string()),
            repo_version: Some("0.9.25-dev".to_string()),
            cli_version: Some("0.9.22-dev".to_string()),
            daemon_version: Some("0.9.23-dev".to_string()),
        });

        assert_eq!(envelope.verdict, PreflightVerdict::Block);
        assert!(envelope.conflicts.iter().any(|conflict| {
            conflict.class == "consumer_install_path_conflicts_with_live_build_host"
        }));
        assert!(envelope
            .safe_alternative
            .as_deref()
            .unwrap_or_default()
            .contains("local Focusa repo"));
    }
}
