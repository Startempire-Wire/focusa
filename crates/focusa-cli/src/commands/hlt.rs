//! Trajectory Ladder CLI — Spec96/102: north-star control, scope-bounded, CRDT-grade.
//! HLT is the ultimate project direction. All commands are project-scoped.

use crate::api_client::ApiClient;
use crate::commands::scope::ensure_project_root_scope_safe;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum HltCmd {
    /// ls — view full Trajectory Ladder (HLT → MLG → STG → Waypoints)
    Ls {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long, default_value = "full")]
        mode: String,
        #[arg(long)]
        json: bool,
    },
    /// set — set HLT (High-Level Trajectory) — north-star goal
    Set {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        hlt: String,
        #[arg(long)]
        desired_end_state: String,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        mlg: Option<String>,
        #[arg(long)]
        stg: Option<String>,
        #[arg(long = "waypoint")]
        waypoints: Vec<String>,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        confirm: bool,
    },
    /// reset — reset HLT to previous value
    Reset {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long, default_value = "1")]
        steps: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// history — show HLT change history (append-only ledger)
    History {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        /// Spec 125 §7.6: filter by session. 'current' resolves to active session.
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long, default_value = "20")]
        limit: String,
        #[arg(long)]
        json: bool,
    },
    /// sessions — list distinct HLT history sessions for a project
    Sessions {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long, default_value = "50")]
        limit: String,
        #[arg(long)]
        json: bool,
    },
    /// fallback — show latest valid HLT fallback for a session/continuity/project
    Fallback {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        /// Spec 125 §7.6: 'current' resolves to active session.
        #[arg(long)]
        session_id: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// diff — compare trajectory between versions
    Diff {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long, default_value = "1")]
        from: String,
        #[arg(long, default_value = "0")]
        to: String,
    },
    /// mlg — view or set MLG (Mid-Level Goal)
    Mlg {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        set: Option<String>,
        #[arg(long)]
        list: bool,
    },
    /// stg — view or set STG (Short-Term Goal)
    Stg {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        set: Option<String>,
        #[arg(long)]
        list: bool,
    },
    /// waypoint — add or list waypoints
    Waypoint {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        add: Option<String>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        complete: Option<String>,
    },
    /// assess — assess current state vs desired end state
    Assess {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        current_state: String,
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
    },
    /// checkpoint — create trajectory checkpoint
    Checkpoint {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        summary: Option<String>,
    },
    /// supersede — supersede HLT with evidence path
    Supersede {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        new_hlt: String,
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        continuity_id: Option<String>,
    },
    /// export — export trajectory ledger to file
    Export {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        output: String,
        #[arg(long, default_value = "jsonl")]
        format: String,
    },
    /// verify — verify trajectory integrity
    Verify {
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
    },
}

