//! Spec96 ProjectIdentity CLI parity commands.

use crate::api_client::ApiClient;
use crate::commands::{scope::ensure_project_root_scope_safe, scope_resolver};
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ProjectCmd {
    /// Read hot-path ProjectIdentity for cwd/project_root.
    Identity {
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        remote_host: Option<String>,
        #[arg(long)]
        remote_user: Option<String>,
        #[arg(long)]
        remote_port: Option<u16>,
        #[arg(long)]
        remote_repo_remote: Option<String>,
        #[arg(long)]
        remote_workspace_kind: Option<String>,
        #[arg(long)]
        remote_deploy_root: Option<String>,
        #[arg(long)]
        persisted_project_root: Option<String>,
        #[arg(long)]
        persisted_project_fingerprint: Option<String>,
        #[arg(long)]
        persisted_project_id: Option<String>,
        #[arg(long)]
        persisted_canonical_name: Option<String>,
    },
    /// Build advisory Project Card from identity, ontology, trajectory, prediction, evidence, and learning-loop signals.
    Card {
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        current_ask: Option<String>,
        #[arg(long)]
        remote_host: Option<String>,
        #[arg(long)]
        remote_user: Option<String>,
        #[arg(long)]
        remote_port: Option<u16>,
        #[arg(long)]
        remote_repo_remote: Option<String>,
        #[arg(long)]
        remote_workspace_kind: Option<String>,
        #[arg(long)]
        remote_deploy_root: Option<String>,
    },
    /// Attach final outcome/evaluation to a project-card algorithm_run_id.
    CardOutcome {
        #[arg(long)]
        algorithm_run_id: String,
        #[arg(long)]
        actual_outcome: String,
        #[arg(long)]
        score: Option<f64>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Project dashboard: saved profile + observed runtime state.
    List {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        from: Option<String>,
    },
    /// Discover nearby candidate projects.
    Discover {
        #[arg(long)]
        from: Option<String>,
        #[arg(long, default_value_t = 3)]
        max_depth: u32,
        #[arg(long, default_value_t = 40)]
        max_results: usize,
        #[arg(long)]
        include_git_only: Option<bool>,
    },
    /// Use/save a project as the selected convenience profile (non-authoritative).
    Use {
        project_root: String,
        #[arg(long)]
        selected_by: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Alias for use.
    Bind {
        project_root: String,
        #[arg(long)]
        selected_by: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Alias for use.
    Switch {
        project_root: String,
        #[arg(long)]
        selected_by: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Show current project dashboard for selected/runtime observed scope.
    Current {
        #[arg(long)]
        project_root: Option<String>,
    },
    /// Show current project dashboard for selected/runtime observed scope.
    Status {
        #[arg(long)]
        project_root: Option<String>,
    },
    /// Remove selected-project convenience profile (non-authoritative).
    Remove,
    /// Create a new project from scratch.
    New {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        working_dir: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        canonical_name: Option<String>,
        #[arg(long)]
        template: Option<String>,
        #[arg(long)]
        workspace_kind: Option<String>,
        #[arg(long)]
        git: bool,
        #[arg(long)]
        use_selected: bool,
        #[arg(long)]
        force: bool,
    },
    /// Preview, apply, inspect, or repair the project discipline baseline.
    Bootstrap {
        #[command(subcommand)]
        cmd: ProjectBootstrapCmd,
    },
    /// Run or inspect the atomic Project Genesis journey.
    Genesis {
        #[command(subcommand)]
        cmd: ProjectGenesisCmd,
    },
    /// Project template list/show.
    Templates {
        #[command(subcommand)]
        cmd: ProjectTemplateCmd,
    },
    /// Per-project settings get/list/set/unset.
    Settings {
        #[command(subcommand)]
        cmd: ProjectSettingsCmd,
    },
    /// Save or continue a Focusa session-transfer packet.
    SessionTransfer {
        #[arg(long, default_value = "status")]
        action: String,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        current_ask: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        mission: Option<String>,
        #[arg(long)]
        next_action: Option<String>,
    },
    /// Verify, migrate, or repair the project Trajectory marker guard.
    TrajectoryGuard {
        #[arg(long, default_value = "verify")]
        action: String,
        #[arg(long)]
        project_root: String,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        expected_trajectory_id: Option<String>,
        #[arg(long)]
        expected_hlt_version: Option<u64>,
        #[arg(long)]
        confirm: bool,
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Verify expected project identity signals against discovered ProjectIdentity.
    Verify {
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        canonical_name: Option<String>,
        #[arg(long)]
        repo_remote: Option<String>,
        #[arg(long)]
        remote_host: Option<String>,
        #[arg(long)]
        remote_user: Option<String>,
        #[arg(long)]
        remote_port: Option<u16>,
        #[arg(long)]
        remote_repo_remote: Option<String>,
        #[arg(long)]
        remote_workspace_kind: Option<String>,
        #[arg(long)]
        remote_deploy_root: Option<String>,
        #[arg(long)]
        persisted_project_root: Option<String>,
        #[arg(long)]
        persisted_project_fingerprint: Option<String>,
        #[arg(long)]
        persisted_project_id: Option<String>,
        #[arg(long)]
        persisted_canonical_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ProjectBootstrapCmd {
    Preview {
        #[command(flatten)]
        args: ProjectBootstrapMutationArgs,
    },
    Apply {
        #[command(flatten)]
        args: ProjectBootstrapMutationArgs,
    },
    Status {
        #[arg(long)]
        project_root: String,
    },
    Repair {
        #[command(flatten)]
        args: ProjectBootstrapMutationArgs,
    },
}

#[derive(Args, Clone)]
pub struct ProjectBootstrapMutationArgs {
    #[arg(long)]
    project_root: String,
    #[arg(long)]
    project_id: String,
    #[arg(long)]
    canonical_name: String,
    #[arg(long)]
    continuity_id: String,
    #[arg(long)]
    idempotency_key: String,
    #[arg(long, default_value = "standard_software_project")]
    discipline_profile: String,
    #[arg(long)]
    initialize_git: Option<bool>,
    #[arg(long)]
    initialize_task_provider: Option<bool>,
    #[arg(long)]
    task_provider: Option<String>,
    #[arg(long)]
    hlt: Option<String>,
    #[arg(long)]
    hlt_confirmed: bool,
    #[arg(long)]
    desired_end_state: Option<String>,
    #[arg(long)]
    current_state: Option<String>,
    #[arg(long)]
    specification_ref: Option<String>,
    #[arg(long = "acceptance")]
    acceptance_criteria: Vec<String>,
    #[arg(long)]
    confirm: bool,
    #[arg(long)]
    repair_action: Option<String>,
}

fn bootstrap_body(args: ProjectBootstrapMutationArgs) -> Value {
    json!({
        "project_root": args.project_root,
        "project_id": args.project_id,
        "canonical_name": args.canonical_name,
        "continuity_id": args.continuity_id,
        "idempotency_key": args.idempotency_key,
        "discipline_profile": args.discipline_profile,
        "initialize_git": args.initialize_git,
        "initialize_task_provider": args.initialize_task_provider,
        "task_provider": args.task_provider,
        "hlt": args.hlt,
        "hlt_confirmed": args.hlt_confirmed,
        "desired_end_state": args.desired_end_state,
        "current_state": args.current_state,
        "specification_ref": args.specification_ref,
        "acceptance_criteria": args.acceptance_criteria,
        "confirm": args.confirm,
        "repair_action": args.repair_action,
    })
}

#[derive(Subcommand)]
pub enum ProjectGenesisCmd {
    /// Inventory project authority and stage Genesis; enters HLT Impasse when required.
    Start {
        #[command(flatten)]
        args: ProjectGenesisMutationArgs,
    },
    /// Resume an interrupted/idempotent Genesis transaction.
    Resume {
        #[command(flatten)]
        args: ProjectGenesisMutationArgs,
    },
    /// Read the durable Genesis packet.
    Status {
        #[arg(long)]
        project_root: String,
    },
    /// Atomically commit Trajectory, first Workpoint, coordination, and readiness.
    Commit {
        #[command(flatten)]
        args: ProjectGenesisMutationArgs,
    },
}

#[derive(Args, Clone)]
pub struct ProjectGenesisMutationArgs {
    #[arg(long)]
    project_root: String,
    #[arg(long)]
    continuity_id: String,
    #[arg(long)]
    idempotency_key: String,
    #[arg(long)]
    hlt: Option<String>,
    #[arg(long)]
    hlt_confirmed: bool,
    #[arg(long)]
    desired_end_state: Option<String>,
    #[arg(long)]
    current_state: Option<String>,
    #[arg(long)]
    specification_ref: Option<String>,
    #[arg(long = "acceptance")]
    acceptance_criteria: Vec<String>,
    #[arg(long)]
    mid_level_goal: Option<String>,
    #[arg(long)]
    short_term_goal: Option<String>,
    #[arg(long = "waypoint")]
    waypoints: Vec<String>,
    #[arg(long)]
    task_provider: Option<String>,
    #[arg(long)]
    allow_task_decomposition: bool,
    #[arg(long)]
    confirm: bool,
    #[arg(long)]
    takeover: bool,
}

fn genesis_body(args: ProjectGenesisMutationArgs) -> Value {
    json!({
        "project_root": args.project_root,
        "continuity_id": args.continuity_id,
        "idempotency_key": args.idempotency_key,
        "hlt": args.hlt,
        "hlt_confirmed": args.hlt_confirmed,
        "desired_end_state": args.desired_end_state,
        "current_state": args.current_state,
        "specification_ref": args.specification_ref,
        "acceptance_criteria": args.acceptance_criteria,
        "mid_level_goal": args.mid_level_goal,
        "short_term_goal": args.short_term_goal,
        "waypoints": args.waypoints,
        "task_provider": args.task_provider,
        "allow_task_decomposition": args.allow_task_decomposition,
        "confirm": args.confirm,
        "takeover": args.takeover,
    })
}

#[derive(Subcommand)]
pub enum ProjectTemplateCmd {
    /// List available project templates.
    List,
    /// Show one template metadata.
    Show {
        #[arg(long)]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ProjectSettingsCmd {
    /// Show all local settings keys.
    List {
        #[arg(long)]
        project_root: Option<String>,
    },
    /// Show one local setting key.
    Get {
        key: String,
        #[arg(long)]
        project_root: Option<String>,
    },
    /// Set one local setting key.
    Set {
        key: String,
        value: String,
        #[arg(long)]
        project_root: Option<String>,
    },
    /// Unset one local setting key.
    Unset {
        key: String,
        #[arg(long)]
        project_root: Option<String>,
    },
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                vec![b as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

fn push_query(qs: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        qs.push(format!("{key}={}", encode(value)));
    }
}

fn push_query_bool(qs: &mut Vec<String>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        qs.push(format!("{}={value}", key));
    }
}

fn slugify_project_id(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "focusa-project".to_string()
    } else {
        trimmed
    }
}

fn project_root_from_new_args(
    project_root: Option<String>,
    working_dir: Option<String>,
    name: Option<&str>,
) -> anyhow::Result<String> {
    if let Some(project_root) = project_root {
        ensure_project_root_scope_safe(Some(project_root.as_str()), "project new: project_root")?;
        return Ok(project_root);
    }
    let Some(name) = name else {
        anyhow::bail!("project new requires --project-root or --name");
    };
    let base = working_dir.unwrap_or_else(|| ".".to_string());
    let root = PathBuf::from(base).join(slugify_project_id(name));
    let root = root.to_string_lossy().to_string();
    ensure_project_root_scope_safe(Some(root.as_str()), "project new: derived project_root")?;
    Ok(root)
}

fn print_summary(label: &str, resp: &Value) {
    let status = resp
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let canonical = resp
        .get("canonical")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let null_value = Value::Null;
    let project = resp.get("project_identity").unwrap_or(&null_value);
    let root = project
        .get("project_root")
        .and_then(Value::as_str)
        .unwrap_or("unbound");
    let confidence = project
        .get("confidence")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let project_status = project
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!(
        "project {label}: status={status} canonical={canonical} project_status={project_status} confidence={confidence}"
    );
    println!("  project_root: {root}");
    if let Some(binding) = resp.get("binding_decision") {
        let binding_status = binding
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let selected = binding
            .get("selected_project_root")
            .and_then(Value::as_str)
            .unwrap_or("none");
        let ambiguous = binding
            .get("ambiguous")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        println!("  binding: status={binding_status} selected={selected} ambiguous={ambiguous}");
        if let Some(candidates) = resp.get("binding_candidates").and_then(Value::as_array) {
            for candidate in candidates.iter().take(5) {
                let root = candidate
                    .get("project_root")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let score = candidate.get("score").and_then(Value::as_u64).unwrap_or(0);
                let sources = candidate
                    .get("sources")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_default();
                println!("    candidate: score={score} root={root} sources={sources}");
            }
        }
    }
    if let Some(next) = resp
        .get("verification")
        .and_then(|v| v.get("required_recovery"))
        .and_then(Value::as_str)
    {
        println!("  recovery: {next}");
    }
}

fn resolve_input_project_root(
    cwd: Option<&str>,
    project_root: Option<&str>,
) -> anyhow::Result<String> {
    let resolved = scope_resolver::resolve_project_scope(project_root, None, cwd)?;
    ensure_project_root_scope_safe(
        Some(resolved.project_root.as_str()),
        "project resolved project_root",
    )?;
    Ok(resolved.project_root)
}

fn render_response(label: &str, resp: &Value) {
    if label == "identity"
        || label == "card"
        || label == "card-outcome"
        || label == "session-transfer"
        || label == "verify"
    {
        print_summary(label, resp);
        return;
    }
    println!(
        "{label}: {}",
        serde_json::to_string_pretty(resp).unwrap_or_else(|_| "{}".to_string())
    );
}

pub async fn run(cmd: ProjectCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let (label, resp) = match cmd {
        ProjectCmd::Identity {
            cwd,
            project_root,
            remote_host,
            remote_user,
            remote_port,
            remote_repo_remote,
            remote_workspace_kind,
            remote_deploy_root,
            persisted_project_root,
            persisted_project_fingerprint,
            persisted_project_id,
            persisted_canonical_name,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project identity: cwd")?;
            let resolved_project_root = project_root
                .as_deref()
                .map(|root| resolve_input_project_root(cwd.as_deref(), Some(root)))
                .transpose()?;
            ensure_project_root_scope_safe(
                persisted_project_root.as_deref(),
                "project identity: persisted_project_root",
            )?;
            let mut qs = Vec::new();
            push_query(&mut qs, "cwd", cwd.as_deref());
            push_query(&mut qs, "project_root", resolved_project_root.as_deref());
            push_query(&mut qs, "remote_host", remote_host.as_deref());
            push_query(&mut qs, "remote_user", remote_user.as_deref());
            if let Some(port) = remote_port {
                qs.push(format!("remote_port={port}"));
            }
            push_query(&mut qs, "remote_repo_remote", remote_repo_remote.as_deref());
            push_query(
                &mut qs,
                "remote_workspace_kind",
                remote_workspace_kind.as_deref(),
            );
            push_query(&mut qs, "remote_deploy_root", remote_deploy_root.as_deref());
            push_query(
                &mut qs,
                "persisted_project_root",
                persisted_project_root.as_deref(),
            );
            push_query(
                &mut qs,
                "persisted_project_fingerprint",
                persisted_project_fingerprint.as_deref(),
            );
            push_query(
                &mut qs,
                "persisted_project_id",
                persisted_project_id.as_deref(),
            );
            push_query(
                &mut qs,
                "persisted_canonical_name",
                persisted_canonical_name.as_deref(),
            );
            let path = if qs.is_empty() {
                "/v1/project/identity".to_string()
            } else {
                format!("/v1/project/identity?{}", qs.join("&"))
            };
            ("identity", api.get(&path).await?)
        }
        ProjectCmd::Card {
            cwd,
            project_root,
            current_ask,
            remote_host,
            remote_user,
            remote_port,
            remote_repo_remote,
            remote_workspace_kind,
            remote_deploy_root,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project card: cwd")?;
            let resolved_project_root =
                resolve_input_project_root(cwd.as_deref(), project_root.as_deref())?;
            let mut qs = Vec::new();
            push_query(&mut qs, "cwd", cwd.as_deref());
            push_query(
                &mut qs,
                "project_root",
                Some(resolved_project_root.as_str()),
            );
            push_query(&mut qs, "current_ask", current_ask.as_deref());
            push_query(&mut qs, "remote_host", remote_host.as_deref());
            push_query(&mut qs, "remote_user", remote_user.as_deref());
            if let Some(port) = remote_port {
                qs.push(format!("remote_port={port}"));
            }
            push_query(&mut qs, "remote_repo_remote", remote_repo_remote.as_deref());
            push_query(
                &mut qs,
                "remote_workspace_kind",
                remote_workspace_kind.as_deref(),
            );
            push_query(&mut qs, "remote_deploy_root", remote_deploy_root.as_deref());
            let path = if qs.is_empty() {
                "/v1/project/card".to_string()
            } else {
                format!("/v1/project/card?{}", qs.join("&"))
            };
            ("card", api.get(&path).await?)
        }
        ProjectCmd::CardOutcome {
            algorithm_run_id,
            actual_outcome,
            score,
            project_root,
            evidence_refs,
            notes,
        } => {
            let resolved_project_root = resolve_input_project_root(None, project_root.as_deref())?;
            let body = json!({
                "algorithm_run_id": algorithm_run_id,
                "actual_outcome": actual_outcome,
                "score": score,
                "project_root": resolved_project_root,
                "evidence_refs": evidence_refs,
                "notes": notes,
            });
            (
                "card-outcome",
                api.post("/v1/project/card/outcome", &body).await?,
            )
        }
        ProjectCmd::List { project_root, from } => {
            let resolved_project_root = match (project_root, &from) {
                (Some(root), _) => Some(resolve_input_project_root(None, Some(root.as_str()))?),
                (None, Some(from_root)) => {
                    Some(resolve_input_project_root(Some(from_root.as_str()), None)?)
                }
                _ => None,
            };
            let mut qs = Vec::new();
            push_query(&mut qs, "project_root", resolved_project_root.as_deref());
            push_query(&mut qs, "from", from.as_deref());
            let path = if qs.is_empty() {
                "/v1/project/list".to_string()
            } else {
                format!("/v1/project/list?{}", qs.join("&"))
            };
            ("list", api.get(&path).await?)
        }
        ProjectCmd::Discover {
            from,
            max_depth,
            max_results,
            include_git_only,
        } => {
            let mut qs = Vec::new();
            push_query(&mut qs, "from", from.as_deref());
            push_query(&mut qs, "max_depth", Some(&max_depth.to_string()));
            push_query(&mut qs, "max_results", Some(&max_results.to_string()));
            push_query_bool(&mut qs, "include_git_only", include_git_only);
            let path = if qs.is_empty() {
                "/v1/project/discover".to_string()
            } else {
                format!("/v1/project/discover?{}", qs.join("&"))
            };
            ("discover", api.get(&path).await?)
        }
        ProjectCmd::Use {
            project_root,
            selected_by,
            note,
        }
        | ProjectCmd::Bind {
            project_root,
            selected_by,
            note,
        }
        | ProjectCmd::Switch {
            project_root,
            selected_by,
            note,
        } => {
            let resolved =
                scope_resolver::resolve_project_scope(Some(project_root.as_str()), None, None)?;
            ensure_project_root_scope_safe(
                Some(resolved.project_root.as_str()),
                "project resolved project_root",
            )?;
            let body = json!({
                "project_root": resolved.canonical_parent_root,
                "active_worktree_root": resolved.active_worktree_root,
                "working_subpath_id": resolved.working_subpath_id,
                "selected_by": selected_by,
                "note": note,
            });
            ("use", api.post("/v1/project/use", &body).await?)
        }
        ProjectCmd::Current { project_root } => {
            let mut qs = Vec::new();
            if let Some(root) = project_root {
                let resolved = resolve_input_project_root(None, Some(root.as_str()))?;
                push_query(&mut qs, "project_root", Some(resolved.as_str()));
            } else if let Ok(cwd) = std::env::current_dir() {
                push_query(&mut qs, "from", cwd.to_str());
            }
            let path = format!("/v1/project/current?{}", qs.join("&"));
            ("current", api.get(&path).await?)
        }
        ProjectCmd::Status { project_root } => {
            let mut qs = Vec::new();
            if let Some(root) = project_root {
                let resolved = resolve_input_project_root(None, Some(root.as_str()))?;
                push_query(&mut qs, "project_root", Some(resolved.as_str()));
            } else if let Ok(cwd) = std::env::current_dir() {
                push_query(&mut qs, "from", cwd.to_str());
            }
            let path = format!("/v1/project/status?{}", qs.join("&"));
            ("status", api.get(&path).await?)
        }
        ProjectCmd::Remove => (
            "remove",
            api.post("/v1/project/remove", &json!({"clear": true}))
                .await?,
        ),
        ProjectCmd::New {
            project_root,
            working_dir,
            name,
            project_id,
            canonical_name,
            template,
            workspace_kind,
            git,
            use_selected,
            force,
        } => {
            let resolved_project_root =
                project_root_from_new_args(project_root, working_dir, name.as_deref())?;
            let inferred_name = name
                .or_else(|| {
                    PathBuf::from(&resolved_project_root)
                        .file_name()
                        .map(|value| value.to_string_lossy().to_string())
                })
                .unwrap_or_else(|| "Focusa Project".to_string());
            let project_id = project_id.unwrap_or_else(|| slugify_project_id(&inferred_name));
            let canonical_name = canonical_name.unwrap_or(inferred_name);
            let body = json!({
                "project_root": resolved_project_root,
                "project_id": project_id,
                "canonical_name": canonical_name,
                "template": template,
                "workspace_kind": workspace_kind,
                "create_git": git,
                "use_selected": use_selected,
                "force": force,
            });
            ("new", api.post("/v1/project/new", &body).await?)
        }
        ProjectCmd::Bootstrap { cmd } => match cmd {
            ProjectBootstrapCmd::Status { project_root } => {
                ensure_project_root_scope_safe(
                    Some(project_root.as_str()),
                    "project bootstrap status",
                )?;
                let mut query = Vec::new();
                push_query(&mut query, "project_root", Some(project_root.as_str()));
                let path = format!("/v1/project/bootstrap/status?{}", query.join("&"));
                ("bootstrap status", api.get(&path).await?)
            }
            ProjectBootstrapCmd::Preview { args } => {
                ensure_project_root_scope_safe(
                    Some(args.project_root.as_str()),
                    "project bootstrap preview",
                )?;
                (
                    "bootstrap preview",
                    api.post("/v1/project/bootstrap/preview", &bootstrap_body(args))
                        .await?,
                )
            }
            ProjectBootstrapCmd::Apply { args } => {
                ensure_project_root_scope_safe(
                    Some(args.project_root.as_str()),
                    "project bootstrap apply",
                )?;
                (
                    "bootstrap apply",
                    api.post("/v1/project/bootstrap/apply", &bootstrap_body(args))
                        .await?,
                )
            }
            ProjectBootstrapCmd::Repair { args } => {
                ensure_project_root_scope_safe(
                    Some(args.project_root.as_str()),
                    "project bootstrap repair",
                )?;
                (
                    "bootstrap repair",
                    api.post("/v1/project/bootstrap/repair", &bootstrap_body(args))
                        .await?,
                )
            }
        },
        ProjectCmd::Genesis { cmd } => match cmd {
            ProjectGenesisCmd::Status { project_root } => {
                let root = resolve_input_project_root(None, Some(project_root.as_str()))?;
                let mut query = Vec::new();
                push_query(&mut query, "project_root", Some(root.as_str()));
                let path = format!("/v1/project/genesis/status?{}", query.join("&"));
                ("genesis status", api.get(&path).await?)
            }
            ProjectGenesisCmd::Start { mut args } => {
                args.project_root =
                    resolve_input_project_root(None, Some(args.project_root.as_str()))?;
                (
                    "genesis start",
                    api.post("/v1/project/genesis/start", &genesis_body(args))
                        .await?,
                )
            }
            ProjectGenesisCmd::Resume { mut args } => {
                args.project_root =
                    resolve_input_project_root(None, Some(args.project_root.as_str()))?;
                (
                    "genesis resume",
                    api.post("/v1/project/genesis/resume", &genesis_body(args))
                        .await?,
                )
            }
            ProjectGenesisCmd::Commit { mut args } => {
                args.project_root =
                    resolve_input_project_root(None, Some(args.project_root.as_str()))?;
                (
                    "genesis commit",
                    api.post("/v1/project/genesis/commit", &genesis_body(args))
                        .await?,
                )
            }
        },
        ProjectCmd::Templates { cmd } => {
            let (path, body_opt) = match cmd {
                ProjectTemplateCmd::List => ("/v1/project/templates".to_string(), None),
                ProjectTemplateCmd::Show { name } => {
                    let mut qs = Vec::new();
                    push_query(&mut qs, "name", Some(name.as_str()));
                    let path = format!("/v1/project/templates?{}", qs.join("&"));
                    (path, None)
                }
            };
            match body_opt {
                Some(body) => ("templates", api.post(&path, &body).await?),
                None => ("templates", api.get(&path).await?),
            }
        }
        ProjectCmd::Settings { cmd } => match cmd {
            ProjectSettingsCmd::List { project_root } => {
                let resolved = project_root
                    .and_then(|root| resolve_input_project_root(None, Some(root.as_str())).ok());
                let mut qs = Vec::new();
                if let Some(root) = resolved {
                    push_query(&mut qs, "project_root", Some(root.as_str()));
                }
                let path = if qs.is_empty() {
                    "/v1/project/settings".to_string()
                } else {
                    format!("/v1/project/settings?{}", qs.join("&"))
                };
                ("settings", api.get(&path).await?)
            }
            ProjectSettingsCmd::Get { key, project_root } => {
                let resolved = if let Some(root) = project_root {
                    Some(resolve_input_project_root(None, Some(root.as_str()))?)
                } else {
                    None
                };
                let mut qs = Vec::new();
                push_query(&mut qs, "project_root", resolved.as_deref());
                push_query(&mut qs, "key", Some(key.as_str()));
                let path = if qs.is_empty() {
                    "/v1/project/settings".to_string()
                } else {
                    format!("/v1/project/settings?{}", qs.join("&"))
                };
                ("settings", api.get(&path).await?)
            }
            ProjectSettingsCmd::Set {
                key,
                value,
                project_root,
            } => {
                let resolved = if let Some(root) = project_root {
                    Some(resolve_input_project_root(None, Some(root.as_str()))?)
                } else {
                    None
                };
                let body =
                    json!({"action": "set", "project_root": resolved, "key": key, "value": value});
                ("settings", api.post("/v1/project/settings", &body).await?)
            }
            ProjectSettingsCmd::Unset { key, project_root } => {
                let resolved = if let Some(root) = project_root {
                    Some(resolve_input_project_root(None, Some(root.as_str()))?)
                } else {
                    None
                };
                let body = json!({"action": "unset", "project_root": resolved, "key": key});
                ("settings", api.post("/v1/project/settings", &body).await?)
            }
        },
        ProjectCmd::SessionTransfer {
            action,
            cwd,
            project_root,
            current_ask,
            continuity_id,
            mission,
            next_action,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project session-transfer: cwd")?;
            let resolved = scope_resolver::resolve_project_scope(
                project_root.as_deref(),
                None,
                cwd.as_deref(),
            )?;
            let body = json!({
                "action": action,
                "cwd": cwd,
                "project_root": resolved.canonical_parent_root,
                "source_working_subpath_id": resolved.working_subpath_id.clone(),
                "target_working_subpath_id": resolved.working_subpath_id,
                "current_ask": current_ask,
                "continuity_id": continuity_id,
                "mission": mission,
                "next_action": next_action,
            });
            (
                "session-transfer",
                api.post("/v1/project/session-transfer", &body).await?,
            )
        }
        ProjectCmd::TrajectoryGuard {
            action,
            project_root,
            continuity_id,
            expected_trajectory_id,
            expected_hlt_version,
            confirm,
            idempotency_key,
        } => {
            let resolved = resolve_input_project_root(None, Some(project_root.as_str()))?;
            ensure_project_root_scope_safe(
                Some(resolved.as_str()),
                "project trajectory-guard: project_root",
            )?;
            let body = json!({
                "action": action,
                "project_root": resolved,
                "continuity_id": continuity_id,
                "expected_trajectory_id": expected_trajectory_id,
                "expected_hlt_version": expected_hlt_version,
                "confirm": confirm,
                "idempotency_key": idempotency_key,
            });
            (
                "trajectory-guard",
                api.post("/v1/project/trajectory-guard", &body).await?,
            )
        }
        ProjectCmd::Verify {
            cwd,
            project_root,
            project_id,
            canonical_name,
            repo_remote,
            remote_host,
            remote_user,
            remote_port,
            remote_repo_remote,
            remote_workspace_kind,
            remote_deploy_root,
            persisted_project_root,
            persisted_project_fingerprint,
            persisted_project_id,
            persisted_canonical_name,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project verify: cwd")?;
            let resolved_project_root = project_root
                .as_deref()
                .map(|root| resolve_input_project_root(cwd.as_deref(), Some(root)))
                .transpose()?;
            ensure_project_root_scope_safe(
                persisted_project_root.as_deref(),
                "project verify: persisted_project_root",
            )?;
            let body = json!({
                "cwd": cwd,
                "project_root": resolved_project_root,
                "project_id": project_id,
                "canonical_name": canonical_name,
                "repo_remote": repo_remote,
                "remote_host": remote_host,
                "remote_user": remote_user,
                "remote_port": remote_port,
                "remote_repo_remote": remote_repo_remote,
                "remote_workspace_kind": remote_workspace_kind,
                "remote_deploy_root": remote_deploy_root,
                "persisted_project_root": persisted_project_root,
                "persisted_project_fingerprint": persisted_project_fingerprint,
                "persisted_project_id": persisted_project_id,
                "persisted_canonical_name": persisted_canonical_name,
            });
            ("verify", api.post("/v1/project/verify", &body).await?)
        }
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        render_response(label, &resp);
    }
    Ok(())
}
