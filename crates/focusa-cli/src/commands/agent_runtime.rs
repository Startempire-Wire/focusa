//! Spec 140 Runtime Constitution CLI.

use crate::api_client::ApiClient;
use clap::{Args, Subcommand};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[derive(Subcommand)]
pub enum AgentRuntimeCmd {
    Scan(ProjectArgs),
    Sources(ProjectArgs),
    Claims(ProjectArgs),
    Conflicts(ProjectArgs),
    Reconcile(InputArgs),
    Simulate(SimulateArgs),
    Effective(ProjectArgs),
    Drift(ProjectArgs),
    /// Preview zero-hidden-change instruction migration and quarantine.
    Migration(ProjectArgs),
    /// Evaluate the foundational instruction-integrity guard from typed JSON.
    IntegrityEvaluate(InputArgs),
    /// Read foundational instruction-integrity availability and outage posture.
    IntegrityStatus,
    /// Propose a canonical instruction amendment from typed JSON.
    AmendmentPropose(InputArgs),
    /// Activate a separately approved amendment and documentation sweep.
    AmendmentActivate(InputArgs),
    /// Verify Mission Canvas-independent headless capability parity.
    HeadlessVerify(InputArgs),
    #[command(subcommand)]
    Constitution(ConstitutionCmd),
    #[command(subcommand)]
    Prompt(PromptCmd),
    #[command(subcommand)]
    Artifacts(ArtifactsCmd),
    /// Render the Agent Runtime Studio in the terminal.
    Studio {
        constitution_id: String,
    },
    Doctor,
}

#[derive(Args)]
pub struct ProjectArgs {
    #[arg(long, default_value = ".")]
    project_root: PathBuf,
    #[arg(long, default_value_t = 262_144)]
    max_source_bytes: u64,
}

#[derive(Args)]
pub struct SimulateArgs {
    #[arg(long, default_value = ".")]
    project_root: PathBuf,
    #[arg(long)]
    path: Option<String>,
    #[arg(long)]
    profile: Option<String>,
    #[arg(long)]
    target: Option<String>,
}

#[derive(Args)]
pub struct InputArgs {
    #[arg(long)]
    input: PathBuf,
}

#[derive(Args)]
pub struct IdInputArgs {
    id: String,
    #[arg(long)]
    input: PathBuf,
}

#[derive(Subcommand)]
pub enum ConstitutionCmd {
    Draft(InputArgs),
    Show { id: String },
    Preview(IdInputArgs),
    Approve(IdInputArgs),
    Activate(IdInputArgs),
    Revoke(IdInputArgs),
    Rollback(IdInputArgs),
}

#[derive(Subcommand)]
pub enum PromptCmd {
    Compile(PromptCompileArgs),
    Preview(IdInputArgs),
    Evaluate(InputArgs),
}

#[derive(Args)]
pub struct PromptCompileArgs {
    #[arg(long)]
    input: PathBuf,
    #[arg(long, default_value = "pi")]
    target: String,
    #[arg(long, default_value = "append")]
    mode: String,
}

#[derive(Subcommand)]
pub enum ArtifactsCmd {
    Compile(InputArgs),
    Preview(InputArgs),
    Apply(InputArgs),
    Verify(InputArgs),
}

pub async fn run(command: AgentRuntimeCmd, output_json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let response = match command {
        AgentRuntimeCmd::Scan(args) => {
            api.post("/v1/agent-runtime/instructions/scan", &project_body(&args)?)
                .await?
        }
        AgentRuntimeCmd::Sources(args) => api.get(&project_query("sources", &args)?).await?,
        AgentRuntimeCmd::Claims(args) => api.get(&project_query("claims", &args)?).await?,
        AgentRuntimeCmd::Conflicts(args) => api.get(&project_query("conflicts", &args)?).await?,
        AgentRuntimeCmd::Reconcile(args) => {
            api.post(
                "/v1/agent-runtime/instructions/reconcile",
                &read_input(&args.input)?,
            )
            .await?
        }
        AgentRuntimeCmd::Simulate(args) => {
            api.post(
                "/v1/agent-runtime/instructions/simulate",
                &json!({
                    "project_root": canonical_root(&args.project_root)?,
                    "path": args.path,
                    "profile": args.profile,
                    "target": args.target,
                }),
            )
            .await?
        }
        AgentRuntimeCmd::Effective(args) => api.get(&project_query("effective", &args)?).await?,
        AgentRuntimeCmd::Drift(args) => api.get(&project_query("drift", &args)?).await?,
        AgentRuntimeCmd::Migration(args) => {
            api.post("/v1/agent-runtime/migration/preview", &project_body(&args)?)
                .await?
        }
        AgentRuntimeCmd::IntegrityEvaluate(args) => {
            api.post(
                "/v1/agent-runtime/instruction-integrity/evaluate",
                &read_input(&args.input)?,
            )
            .await?
        }
        AgentRuntimeCmd::IntegrityStatus => {
            api.get("/v1/agent-runtime/instruction-integrity/status")
                .await?
        }
        AgentRuntimeCmd::AmendmentPropose(args) => {
            api.post(
                "/v1/agent-runtime/amendments/propose",
                &read_input(&args.input)?,
            )
            .await?
        }
        AgentRuntimeCmd::AmendmentActivate(args) => {
            api.post(
                "/v1/agent-runtime/amendments/activate",
                &read_input(&args.input)?,
            )
            .await?
        }
        AgentRuntimeCmd::HeadlessVerify(args) => {
            api.post(
                "/v1/agent-runtime/headless/verify",
                &read_input(&args.input)?,
            )
            .await?
        }
        AgentRuntimeCmd::Constitution(command) => run_constitution(&api, command).await?,
        AgentRuntimeCmd::Prompt(command) => run_prompt(&api, command).await?,
        AgentRuntimeCmd::Artifacts(command) => run_artifacts(&api, command).await?,
        AgentRuntimeCmd::Studio { constitution_id } => {
            let response = api
                .get(&format!(
                    "/v1/agent-runtime/studio?constitution_id={}",
                    encoded(&constitution_id)
                ))
                .await?;
            if !output_json {
                println!(
                    "{}",
                    focusa_terminal_ui::agent_runtime_studio::render_agent_runtime_studio(
                        &response
                    )?
                );
                return Ok(());
            }
            response
        }
        AgentRuntimeCmd::Doctor => api.get("/v1/agent-runtime/doctor").await?,
    };
    render(&response, output_json)?;
    Ok(())
}