pub async fn run(cmd: HltCmd, _json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    match cmd {
        HltCmd::Ls {
            project_root,
            continuity_id,
            mode,
            json,
        } => {
            run_ls(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                &mode,
                json,
            )
            .await
        }
        HltCmd::Set {
            project_root,
            hlt,
            desired_end_state,
            continuity_id,
            mlg,
            stg,
            waypoints,
            reason,
            evidence_refs,
            confirm,
        } => {
            run_set(
                &api,
                &cwd,
                project_root.as_deref(),
                &hlt,
                &desired_end_state,
                continuity_id.as_deref(),
                mlg.as_deref(),
                stg.as_deref(),
                waypoints,
                reason.as_deref(),
                evidence_refs,
                confirm,
            )
            .await
        }
        HltCmd::Reset {
            project_root,
            continuity_id,
            steps,
            dry_run,
        } => {
            run_reset(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                &steps,
                dry_run,
            )
            .await
        }
        HltCmd::History {
            project_root,
            continuity_id,
            session_id,
            limit,
            json,
        } => {
            run_history(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                session_id.as_deref(),
                &limit,
                json,
            )
            .await
        }
        HltCmd::Sessions {
            project_root,
            continuity_id,
            limit,
            json,
        } => {
            run_sessions(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                &limit,
                json,
            )
            .await
        }
        HltCmd::Fallback {
            project_root,
            continuity_id,
            session_id,
            json,
        } => {
            run_fallback(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                session_id.as_deref(),
                json,
            )
            .await
        }
        HltCmd::Diff {
            project_root,
            continuity_id,
            from,
            to,
        } => {
            run_diff(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                &from,
                &to,
            )
            .await
        }
        HltCmd::Mlg {
            project_root,
            continuity_id,
            set,
            list,
        } => {
            run_mlg(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                set.as_deref(),
                list,
            )
            .await
        }
        HltCmd::Stg {
            project_root,
            continuity_id,
            set,
            list,
        } => {
            run_stg(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                set.as_deref(),
                list,
            )
            .await
        }
        HltCmd::Waypoint {
            project_root,
            continuity_id,
            add,
            list,
            complete,
        } => {
            run_waypoint(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                add.as_deref(),
                list,
                complete.as_deref(),
            )
            .await
        }
        HltCmd::Assess {
            project_root,
            continuity_id,
            current_state,
            evidence_refs,
        } => {
            run_assess(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                &current_state,
                evidence_refs,
            )
            .await
        }
        HltCmd::Checkpoint {
            project_root,
            continuity_id,
            summary,
        } => {
            run_checkpoint(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                summary.as_deref(),
            )
            .await
        }
        HltCmd::Supersede {
            project_root,
            new_hlt,
            evidence_refs,
            reason,
            continuity_id,
        } => {
            run_supersede(
                &api,
                &cwd,
                project_root.as_deref(),
                &new_hlt,
                evidence_refs,
                &reason,
                continuity_id.as_deref(),
            )
            .await
        }
        HltCmd::Export {
            project_root,
            continuity_id,
            output,
            format,
        } => {
            run_export(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
                &output,
                &format,
            )
            .await
        }
        HltCmd::Verify {
            project_root,
            continuity_id,
        } => {
            run_verify(
                &api,
                &cwd,
                project_root.as_deref(),
                continuity_id.as_deref(),
            )
            .await
        }
    }
}

fn get_project_root(project_root: Option<&str>, cwd: &str) -> anyhow::Result<String> {
    let resolved = project_root
        .map(String::from)
        .or_else(|| std::env::var("FOCUSA_PROJECT_ROOT").ok())
        .unwrap_or_else(|| cwd.to_string());
    ensure_project_root_scope_safe(Some(resolved.as_str()), "hlt: project_root")?;
    Ok(resolved)
}

fn build_query(path: &str, project_root: &str, continuity_id: Option<&str>) -> String {
    let mut q = format!(
        "{}?project_root={}",
        path,
        urlencoding::encode(project_root)
    );
    if let Some(cid) = continuity_id {
        q.push_str(&format!("&continuity_id={}", urlencoding::encode(cid)));
    }
    q
}

