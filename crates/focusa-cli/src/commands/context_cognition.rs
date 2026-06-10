//! Spec 100 Context Cognition CLI — view the bounded packet.
//!
//! `focusa context-cognition view --project-root <path> [--continuity-id <id>] [--json]`

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;

#[derive(Subcommand)]
pub enum ContextCognitionCmd {
    /// View the current ContextCognitionPacket (advisory, read-only).
    View {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Render the packet as compact text (for prompt/CLI/menubar).
    Render {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
    },
    /// Map packet surfaces to proof commands (curl + focusa + audits).
    Proof {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
    },
    /// Curate context under a token budget (Spec 100 Phase 3).
    Curate {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, default_value = "2000")]
        token_budget: usize,
        #[arg(long)]
        candidates_json: Option<String>,
    },
    /// Run a curator eval case and compute precision/recall/F1 (Spec 100 Phase 4).
    CurateEval {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, default_value = "2000")]
        token_budget: usize,
        #[arg(long)]
        candidates_json: Option<String>,
        #[arg(long)]
        expected_json: Option<String>,
        #[arg(long, default_value = "0.5")]
        score_threshold: f64,
        #[arg(long, default_value = "0.0")]
        baseline_f1: f64,
    },
    /// List recent curator eval runs for a project (Spec 100 Phase 4).
    CurateEvalRuns {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// List Cognition Optimizer artifacts for a project+module (Spec 100 Phase 5).
    OptimizerArtifacts {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long, default_value = "curator")]
        module_name: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Submit a Cognition Optimizer artifact and get the promote/rollback decision (Spec 100 Phase 5).
    CurateOptimize {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        prompt_artifact_ref: String,
        #[arg(long, default_value = "curator")]
        module_name: String,
        #[arg(long)]
        eval_score: f64,
        #[arg(long, default_value = "0.0")]
        baseline_score: f64,
        #[arg(long, default_value = "0.5")]
        score_threshold: f64,
        #[arg(long)]
        eval_run_id: Option<String>,
        #[arg(long)]
        rollback: bool,
    },
}

