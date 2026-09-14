//! Shared, credential-free guided lifecycle UX for install/update/uninstall.

use anyhow::{Result, bail};
use chrono::Utc;
use clap::{Args, ValueEnum};
use focusa_core::install_lifecycle::{
    LifecycleAdapterKind, LifecycleOperation, LifecycleScope, LifecycleState,
    LifecycleTransactionKind, LifecycleTransition,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuidedAction {
    Inspect,
    Preview,
    Confirm,
    Apply,
    Resume,
    Repair,
    Rerun,
    Rollback,
    Uninstall,
    Purge,
}

#[derive(Args, Clone, Debug, Default)]
pub struct GuidedLifecycleArgs {
    /// Use the guided lifecycle transaction path.
    #[arg(long = "lifecycle-action", value_enum, value_name = "ACTION")]
    pub action: Option<GuidedAction>,

    /// Confirm the mutation described by --lifecycle-action.
    #[arg(long, requires = "action")]
    pub confirm: bool,

    /// Separately confirm deletion of Focusa user data (purge only).
    #[arg(long, requires = "action")]
    pub confirm_purge_data: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    Install,
    Update,
    Uninstall,
}

#[derive(Debug, Serialize)]
struct GuidedReceipt {
    schema: &'static str,
    status: &'static str,
    operation: LifecycleOperation,
    transaction: LifecycleTransition,
    adapter_contract: LifecycleAdapterKind,
    mutation_started: bool,
    next_command: Option<String>,
    recovery: Option<&'static str>,
}

/// Returns true when guidance was terminal and the caller must not mutate.
pub fn prepare(args: &GuidedLifecycleArgs, flow: Flow, json: bool) -> Result<bool> {
    let Some(action) = args.action else {
        // Compatibility path: existing commands retain their exact behavior.
        return Ok(false);
    };
    let operation = operation_for(action, flow)?;
    let mut terminal = matches!(
        action,
        GuidedAction::Inspect | GuidedAction::Preview | GuidedAction::Confirm
    );
    let mut status = match action {
        GuidedAction::Inspect => "inspected",
        GuidedAction::Preview => "previewed",
        GuidedAction::Confirm => "confirmed",
        GuidedAction::Resume => "resume_ready",
        _ => "apply_ready",
    };
    let mut recovery = None;

    if is_mutating(action) && !args.confirm {
        terminal = true;
        status = "operator_required";
        recovery = Some("Review the preview, then repeat the command with --confirm.");
    }
    if action == GuidedAction::Purge && !args.confirm_purge_data {
        terminal = true;
        status = "operator_required";
        recovery = Some(
            "Purge is separate from uninstall. Repeat with both --confirm and --confirm-purge-data only if user-data deletion is intended.",
        );
    }

    let transaction_id = format!("lifecycle-{}", Uuid::now_v7());
    // Durable resume is executed by the lifecycle orchestrator after loading its journal;
    // this pre-mutation projection starts unknown rather than fabricating persisted state.
    let prior_state = LifecycleState::Uninspected;
    let new_state = if terminal {
        match status {
            "operator_required" => LifecycleState::OperatorActionRequired,
            "previewed" | "inspected" | "confirmed" => LifecycleState::Preflighted,
            _ => prior_state,
        }
    } else {
        LifecycleState::Preflighted
    };
    let transaction = LifecycleTransition {
        transaction_id,
        transaction_kind: LifecycleTransactionKind::LifecycleMaintenance,
        scope: LifecycleScope {
            host_id: "local-host".into(),
            project_root: None,
            continuity_id: None,
        },
        prior_state,
        new_state,
        action: format!("{:?}", action).to_ascii_lowercase(),
        status: status.into(),
        evidence_refs: vec!["cli:guided-lifecycle-v1".into()],
        recovery: recovery.map(str::to_owned),
        occurred_at: Utc::now(),
    };
    let next_command = exact_command(flow, action, args);
    let receipt = GuidedReceipt {
        schema: "focusa.cli.lifecycle.receipt.v1",
        status,
        operation,
        transaction,
        adapter_contract: LifecycleAdapterKind::Pi,
        mutation_started: !terminal,
        next_command,
        recovery,
    };
    render(&receipt, json)?;
    Ok(terminal)
}

fn operation_for(action: GuidedAction, flow: Flow) -> Result<LifecycleOperation> {
    let operation = match (flow, action) {
        (Flow::Install, GuidedAction::Repair) => LifecycleOperation::Repair,
        (Flow::Install, GuidedAction::Rerun) => LifecycleOperation::Rerun,
        (
            Flow::Install,
            GuidedAction::Inspect
            | GuidedAction::Preview
            | GuidedAction::Confirm
            | GuidedAction::Apply
            | GuidedAction::Resume,
        ) => LifecycleOperation::Install,
        (Flow::Update, GuidedAction::Rollback) => LifecycleOperation::Rollback,
        (
            Flow::Update,
            GuidedAction::Inspect
            | GuidedAction::Preview
            | GuidedAction::Confirm
            | GuidedAction::Apply
            | GuidedAction::Resume,
        ) => LifecycleOperation::Update,
        (Flow::Uninstall, GuidedAction::Purge) => LifecycleOperation::Purge,
        (
            Flow::Uninstall,
            GuidedAction::Inspect
            | GuidedAction::Preview
            | GuidedAction::Confirm
            | GuidedAction::Apply
            | GuidedAction::Resume
            | GuidedAction::Uninstall,
        ) => LifecycleOperation::Uninstall,
        _ => bail!(
            "that lifecycle action is not valid for this command; inspect the command help for supported recovery actions"
        ),
    };
    Ok(operation)
}

fn is_mutating(action: GuidedAction) -> bool {
    matches!(
        action,
        GuidedAction::Apply
            | GuidedAction::Resume
            | GuidedAction::Repair
            | GuidedAction::Rerun
            | GuidedAction::Rollback
            | GuidedAction::Uninstall
            | GuidedAction::Purge
    )
}

fn exact_command(flow: Flow, action: GuidedAction, args: &GuidedLifecycleArgs) -> Option<String> {
    if !is_mutating(action)
        || args.confirm && (action != GuidedAction::Purge || args.confirm_purge_data)
    {
        return None;
    }
    let command = match flow {
        Flow::Install => "focusa install",
        Flow::Update => "focusa update",
        Flow::Uninstall => "focusa uninstall",
    };
    let mut value = format!(
        "{command} --lifecycle-action {} --confirm",
        action_name(action)
    );
    if action == GuidedAction::Purge {
        value.push_str(" --confirm-purge-data");
    }
    Some(value)
}

fn action_name(action: GuidedAction) -> &'static str {
    match action {
        GuidedAction::Inspect => "inspect",
        GuidedAction::Preview => "preview",
        GuidedAction::Confirm => "confirm",
        GuidedAction::Apply => "apply",
        GuidedAction::Resume => "resume",
        GuidedAction::Repair => "repair",
        GuidedAction::Rerun => "rerun",
        GuidedAction::Rollback => "rollback",
        GuidedAction::Uninstall => "uninstall",
        GuidedAction::Purge => "purge",
    }
}

fn render(receipt: &GuidedReceipt, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(receipt)?);
    } else {
        println!("Lifecycle: {} ({:?})", receipt.status, receipt.operation);
        if let Some(next) = &receipt.next_command {
            println!("Next: {next}");
        }
        if let Some(recovery) = receipt.recovery {
            println!("Recovery: {recovery}");
        }
        println!("Receipt: {}", receipt.transaction.transaction_id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purge_requires_separate_confirmation() {
        let args = GuidedLifecycleArgs {
            action: Some(GuidedAction::Purge),
            confirm: true,
            confirm_purge_data: false,
        };
        assert!(prepare(&args, Flow::Uninstall, true).unwrap());
    }

    #[test]
    fn legacy_path_is_untouched() {
        assert!(!prepare(&GuidedLifecycleArgs::default(), Flow::Install, true).unwrap());
    }

    #[test]
    fn cross_flow_recovery_is_rejected() {
        let args = GuidedLifecycleArgs {
            action: Some(GuidedAction::Rollback),
            confirm: true,
            confirm_purge_data: false,
        };
        assert!(prepare(&args, Flow::Install, true).is_err());
    }
}