async fn run_ls(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    mode: &str,
    json_output: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;
    let path = build_query("/v1/trajectory/view", &project_root, continuity_id);
    let url = format!("{}&mode={}", path, mode);
    let response: Value = api.get(&url).await?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Ok(());
    }

    let hlt = response
        .pointer("/trajectory/long_term_goal")
        .and_then(|v| v.as_str())
        .unwrap_or("(not set)");
    let mlg = response
        .pointer("/trajectory/mid_level_goal")
        .and_then(|v| v.as_str());
    let stg = response
        .pointer("/trajectory/short_term_goal")
        .and_then(|v| v.as_str());
    let desired = response
        .pointer("/trajectory/desired_end_state")
        .and_then(|v| v.as_str())
        .unwrap_or("(not set)");
    let waypoints = response
        .pointer("/trajectory/waypoints")
        .and_then(|v| v.as_array());

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Trajectory Ladder — North Star View                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Project: {}", project_root);
    if let Some(cid) = continuity_id {
        println!("  Continuity: {}", cid);
    }
    println!();
    println!("  ┌─ HLT (High-Level Trajectory) ─────────────────────────┐");
    println!("  │ {} │", wrap_text(hlt, 54));
    println!("  └───────────────────────────────────────────────────────┘");

    if let Some(mlg) = mlg {
        println!();
        println!("  ┌─ MLG (Mid-Level Goal) ───────────────────────────────┐");
        println!("  │ {} │", wrap_text(mlg, 54));
        println!("  └───────────────────────────────────────────────────────┘");
    }

    if let Some(stg) = stg {
        println!();
        println!("  ┌─ STG (Short-Term Goal) ──────────────────────────────┐");
        println!("  │ {} │", wrap_text(stg, 54));
        println!("  └───────────────────────────────────────────────────────┘");
    }

    if let Some(wps) = waypoints.filter(|wps| !wps.is_empty()) {
        println!();
        println!("  Waypoints:");
        for (i, wp) in wps.iter().enumerate() {
            if let Some(wp_str) = wp.as_str() {
                println!("    {}. {}", i + 1, wp_str);
            }
        }
    }

    println!();
    println!("  Desired End State: {}", wrap_text(desired, 60));

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_set(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    hlt: &str,
    desired_end_state: &str,
    continuity_id: Option<&str>,
    mlg: Option<&str>,
    stg: Option<&str>,
    waypoints: Vec<String>,
    _reason: Option<&str>,
    evidence_refs: Vec<String>,
    confirm: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    if !confirm {
        println!("Setting HLT for: {}", project_root);
        println!();
        println!("  HLT: {}", wrap_text(hlt, 70));
        println!("  Desired End State: {}", wrap_text(desired_end_state, 70));
        if let Some(mlg) = mlg {
            println!("  MLG: {}", wrap_text(mlg, 70));
        }
        if let Some(stg) = stg {
            println!("  STG: {}", wrap_text(stg, 70));
        }
        if !waypoints.is_empty() {
            println!("  Waypoints: {:?}", waypoints);
        }
        println!();
        println!("Use --confirm to apply.");
        return Ok(());
    }

    let mut body = serde_json::json!({
        "project_root": project_root,
        "long_term_goal": hlt,
        "desired_end_state": desired_end_state,
        "goal_source": "cli",
        "operator_confirmed": true,
        "waypoints": waypoints,
    });

    if let Some(cid) = continuity_id {
        body["continuity_id"] = serde_json::json!(cid);
    }
    if let Some(mlg) = mlg {
        body["mid_level_goal"] = serde_json::json!(mlg);
    }
    if let Some(stg) = stg {
        body["short_term_goal"] = serde_json::json!(stg);
    }
    if !evidence_refs.is_empty() {
        body["supersession_evidence_refs"] = serde_json::json!(evidence_refs);
    }

    let response: Value = api.post("/v1/trajectory/define-goal", &body).await?;
    let status = response
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let persisted = response
        .get("persisted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    println!("  Status: {}", status);
    println!("  Persisted: {}", persisted);
    if persisted {
        println!("✓ HLT set successfully");
    }

    Ok(())
}

async fn run_reset(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    steps: &str,
    dry_run: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;
    let steps: usize = steps.parse().unwrap_or(1);

    let path = build_query("/v1/hlt/history", &project_root, continuity_id);
    let url = format!("{}&limit={}", path, steps.min(10));
    let response: Value = api.get(&url).await?;

    let entries = response
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("No history entries"))?;

    if entries.len() < 2 {
        anyhow::bail!("No previous HLT to reset to");
    }

    let target_idx = entries
        .len()
        .saturating_sub(1 + steps.min(entries.len() - 1));
    let target_entry = &entries[target_idx];

    let prev_hlt = target_entry
        .get("old_hlt")
        .and_then(|v| v.as_str())
        .unwrap_or("(initial)");
    let new_hlt = target_entry
        .get("new_hlt")
        .and_then(|v| v.as_str())
        .unwrap_or("(none)");

    println!("Reset HLT for: {}", project_root);
    println!("  Current: {}", wrap_text(new_hlt, 60));
    println!(
        "  Reset to ({} steps back): {}",
        steps,
        wrap_text(prev_hlt, 60)
    );

    if dry_run {
        println!("[dry-run] Would reset HLT");
        return Ok(());
    }

    if prev_hlt == "(initial)" || prev_hlt == "(none)" {
        anyhow::bail!("Cannot reset to initial (no previous HLT)");
    }

    let mut body = serde_json::json!({
        "project_root": project_root,
        "long_term_goal": prev_hlt,
        "desired_end_state": "Reset to previous HLT via CLI",
        "goal_source": "cli_reset",
        "operator_confirmed": true,
    });
    if let Some(cid) = continuity_id {
        body["continuity_id"] = serde_json::json!(cid);
    }

    let response: Value = api.post("/v1/trajectory/define-goal", &body).await?;
    let persisted = response
        .get("persisted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if persisted {
        println!("✓ HLT reset successfully");
    }

    Ok(())
}

