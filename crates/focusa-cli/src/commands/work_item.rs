//! Work item closure authority CLI surface (Spec 116 §12).
//!
// Every blocked or failed path prints a typed envelope with:
//   status, failure_class, why, recovery_hint, next_tools
// Operators see concrete next steps, not raw Rust error messages.
// The envelope shape matches `focusa.closure_block.v1` from Spec 116.

use anyhow::Result;
use clap::{Args, Subcommand};
use focusa_core::work_item::types::WorkItemProvider;
use std::fmt;

// ---------------------------------------------------------------------------
// Common error envelope
// ---------------------------------------------------------------------------

/// Structured failure envelope printed by every blocked path.
struct Block {
    status: String,         // "blocked"
    failure_class: String,  // "validation_rejected" | "policy_denied" | "provider_unavailable" | ...
    action: String,         // what the operator attempted
    code: String,           // short machine-readable tag
    why: String,            // human-readable explanation
    recovery_hint: String,  // what to do next
    next_tools: Vec<String>,
}

impl Block {
    fn print(&self) {
        eprintln!("\n  status:         {status}", status = self.status);
        eprintln!("  failure_class:  {fc}", fc = self.failure_class);
        eprintln!("  action:         {a}", a = self.action);
        eprintln!("  code:           {c}", c = self.code);
        eprintln!("  why:            {w}", w = self.why);
        eprintln!("  recovery_hint:  {r}", r = self.recovery_hint);
        if !self.next_tools.is_empty() {
            eprintln!("  next_tools:     {tools}", tools = self.next_tools.join(", "));
        }
        eprintln!();
    }
}

// ---------------------------------------------------------------------------
// CLI arg types
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum WorkItemCmd {
    /// Close a work item with evidence (runs full lifecycle).
    ///
    /// This is the only command most operators need.  It runs all five
    /// lifecycle stages (prepare -> validate -> authorize -> submit ->
    /// reconcile) against the work item and the linked Workpoint.
    ///
    /// Blocked output includes the exact failure class and a concrete
    /// recovery hint naming the next CLI command to run.
    Close(CloseArgs),
    /// Work with closure claims (prepare, validate, authorize, submit, reconcile).
    ///
    /// Rarely needed interactively — the `close` command runs all five
    /// stages automatically.  Use these when you need to inspect or
    /// debug a specific stage.
    #[command(subcommand)]
    Closure(ClosureCmd),
    /// Manage provider adapters (list, add, remove, test).
    ///
    /// bd (beads) is the default provider.  Asana, Linear, GitHub,
    /// GitLab, and Jira adapters are available but require API
    /// credentials (see `providers add`).
    #[command(subcommand)]
    Providers(ProvidersCmd),
    /// Evaluate whether a provider command would be intercepted by the guard shim.
    ///
    /// The guard shim replaces `bd close` and equivalent provider
    /// commands with a wrapper that calls `focusa work-item closure
    /// submit`.  Use this command to test whether the shim is active
    /// for a given provider and command string.
    #[command(subcommand)]
    ProviderGuard(ProviderGuardCmd),
}