async fn run_constitution(api: &ApiClient, command: ConstitutionCmd) -> anyhow::Result<Value> {
    Ok(match command {
        ConstitutionCmd::Draft(args) => {
            api.post(
                "/v1/agent-runtime/constitutions/draft",
                &read_input(&args.input)?,
            )
            .await?
        }
        ConstitutionCmd::Show { id } => {
            api.get(&format!("/v1/agent-runtime/constitutions/{}", encoded(&id)))
                .await?
        }
        ConstitutionCmd::Preview(args) => post_id(api, &args, "preview").await?,
        ConstitutionCmd::Approve(args) => post_id(api, &args, "approve").await?,
        ConstitutionCmd::Activate(args) => post_id(api, &args, "activate").await?,
        ConstitutionCmd::Revoke(args) => post_id(api, &args, "revoke").await?,
        ConstitutionCmd::Rollback(args) => post_id(api, &args, "rollback").await?,
    })
}

async fn run_prompt(api: &ApiClient, command: PromptCmd) -> anyhow::Result<Value> {
    Ok(match command {
        PromptCmd::Compile(args) => {
            let mut body = read_input(&args.input)?;
            body["target"] = Value::String(args.target);
            body["mode"] = Value::String(args.mode);
            api.post("/v1/agent-runtime/compile/system-prompt", &body)
                .await?
        }
        PromptCmd::Preview(args) => post_id(api, &args, "preview").await?,
        PromptCmd::Evaluate(args) => {
            api.post("/v1/agent-runtime/evaluations", &read_input(&args.input)?)
                .await?
        }
    })
}

async fn run_artifacts(api: &ApiClient, command: ArtifactsCmd) -> anyhow::Result<Value> {
    let (path, input) = match command {
        ArtifactsCmd::Compile(args) => ("/v1/agent-runtime/compile/agents-md", args.input),
        ArtifactsCmd::Preview(args) => ("/v1/agent-runtime/delivery/preview", args.input),
        ArtifactsCmd::Apply(args) => ("/v1/agent-runtime/delivery/commit", args.input),
        ArtifactsCmd::Verify(args) => ("/v1/agent-runtime/delivery/verify", args.input),
    };
    api.post(path, &read_input(&input)?).await
}

async fn post_id(api: &ApiClient, args: &IdInputArgs, action: &str) -> anyhow::Result<Value> {
    api.post(
        &format!(
            "/v1/agent-runtime/constitutions/{}/{}",
            encoded(&args.id),
            action
        ),
        &read_input(&args.input)?,
    )
    .await
}

fn project_body(args: &ProjectArgs) -> anyhow::Result<Value> {
    Ok(
        json!({"project_root":canonical_root(&args.project_root)?,"max_source_bytes":args.max_source_bytes}),
    )
}

fn project_query(action: &str, args: &ProjectArgs) -> anyhow::Result<String> {
    Ok(format!(
        "/v1/agent-runtime/instructions/{action}?project_root={}&max_source_bytes={}",
        encoded(&canonical_root(&args.project_root)?),
        args.max_source_bytes
    ))
}

fn canonical_root(path: &PathBuf) -> anyhow::Result<String> {
    Ok(path.canonicalize()?.to_string_lossy().into_owned())
}

fn read_input(path: &PathBuf) -> anyhow::Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn encoded(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

fn render(response: &Value, output_json: bool) -> anyhow::Result<()> {
    if output_json {
        println!("{}", serde_json::to_string_pretty(response)?);
    } else if let Some(error) = response.get("error") {
        anyhow::bail!("agent-runtime operation rejected: {error}");
    } else {
        println!("Agent Runtime Constitution operation completed.");
        println!("{}", serde_json::to_string_pretty(response)?);
    }
    Ok(())
}