async fn run_history(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    session_id: Option<&str>,
    limit: &str,
    json_output: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;
    let limit: usize = limit.parse().unwrap_or(20);

    let path = build_query("/v1/hlt/history", &project_root, continuity_id);
    let mut url = format!("{}&limit={}", path, limit);
    if let Some(sid) = session_id {
        url = format!("{}&session_id={}", url, sid);
    }
    let response: Value = api.get(&url).await?;
    let ledger_file = response
        .get("ledger_file")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let count = response.get("count").and_then(|v| v.as_u64()).unwrap_or(0);

    if json_output {
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Ok(());
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Trajectory History (append-only ledger)                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Project: {}", project_root);
    if let Some(cid) = continuity_id {
        println!("  Continuity: {}", cid);
    }
    println!("  Entries: {}", count);
    println!("  Ledger: {}", ledger_file);
    println!();
    println!("{}", "─".repeat(72));

    if let Some(entries) = response.get("entries").and_then(|v| v.as_array()) {
        for (i, entry) in entries.iter().enumerate() {
            let old_hlt = entry
                .get("old_hlt")
                .and_then(|v| v.as_str())
                .unwrap_or("(initial)");
            let new_hlt = entry
                .get("new_hlt")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let ts = entry
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let source = entry
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let entry_num = entries.len() - i;
            println!();
            println!("  [{}] {}", entry_num, ts);
            println!("    old: {}", wrap_text(old_hlt, 66));
            println!("    new: {}", wrap_text(new_hlt, 66));
            println!("    src: {}", source);
        }
    }

    println!();
    println!("{}", "─".repeat(72));

    Ok(())
}

async fn run_sessions(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    limit: &str,
    json_output: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;
    let limit: usize = limit.parse().unwrap_or(50);
    // Fetch all entries to extract distinct sessions.
    let path = build_query("/v1/hlt/history", &project_root, continuity_id);
    let url = format!("{}&limit={}", path, limit);
    let response: Value = api.get(&url).await?;
    let entries = response
        .get("entries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    // Collect distinct session_ids with their latest entry.
    let mut seen: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for entry in &entries {
        if let Some(sid) = entry.get("session_id").and_then(|v| v.as_str()) {
            if !sid.is_empty() {
                seen.entry(sid.to_string()).or_insert_with(|| entry.clone());
            }
        }
    }
    let sessions: Vec<Value> = seen.into_values().collect();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "completed",
                "project_root": project_root,
                "continuity_id": continuity_id,
                "count": sessions.len(),
                "sessions": sessions,
            }))?
        );
        return Ok(());
    }
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  HLT History Sessions                                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Project: {}", project_root);
    println!("  Sessions: {}", sessions.len());
    println!();
    for session in &sessions {
        let sid = session
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let hlt = session
            .get("new_hlt")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let ts = session
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        println!("  {} │ {} │ {}", sid, &hlt[..hlt.len().min(50)], ts);
    }
    println!();
    Ok(())
}

