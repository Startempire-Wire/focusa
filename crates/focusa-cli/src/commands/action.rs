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
    /// Classify an operator prompt before mutation.
    ClassifyIntent(IntentClassifyArgs),
}

#[derive(Args, Debug, Clone)]
pub struct IntentClassifyArgs {
    /// Operator prompt/current ask to classify.
    #[arg(long)]
    pub prompt: String,
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

#[derive(Debug, Serialize)]
pub struct IntentClassificationEnvelope {
    pub schema: &'static str,
    pub mode: &'static str,
    pub mutation_allowed: bool,
    pub requires_preflight: bool,
    pub recommended_action: &'static str,
}

pub fn classify_intent(prompt: &str) -> IntentClassificationEnvelope {
    let lower = prompt.trim().to_ascii_lowercase();
    let planning_markers = [
        "maybe", "what if", "could we", "can we", "discuss", "explore", "plan", "spec",
    ];
    let diagnosis_markers = [
        "read",
        "inspect",
        "investigate",
        "diagnose",
        "why",
        "what happened",
    ];
    let implementation_markers = [
        "implement",
        "build",
        "add",
        "fix",
        "patch",
        "change",
        "create",
    ];
    let runtime_markers = ["restart", "start daemon", "stop daemon", "pair", "deploy"];
    let destructive_markers = ["delete", "remove", "overwrite", "reset", "clean", "kill"];

    if destructive_markers
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return IntentClassificationEnvelope {
            schema: "focusa.intent_mode_gate.v1",
            mode: "destructive_or_high_risk_requires_confirmation",
            mutation_allowed: false,
            requires_preflight: true,
            recommended_action: "require explicit confirmation and operational context preflight before mutation",
        };
    }
    if planning_markers.iter().any(|marker| lower.contains(marker)) {
        return IntentClassificationEnvelope {
            schema: "focusa.intent_mode_gate.v1",
            mode: "planning_discussion",
            mutation_allowed: false,
            requires_preflight: false,
            recommended_action: "produce plan/spec only; do not mutate files or runtime",
        };
    }
    if runtime_markers.iter().any(|marker| lower.contains(marker)) {
        return IntentClassificationEnvelope {
            schema: "focusa.intent_mode_gate.v1",
            mode: "runtime_operation_authorized",
            mutation_allowed: true,
            requires_preflight: true,
            recommended_action: "run operational context preflight before runtime mutation",
        };
    }
    if implementation_markers
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return IntentClassificationEnvelope {
            schema: "focusa.intent_mode_gate.v1",
            mode: "implementation_authorized",
            mutation_allowed: true,
            requires_preflight: true,
            recommended_action: "run repo/status and operational context preflight before implementation",
        };
    }
    if diagnosis_markers
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return IntentClassificationEnvelope {
            schema: "focusa.intent_mode_gate.v1",
            mode: "diagnosis",
            mutation_allowed: false,
            requires_preflight: false,
            recommended_action: "read/inspect only; no mutation",
        };
    }
    IntentClassificationEnvelope {
        schema: "focusa.intent_mode_gate.v1",
        mode: "diagnosis",
        mutation_allowed: false,
        requires_preflight: false,
        recommended_action: "treat ambiguous prompt as read-only until implementation intent is explicit",
    }
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
        ActionCmd::ClassifyIntent(args) => {
            let envelope = classify_intent(args.prompt.as_str());
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("mode: {}", envelope.mode);
                println!("mutation_allowed: {}", envelope.mutation_allowed);
                println!("recommended_action: {}", envelope.recommended_action);
            }
        }
    }
    Ok(())
}

pub fn evaluate_preflight(args: ActionPreflightArgs) -> ActionPreflightEnvelope {
    let kind = args.kind.trim().to_ascii_lowercase();
    let source = args
        .source
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
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
            "Inspect existing runtime, repair from local repo if needed, then run focusa pair."
                .to_string()
        });
    }

    if install_role == "unknown" && is_release_binary_replace && verdict == PreflightVerdict::Allow
    {
        verdict = PreflightVerdict::AskOperator;
        conflicts.push(PreflightConflict {
            class: "environment_role_unknown_for_risky_mutation",
            why: "Binary replacement requires a verified install role before mutation.",
        });
        safe_alternative =
            Some("Verify environment contract before replacing Focusa binaries.".to_string());
    }

    ActionPreflightEnvelope {
        schema: "focusa.operational_context_gate.v1",
        verdict,
        risk_class: if kind == "binary_replace" {
            "high"
        } else {
            "medium"
        },
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
    fn classifies_maybe_prompt_as_planning_without_mutation() {
        let envelope = classify_intent("Maybe we can add a flag for install context");
        assert_eq!(envelope.mode, "planning_discussion");
        assert!(!envelope.mutation_allowed);
        assert!(!envelope.requires_preflight);
    }

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
        assert!(
            envelope
                .safe_alternative
                .as_deref()
                .unwrap_or_default()
                .contains("local Focusa repo")
        );
    }
}
