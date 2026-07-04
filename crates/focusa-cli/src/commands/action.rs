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
    /// Plain-language error for agents/humans. This must be present when
    /// verdict=Block so agents don't see only opaque failure codes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plain_language_error: Option<String>,
    pub safe_alternative: Option<String>,
    pub evidence_refs: Vec<String>,
    /// Spec 109 / transcript gap 2026-07-03: per-check audit trail. Each
    /// check the preflight ran is listed with name / passed / observed
    /// value / threshold / recovery_hint so the agent knows WHY the verdict
    /// came out the way it did.
    pub checks: Vec<PreflightCheck>,
}

#[derive(Debug, Serialize)]
pub struct PreflightCheck {
    /// Stable name like "scope_resolution", "task_substitution", or
    /// "environment_role_known".
    pub name: &'static str,
    pub passed: bool,
    /// What the check observed (raw value, normalized form, etc).
    pub value_observed: String,
    /// Threshold / rule that value_observed was compared against.
    pub threshold: String,
    /// Hint if this check failed. Absent when passed.
    pub recovery_hint: Option<String>,
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
                if let Some(error) = &envelope.plain_language_error {
                    println!("error: {error}");
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
    let mut plain_language_error: Option<String> = None;
    let mut verdict = PreflightVerdict::Allow;
    let mut checks: Vec<PreflightCheck> = Vec::new();

    let target_lc = args
        .target
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let is_release_binary_replace = kind == "binary_replace"
        && (source == "github_release_asset" || source == "release_asset")
        && args
            .target
            .as_deref()
            .map(|target| target.ends_with("/focusa") || target.ends_with("/focusa-daemon"))
            .unwrap_or(false);
    let is_full_live_pipeline = kind == "full_live_release_pipeline"
        || (source == "github_actions_release_pipeline"
            && target_lc.contains("scripts/create-dev-release-tag.sh --base 0.9 --push"));
    let bypasses_full_live_pipeline = !is_full_live_pipeline
        && (kind == "local_release_build"
            || kind == "partial_deploy"
            || kind == "deploy_live_daemon"
            || kind == "daemon_deploy"
            || kind == "release_build"
            || source == "local_toolchain"
            || source == "local_repo_build"
            || source == "target_release"
            || target_lc.contains("target/release")
            || target_lc.contains("cargo build --release")
            || target_lc.contains("install-daemon.sh --binary")
            || target_lc.contains("gh workflow run 'deploy live daemon'")
            || target_lc.contains("gh workflow run deploy live daemon"));

    // Check 1: scope_resolution — project_root is safe (not /, /root, /home, etc.)
    let scope_safe = args
        .project_root
        .as_deref()
        .map(|p| {
            let r = p.trim().trim_end_matches('/');
            !r.is_empty()
                && r != "/"
                && r != "/root"
                && r != "/home"
                && r != "/tmp"
                && r != "/var"
                && r != "/usr"
                && r != "/opt"
        })
        .unwrap_or(false);
    checks.push(PreflightCheck {
        name: "scope_resolution",
        passed: scope_safe || !is_release_binary_replace,
        value_observed: args.project_root.clone().unwrap_or_default(),
        threshold: "project_root must be a non-trivial focused directory".to_string(),
        recovery_hint: if !scope_safe && is_release_binary_replace {
            Some("Pass --project-root to focusa install or set FOCUSA_PROJECT_ROOT env to a focused dir.".to_string())
        } else { None },
    });

    // Check 2: task_substitution — current_ask is not inconsistent with action
    let ask_consistent = !(current_ask.contains("pair") && is_release_binary_replace);
    checks.push(PreflightCheck {
        name: "task_substitution",
        passed: ask_consistent,
        value_observed: format!("current_ask={:?} action={:?}", args.current_ask, args.kind),
        threshold: "current_ask and proposed action should target the same task type".to_string(),
        recovery_hint: if !ask_consistent {
            Some("Inspect existing runtime, repair from local repo if needed, then run focusa pair.".to_string())
        } else { None },
    });

    // Check 3: full_live_pipeline_required — release/deploy must use full GH pipeline, never local toolchains or partial workflow shortcuts.
    checks.push(PreflightCheck {
        name: "full_live_pipeline_required",
        passed: !bypasses_full_live_pipeline,
        value_observed: format!("kind={:?} source={:?} target={:?}", args.kind, args.source, args.target),
        threshold: "release/deploy actions must use scripts/create-dev-release-tag.sh --base 0.9 --push, then GitHub CI -> Release -> Deploy Live Daemon".to_string(),
        recovery_hint: if bypasses_full_live_pipeline {
            Some("Blocked: use gh as the toolchain. Run scripts/create-dev-release-tag.sh --base 0.9 --push, then inspect with gh run list/view. Do not build locally or run only Deploy Live Daemon.".to_string())
        } else { None },
    });

    // Check 4: environment_role_known — install_role is not "unknown" for risky mutations
    let role_known = install_role != "unknown" || !is_release_binary_replace;
    checks.push(PreflightCheck {
        name: "environment_role_known",
        passed: role_known,
        value_observed: format!("install_role={:?}", args.install_role),
        threshold: "environment_role must be a verified focusa role (live_build_host, consumer, dev, etc.) for risky mutations".to_string(),
        recovery_hint: if !role_known {
            Some("Verify environment contract before replacing Focusa binaries.".to_string())
        } else { None },
    });

    // Check 5: live_build_host_safety — release-binary on a live_build_host is blocked
    let live_host_safe = !(install_role == "live_build_host" && is_release_binary_replace);
    checks.push(PreflightCheck {
        name: "live_build_host_safety",
        passed: live_host_safe,
        value_observed: format!("install_role={:?} is_release_binary_replace={}",
            args.install_role, is_release_binary_replace),
        threshold: "release_binary_replace must NOT run on a live_build_host role".to_string(),
        recovery_hint: if !live_host_safe {
            Some("Build from the verified local Focusa repo and restart the daemon as the project owner.".to_string())
        } else { None },
    });

    if bypasses_full_live_pipeline {
        verdict = PreflightVerdict::Block;
        conflicts.push(PreflightConflict {
            class: "full_live_release_pipeline_required",
            why: "Release/deploy attempted to bypass the full GitHub CI -> Release -> Deploy Live Daemon pipeline.",
        });
        plain_language_error = Some("Blocked: this would bypass the full live GitHub release pipeline. Focusa releases must be built and deployed by GitHub Actions, not local toolchains or partial deploy workflows.".to_string());
        safe_alternative = Some("Use gh as the release toolchain: run `scripts/create-dev-release-tag.sh --base 0.9 --push`, then inspect with `gh run list`, `gh run view`, and `gh release view`.".to_string());
    }

    if install_role == "live_build_host" && is_release_binary_replace {
        verdict = PreflightVerdict::Block;
        conflicts.push(PreflightConflict {
            class: "consumer_install_path_conflicts_with_live_build_host",
            why: "This host is the live Focusa build host; release assets are not the repair source.",
        });
        plain_language_error.get_or_insert_with(|| "Blocked: release assets are not the repair source for the live Focusa build host.".to_string());
        safe_alternative = Some("Use the full live GitHub release pipeline; do not replace live binaries by hand.".to_string());
    }

    if !ask_consistent {
        verdict = PreflightVerdict::Block;
        conflicts.push(PreflightConflict {
            class: "task_substitution_detected",
            why: "The current ask is pairing initiation, but the proposed action is binary installation/replacement.",
        });
        plain_language_error.get_or_insert_with(|| "Blocked: this action does not match the current user ask.".to_string());
        safe_alternative.get_or_insert_with(|| {
            "Inspect existing runtime, then run the requested pairing action; do not substitute a binary install."
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
        plain_language_error.get_or_insert_with(|| "Blocked until the environment role is verified for this risky mutation.".to_string());
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
        plain_language_error,
        safe_alternative,
        evidence_refs: Vec::new(),
        checks,
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
            project_root: Some("/workspace/focusa-project".to_string()),
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
                .plain_language_error
                .as_deref()
                .unwrap_or_default()
                .contains("Blocked")
        );
        assert!(
            envelope
                .safe_alternative
                .as_deref()
                .unwrap_or_default()
                .contains("full live GitHub release pipeline")
        );
    }
}