async fn run_fallback(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    session_id: Option<&str>,
    json_output: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;
    let path = build_query("/v1/hlt/history", &project_root, continuity_id);
    let mut url = format!("{}&limit=50", path);
    if let Some(sid) = session_id {
        url = format!("{}&session_id={}", url, sid);
    }
    let response: Value = api.get(&url).await?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "completed",
                "project_root": project_root,
                "continuity_id": continuity_id,
                "session_id": session_id,
                "fallback_candidates": response.get("fallback_candidates"),
                "latest_valid_for_session": response.get("latest_valid_for_session"),
                "latest_valid_for_continuity": response.get("latest_valid_for_continuity"),
                "latest_valid_for_project": response.get("latest_valid_for_project"),
                "generic_skipped": response.get("generic_skipped"),
                "warnings": response.get("warnings"),
            }))?
        );
        return Ok(());
    }
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  HLT Fallback Candidates                                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Project: {}", project_root);
    if let Some(cid) = continuity_id {
        println!("  Continuity: {}", cid);
    }
    if let Some(sid) = session_id {
        println!("  Session: {}", sid);
    }
    println!();
    let print_latest = |label: &str, val: &Value| match val
        .get("latest_valid_for_".to_string() + label)
        .or_else(|| val.as_str().map(|_| val))
    {
        Some(v) if !v.is_null() => {
            let hlt = v.as_str().unwrap_or("?");
            println!("  Latest valid HLT for {}: {}", label, hlt);
        }
        _ => println!("  Latest valid HLT for {}: (none)", label),
    };
    print_latest("session", &response);
    print_latest("continuity", &response);
    print_latest("project", &response);
    println!();
    if let Some(candidates) = response
        .get("fallback_candidates")
        .and_then(|v| v.as_array())
    {
        if candidates.is_empty() {
            println!("  Fallback: unavailable");
        } else {
            for c in candidates {
                let kind = c.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                let hlt = c.get("hlt").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  Fallback candidate ({}): {}", kind, hlt);
            }
        }
    }
    if let Some(skipped) = response.get("generic_skipped").and_then(|v| v.as_u64()) {
        if skipped > 0 {
            println!("  Generic HLT entries skipped: {}", skipped);
        }
    }
    if let Some(warnings) = response.get("warnings").and_then(|v| v.as_array()) {
        for w in warnings {
            println!("  Warning: {}", w.as_str().unwrap_or("?"));
        }
    }
    println!();
    Ok(())
}

async fn run_diff(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    from: &str,
    to: &str,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;
    let from: usize = from.parse().unwrap_or(1);
    let to: usize = to.parse().unwrap_or(0);

    let limit = from.max(to + 1).max(2);
    let path = build_query("/v1/hlt/history", &project_root, continuity_id);
    let url = format!("{}&limit={}", path, limit);
    let response: Value = api.get(&url).await?;

    let entries = response
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("No history entries"))?;

    let from_entry = entries
        .get(entries.len().saturating_sub(from))
        .ok_or_else(|| anyhow::anyhow!("Entry index {} not found", from))?;
    let to_entry = if to == 0 {
        entries.last()
    } else {
        entries.get(entries.len().saturating_sub(to))
    }
    .ok_or_else(|| anyhow::anyhow!("Entry index {} not found", to))?;

    let from_hlt = from_entry
        .get("new_hlt")
        .and_then(|v| v.as_str())
        .unwrap_or("(none)");
    let to_hlt = to_entry
        .get("new_hlt")
        .and_then(|v| v.as_str())
        .unwrap_or("(none)");
    let from_ts = from_entry
        .get("timestamp")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let to_ts = to_entry
        .get("timestamp")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("Trajectory Diff: Entry {} vs Entry {} (latest)", from, to);
    println!();
    println!("  Entry {} ({}):", from, from_ts);
    println!("    HLT: {}", wrap_text(from_hlt, 60));
    println!();
    let to_label = if to == 0 { "latest" } else { &to.to_string() };
    println!("  Entry {} ({}):", to_label, to_ts);
    println!("    HLT: {}", wrap_text(to_hlt, 60));
    println!();

    if from_hlt != to_hlt {
        println!("  ✗ HLT changed between entries");
    } else {
        println!("  ✓ HLT unchanged");
    }

    Ok(())
}