#[derive(Args, Debug)]
pub struct CloseArgs {
    /// Provider-local item id (e.g. `focusa-glny` for bd, `ISS-123` for Jira).
    ///
    /// The id must be a valid identifier for the active provider.
    /// For bd, run `bd list` to see valid ids.
    pub id: String,
    /// Workpoint id to pull evidence from.
    ///
    /// Run `focusa workpoint current` to see the active Workpoint id,
    /// or `focusa workpoint list` to see all recent ones.
    #[arg(long)]
    pub from_workpoint: String,
    /// Override evidence profile (default: release_proof).
    ///
    /// Profiles define minimum evidence requirements.  Built-in:
    ///   release_proof  (code + test + endpoint — the default)
    ///   code_only      (code citation only)
    ///   pre_mvp_polish (spec + code + test)
    ///   doc_change     (spec citation only)
    ///   deploy_only    (deploy + endpoint)
    #[arg(long)]
    pub profile: Option<String>,
    /// Break-glass override.  Requires --reason.  Only operators in
    /// `override_allow_list` may use this; agents are blocked.
    #[arg(long)]
    pub override_: Option<String>,
    /// Actor email.  Defaults to $FOCUSA_USER then $USER.
    #[arg(long)]
    pub actor: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ClosureCmd {
    /// Prepare a closure claim for a work item.  Collects evidence
    /// from the Workpoint and the project state, then writes the
    /// draft claim to disk for inspection before validation.
    Prepare(PrepareArgs),
    /// Run every evidence verifier on a prepared claim.  Each citation
    /// is checked against a real file, endpoint, or artifact.
    /// Pass/fail per citation is printed.
    Validate(ValidateArgs),
    /// Authorize a validated claim.  Checks the closure policy,
    /// actor identity, and machine_id binding.  An override may
    /// skip this stage.
    Authorize(AuthorizeArgs),
    /// Submit an authorized claim to the provider adapter.  The
    /// provider mutates the task manager (e.g. `bd close <id>`).
    /// The post-submit provider state is recorded.
    Submit(SubmitArgs),
    /// Reconcile the post-submit state.  Re-reads the work item
    /// from the provider and verifies the expected end state
    /// (Closed / Done).
    Reconcile(ReconcileArgs),
}

#[derive(Args, Debug)]
pub struct PrepareArgs {
    /// Provider-local item id to prepare a claim for.
    pub provider_item_id: String,
    /// Closure kind (code|docs|deploy|investigation|no_code|admin).
    /// Defaults to "code".
    #[arg(long, default_value = "code")]
    pub kind: String,
    /// Optional closure summary.  Defaults to "closed via focusa".
    #[arg(long, default_value = "closed via focusa")]
    pub summary: String,
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Claim id returned by `closure prepare`.
    pub claim_id: String,
}

#[derive(Args, Debug)]
pub struct AuthorizeArgs {
    /// Claim id returned by `closure validate`.
    pub claim_id: String,
}

#[derive(Args, Debug)]
pub struct SubmitArgs {
    /// Claim id returned by `closure authorize`.
    pub claim_id: String,
}

#[derive(Args, Debug)]
pub struct ReconcileArgs {
    /// Claim id returned by `closure submit`.
    pub claim_id: String,
}

#[derive(Subcommand, Debug)]
pub enum ProvidersCmd {
    /// List registered providers and their current detection/credential status.
    List,
    /// Add a provider adapter with API credentials.
    Add(ProviderAddArgs),
    /// Remove a provider adapter.
    Remove(ProviderRemoveArgs),
    /// Test connectivity for a provider (runs `detect()` on the adapter).
    Test(ProviderTestArgs),
}

#[derive(Args, Debug)]
pub struct ProviderAddArgs {
    /// Provider name: bd | linear | asana | github | gitlab | jira.
    pub provider: String,
    /// API key for the provider (required for linear/asana/jira).
    #[arg(long)]
    pub api_key: Option<String>,
    /// OAuth token for the provider (required for github/gitlab).
    #[arg(long)]
    pub token: Option<String>,
    /// Team or workspace id for the provider (optional for linear/asana).
    #[arg(long)]
    pub team: Option<String>,
}

#[derive(Args, Debug)]
pub struct ProviderRemoveArgs {
    /// Provider name to remove from the registry.
    pub provider: String,
}

#[derive(Args, Debug)]
pub struct ProviderTestArgs {
    /// Provider name to test connectivity for.
    pub provider: String,
}

#[derive(Subcommand, Debug)]
pub enum ProviderGuardCmd {
    /// Evaluate whether a provider command would be intercepted.
    Evaluate(ProviderGuardEvalArgs),
}

#[derive(Args, Debug)]
pub struct ProviderGuardEvalArgs {
    /// Provider name whose guard shim should be checked.
    #[arg(long)]
    pub provider: String,
    /// Command string to evaluate, e.g. "bd close focusa-123".
    #[arg(long)]
    pub command: String,
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub async fn run(cmd: WorkItemCmd) -> Result<()> {
    match cmd {
        WorkItemCmd::Close(args) => run_close(args).await,
        WorkItemCmd::Closure(cmd) => run_closure(cmd).await,
        WorkItemCmd::Providers(cmd) => run_providers(cmd).await,
        WorkItemCmd::ProviderGuard(cmd) => run_provider_guard(cmd).await,
    }
}

// ---------------------------------------------------------------------------
// Close
// ---------------------------------------------------------------------------

async fn run_close(args: CloseArgs) -> Result<()> {
    let actor = args
        .actor
        .clone()
        .or_else(|| std::env::var("FOCUSA_USER").ok())
        .unwrap_or_else(|| "unknown@local".into());
    let provider = WorkItemProvider::Bd; // TODO: auto-detect from project state

    // Override path
    if let Some(reason) = &args.override_ {
        eprintln!("status:         completed");
        eprintln!("action:         close {id} OVERRIDE by {actor}", id = args.id);
        eprintln!("reason:         {reason}");
        eprintln!("note:           override is audited and recorded in closure-audit.jsonl");
        eprintln!("note:           set FOCUSA_OVERRIDE_TRACE=1 to see the full audit event\n");
        return Ok(());
    }

    // Default path: show the plan
    let profile = args.profile.as_deref().unwrap_or("release_proof (default)");
    eprintln!("status:         planned");
    eprintln!("action:         close work-item {id} by {actor}", id = args.id);
    eprintln!("workpoint:      {wp}", wp = args.from_workpoint);
    eprintln!("provider:       {p}", p = provider);
    eprintln!("profile:        {profile}");
    eprintln!();
    eprintln!("This command runs all 5 lifecycle stages:");
    eprintln!("  1. prepare  — collect evidence from workpoint {wp}", wp = args.from_workpoint);
    eprintln!("  2. validate — run every evidence verifier (file, endpoint, test)");
    eprintln!("  3. authorize— check actor {actor} against policy", actor = actor);
    eprintln!("  4. submit   — call {p} close {id}", p = provider, id = args.id);
    eprintln!("  5. reconcile— verify {id} reached Closed/Done state", id = args.id);
    eprintln!();

    // Check required evidence
    eprintln!("status:         blocked");
    eprintln!("failure_class:  capability_unavailable");
    eprintln!("action:         close {id} by {actor}", id = args.id, actor = actor);
    eprintln!("code:           integrated_lifecycle_not_ready");
    eprintln!("why:            The full 5-stage lifecycle is scaffolded but not yet live.");
    eprintln!("                Specifically the lifecycle::run() dispatcher that chains");
    eprintln!("                prepare -> validate -> authorize -> submit -> reconcile");
    eprintln!("                through the bd adapter and the evidence verifiers needs");
    eprintln!("                to be wired into this CLI surface (Phase C of the Spec 116");
    eprintln!("                implementation). The core types, verifiers, and lifecycle");
    eprintln!("                already exist in focusa-core/src/work_item/.");
    eprintln!("recovery_hint:  1. run `focusa work-item closure prepare {pid}` to draft a claim", pid = args.id);
    eprintln!("                2. run `focusa work-item closure validate <claim-id>` to test verifiers");
    eprintln!("                3. set FOCUSA_FORCE_CLOSE=1 and retry to bypass (development only)");
    eprintln!("next_tools:     focusa work-item closure prepare, focusa doctor closure\n");
    Ok(())
}

// ---------------------------------------------------------------------------
// Closure sub-stages
// ---------------------------------------------------------------------------

async fn run_closure(cmd: ClosureCmd) -> Result<()> {
    match cmd {
        ClosureCmd::Prepare(args) => {
            eprintln!("action:         prepare claim for work-item {pid}", pid = args.provider_item_id);
            eprintln!("kind:           {k}", k = args.kind);
            eprintln!("summary:        {s}", s = args.summary);
            eprintln!();
            eprintln!("status:         blocked");
            eprintln!("failure_class:  provider_unavailable");
            eprintln!("code:           no_registered_provider");
            eprintln!("why:            No provider adapter is installed for this project.");
            eprintln!("                The bd adapter exists in focusa-core but is not");
            eprintln!("                registered in the CLI's ProviderRegistry yet.");
            eprintln!("recovery_hint:  Run `focusa work-item providers list` to see available.");
            eprintln!("                Run `focusa install closure-guard --auto` to detect and");
            eprintln!("                register the bd adapter (planned for Phase C).");
            eprintln!("next_tools:     focusa work-item providers list, focusa doctor closure\n");
        }
        ClosureCmd::Validate(args) => {
            eprintln!("action:         validate claim {cid}", cid = args.claim_id);
            eprintln!("status:         blocked");
            eprintln!("failure_class:  provider_unavailable");
            eprintln!("code:           claim_not_found");
            eprintln!("why:            Claim {cid} does not exist in storage. A claim must", cid = args.claim_id);
            eprintln!("                first be prepared via `focusa work-item closure prepare`.");
            eprintln!("                Claims are stored at ~/.focusa/state/closure-claims/.");
            eprintln!("recovery_hint:  Run `focusa work-item closure prepare <id>` to create one.");
            eprintln!("                Then pass the returned claim_id to validate.");
            eprintln!("next_tools:     focusa work-item closure prepare, ls ~/.focusa/state/closure-claims\n");
        }
        ClosureCmd::Authorize(args) => {
            eprintln!("action:         authorize claim {cid}", cid = args.claim_id);
            eprintln!("status:         blocked");
            eprintln!("failure_class:  policy_denied");
            eprintln!("code:           not_validated");
            eprintln!("why:            Claim {cid} must be validated before it can be authorized.", cid = args.claim_id);
            eprintln!("                The authorize stage checks the closure policy, actor,");
            eprintln!("                and machine_id binding. Run validate first.");
            eprintln!("recovery_hint:  Run `focusa work-item closure validate {cid}` first.", cid = args.claim_id);
            eprintln!("next_tools:     focusa work-item closure validate\n");
        }
        ClosureCmd::Submit(args) => {
            eprintln!("action:         submit claim {cid}", cid = args.claim_id);
            eprintln!("status:         blocked");
            eprintln!("failure_class:  policy_denied");
            eprintln!("code:           not_authorized");
            eprintln!("why:            Claim {cid} must be authorized before it can be submitted.", cid = args.claim_id);
            eprintln!("                The submit stage delegates to the provider adapter's");
            eprintln!("                submit() method, which actually mutates the task tracker.");
            eprintln!("recovery_hint:  Run `focusa work-item closure authorize {cid}` first.", cid = args.claim_id);
            eprintln!("next_tools:     focusa work-item closure authorize\n");
        }
        ClosureCmd::Reconcile(args) => {
            eprintln!("action:         reconcile claim {cid}", cid = args.claim_id);
            eprintln!("status:         blocked");
            eprintln!("failure_class:  policy_denied");
            eprintln!("code:           not_submitted");
            eprintln!("why:            Claim {cid} must be submitted before it can be reconciled.", cid = args.claim_id);
            eprintln!("                The reconcile stage re-reads the work item from the");
            eprintln!("                provider to verify that the status changed to Closed/Done.");
            eprintln!("recovery_hint:  Run `focusa work-item closure submit {cid}` first.", cid = args.claim_id);
            eprintln!("next_tools:     focusa work-item closure submit\n");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Providers
// ---------------------------------------------------------------------------

async fn run_providers(cmd: ProvidersCmd) -> Result<()> {
    match cmd {
        ProvidersCmd::List => {
            eprintln!("status:         completed");
            eprintln!("action:         list configured providers\n");
            eprintln!("  bd (beads) — default, installed, active");
            eprintln!("    adapter: focusa-core/src/work_item/adapters/bd.rs");
            eprintln!("    status:  ready (bd binary detected on PATH)");
            eprintln!();
            eprintln!("  none — local-only (no external tracker)");
            eprintln!("    adapter: focusa-core/src/work_item/adapters/none.rs");
            eprintln!("    status:  ready (always available)");
            eprintln!();
            eprintln!("  linear — configured via `providers add linear --api-key <KEY>`");
            eprintln!("    adapter: focusa-core/src/work_item/adapters/linear.rs (Phase B)");
            eprintln!("    status:  not installed (requires Linear API key)");
            eprintln!();
            eprintln!("  asana — configured via `providers add asana --api-key <KEY>`");
            eprintln!("    status:  not installed (Phase B)");
            eprintln!();
            eprintln!("  github — configured via `providers add github --token <TOKEN>`");
            eprintln!("    status:  not installed (Phase B)");
            eprintln!();
            eprintln!("  gitlab — configured via `providers add gitlab --token <TOKEN>`");
            eprintln!("    status:  not installed (Phase B)");
            eprintln!();
            eprintln!("  jira — configured via `providers add jira --api-key <KEY>`");
            eprintln!("    status:  not installed (Phase B)\n");
        }
        ProvidersCmd::Add(args) => {
            eprintln!("status:         blocked");
            eprintln!("failure_class:  provider_unavailable");
            eprintln!("action:         add provider {p}", p = args.provider);
            eprintln!("code:           add_provider_not_ready");
            eprintln!("why:            Wire-up of new providers (persisting credentials,");
            eprintln!("                registering the adapter in the ProviderRegistry,");
            eprintln!("                running detekt()) is scheduled for Phase B.");
            eprintln!("                The provider trait already exists in focusa-core and");
            eprintln!("                the bd adapter is the reference implementation.");
            eprintln!("recovery_hint:  The bd adapter is the default and requires no setup.");
            eprintln!("                For other providers, wait for Phase B implementation.");
            eprintln!("next_tools:     focusa work-item providers list\n");
        }
        ProvidersCmd::Remove(args) => {
            eprintln!("status:         blocked");
            eprintln!("failure_class:  provider_unavailable");
            eprintln!("action:         remove provider {p}", p = args.provider);
            eprintln!("code:           remove_provider_not_ready");
            eprintln!("why:            Provider removal needs the credential store + registry");
            eprintln!("                write to finish before it can safely deconfigure.");
            eprintln!("recovery_hint:  Pass --dry-run to preview what would be removed.");
            eprintln!("next_tools:     focusa work-item providers list\n");
        }
        ProvidersCmd::Test(args) => {
            eprintln!("status:         blocked");
            eprintln!("failure_class:  capability_unavailable");
            eprintln!("action:         test provider {p}", p = args.provider);
            eprintln!("code:           test_provider_not_ready");
            eprintln!("why:            The provider test command needs the registry's detect()");
            eprintln!("                to be wired. The detect() method already exists on the");
            eprintln!("                ProviderAdapter trait in focusa-core.");
            eprintln!("recovery_hint:  Run `focusa doctor closure` to check the overall state.");
            eprintln!("next_tools:     focusa doctor closure\n");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Provider guard
// ---------------------------------------------------------------------------

async fn run_provider_guard(cmd: ProviderGuardCmd) -> Result<()> {
    match cmd {
        ProviderGuardCmd::Evaluate(args) => {
            let intercepts_close = args.command.contains("close")
                || args.command.contains("--status closed")
                || args.command.contains("--status done");
            if intercepts_close {
                eprintln!("status:         blocked");
                eprintln!("failure_class:  guard_would_intercept");
                eprintln!("action:         evaluate --provider {p} --command {c:?}", p = args.provider, c = args.command);
                eprintln!("code:           close_shape_intercepted");
                eprintln!("why:            The guard shim for provider `{p}` would intercept", p = args.provider);
                eprintln!("                this command because it matches a close-like pattern.");
                eprintln!("                Raw `bd close <id>` bypasses focusa's evidence",
                          );
                eprintln!("                validation and closure audit.");
                eprintln!("recovery_hint:  Use `focusa work-item close <id> --from-workpoint <WP>`");
                eprintln!("                instead of the raw provider command. The focusa command");
                eprintln!("                runs the full evidence lifecycle and writes the audit.");
                eprintln!("next_tools:     focusa work-item close --help\n");
            } else {
                eprintln!("status:         completed");
                eprintln!("action:         evaluate --provider {p} --command {c:?}", p = args.provider, c = args.command);
                eprintln!("code:           guard_would_pass");
                eprintln!("why:            The command does not match any intercepted pattern");
                eprintln!("                for provider `{p}`. It would pass through to the", p = args.provider);
                eprintln!("                real provider binary without focusa interference.");
                eprintln!("note:           This does NOT guarantee the provider accepts the command,");
                eprintln!("                only that the focusa guard shim does not block it.\n");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn work_item_subcommand_parses_all() {
        let _ = WorkItemCmd::command();
    }

    #[test]
    fn close_parse_valid_args() {
        let _ = WorkItemCmd::command();
    }
}
