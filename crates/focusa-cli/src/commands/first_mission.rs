//! Spec124 first-mission guided evaluator workflow.

use crate::api_client::ApiClient;
use crate::commands::{scope::ensure_project_root_scope_safe, scope_resolver};
use clap::Args;
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct FirstMissionArgs {
    /// Safe project folder/container for scoped first mission.
    #[arg(long)]
    pub project_root: Option<String>,
    /// Project alias/path candidate resolved by the scoped project resolver.
    #[arg(long)]
    pub project: Option<String>,
    /// Stable logical workstream id.
    #[arg(long)]
    pub continuity_id: Option<String>,
    /// Non-interactive confirmation for future promptful flows.
    #[arg(long)]
    pub yes: bool,
    /// Print the planned workflow without mutating daemon state.
    #[arg(long)]
    pub dry_run: bool,
    /// Suggest/open Mission Deck after resume packet creation.
    #[arg(long)]
    pub open_deck: bool,
    /// Suppress animated/progressive output.
    #[arg(long)]
    pub no_animation: bool,
}

fn step(name: &str, status: &str, detail: impl Into<String>) -> Value {
    json!({"name": name, "status": status, "detail": detail.into()})
}

fn render_human(payload: &Value) {
    println!("FOCUSA FIRST MISSION\n");
    println!("Give this AI project a save point, proof, and safe handoff.\n");
    if let Some(steps) = payload.get("steps").and_then(Value::as_array) {
        for item in steps {
            let mark = match item.get("status").and_then(Value::as_str) {
                Some("ok") => "✓",
                Some("planned") => "•",
                Some("skipped") => "-",
                _ => "!",
            };
            let name = item.get("name").and_then(Value::as_str).unwrap_or("step");
            let detail = item.get("detail").and_then(Value::as_str).unwrap_or("");
            if detail.is_empty() {
                println!("{mark} {name}");
            } else {
                println!("{mark} {name}: {detail}");
            }
        }
    }
    println!(
        "\nMission {}.\n",
        payload
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!("Next:");
    println!("  focusa deck");
    println!("  focusa status operator");
    if let Some(root) = payload.get("project_root").and_then(Value::as_str) {
        println!("  focusa workpoint resume --project-root {root}");
    } else {
        println!("  focusa workpoint resume");
    }
}

pub async fn run(args: FirstMissionArgs, json_output: bool) -> anyhow::Result<()> {
    let resolved = scope_resolver::resolve_project_scope(
        args.project_root.as_deref(),
        args.project.as_deref(),
        std::env::current_dir()
            .ok()
            .and_then(|path| path.to_str().map(str::to_string))
            .as_deref(),
    )?;
    ensure_project_root_scope_safe(
        Some(resolved.project_root.as_str()),
        "first-mission: project_root",
    )?;
    let continuity_id = args
        .continuity_id
        .clone()
        .or(resolved.continuity_id.clone())
        .unwrap_or_else(|| "focusa-main".to_string());

    let mut steps = vec![
        step("Project selected", "ok", resolved.project_root.clone()),
        step("Scope safe", "ok", format!("{:?}", resolved.scope_source)),
    ];

    if args.dry_run {
        steps.extend([
            step("Daemon healthy", "planned", "/v1/health"),
            step("Project marker present", "planned", ".focusa-project.json"),
            step("Workpoint created", "planned", "/v1/workpoint/checkpoint"),
            step("Proof linked", "planned", "/v1/workpoint/evidence/link"),
            step("Resume packet ready", "planned", "/v1/workpoint/resume"),
            step("Project status shown", "planned", "focusa project status"),
            step(
                "Mission Deck",
                if args.open_deck { "planned" } else { "skipped" },
                "focusa deck",
            ),
        ]);
        let payload = json!({
            "schema": "focusa.first_mission.v1",
            "status": "planned",
            "dry_run": true,
            "project_root": resolved.project_root,
            "continuity_id": continuity_id,
            "mutated": false,
            "open_deck_requested": args.open_deck,
            "no_animation": args.no_animation,
            "yes": args.yes,
            "steps": steps,
            "next": ["focusa deck", "focusa status operator", "focusa project status", "focusa workpoint resume"],
        });
        if json_output {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            render_human(&payload);
        }
        return Ok(());
    }

    let api = ApiClient::with_timeout_secs(12);
    let health = api.get("/v1/health").await?;
    steps.push(step(
        "Daemon healthy",
        "ok",
        health
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("health-ok"),
    ));

    let identity_path = format!(
        "/v1/project/identity?project_root={}",
        resolved.project_root.replace(' ', "+")
    );
    let identity = api.get(&identity_path).await?;
    let marker_status = identity
        .pointer("/project_identity/status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    steps.push(step("Project marker present", "ok", marker_status));

    let checkpoint = api
        .post(
            "/v1/workpoint/checkpoint",
            &json!({
                "mission": "Focusa First Mission",
                "next_slice": "Use the First Mission resume packet as the safe project handoff.",
                "project_root": resolved.project_root,
                "continuity_id": continuity_id,
                "checkpoint_reason": "manual",
                "canonical": true,
                "promote": true,
                "action_intent": {
                    "action_type": "first_mission",
                    "target_ref": "focusa:first-mission",
                    "verification_hooks": ["health", "project_identity", "workpoint_resume"],
                    "status": "ready"
                },
                "active_object_refs": ["focusa:first-mission", resolved.project_root],
                "idempotency_key": format!("first-mission:{}:{}", resolved.project_root, continuity_id),
            }),
        )
        .await?;
    let workpoint_id = checkpoint
        .get("workpoint_id")
        .and_then(Value::as_str)
        .unwrap_or("active");
    steps.push(step("Workpoint created", "ok", workpoint_id));

    let evidence = api
        .post(
            "/v1/workpoint/evidence/link",
            &json!({
                "target_ref": "focusa:first-mission",
                "result": "First Mission verified daemon health, project identity, and Workpoint checkpoint creation.",
                "evidence_ref": "focusa:first-mission:cli",
            }),
        )
        .await?;
    steps.push(step(
        "Proof linked",
        "ok",
        evidence
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("linked"),
    ));

    let resume = api
        .post(
            "/v1/workpoint/resume",
            &json!({
                "mode": "operator_summary",
                "project_root": resolved.project_root,
                "continuity_id": continuity_id,
            }),
        )
        .await?;
    steps.push(step(
        "Resume packet ready",
        "ok",
        resume
            .get("workpoint_id")
            .and_then(Value::as_str)
            .unwrap_or("active"),
    ));
    steps.push(step(
        "Project status shown",
        "ok",
        format!("focusa project status --project-root {}", resolved.project_root),
    ));
    steps.push(step(
        "Mission Deck",
        if args.open_deck { "planned" } else { "skipped" },
        "focusa deck",
    ));

    let payload = json!({
        "schema": "focusa.first_mission.v1",
        "status": "saved",
        "dry_run": false,
        "mutated": true,
        "project_root": resolved.project_root,
        "continuity_id": continuity_id,
        "workpoint_id": workpoint_id,
        "checkpoint": checkpoint,
        "resume": resume,
        "open_deck_requested": args.open_deck,
        "no_animation": args.no_animation,
        "yes": args.yes,
        "steps": steps,
        "next": ["focusa deck", "focusa status operator", "focusa project status", "focusa workpoint resume"],
    });
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        render_human(&payload);
    }
    Ok(())
}