async fn run_mlg(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    set: Option<&str>,
    list: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    if list {
        let path = build_query("/v1/trajectory/view", &project_root, continuity_id);
        let url = format!("{}&mode=summary", path);
        let response: Value = api.get(&url).await?;
        let mlg = response
            .pointer("/trajectory/mid_level_goal")
            .and_then(|v| v.as_str())
            .unwrap_or("(not set)");
        println!("MLG for {}: {}", project_root, mlg);
        return Ok(());
    }

    if let Some(set_mlg) = set {
        // Read current trajectory to get HLT
        let view_url = format!(
            "/v1/trajectory/view?project_root={}",
            urlencoding::encode(&project_root)
        );
        let view_response = api.get(&view_url).await;
        let current_hlt = view_response
            .as_ref()
            .ok()
            .and_then(|v: &Value| v.pointer("/trajectory/long_term_goal"))
            .and_then(|v| v.as_str())
            .unwrap_or("Current trajectory");

        let mut body = serde_json::json!({
            "project_root": project_root,
            "long_term_goal": current_hlt,
            "mid_level_goal": set_mlg,
            "desired_end_state": "Updated via CLI",
            "goal_source": "cli",
            "operator_confirmed": true,
        });
        if let Some(cid) = continuity_id {
            body["continuity_id"] = serde_json::json!(cid);
        }
        let _: Value = api.post("/v1/trajectory/define-goal", &body).await?;
        println!("✓ MLG set: {}", set_mlg);
        return Ok(());
    }

    println!("Use --set <value> or --list");
    Ok(())
}

async fn run_stg(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    set: Option<&str>,
    list: bool,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    if list {
        let path = build_query("/v1/trajectory/view", &project_root, continuity_id);
        let url = format!("{}&mode=summary", path);
        let response: Value = api.get(&url).await?;
        let stg = response
            .pointer("/trajectory/short_term_goal")
            .and_then(|v| v.as_str())
            .unwrap_or("(not set)");
        println!("STG for {}: {}", project_root, stg);
        return Ok(());
    }

    if let Some(set_stg) = set {
        // Read current trajectory to get HLT and MLG
        let view_url = format!(
            "/v1/trajectory/view?project_root={}",
            urlencoding::encode(&project_root)
        );
        let view_response = api.get(&view_url).await;
        let current_hlt = view_response
            .as_ref()
            .ok()
            .and_then(|v: &Value| v.pointer("/trajectory/long_term_goal"))
            .and_then(|v| v.as_str())
            .unwrap_or("Current trajectory");
        let current_mlg = view_response
            .as_ref()
            .ok()
            .and_then(|v: &Value| v.pointer("/trajectory/mid_level_goal"))
            .and_then(|v| v.as_str());

        let mut body = serde_json::json!({
            "project_root": project_root,
            "long_term_goal": current_hlt,
            "short_term_goal": set_stg,
            "desired_end_state": "Updated via CLI",
            "goal_source": "cli",
            "operator_confirmed": true,
        });
        if let Some(mlg) = current_mlg {
            body["mid_level_goal"] = serde_json::json!(mlg);
        }
        if let Some(cid) = continuity_id {
            body["continuity_id"] = serde_json::json!(cid);
        }
        let _: Value = api.post("/v1/trajectory/define-goal", &body).await?;
        println!("✓ STG set: {}", set_stg);
        return Ok(());
    }

    println!("Use --set <value> or --list");
    Ok(())
}

