//! GET /v1/health

use crate::routes::bounded::resource_mode_status;
use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "daemon": &state.daemon_runtime_identity.process,
        "uptime_ms": state.started_at.elapsed().as_millis() as u64,
        "persistence": state.persistence_actor.as_ref().map(|actor| actor.metrics()),
    }))
}

async fn about(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "schema": "focusa.about.v1",
        "project": "Focusa",
        "version": env!("CARGO_PKG_VERSION"),
        "one_line": "Focusa turns long AI chat into long-running AI project work.",
        "quickstart": {
            "summary": "Three commands to a green Focusa install on this host.",
            "commands": [
                "bash scripts/install-daemon.sh /usr/local",
                "focusa start && sleep 2",
                "focusa init --quickstart"
            ],
        },
        "interactive_first_run": [
            "focusa onboard",
            "focusa init --quickstart",
        ],
        "uptime_ms": state.started_at.elapsed().as_millis() as u64,
        "next_commands": {
            "init": "focusa init [--quickstart] [--project-root PATH]",
            "onboard": "focusa onboard [--scope project|host] [--remote <git-url>]",
            "tui": "focusa tui [--headless-self-test]",
            "doctor": "focusa doctor",
            "audit": "focusa audit-failure-summary",
            "pi_install": "bash scripts/install-pi-skill.sh",
        },
    }))
}

const BUNDLED_TOOL_CONTRACTS_JSON: &str =
    include_str!("../../../../docs/current/focusa-tool-contracts.json");

fn bundled_tool_contract_count() -> usize {
    serde_json::from_str::<Value>(BUNDLED_TOOL_CONTRACTS_JSON)
        .ok()
        .and_then(|registry| {
            registry
                .get("contracts")
                .and_then(Value::as_array)
                .map(Vec::len)
                .or_else(|| {
                    registry
                        .get("tool_count")
                        .and_then(Value::as_u64)
                        .map(|count| count as usize)
                })
        })
        .unwrap_or(0)
}

fn path_has_command(command: &str) -> bool {
    let path_hit = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| command_exists_in_dir(&dir, command)))
        .unwrap_or(false);
    if path_hit {
        return true;
    }
    common_command_paths(command)
        .iter()
        .any(|path| Path::new(path).is_file())
}

fn common_command_paths(command: &str) -> &'static [&'static str] {
    match command {
        "cargo" => &["/root/.cargo/bin/cargo", "/usr/local/cargo/bin/cargo"],
        "rustc" => &["/root/.cargo/bin/rustc", "/usr/local/cargo/bin/rustc"],
        "node" => &[
            "/opt/node-v22.22.3-linux-x64/bin/node",
            "/usr/local/bin/node",
            "/usr/bin/node",
        ],
        "npm" => &[
            "/opt/node-v22.22.3-linux-x64/bin/npm",
            "/usr/local/bin/npm",
            "/usr/bin/npm",
        ],
        "gh" => &["/usr/bin/gh", "/usr/local/bin/gh"],
        _ => &[],
    }
}

fn command_exists_in_dir(dir: &Path, command: &str) -> bool {
    let candidate = dir.join(command);
    candidate.is_file()
        || (cfg!(windows)
            && ["exe", "cmd", "bat", "ps1"]
                .iter()
                .any(|ext| dir.join(format!("{command}.{ext}")).is_file()))
}

fn portability_tool(name: &str, required_for: &str, note: &str) -> Value {
    let available = path_has_command(name);
    json!({
        "name": name,
        "available": available,
        "status": if available { "ok" } else { "missing" },
        "required_for": required_for,
        "note": note,
    })
}