pub async fn handle(client: &mut ApiClient, cmd: ContextCognitionCmd) -> anyhow::Result<()> {
    match cmd {
        ContextCognitionCmd::View {
            project_root,
            continuity_id,
            json,
        } => {
            let mut path = String::from("/v1/context-cognition");
            let mut sep = "?";
            if let Some(pr) = project_root.as_deref() {
                path.push_str(sep);
                path.push_str("project_root=");
                path.push_str(&urlencoding_minimal(pr));
                sep = "&";
            }
            if let Some(cid) = continuity_id.as_deref() {
                path.push_str(sep);
                path.push_str("continuity_id=");
                path.push_str(&urlencoding_minimal(cid));
            }
            let resp = client.get(&path).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
                return Ok(());
            }
            print_human(&resp);
            Ok(())
        }
        ContextCognitionCmd::Render {
            project_root,
            continuity_id,
        } => {
            let path = build_query("/v1/context-cognition/render", project_root, continuity_id);
            let resp = client.get(&path).await?;
            if let Some(render) = resp.get("render").and_then(Value::as_str) {
                println!("{render}");
            } else {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            Ok(())
        }
        ContextCognitionCmd::Proof {
            project_root,
            continuity_id,
        } => {
            let path = build_query("/v1/context-cognition/proof", project_root, continuity_id);
            let resp = client.get(&path).await?;
            if let Some(commands) = resp.get("proof_commands").and_then(Value::as_array) {
                for c in commands {
                    if let Some(s) = c.as_str() {
                        println!("{s}");
                    }
                }
            } else {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            Ok(())
        }
        ContextCognitionCmd::Curate {
            project_root,
            target,
            token_budget,
            candidates_json,
        } => {
            let project_root = project_root
                .ok_or_else(|| anyhow::anyhow!("--project-root is required for curate"))?;
            let candidates: Vec<Value> = match candidates_json.as_deref() {
                Some(s) => serde_json::from_str(s)
                    .map_err(|e| anyhow::anyhow!("invalid --candidates-json: {e}"))?,
                None => Vec::new(),
            };
            let body = serde_json::json!({
                "project_root": project_root,
                "target": target,
                "token_budget": token_budget,
                "candidates": candidates,
            });
            let resp = client.post("/v1/context-cognition/curate", &body).await?;
            print_curated_human(&resp);
            Ok(())
        }
        ContextCognitionCmd::CurateEval {
            project_root,
            target,
            token_budget,
            candidates_json,
            expected_json,
            score_threshold,
            baseline_f1,
        } => {
            let project_root = project_root
                .ok_or_else(|| anyhow::anyhow!("--project-root is required for curate-eval"))?;
            let candidates: Vec<Value> = match candidates_json.as_deref() {
                Some(s) => serde_json::from_str(s)
                    .map_err(|e| anyhow::anyhow!("invalid --candidates-json: {e}"))?,
                None => Vec::new(),
            };
            let expected: Vec<String> = match expected_json.as_deref() {
                Some(s) => serde_json::from_str(s)
                    .map_err(|e| anyhow::anyhow!("invalid --expected-json: {e}"))?,
                None => Vec::new(),
            };
            let body = serde_json::json!({
                "project_root": project_root,
                "target": target,
                "token_budget": token_budget,
                "candidates": candidates,
                "expected_selected_paths": expected,
                "score_threshold": score_threshold,
                "baseline_f1": baseline_f1,
            });
            let resp = client
                .post("/v1/context-cognition/curate/eval", &body)
                .await?;
            print_eval_human(&resp);
            Ok(())
        }
        ContextCognitionCmd::CurateEvalRuns {
            project_root,
            limit,
        } => {
            let project_root = project_root.ok_or_else(|| {
                anyhow::anyhow!("--project-root is required for curate-eval-runs")
            })?;
            let body = serde_json::json!({
                "project_root": project_root,
                "limit": limit,
            });
            let resp = client.get(&format!("/v1/context-cognition/curate/eval/runs?project_root={project_root}&limit={limit}")).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        ContextCognitionCmd::OptimizerArtifacts {
            project_root,
            module_name,
            limit,
        } => {
            let project_root = project_root.ok_or_else(|| {
                anyhow::anyhow!("--project-root is required for optimizer-artifacts")
            })?;
            let body = serde_json::json!({
                "project_root": project_root,
                "module_name": module_name,
                "limit": limit,
            });
            let resp = client.get(&format!("/v1/context-cognition/optimizer/artifacts?project_root={project_root}&module_name={module_name}&limit={limit}")).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        ContextCognitionCmd::CurateOptimize {
            project_root,
            prompt_artifact_ref,
            module_name,
            eval_score,
            baseline_score,
            score_threshold,
            eval_run_id,
            rollback,
        } => {
            let project_root = project_root
                .ok_or_else(|| anyhow::anyhow!("--project-root is required for curate-optimize"))?;
            let body = serde_json::json!({
                "project_root": project_root,
                "module_name": module_name,
                "prompt_artifact_ref": prompt_artifact_ref,
                "eval_score": eval_score,
                "baseline_score": baseline_score,
                "score_threshold": score_threshold,
                "eval_run_id": eval_run_id,
                "rollback": rollback,
            });
            let resp = client
                .post("/v1/context-cognition/curate/optimize", &body)
                .await?;
            print_optimize_human(&resp);
            Ok(())
        }
    }
}

fn print_curated_human(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let target = payload
        .get("target")
        .and_then(Value::as_str)
        .unwrap_or("<none>");
    let budget = payload
        .get("token_budget")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let used = payload
        .get("tokens_used")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let selected_count = payload
        .get("selected_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let excluded_count = payload
        .get("excluded_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let rehydrate = payload
        .get("rehydrate_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    println!("context cognition curate {status} | target=\"{target}\" budget={budget} used={used}");
    println!(
        "fields: selected={selected_count} excluded={excluded_count} rehydrate_id={rehydrate}"
    );
    if let Some(arr) = payload.get("selected_context").and_then(Value::as_array) {
        for s in arr.iter().take(5) {
            let path = s.get("path").and_then(Value::as_str).unwrap_or("?");
            let kind = s.get("kind").and_then(Value::as_str).unwrap_or("?");
            let tokens = s.get("tokens").and_then(Value::as_u64).unwrap_or(0);
            let score = s.get("score").and_then(Value::as_f64).unwrap_or(0.0);
            println!("  selected: {kind} {path} tokens={tokens} score={score:.2}");
        }
    }
    if let Some(arr) = payload.get("excluded_context").and_then(Value::as_array) {
        for e in arr.iter().take(5) {
            let path = e.get("path").and_then(Value::as_str).unwrap_or("?");
            let reason = e.get("reason").and_then(Value::as_str).unwrap_or("?");
            println!("  excluded: {path} reason={reason}");
        }
    }
}

fn print_eval_human(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let run_id = payload
        .get("run_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let precision = payload
        .get("precision")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let recall = payload.get("recall").and_then(Value::as_f64).unwrap_or(0.0);
    let f1 = payload.get("f1").and_then(Value::as_f64).unwrap_or(0.0);
    let baseline_f1 = payload
        .get("baseline_f1")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let tokens_used = payload
        .get("tokens_used")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let promovido = payload
        .get("promoted")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    println!(
        "context cognition curate-eval {status} | run_id={run_id} precision={precision:.2} recall={recall:.2} f1={f1:.2} baseline_f1={baseline_f1:.2}"
    );
    println!(
        "fields: tokens_used={tokens_used} promoted={} rehydrate_id={}",
        if promovido { "yes" } else { "no" },
        run_id
    );
    if let Some(arr) = payload.get("selected_paths").and_then(Value::as_array) {
        for p in arr.iter().take(5) {
            if let Some(s) = p.as_str() {
                println!("  selected: {s}");
            }
        }
    }
}

fn print_optimize_human(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let artifact_id = payload
        .get("artifact_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let decision = payload
        .get("decision")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let eval_score = payload
        .get("eval_score")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let baseline_score = payload
        .get("baseline_score")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let score_threshold = payload
        .get("score_threshold")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let rollback_ref = payload
        .get("rollback_ref")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let promoted = payload
        .get("promoted")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    println!(
        "context cognition curate-optimize {status} | decision={decision} artifact_id={artifact_id}"
    );
    println!(
        "fields: eval_score={eval_score:.2} baseline_score={baseline_score:.2} score_threshold={score_threshold:.2} rollback_ref={rollback_ref} promoted={}",
        if promoted { "yes" } else { "no" }
    );
}

fn build_query(base: &str, project_root: Option<String>, continuity_id: Option<String>) -> String {
    let mut path = String::from(base);
    let mut sep = "?";
    if let Some(pr) = project_root.as_deref() {
        path.push_str(sep);
        path.push_str("project_root=");
        path.push_str(&urlencoding_minimal(pr));
        sep = "&";
    }
    if let Some(cid) = continuity_id.as_deref() {
        path.push_str(sep);
        path.push_str("continuity_id=");
        path.push_str(&urlencoding_minimal(cid));
    }
    path
}

fn print_human(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let scope_status = payload
        .get("scope_status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let schema = payload
        .pointer("/packet/schema_version")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let workpoint_id = payload
        .pointer("/packet/scope/workpoint_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let trajectory_id = payload
        .pointer("/packet/scope/trajectory_id")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let action_authority = payload
        .pointer("/packet/authority/action_authority")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let next_tools = payload
        .get("next_tools")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let evidence_count = payload
        .pointer("/packet/evidence_refs")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);

    println!("context cognition {status} | scope={scope_status} schema={schema}");
    println!(
        "ids: workpoint_id={workpoint_id} trajectory_id={trajectory_id} action_authority={action_authority}"
    );
    println!("fields: evidence_refs={evidence_count}");
    if !next_tools.is_empty() {
        println!("next: {next_tools}");
    }
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(c),
            _ => out.push_str(&format!("%{:02X}", c as u32)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn print_human_renders_summary() {
        let payload = json!({
            "status": "completed",
            "scope_status": "matched",
            "packet": {
                "schema_version": "focusa.context_cognition_packet.v1",
                "scope": {
                    "workpoint_id": "019eacb8-c8be-7f63-ae40-a16da6600110",
                    "trajectory_id": "trajectory:project-fnv1a64:8aab637a4a87e459:defined-goal"
                },
                "authority": {"action_authority": "workpoint"},
                "evidence_refs": ["ev:1", "ev:2"]
            },
            "next_tools": ["focusa_active_object_resolve", "focusa_workpoint_checkpoint"]
        });
        // Just ensure no panic
        print_human(&payload);
    }

    #[test]
    fn urlencoding_escapes_paths() {
        assert_eq!(
            urlencoding_minimal("/home/wirebot/focusa"),
            "%2Fhome%2Fwirebot%2Ffocusa"
        );
        assert_eq!(urlencoding_minimal("a-b_c.d~e"), "a-b_c.d~e");
    }
}