async fn run_waypoint(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    add: Option<&str>,
    list: bool,
    _complete: Option<&str>,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    let path = build_query("/v1/trajectory/view", &project_root, continuity_id);
    let url = format!("{}&mode=summary", path);
    let response: Value = api.get(&url).await?;
    let waypoints = response
        .get("waypoints")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if list {
        println!("Waypoints for {}:", project_root);
        if waypoints.is_empty() {
            println!("  (none)");
        } else {
            for (i, wp) in waypoints.iter().enumerate() {
                println!("  {}. {}", i + 1, wp);
            }
        }
        return Ok(());
    }

    if let Some(add_wp) = add {
        // Read current trajectory to get HLT and desired_end_state for the request
        let view_url = build_query("/v1/trajectory/view", &project_root, continuity_id);
        let current_trajectory = api.get(&view_url).await.ok();
        let current_hlt = current_trajectory
            .as_ref()
            .and_then(|v| v.pointer("/trajectory/long_term_goal"))
            .and_then(|v| v.as_str())
            .unwrap_or("Current trajectory");
        let current_desired = current_trajectory
            .as_ref()
            .and_then(|v| v.pointer("/trajectory/desired_end_state"))
            .and_then(|v| v.as_str())
            .unwrap_or("Updated via CLI");

        let mut all_wps = waypoints.clone();
        all_wps.push(add_wp.to_string());
        let mut body = serde_json::json!({
            "project_root": project_root,
            "long_term_goal": current_hlt,
            "desired_end_state": current_desired,
            "waypoints": all_wps,
            "goal_source": "cli",
            "operator_confirmed": true,
        });
        if let Some(cid) = continuity_id {
            body["continuity_id"] = serde_json::json!(cid);
        }
        let _: Value = api.post("/v1/trajectory/define-goal", &body).await?;
        println!("✓ Waypoint added: {}", add_wp);
        return Ok(());
    }

    println!("Use --add <value> or --list");
    Ok(())
}

async fn run_assess(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    current_state: &str,
    evidence_refs: Vec<String>,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    let mut body = serde_json::json!({
        "project_root": project_root,
        "observed_state": current_state,
        "evidence_refs": evidence_refs,
    });
    if let Some(cid) = continuity_id {
        body["continuity_id"] = serde_json::json!(cid);
    }

    let response: Value = api.post("/v1/trajectory/assess", &body).await?;

    println!("Assessment for: {}", project_root);
    println!();
    println!("  Current State: {}", wrap_text(current_state, 60));

    if let Some(gap) = response
        .get("gaps")
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_str())
    {
        println!();
        println!("  Gap: {}", wrap_text(gap, 60));
    }

    if let Some(posture) = response.get("recommended_action").and_then(|v| v.as_str()) {
        println!();
        println!("  Posture: {}", posture);
    }

    Ok(())
}

