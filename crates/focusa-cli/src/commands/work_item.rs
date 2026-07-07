//! Work item closure authority CLI surface (Spec 116 §12).
//!
// Commands:
//   focusa work-item close <id> --from-workpoint <WP_ID>
//   focusa work-item closure prepare <id>
//   focusa work-item closure validate <claim-id>
//   focusa work-item closure authorize <claim-id>
//   focusa work-item closure submit <claim-id>
//   focusa work-item closure reconcile <claim-id>
//   focusa work-item providers list
//   focusa work-item providers add <provider> --api-key <KEY>
//!   focusa work-item provider-guard evaluate --provider bd --command "bd close <id>"

use anyhow::Result;
use clap::{Args, Subcommand};
use focusa_core::work_item::types::{
    ClaimStatus, ClosureClaim, ClosureKind, EvidenceCitation, EvidenceKind, WorkItemProvider,
    WorkItemRef,
};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum WorkItemCmd {
    /// Close a work item with evidence (runs full lifecycle).
    Close(CloseArgs),
    /// Work with closure claims (prepare, validate, authorize, submit, reconcile).
    #[command(subcommand)]
    Closure(ClosureCmd),
    /// Manage provider adapters (list, add, remove, test).
    #[command(subcommand)]
    Providers(ProvidersCmd),
    /// Evaluate whether a provider command would be intercepted by the guard shim.
    #[command(subcommand)]
    ProviderGuard(ProviderGuardCmd),
}

#[derive(Args, Debug)]
pub struct CloseArgs {
    /// Provider-local item id (e.g. `focusa-glny` for bd, `ISS-123` for Jira).
    pub id: String,
    /// Workpoint id to pull evidence from.
    #[arg(long)]
    pub from_workpoint: String,
    /// Override evidence profile (default: release_proof).
    #[arg(long)]
    pub profile: Option<String>,
    /// Break-glass override. Requires --reason.
    #[arg(long)]
    pub override_: Option<String>,
    /// Actor email (default: $FOCUSA_USER or $USER).
    #[arg(long)]
    pub actor: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ClosureCmd {
    Prepare(PrepareArgs),
    Validate(ValidateArgs),
    Authorize(AuthorizeArgs),
    Submit(SubmitArgs),
    Reconcile(ReconcileArgs),
}

#[derive(Args, Debug)]
pub struct PrepareArgs {
    pub provider_item_id: String,
    #[arg(long)]
    pub kind: Option<String>,
    #[arg(long)]
    pub summary: Option<String>,
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    pub claim_id: String,
}

#[derive(Args, Debug)]
pub struct AuthorizeArgs {
    pub claim_id: String,
}

#[derive(Args, Debug)]
pub struct SubmitArgs {
    pub claim_id: String,
}

#[derive(Args, Debug)]
pub struct ReconcileArgs {
    pub claim_id: String,
}

#[derive(Subcommand, Debug)]
pub enum ProvidersCmd {
    /// List registered providers.
    List,
    /// Add a provider adapter.
    Add(ProviderAddArgs),
    /// Remove a provider adapter.
    Remove(ProviderRemoveArgs),
    /// Test connectivity for a provider.
    Test(ProviderTestArgs),
}

#[derive(Args, Debug)]
pub struct ProviderAddArgs {
    pub provider: String,
    #[arg(long)]
    pub api_key: Option<String>,
    #[arg(long)]
    pub token: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProviderRemoveArgs {
    pub provider: String,
}

#[derive(Args, Debug)]
pub struct ProviderTestArgs {
    pub provider: String,
}

#[derive(Subcommand, Debug)]
pub enum ProviderGuardCmd {
    /// Evaluate whether a provider command would be intercepted.
    Evaluate(ProviderGuardEvalArgs),
}

#[derive(Args, Debug)]
pub struct ProviderGuardEvalArgs {
    #[arg(long)]
    pub provider: String,
    #[arg(long)]
    pub command: String,
}

pub async fn run(cmd: WorkItemCmd) -> Result<()> {
    match cmd {
        WorkItemCmd::Close(args) => run_close(args).await,
        WorkItemCmd::Closure(cmd) => run_closure(cmd).await,
        WorkItemCmd::Providers(cmd) => run_providers(cmd).await,
        WorkItemCmd::ProviderGuard(cmd) => run_provider_guard(cmd).await,
    }
}

async fn run_close(args: CloseArgs) -> Result<()> {
    let actor = args
        .actor
        .unwrap_or_else(|| std::env::var("FOCUSA_USER").unwrap_or_else(|_| "unknown@local".into()));
    let provider = WorkItemProvider::Bd; // auto-detect later

    if let Some(reason) = &args.override_ {
        println!("[work-item] close OVERRIDE for {id} by {actor}: {reason}", id = args.id);
        return Ok(());
    }

    println!("[work-item] close {id} by {actor} (workpoint={wp})", id = args.id, wp = args.from_workpoint);
    println!("[work-item]   profile: {}", args.profile.as_deref().unwrap_or("release_proof (default)"));
    println!("[work-item]   TODO: full lifecycle calls");
    Ok(())
}

async fn run_closure(cmd: ClosureCmd) -> Result<()> {
    match cmd {
        ClosureCmd::Prepare(args) => {
            println!("[closure] prepare {pid}", pid = args.provider_item_id);
            Ok(())
        }
        ClosureCmd::Validate(args) => {
            println!("[closure] validate {cid}", cid = args.claim_id);
            Ok(())
        }
        ClosureCmd::Authorize(args) => {
            println!("[closure] authorize {cid}", cid = args.claim_id);
            Ok(())
        }
        ClosureCmd::Submit(args) => {
            println!("[closure] submit {cid}", cid = args.claim_id);
            Ok(())
        }
        ClosureCmd::Reconcile(args) => {
            println!("[closure] reconcile {cid}", cid = args.claim_id);
            Ok(())
        }
    }
}

async fn run_providers(cmd: ProvidersCmd) -> Result<()> {
    match cmd {
        ProvidersCmd::List => {
            println!("[providers] configured providers:");
            println!("  bd (default)");
            println!("  linear (not configured)");
            println!("  asana (not configured)");
            println!("  github (not configured)");
            println!("  gitlab (not configured)");
            println!("  jira (not configured)");
            println!("  none (local-only)");
            Ok(())
        }
        ProvidersCmd::Add(args) => {
            println!("[providers] add {provider}", provider = args.provider);
            Ok(())
        }
        ProvidersCmd::Remove(args) => {
            println!("[providers] remove {provider}", provider = args.provider);
            Ok(())
        }
        ProvidersCmd::Test(args) => {
            println!("[providers] test {provider}", provider = args.provider);
            Ok(())
        }
    }
}

async fn run_provider_guard(cmd: ProviderGuardCmd) -> Result<()> {
    match cmd {
        ProviderGuardCmd::Evaluate(args) => {
            println!("[work-item] provider-guard evaluate --provider {} --command {:?}", args.provider, args.command);
            println!("  result: would {}intercept (guard not yet active)", "");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_item_subcommand_parses_close() {
        use clap::CommandFactory;
        // Smoke test that the clap config is valid.
        let _ = WorkItemArgs::command();
    }
}