fn portability_doctor_payload() -> Value {
    let tools = vec![
        portability_tool("focusa-daemon", "runtime", "daemon/API binary on PATH"),
        portability_tool(
            "cargo",
            "source_build",
            "Rust source builds and maintainer gates",
        ),
        portability_tool("rustc", "source_build", "Rust compiler for source builds"),
        portability_tool(
            "node",
            "pi_extension_menubar",
            "Pi extension and menubar checks require Node >=20",
        ),
        portability_tool("npm", "menubar", "menubar npm install/check/build path"),
        portability_tool(
            "bash",
            "scripts",
            "spec/release helper scripts assume POSIX shell",
        ),
        portability_tool("curl", "api_scripts", "API smoke and live proof scripts"),
        portability_tool("jq", "api_scripts", "JSON assertions in scripts and docs"),
        portability_tool(
            "python3",
            "spec_gates",
            "API contract probe and helper scripts",
        ),
        portability_tool(
            "rg",
            "developer_gates",
            "fast static checks and grep-style gates",
        ),
        portability_tool(
            "gh",
            "maintainer_release",
            "GitHub CI/release inspection via CLI",
        ),
    ];
    let missing_runtime: Vec<String> = tools
        .iter()
        .filter(|tool| {
            tool.get("available").and_then(Value::as_bool) == Some(false)
                && tool.get("required_for").and_then(Value::as_str) == Some("runtime")
        })
        .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    let missing_source_build: Vec<String> = tools
        .iter()
        .filter(|tool| {
            tool.get("available").and_then(Value::as_bool) == Some(false)
                && tool.get("required_for").and_then(Value::as_str) == Some("source_build")
        })
        .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    let missing_helpers: Vec<String> = tools
        .iter()
        .filter(|tool| tool.get("available").and_then(Value::as_bool) == Some(false))
        .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    json!({
        "status": if missing_runtime.is_empty() { "ok" } else { "warn" },
        "host_os": std::env::consts::OS,
        "host_arch": std::env::consts::ARCH,
        "path_entries": std::env::var_os("PATH").map(|paths| std::env::split_paths(&paths).count()).unwrap_or(0),
        "tools": tools,
        "missing_runtime": missing_runtime,
        "missing_source_build": missing_source_build,
        "missing_helpers": missing_helpers,
        "source": "docs/current/PORTABILITY_AUDIT.md",
        "note": "Availability is PATH-based and non-spawning; source-build and maintainer helpers are informational for binary runtime installs.",
    })
}

