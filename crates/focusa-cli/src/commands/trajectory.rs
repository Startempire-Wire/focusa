//! Spec96 Trajectory Projection CLI parity commands.

use crate::api_client::ApiClient;
use crate::commands::scope::ensure_project_root_scope_safe;
use clap::Subcommand;
use focusa_core::working_subpath::resolve_git_working_context;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum TrajectoryCmd {
    /// Read per-project Trajectory Intelligence view.
    View(ScopeArgs),
    /// Persist/define a trajectory goal candidate.
    DefineGoal {
        #[arg(long)]
        long_term_goal: String,
        #[arg(long)]
        desired_end_state: String,
        #[arg(long)]
        mid_level_goal: Option<String>,
        #[arg(long)]
        short_term_goal: Option<String>,
        #[arg(long = "waypoint")]
        waypoints: Vec<String>,
        #[arg(long)]
        current_state: Option<String>,
        #[arg(long)]
        goal_source: Option<String>,
        #[arg(long)]
        supersedes_trajectory_id: Option<String>,
        #[arg(long)]
        operator_confirmed: bool,
        #[arg(long = "supersession-evidence-ref")]
        supersession_evidence_refs: Vec<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Assess current project state against trajectory desired end state.
    Assess {
        #[arg(long)]
        observed_state: Option<String>,
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
    },
    /// Propose an advisory Workpoint candidate from active trajectory gap.
    ProposeWorkpoint {
        #[arg(long)]
        trajectory_id: Option<String>,
        #[arg(long)]
        target_ref: Option<String>,
        #[arg(long)]
        action_type: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
    },
    /// Persist a trajectory checkpoint packet.
    Checkpoint {
        #[arg(long)]
        summary: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Resume trajectory orientation after compaction/model switch/session resume.
    Resume(ScopeArgs),
}

#[derive(clap::Args, Clone)]
pub struct ScopeArgs {
    #[arg(long)]
    pub project_root: Option<String>,
    #[arg(long)]
    pub session_id: Option<String>,
    #[arg(long)]
    pub continuity_id: Option<String>,
    #[arg(long, default_value = "summary")]
    pub mode: String,
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

fn path_for_view(scope: &ScopeArgs) -> String {
    let mut qs = Vec::new();
    push_query(&mut qs, "project_root", scope.project_root.as_deref());
    push_query(&mut qs, "session_id", scope.session_id.as_deref());
    push_query(&mut qs, "continuity_id", scope.continuity_id.as_deref());
    push_query(&mut qs, "mode", Some(scope.mode.as_str()));
    format!("/v1/trajectory/view?{}", qs.join("&"))
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
    let trajectory = resp
        .get("trajectory")
        .or_else(|| resp.pointer("/resume_packet/trajectory"))
        .unwrap_or(&null_value);
    let definition = trajectory
        .get("definition_status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let project = resp
        .pointer("/project_identity/status")
        .or_else(|| resp.pointer("/resume_packet/project_identity/status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!(
        "trajectory {label}: status={status} canonical={canonical} project={project} definition={definition}"
    );
    if let Some(next) = resp
        .get("recommended_action")
        .and_then(Value::as_str)
        .or_else(|| {
            resp.pointer("/intelligence_view/context_sufficiency/recommended_action")
                .and_then(Value::as_str)
        })
    {
        println!("  action: {next}");
    }
    if let Some(ladder) = trajectory.get("trajectory_ladder") {
        let hlt = ladder
            .get("hlt")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let mlg = ladder
            .get("mlg")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let stg = ladder
            .get("stg")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("  ladder: HLT={hlt}; MLG={mlg}; STG={stg}");
    }
    if let Some(gap) = trajectory.get("active_gap").and_then(Value::as_str) {
        println!("  gap: {gap}");
    }
}

pub async fn run(mut cmd: TrajectoryCmd, json_output: bool) -> anyhow::Result<()> {
    fn canonicalize(root: &mut Option<String>) {
        let Some(input) = root.clone() else { return };
        if let Ok(Some(context)) = resolve_git_working_context(std::path::Path::new(&input)) {
            *root = Some(context.canonical_parent_root);
        }
    }
    match &mut cmd {
        TrajectoryCmd::View(scope) | TrajectoryCmd::Resume(scope) => {
            canonicalize(&mut scope.project_root)
        }
        TrajectoryCmd::DefineGoal { project_root, .. }
        | TrajectoryCmd::Assess { project_root, .. }
        | TrajectoryCmd::ProposeWorkpoint { project_root, .. }
        | TrajectoryCmd::Checkpoint { project_root, .. } => canonicalize(project_root),
    }
    let api = ApiClient::new();
    let (label, resp) = match cmd {
        TrajectoryCmd::View(scope) => {
            ensure_project_root_scope_safe(
                scope.project_root.as_deref(),
                "trajectory view: project_root",
            )?;
            ("view", api.get(&path_for_view(&scope)).await?)
        }
        TrajectoryCmd::Resume(scope) => {
            ensure_project_root_scope_safe(
                scope.project_root.as_deref(),
                "trajectory resume: project_root",
            )?;
            (
                "resume",
                api.post(
                    "/v1/trajectory/resume",
                    &json!({
                        "project_root": scope.project_root,
                        "session_id": scope.session_id,
                        "continuity_id": scope.continuity_id,
                        "mode": scope.mode,
                    }),
                )
                .await?,
            )
        }
        TrajectoryCmd::DefineGoal {
            long_term_goal,
            desired_end_state,
            mid_level_goal,
            short_term_goal,
            waypoints,
            current_state,
            goal_source,
            supersedes_trajectory_id,
            operator_confirmed,
            supersession_evidence_refs,
            project_root,
            session_id,
            continuity_id,
            idempotency_key,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "trajectory define-goal: project_root",
            )?;
            (
                "define-goal",
                api.post(
                    "/v1/trajectory/define-goal",
                    &json!({
                        "long_term_goal": long_term_goal,
                        "desired_end_state": desired_end_state,
                        "mid_level_goal": mid_level_goal,
                        "short_term_goal": short_term_goal,
                        "waypoints": waypoints,
                        "current_state": current_state,
                        "goal_source": goal_source,
                        "supersedes_trajectory_id": supersedes_trajectory_id,
                        "operator_confirmed": operator_confirmed,
                        "supersession_evidence_refs": supersession_evidence_refs,
                        "project_root": project_root,
                        "session_id": session_id,
                        "continuity_id": continuity_id,
                        "idempotency_key": idempotency_key,
                    }),
                )
                .await?,
            )
        }
        TrajectoryCmd::Assess {
            observed_state,
            evidence_refs,
            project_root,
            session_id,
            continuity_id,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "trajectory assess: project_root",
            )?;
            (
                "assess",
                api.post(
                    "/v1/trajectory/assess",
                    &json!({
                        "observed_state": observed_state,
                        "evidence_refs": evidence_refs,
                        "project_root": project_root,
                        "session_id": session_id,
                        "continuity_id": continuity_id,
                    }),
                )
                .await?,
            )
        }
        TrajectoryCmd::ProposeWorkpoint {
            trajectory_id,
            target_ref,
            action_type,
            project_root,
            session_id,
            continuity_id,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "trajectory propose-workpoint: project_root",
            )?;
            (
                "propose-workpoint",
                api.post(
                    "/v1/trajectory/propose-workpoint",
                    &json!({
                        "trajectory_id": trajectory_id,
                        "target_ref": target_ref,
                        "action_type": action_type,
                        "project_root": project_root,
                        "session_id": session_id,
                        "continuity_id": continuity_id,
                    }),
                )
                .await?,
            )
        }
        TrajectoryCmd::Checkpoint {
            summary,
            project_root,
            session_id,
            continuity_id,
            idempotency_key,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "trajectory checkpoint: project_root",
            )?;
            (
                "checkpoint",
                api.post(
                    "/v1/trajectory/checkpoint",
                    &json!({
                        "summary": summary,
                        "project_root": project_root,
                        "session_id": session_id,
                        "continuity_id": continuity_id,
                        "idempotency_key": idempotency_key,
                    }),
                )
                .await?,
            )
        }
    };
    if json_output {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        print_summary(label, &resp);
    }
    Ok(())
}