async fn run_checkpoint(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    summary: Option<&str>,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    let mut body = serde_json::json!({
        "project_root": project_root,
        "summary": summary.unwrap_or("CLI checkpoint"),
    });
    if let Some(cid) = continuity_id {
        body["continuity_id"] = serde_json::json!(cid);
    }

    let response: Value = api.post("/v1/trajectory/checkpoint", &body).await?;
    let traj_id = response
        .get("trajectory_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("✓ Trajectory checkpointed: {}", traj_id);
    Ok(())
}

async fn run_supersede(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    new_hlt: &str,
    evidence_refs: Vec<String>,
    reason: &str,
    continuity_id: Option<&str>,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    if evidence_refs.is_empty() {
        anyhow::bail!("Supersession requires evidence refs (per §5)");
    }

    let mut body = serde_json::json!({
        "project_root": project_root,
        "long_term_goal": new_hlt,
        "desired_end_state": reason,
        "goal_source": "supersession",
        "operator_confirmed": true,
        "supersession_evidence_refs": evidence_refs,
    });
    if let Some(cid) = continuity_id {
        body["continuity_id"] = serde_json::json!(cid);
    }

    let response: Value = api.post("/v1/trajectory/define-goal", &body).await?;
    let persisted = response
        .get("persisted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if persisted {
        println!("✓ HLT superseded with evidence");
    } else {
        println!("⚠ Supersession failed");
    }

    Ok(())
}

async fn run_export(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
    output: &str,
    format: &str,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    let path = build_query("/v1/hlt/history", &project_root, continuity_id);
    let url = format!("{}&limit=500", path);
    let response: Value = api.get(&url).await?;

    let content = match format {
        "json" => serde_json::to_string_pretty(&response)?,
        "markdown" => {
            let entries = response.get("entries").and_then(|v| v.as_array());
            let mut md = format!("# Trajectory History\n\nProject: {}\n\n", project_root);
            if let Some(entries) = entries {
                for entry in entries {
                    let ts = entry
                        .get("timestamp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let old_hlt = entry
                        .get("old_hlt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("(initial)");
                    let new_hlt = entry.get("new_hlt").and_then(|v| v.as_str()).unwrap_or("");
                    let source = entry.get("source").and_then(|v| v.as_str()).unwrap_or("");
                    md.push_str(&format!(
                        "## {}\n\n**Source:** {}\n\n- **Old:** {}\n- **New:** {}\n\n",
                        ts, source, old_hlt, new_hlt
                    ));
                }
            }
            md
        }
        _ => {
            let entries = response.get("entries").and_then(|v| v.as_array());
            entries
                .map(|e| {
                    e.iter()
                        .filter_map(|v| serde_json::to_string(v).ok())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default()
        }
    };

    std::fs::write(output, content)?;
    println!("✓ Exported to: {}", output);
    Ok(())
}

async fn run_verify(
    api: &ApiClient,
    cwd: &str,
    project_root: Option<&str>,
    continuity_id: Option<&str>,
) -> anyhow::Result<()> {
    let project_root = get_project_root(project_root, cwd)?;

    println!("Verifying trajectory for: {}", project_root);
    println!();

    // Check ledger
    let ledger_path = build_query("/v1/hlt/history", &project_root, continuity_id);
    let ledger_url = format!("{}&limit=1", ledger_path);
    let ledger_response: Value = api.get(&ledger_url).await?;
    let ledger_file = ledger_response
        .get("ledger_file")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let count = ledger_response
        .get("count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    println!("  Ledger file: {}", ledger_file);
    println!("  Entry count: {}", count);

    // Check trajectory consistency
    let traj_path = build_query("/v1/trajectory/view", &project_root, continuity_id);
    let traj_url = format!("{}&mode=summary", traj_path);
    let traj_response: Value = api.get(&traj_url).await?;
    let current_hlt = traj_response
        .get("long_term_goal")
        .and_then(|v| v.as_str())
        .unwrap_or("(not set)");

    println!();
    println!("  Current HLT: {}", wrap_text(current_hlt, 50));

    // Verify match
    if let Some(latest) = ledger_response
        .get("entries")
        .and_then(|v| v.as_array())
        .and_then(|entries| entries.last())
    {
        let ledger_hlt = latest.get("new_hlt").and_then(|v| v.as_str()).unwrap_or("");
        let matches = ledger_hlt == current_hlt;
        println!();
        println!(
            "  Ledger ↔ Trajectory: {}",
            if matches { "✓" } else { "⚠ mismatch" }
        );
    }

    println!();
    println!(
        "✓ Verification complete (scope: {})",
        if continuity_id.is_some() {
            "continuity"
        } else {
            "project"
        }
    );

    Ok(())
}

fn wrap_text(text: &str, width: usize) -> String {
    if text.len() <= width {
        return text.to_string();
    }
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() < width {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        } else {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
            }
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.len() == 1 {
        return lines[0].clone();
    }

    lines.join(&format!("\n{}", " ".repeat(7)))
}
