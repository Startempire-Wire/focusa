//! Canonical Spec 137/137A client command families.
//! These commands are thin typed clients: authority and persistence stay in the daemon.
use crate::{api_client::ApiClient, commands::scope::ensure_project_root_scope_safe};
use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Value, json};
#[derive(Args, Clone)]
pub struct ScopeArgs {
    #[arg(long, alias = "project")]
    project_root: String,
    #[arg(long)]
    continuity_id: String,
}
impl ScopeArgs {
    fn query(&self) -> String {
        format!("project_root={}&continuity_id={}", urlencoding::encode(&self.project_root), urlencoding::encode(&self.continuity_id))
    }
    fn body(&self) -> Value {
        json!({"project_root":self.project_root,"continuity_id":self.continuity_id})
    }
    fn validate(&self, label: &str) -> Result<()> {
        ensure_project_root_scope_safe(Some(&self.project_root), label)
    }
}
fn query_arg(path: &str, scope: &ScopeArgs, key: &str, value: &str) -> String {
    format!("{path}?{}&{key}={}", scope.query(), urlencoding::encode(value))
}
fn packet(path: &std::path::Path) -> Result<Value> {
    let value: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    if !value.is_object() {
        bail!("temporal request packet must be a JSON object");
    }
    Ok(value)
}
fn scoped_packet(scope: &ScopeArgs, path: &std::path::Path) -> Result<Value> {
    let mut value = packet(path)?;
    let object = value.as_object_mut().expect("validated object");
    object.insert("project_root".into(), json!(scope.project_root));
    object.insert("continuity_id".into(), json!(scope.continuity_id));
    Ok(value)
}
async fn output(label: &str, response: Value, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("{label}: {}", response.get("status").and_then(Value::as_str).unwrap_or("unknown"));
        if let Some(next) = response.get("next_action").and_then(Value::as_str) {
            println!("  next: {next}");
        }
    }
    Ok(())
}
#[derive(Subcommand)]
pub enum TimeCmd {
    Now,
    Status {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        workpoint: Option<String>,
        #[arg(long)]
        task: Option<String>,
    },
    #[command(subcommand)]
    Trust(TimeTrustCmd),
    #[command(subcommand)]
    Samples(TimeSamplesCmd),
    Capabilities {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        host: String,
    },
    Doctor,
}
#[derive(Subcommand)]
pub enum TimeTrustCmd {
    Inspect {
        #[command(flatten)]
        scope: ScopeArgs,
    },
}
#[derive(Subcommand)]
pub enum TimeSamplesCmd {
    List {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        host: String,
    },
}
pub async fn run_time(cmd: TimeCmd, json_output: bool) -> Result<()> {
    let api = ApiClient::new();
    let response = match cmd {
        TimeCmd::Now => api.get("/v1/time/now").await?,
        TimeCmd::Status { scope, workpoint, task } => {
            scope.validate("time status")?;
            let mut p = format!("/v1/time/status?{}", scope.query());
            if let Some(v) = workpoint {
                p += &format!("&workpoint_id={}", urlencoding::encode(&v));
            }
            if let Some(v) = task {
                p += &format!("&task_id={}", urlencoding::encode(&v));
            }
            api.get(&p).await?
        }
        TimeCmd::Trust(TimeTrustCmd::Inspect { scope }) => {
            scope.validate("time trust inspect")?;
            api.get(&format!("/v1/time/trust?{}", scope.query())).await?
        }
        TimeCmd::Samples(TimeSamplesCmd::List { scope, host }) => {
            scope.validate("time samples list")?;
            api.get(&format!("/v1/time/samples?{}&host_id={}", scope.query(), urlencoding::encode(&host))).await?
        }
        TimeCmd::Capabilities { scope, host } => {
            scope.validate("time capabilities")?;
            api.get(&format!("/v1/time/capabilities?{}&host_id={}", scope.query(), urlencoding::encode(&host))).await?
        }
        TimeCmd::Doctor => api.get("/v1/time/doctor").await?,
    };
    output("time", response, json_output).await
}
#[derive(Subcommand)]
pub enum DeadlineCmd {
    Set {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        at: String,
        #[arg(long)]
        timezone: String,
        #[arg(long)]
        readiness_target: Option<String>,
        #[arg(long)]
        completion_target: String,
        #[arg(long = "evidence")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        idempotency_key: String,
        #[arg(long)]
        confirm: bool,
    },
    SetCivil {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        local: String,
        #[arg(long)]
        timezone: String,
        #[arg(long)]
        fold_policy: String,
        #[arg(long)]
        gap_policy: String,
        #[arg(long)]
        calendar: String,
        #[arg(long)]
        calendar_version: String,
        #[arg(long)]
        tzdb_version: String,
        #[arg(long)]
        completion_target: String,
        #[arg(long = "evidence")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        idempotency_key: String,
        #[arg(long)]
        confirm: bool,
    },
    Inspect {
        deadline_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
    },
    ResolveCivil {
        deadline_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        tzdb_version: String,
    },
    Conflicts {
        #[command(flatten)]
        scope: ScopeArgs,
    },
    Revise {
        deadline_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        expected_revision: u64,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        at: Option<String>,
        #[arg(long)]
        idempotency_key: String,
        #[arg(long)]
        confirm: bool,
    },
    Clear {
        deadline_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        expected_revision: u64,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        idempotency_key: String,
        #[arg(long)]
        confirm: bool,
    },
    List {
        #[command(flatten)]
        scope: ScopeArgs,
    },
}
pub async fn run_deadline(cmd: DeadlineCmd, json_output: bool) -> Result<()> {
    let api = ApiClient::new();
    let response = match cmd {
        DeadlineCmd::Set { scope, subject, at, timezone, readiness_target, completion_target, evidence_refs, idempotency_key, confirm } => {
            scope.validate("deadline set")?;
            let mut b = scope.body();
            b["subject_ref"] = json!(subject);
            b["deadline_at"] = json!(at);
            b["timezone"] = json!(timezone);
            b["readiness_target"] = json!(readiness_target);
            b["completion_target_ref"] = json!(completion_target);
            b["evidence_refs"] = json!(evidence_refs);
            b["idempotency_key"] = json!(idempotency_key);
            b["confirm"] = json!(confirm);
            api.post("/v1/deadline/set", &b).await?
        }
        DeadlineCmd::SetCivil { scope, subject, local, timezone, fold_policy, gap_policy, calendar, calendar_version, tzdb_version, completion_target, evidence_refs, idempotency_key, confirm } => {
            scope.validate("deadline set-civil")?;
            let mut b = scope.body();
            b["subject_ref"] = json!(subject);
            b["local_time"] = json!(local);
            b["timezone"] = json!(timezone);
            b["fold_policy"] = json!(fold_policy);
            b["gap_policy"] = json!(gap_policy);
            b["calendar_ref"] = json!(calendar);
            b["calendar_version"] = json!(calendar_version);
            b["tzdb_version"] = json!(tzdb_version);
            b["completion_target_ref"] = json!(completion_target);
            b["evidence_refs"] = json!(evidence_refs);
            b["idempotency_key"] = json!(idempotency_key);
            b["confirm"] = json!(confirm);
            api.post("/v1/deadline/set-civil", &b).await?
        }
        DeadlineCmd::Inspect { deadline_id, scope } => {
            scope.validate("deadline inspect")?;
            api.get(&query_arg(&format!("/v1/deadline/{}", urlencoding::encode(&deadline_id)), &scope, "view", "canonical")).await?
        }
        DeadlineCmd::ResolveCivil { deadline_id, scope, tzdb_version } => {
            scope.validate("deadline resolve-civil")?;
            let mut b = scope.body();
            b["deadline_id"] = json!(deadline_id);
            b["tzdb_version"] = json!(tzdb_version);
            api.post("/v1/deadline/resolve-civil", &b).await?
        }
        DeadlineCmd::Conflicts { scope } => {
            scope.validate("deadline conflicts")?;
            api.get(&format!("/v1/deadline/conflicts?{}", scope.query())).await?
        }
        DeadlineCmd::Revise { deadline_id, scope, expected_revision, reason, at, idempotency_key, confirm } => {
            scope.validate("deadline revise")?;
            let mut b = scope.body();
            b["deadline_id"] = json!(deadline_id);
            b["expected_revision"] = json!(expected_revision);
            b["reason"] = json!(reason);
            b["deadline_at"] = json!(at);
            b["idempotency_key"] = json!(idempotency_key);
            b["confirm"] = json!(confirm);
            api.post("/v1/deadline/revise", &b).await?
        }
        DeadlineCmd::Clear { deadline_id, scope, expected_revision, reason, idempotency_key, confirm } => {
            scope.validate("deadline clear")?;
            let mut b = scope.body();
            b["deadline_id"] = json!(deadline_id);
            b["expected_revision"] = json!(expected_revision);
            b["reason"] = json!(reason);
            b["idempotency_key"] = json!(idempotency_key);
            b["confirm"] = json!(confirm);
            api.post("/v1/deadline/clear", &b).await?
        }
        DeadlineCmd::List { scope } => {
            scope.validate("deadline list")?;
            api.get(&format!("/v1/deadlines?{}", scope.query())).await?
        }
    };
    output("deadline", response, json_output).await
}
#[derive(Subcommand)]
pub enum EstimateCmd {
    Request {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        target_state: String,
        #[arg(long)]
        packet: std::path::PathBuf,
    },
    Inspect {
        estimate_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
    },
    Validate {
        estimate_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
    },
    Evaluate {
        estimate_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        actual_event: String,
        #[arg(long)]
        packet: std::path::PathBuf,
    },
    History {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        task_family: String,
        #[arg(long)]
        include_censored: bool,
    },
}
pub async fn run_estimate(cmd: EstimateCmd, json_output: bool) -> Result<()> {
    let api = ApiClient::new();
    let response = match cmd {
        EstimateCmd::Request { scope, subject, target_state, packet: path } => {
            scope.validate("estimate request")?;
            let mut b = scoped_packet(&scope, &path)?;
            b["subject_ref"] = json!(subject);
            b["target_state"] = json!(target_state);
            api.post("/v1/estimate/request", &b).await?
        }
        EstimateCmd::Inspect { estimate_id, scope } => {
            scope.validate("estimate inspect")?;
            api.get(&query_arg(&format!("/v1/estimate/{}", urlencoding::encode(&estimate_id)), &scope, "view", "canonical")).await?
        }
        EstimateCmd::Validate { estimate_id, scope } => {
            scope.validate("estimate validate")?;
            let mut b = scope.body();
            b["estimate_id"] = json!(estimate_id);
            api.post("/v1/estimate/validate", &b).await?
        }
        EstimateCmd::Evaluate { estimate_id, scope, actual_event, packet: path } => {
            scope.validate("estimate evaluate")?;
            let mut b = scoped_packet(&scope, &path)?;
            b["estimate_id"] = json!(estimate_id);
            b["actual_event_ref"] = json!(actual_event);
            api.post("/v1/estimate/evaluate", &b).await?
        }
        EstimateCmd::History { scope, task_family, include_censored } => {
            scope.validate("estimate history")?;
            api.get(&format!("/v1/estimate/history?{}&task_family={}&include_censored={}", scope.query(), urlencoding::encode(&task_family), include_censored)).await?
        }
    };
    output("estimate", response, json_output).await
}
#[derive(Subcommand)]
pub enum ProgressCmd {
    Record {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        item: String,
        #[arg(long)]
        kind: String,
        #[arg(long = "evidence")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        idempotency_key: String,
    },
    Status {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        item: String,
    },
}
pub async fn run_progress(cmd: ProgressCmd, json_output: bool) -> Result<()> {
    let api = ApiClient::new();
    let response = match cmd {
        ProgressCmd::Record { scope, item, kind, evidence_refs, idempotency_key } => {
            scope.validate("progress record")?;
            let mut b = scope.body();
            b["item_id"] = json!(item);
            b["kind"] = json!(kind);
            b["evidence_refs"] = json!(evidence_refs);
            b["idempotency_key"] = json!(idempotency_key);
            api.post("/v1/progress/record", &b).await?
        }
        ProgressCmd::Status { scope, item } => {
            scope.validate("progress status")?;
            api.get(&query_arg("/v1/progress/status", &scope, "item_id", &item)).await?
        }
    };
    output("progress", response, json_output).await
}
#[derive(Subcommand)]
pub enum NoProgressCmd {
    Inspect {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        item: String,
    },
}
#[derive(Subcommand)]
pub enum LostTimeCmd {
    List {
        #[command(flatten)]
        scope: ScopeArgs,
        #[arg(long)]
        subject: String,
    },
    Inspect {
        incident_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
    },
}
#[derive(Subcommand)]
pub enum OpportunityCmd {
    Inspect {
        subject_ref: String,
        #[command(flatten)]
        scope: ScopeArgs,
    },
}
#[derive(Subcommand)]
pub enum CancellationCmd {
    Inspect {
        cancellation_id: String,
        #[command(flatten)]
        scope: ScopeArgs,
    },
}
async fn scoped_get(label: &str, scope: ScopeArgs, path: String, json_output: bool) -> Result<()> {
    scope.validate(label)?;
    output(label, ApiClient::new().get(&format!("{path}?{}", scope.query())).await?, json_output).await
}
pub async fn run_no_progress(cmd: NoProgressCmd, j: bool) -> Result<()> {
    match cmd {
        NoProgressCmd::Inspect { scope, item } => scoped_get("no-progress", scope, format!("/v1/no-progress/incidents?item_id={}", urlencoding::encode(&item)), j).await,
    }
}
pub async fn run_lost_time(cmd: LostTimeCmd, j: bool) -> Result<()> {
    match cmd {
        LostTimeCmd::List { scope, subject } => scoped_get("lost-time", scope, format!("/v1/lost-time/incidents?subject_ref={}", urlencoding::encode(&subject)), j).await,
        LostTimeCmd::Inspect { incident_id, scope } => scoped_get("lost-time", scope, format!("/v1/lost-time/incidents/{}", urlencoding::encode(&incident_id)), j).await,
    }
}
pub async fn run_opportunity(cmd: OpportunityCmd, j: bool) -> Result<()> {
    match cmd {
        OpportunityCmd::Inspect { subject_ref, scope } => scoped_get("opportunity", scope, format!("/v1/opportunities/{}", urlencoding::encode(&subject_ref)), j).await,
    }
}
pub async fn run_cancellation(cmd: CancellationCmd, j: bool) -> Result<()> {
    match cmd {
        CancellationCmd::Inspect { cancellation_id, scope } => scoped_get("cancellation", scope, format!("/v1/cancellation/{}", urlencoding::encode(&cancellation_id)), j).await,
    }
}