async fn doctor(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let resource_mode = resource_mode_status();
    let portability = portability_doctor_payload();
    let s = state.focusa.read().await;
    let token_records = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_token_budget")
        })
        .count();
    let cache_records = s
        .telemetry
        .trace_events
        .iter()
        .filter(|event| {
            event.get("event_type").and_then(|v| v.as_str()) == Some("spec92_cache_metadata")
        })
        .count();
    let active_frame = s
        .focus_stack
        .active_id
        .and_then(|id| s.focus_stack.frames.iter().find(|frame| frame.id == id));
    let perf = &state.supervisor_perf;
    let driver_start_attempts = perf.driver_start_attempts.load(Ordering::Relaxed);
    let driver_stop_attempts = perf.driver_stop_attempts.load(Ordering::Relaxed);
    let dispatch_recovery_restarts = perf.dispatch_recovery_restarts.load(Ordering::Relaxed);
    let current_task_present = s.work_loop.current_task.is_some();
    let idle_without_task =
        s.work_loop.status == focusa_core::types::WorkLoopStatus::Idle && !current_task_present;
    let churn_risk = idle_without_task && (driver_start_attempts > 0 || driver_stop_attempts > 0);
    let active_workpoint = s.workpoint.active_workpoint_id.and_then(|id| {
        s.workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == id)
    });
    let base_product =
        focusa_license::base_product_projection(state.license_guard.entitlement.as_ref()).ok();
    let base_mutations_allowed = base_product
        .as_ref()
        .is_some_and(|projection| projection.permits_base_mutations);
    let project_scope_ready = base_mutations_allowed
        && active_workpoint.is_some_and(|record| {
            record.canonical
                && record
                    .project_root
                    .as_deref()
                    .is_some_and(|root| !root.trim().is_empty())
                && record
                    .continuity_id
                    .as_deref()
                    .is_some_and(|continuity| !continuity.trim().is_empty())
        });
    let readiness_categories = json!({
        "runtime_readiness": {
            "status": if base_mutations_allowed { "ready" } else { "blocked" },
            "reason": if base_mutations_allowed { "daemon is reachable and base mutations are allowed" } else { "daemon is reachable but signed authority blocks base mutations" },
            "scope": "runtime",
            "process": &state.daemon_runtime_identity.process,
            "data_store": crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir),
            "base_product": base_product,
        },
        "project_scope_readiness": {
            "status": if project_scope_ready { "ready" } else { "blocked" },
            "reason": if project_scope_ready { "canonical Workpoint, project root, continuity, and write authority agree" } else { "project readiness requires write authority plus one canonical Workpoint with project root and continuity" },
            "scope": "project_root_plus_continuity_id",
        },
        "workpoint_readiness": {
            "status": if project_scope_ready { "ready" } else { "blocked" },
            "reason": if project_scope_ready { "active canonical Workpoint agrees with project scope" } else { "active Workpoint is missing, noncanonical, incomplete, or blocked by authority" },
            "workpoint_id": active_workpoint.map(|record| record.workpoint_id),
        },
        "trajectory_readiness": {
            "status": if base_mutations_allowed && s.trajectory.active_trajectory_id.is_some() { "ready" } else { "blocked" },
            "reason": if !base_mutations_allowed { "trajectory mutations are blocked by signed authority" } else if s.trajectory.active_trajectory_id.is_some() { "active trajectory id present" } else { "active trajectory is missing" },
            "trajectory_id": s.trajectory.active_trajectory_id,
        },
        "focus_state_readiness": {
            "status": if active_frame.is_some() { "ready" } else { "not_checked" },
            "reason": if active_frame.is_some() { "active Focus frame present" } else { "Focus State frame not required for daemon runtime readiness" },
        },
        "source_build_readiness": {
            "status": if portability.get("missing_source_build").and_then(Value::as_array).is_some_and(|items| items.is_empty()) { "ready" } else { "blocked" },
            "reason": "PATH-based cargo/rustc source-build plane; separate from daemon runtime readiness",
            "missing": portability.get("missing_source_build").cloned().unwrap_or_else(|| json!([])),
        },
        "release_readiness": {
            "status": if portability.get("missing_helpers").and_then(Value::as_array).is_some_and(|items| items.is_empty()) { "ready" } else { "partial" },
            "reason": "release/helper tooling plane; separate from daemon runtime readiness",
            "missing": portability.get("missing_helpers").cloned().unwrap_or_else(|| json!([])),
        },
        "telemetry_readiness": {
            "status": if token_records > 0 || cache_records > 0 { "ready" } else { "not_checked" },
            "reason": if token_records > 0 || cache_records > 0 { "token/cache telemetry records present" } else { "run a Pi/provider turn to populate token/cache telemetry" },
        },
        "ui_browser_readiness": {
            "status": "not_checked",
            "reason": "UIAI browser plane is checked by Pi/UIAI health, not daemon runtime doctor",
        },
    });
    Json(json!({
        "status": if base_mutations_allowed { "ok" } else { "blocked" },
        "summary": if base_mutations_allowed { "Focusa daemon is reachable and permits base mutations" } else { "Focusa daemon is reachable but not operational for governed work" },
        "readiness_categories": readiness_categories,
        "daemon": {
            "ok": true,
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_ms": state.started_at.elapsed().as_millis() as u64,
        },
        "focus": {
            "active_frame_id": active_frame.map(|frame| frame.id.to_string()),
            "active_frame_title": active_frame.map(|frame| frame.title.clone()),
            "stack_depth": s.focus_stack.frames.len(),
        },
        "telemetry": {
            "total_events": s.telemetry.total_events,
            "token_budget_records": token_records,
            "cache_metadata_records": cache_records,
            "tool_calls": s.telemetry.tool_calls.len(),
        },
        "work_loop": {
            "enabled": s.work_loop.enabled,
            "status": s.work_loop.status,
            "current_task_present": current_task_present,
            "supervisor_perf": {
                "supervisor_ticks_total": perf.ticks_total.load(Ordering::Relaxed),
                "driver_start_attempts": driver_start_attempts,
                "driver_stop_attempts": driver_stop_attempts,
                "dispatch_attempts": perf.dispatch_attempts.load(Ordering::Relaxed),
                "dispatch_skipped_disallowed": perf.dispatch_skipped_disallowed.load(Ordering::Relaxed),
                "dispatch_recovery_restarts": dispatch_recovery_restarts,
                "background_throttled_ticks": perf.background_throttled_ticks.load(Ordering::Relaxed),
            },
            "churn_diagnostic": {
                "status": if churn_risk { "warn" } else { "ok" },
                "risk": churn_risk,
                "reason": if churn_risk { "pi-rpc supervisor driver counters changed while work-loop is idle with no current task" } else { "no idle/no-task driver churn detected" },
                "recommended_action": if churn_risk { "inspect /v1/work-loop/status?summary_only=true, stop stale driver if present, and verify idle start gate" } else { "continue normally" },
            },
            // BAD-003 fix: Expose top drift causes when drift is detected
            "drift": {
                "status": if s.workpoint.drift_events.is_empty() { "ok" } else { "warn" },
                "recent_count": s.workpoint.drift_events.len(),
                "top_causes": s.workpoint.drift_events.iter().rev().take(3).map(|event| {
                    json!({
                        "reason": event.reason,
                        "severity": format!("{:?}", event.severity),
                        "recovery_hint": event.recovery_hint.clone().unwrap_or_default(),
                    })
                }).collect::<Vec<_>>(),
                "next_action": if s.workpoint.drift_events.is_empty() {
                    "no drift events recorded"
                } else {
                    "inspect top drift causes and apply recovery_hint; if persistent, run focusa_workpoint_resume to re-align canonical packet"
                },
            }
        },
        "portability": portability,
        "api_cli_parity": {
            "cli_command": "focusa doctor --json",
            "api_route": "/v1/doctor",
            "shared_checks": [
                "daemon health",
                "command-center doctor API",
                "API route inventory surface",
                "Spec90 tool contracts",
                "Workpoint canonicality",
                "Work-loop writer state",
                "token telemetry status",
                "cache metadata status",
                "portability PATH checklist"
            ],
            "recovery_commands": [
                "focusa start",
                "focusa start",
                "focusa-daemon",
                "journalctl -u focusa-daemon -n 80 --no-pager (Linux service installs)"
            ],
            "status_fields": ["status", "summary", "next_action", "why", "commands", "recovery", "details.checks"]
        },
        "checks_summary": {
            "contracts_expected": bundled_tool_contract_count(),
            "scoped_hot_routes": ["/v1/health", "/v1/doctor", "/v1/workpoint/current", "/v1/work-loop/status?summary_only=true"],
            "docs": ["docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md", "docs/current/CLI_REFERENCE_CURRENT.md"]
        },
        "resource_mode": {
            "mode": resource_mode.mode,
            "reason": resource_mode.reason,
            "forced": resource_mode.forced,
            "pressure": resource_mode.pressure,
            "budget": resource_mode.budget,
            "latest_transition": resource_mode.latest_transition,
            "transition_omitted_count": resource_mode.transition_omitted_count,
            "hysteresis": resource_mode.hysteresis,
            "tool_availability_policy": resource_mode.tool_availability_policy,
            "cold_surfaces_deferred": resource_mode.cold_surfaces_deferred,
        },
        "next_action": if churn_risk { "inspect work-loop supervisor churn before broad work" } else if portability.get("status").and_then(Value::as_str) == Some("warn") { "install missing runtime tools from doctor portability checklist" } else if token_records == 0 || cache_records == 0 { "run a Pi/provider turn, then re-run focusa doctor" } else { "continue normally; use focusa telemetry token-budget and focusa cache doctor for detail" },
        "commands": ["focusa resource status", "focusa telemetry token-budget", "focusa cache doctor", "focusa work-loop status", "focusa workpoint current", "focusa doctor --json | jq .portability"],
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/about", get(about))
        .route("/v1/doctor", get(doctor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_contract_count_matches_bundled_registry() {
        let registry: Value = serde_json::from_str(BUNDLED_TOOL_CONTRACTS_JSON)
            .expect("bundled tool contract registry must be valid JSON");
        let contracts = registry
            .get("contracts")
            .and_then(Value::as_array)
            .expect("bundled tool contract registry must contain contracts");
        let metadata_count = registry
            .get("tool_count")
            .and_then(Value::as_u64)
            .expect("bundled tool contract registry must contain tool_count");

        assert!(
            !contracts.is_empty(),
            "bundled tool contract registry must not be empty"
        );
        assert_eq!(bundled_tool_contract_count(), contracts.len());
        assert_eq!(metadata_count as usize, contracts.len());
    }
}
