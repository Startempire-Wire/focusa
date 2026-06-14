//! POST /v1/call-stack/design — Spec 103 Call Stack Architecture Blueprint.
//!
//! Returns a typed, append-only `CallStackDesign` envelope. The tool does
//! not invent feature-specific details; it returns the standard Focusa call
//! stack scaffold that the operator/agent fills in for the specific feature.

use crate::routes::project::project_identity_payload_for_scope;
use crate::server::AppState;
use axum::{Json, extract::{Query, State}, http::StatusCode};
use focusa_core::types::CallStackDesign;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{fs, path::Path, sync::Arc};

const MISSION_MAX_CHARS: usize = 200;
const NOTES_MAX_CHARS: usize = 2048;
const ENTRY_NAME_MAX_CHARS: usize = 120;
const ENTRY_SURFACE_ALLOWED: &[&str] = &["pi_tool", "cli_command", "http_route"];

#[derive(Debug, Deserialize)]
pub struct CallStackDesignRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub mission: Option<String>,
    pub entry_surface: Option<String>,
    pub entry_name: Option<String>,
    pub workpoint_id: Option<String>,
    pub attach_to_workpoint: Option<bool>,
    pub attach_to_stg: Option<bool>,
    pub parent_design_id: Option<String>,
    pub notes: Option<String>,
}

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/call-stack/design", axum::routing::post(design))
        .route("/v1/call-stack/verify", axum::routing::post(verify))
        .route("/v1/call-stack/list", axum::routing::get(list))
        .route("/v1/call-stack/show", axum::routing::get(show))
}

fn rejection(status: StatusCode, body: Value) -> (StatusCode, Json<Value>) {
    (status, Json(body))
}

fn standard_handlers() -> Vec<Value> {
    vec![
        json!({"name": "validation", "purpose": "input schema check"}),
        json!({"name": "scope_binding", "purpose": "project_root + continuity_id"}),
        json!({"name": "workpoint_link", "purpose": "attach evidence to active Workpoint"}),
    ]
}

fn standard_services() -> Vec<Value> {
    vec![
        json!({"name": "spec80_envelope", "purpose": "tool_result_v1 wrapper"}),
        json!({"name": "trajectory_assess", "purpose": "short-term-goal alignment"}),
    ]
}

fn standard_adapters() -> Vec<Value> {
    vec![
        json!({"name": "focusa_fetch", "purpose": "HTTP/JSON to daemon"}),
        json!({"name": "persistence_jsonl", "purpose": "append-only ledger"}),
    ]
}

async fn design(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CallStackDesignRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = body
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if project_root.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "project_root_missing",
                "field": "project_root",
                "message": "project_root is required",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
                "unsafe_reason": "agent_runtime_directory",
                "message": "agent runtime paths are not allowed as project_root",
            }),
        ));
    }

    let identity = project_identity_payload_for_scope(Some(project_root), Some(project_root));
    let identity_status = identity
        .get("project_identity")
        .and_then(|pi: &Value| pi.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if identity_status == "unsafe_project_root" {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "project_root_unverified",
                "field": "project_root",
                "rejected_value": project_root,
                "message": "project_root is not verified",
            }),
        ));
    }

    let mission = body
        .mission
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if mission.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "mission_missing",
                "field": "mission",
                "message": "mission is required",
            }),
        ));
    }
    if mission.chars().count() > MISSION_MAX_CHARS {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "mission_too_long",
                "field": "mission",
                "max_chars": MISSION_MAX_CHARS,
                "message": format!("mission must be <= {} chars", MISSION_MAX_CHARS),
            }),
        ));
    }

    let entry_surface = body
        .entry_surface
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("pi_tool");
    if !ENTRY_SURFACE_ALLOWED.contains(&entry_surface) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "entry_surface_invalid",
                "field": "entry_surface",
                "allowed": ENTRY_SURFACE_ALLOWED,
                "message": "entry_surface must be one of pi_tool|cli_command|http_route",
            }),
        ));
    }

    let entry_name = body
        .entry_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if entry_name.chars().count() > ENTRY_NAME_MAX_CHARS {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "entry_name_too_long",
                "field": "entry_name",
                "max_chars": ENTRY_NAME_MAX_CHARS,
                "message": format!("entry_name must be <= {} chars", ENTRY_NAME_MAX_CHARS),
            }),
        ));
    }
    if entry_name.is_empty() {
        // Derive a deterministic stub from the project + mission.
        let stub = format!("{}.{}", sanitize_stub(project_root), sanitize_stub(mission));
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "entry_name_missing",
                "field": "entry_name",
                "suggested_stub": stub,
                "message": "entry_name is required; suggested stub provided",
            }),
        ));
    }

    let notes = body
        .notes
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(n) = notes
        && n.chars().count() > NOTES_MAX_CHARS
    {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "notes_too_long",
                "field": "notes",
                "max_chars": NOTES_MAX_CHARS,
                "message": format!("notes must be <= {} chars", NOTES_MAX_CHARS),
            }),
        ));
    }

    let attach_to_workpoint = body.attach_to_workpoint.unwrap_or(false);
    let attach_to_stg = body.attach_to_stg.unwrap_or(false);

    if attach_to_workpoint {
        let has_workpoint = body
            .workpoint_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .is_some();
        if !has_workpoint {
            return Err(rejection(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "status": "validation_rejected",
                    "canonical": false,
                    "advisory": true,
                    "failure_class": "workpoint_unavailable",
                    "field": "workpoint_id",
                    "message": "attach_to_workpoint=true requires an explicit workpoint_id",
                }),
            ));
        }
    }
    if attach_to_stg && body.continuity_id.is_none() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "trajectory_unclear",
                "field": "continuity_id",
                "message": "attach_to_stg=true requires an explicit continuity_id",
            }),
        ));
    }

    let design_id = uuid::Uuid::now_v7().to_string();
    let design = CallStackDesign {
        timestamp: chrono::Utc::now(),
        design_id: design_id.clone(),
        parent_design_id: body
            .parent_design_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        project_root: project_root.to_string(),
        continuity_id: body
            .continuity_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        session_id: None,
        workpoint_id: body
            .workpoint_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        mission: mission.to_string(),
        entry_surface: entry_surface.to_string(),
        entry_name: entry_name.to_string(),
        attach_to_workpoint,
        attach_to_stg,
        notes: notes.map(str::to_string),
        handlers: vec![
            focusa_core::types::CallStackStep {
                name: "validation".to_string(),
                purpose: "input schema check".to_string(),
            },
            focusa_core::types::CallStackStep {
                name: "scope_binding".to_string(),
                purpose: "project_root + continuity_id".to_string(),
            },
            focusa_core::types::CallStackStep {
                name: "workpoint_link".to_string(),
                purpose: "attach evidence to active Workpoint".to_string(),
            },
        ],
        services: vec![
            focusa_core::types::CallStackStep {
                name: "spec80_envelope".to_string(),
                purpose: "tool_result_v1 wrapper".to_string(),
            },
            focusa_core::types::CallStackStep {
                name: "trajectory_assess".to_string(),
                purpose: "short-term-goal alignment".to_string(),
            },
        ],
        adapters: vec![
            focusa_core::types::CallStackStep {
                name: "focusa_fetch".to_string(),
                purpose: "HTTP/JSON to daemon".to_string(),
            },
            focusa_core::types::CallStackStep {
                name: "persistence_jsonl".to_string(),
                purpose: "append-only ledger".to_string(),
            },
        ],
        storage_kind: Some("jsonl".to_string()),
        storage_path: Some(format!(
            "data/call-stack-designs/{}/designs.jsonl",
            project_root_hash(project_root)
        )),
        output_envelope: Some("tool_result_v1".to_string()),
        evidence_refs: Vec::new(),
    };

    if let Err(e) = state.persistence.append_call_stack_design(&design) {
        return Err(rejection(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "status": "blocked",
                "canonical": false,
                "advisory": true,
                "failure_class": "storage_unwritable",
                "message": format!("append failed: {}", e),
            }),
        ));
    }

    let ledger_path = state
        .persistence
        .call_stack_designs_path_for_project(project_root);

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "failure_class": null,
        "scope_status": "matched",
        "design_id": design_id,
        "design": design,
        "handlers": standard_handlers(),
        "services": standard_services(),
        "adapters": standard_adapters(),
        "output_envelope": "tool_result_v1",
        "next_tools": [
            "focusa_call_stack_verify",
            "focusa_workpoint_link_evidence",
            "focusa_trajectory_assess"
        ],
        "rehydrate_id": design_id,
        "ledger_file": ledger_path.to_string_lossy(),
        "evidence_refs": []
    })))
}



#[derive(Debug, Deserialize)]
pub struct CallStackListQuery {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub entry_name: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CallStackShowQuery {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub design_id: String,
}

fn require_project_root_param(value: Option<&str>) -> Result<String, (StatusCode, Json<Value>)> {
    let project_root = value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            rejection(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "status": "validation_rejected",
                    "canonical": false,
                    "advisory": true,
                    "failure_class": "project_root_missing",
                    "field": "project_root",
                }),
            )
        })?;
    if is_unsafe_agent_runtime_path_inline(project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }
    Ok(project_root.to_string())
}

async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CallStackListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = require_project_root_param(query.project_root.as_deref())?;
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let designs = state
        .persistence
        .read_call_stack_designs(
            &project_root,
            query.continuity_id.as_deref(),
            query.entry_name.as_deref(),
            limit,
        )
        .map_err(|e| {
            rejection(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "status": "blocked",
                    "canonical": false,
                    "advisory": true,
                    "failure_class": "storage_unreadable",
                    "message": format!("read failed: {}", e),
                }),
            )
        })?;
    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": "matched",
        "project_root": project_root,
        "continuity_id": query.continuity_id,
        "entry_name": query.entry_name,
        "limit": limit,
        "count": designs.len(),
        "designs": designs,
        "next_tools": ["focusa_call_stack_verify", "focusa_call_stack_design"],
        "rehydrate_id": "call_stack_list",
    })))
}

async fn show(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CallStackShowQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = require_project_root_param(query.project_root.as_deref())?;
    let designs = state
        .persistence
        .read_call_stack_designs(&project_root, query.continuity_id.as_deref(), None, 100)
        .map_err(|e| {
            rejection(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "status": "blocked",
                    "canonical": false,
                    "advisory": true,
                    "failure_class": "storage_unreadable",
                    "message": format!("read failed: {}", e),
                }),
            )
        })?;
    let Some(design) = designs.into_iter().find(|d| d.design_id == query.design_id) else {
        return Err(rejection(
            StatusCode::NOT_FOUND,
            json!({
                "status": "not_found",
                "canonical": false,
                "advisory": true,
                "failure_class": "call_stack_design_not_found",
                "design_id": query.design_id,
            }),
        ));
    };
    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": "matched",
        "design_id": design.design_id,
        "design": design,
        "next_tools": ["focusa_call_stack_verify", "focusa_call_stack_design"],
        "rehydrate_id": format!("call_stack_show:{}", query.design_id),
    })))
}

#[derive(Debug, Deserialize)]
pub struct CallStackVerifyRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub design_id: Option<String>,
    pub entry_name: Option<String>,
}

fn call_stack_check(id: &str, status: &str, message: impl Into<String>) -> Value {
    json!({"id": id, "status": status, "message": message.into()})
}

fn source_contains(root: &str, rel_dir: &str, needle: &str, max_files: usize) -> bool {
    if needle.trim().is_empty() {
        return false;
    }
    let dir = Path::new(root).join(rel_dir);
    if !dir.exists() {
        return false;
    }
    let mut stack = vec![dir];
    let mut checked = 0usize;
    while let Some(path) = stack.pop() {
        let Ok(meta) = fs::metadata(&path) else { continue };
        if meta.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else { continue };
            for entry in entries.flatten() {
                let child = entry.path();
                let name = child.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(name, "target" | "node_modules" | ".git" | "data" | "ecs") {
                    continue;
                }
                stack.push(child);
            }
        } else if meta.is_file() {
            checked += 1;
            if checked > max_files {
                return false;
            }
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !matches!(ext, "rs" | "ts" | "tsx" | "js" | "md" | "toml" | "json") {
                continue;
            }
            if let Ok(body) = fs::read_to_string(&path) {
                if body.contains(needle) {
                    return true;
                }
            }
        }
    }
    false
}

fn entry_surface_exists(project_root: &str, design: &CallStackDesign) -> bool {
    match design.entry_surface.as_str() {
        "pi_tool" => source_contains(
            project_root,
            "apps/pi-extension/src",
            &format!("name: \"{}\"", design.entry_name),
            350,
        ),
        "cli_command" => source_contains(project_root, "crates/focusa-cli", &design.entry_name, 350),
        "http_route" => source_contains(project_root, "crates/focusa-api", &design.entry_name, 500),
        _ => false,
    }
}

async fn verify(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CallStackVerifyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = body
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            rejection(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "status": "validation_rejected",
                    "canonical": false,
                    "advisory": true,
                    "failure_class": "project_root_missing",
                    "field": "project_root",
                }),
            )
        })?;
    if is_unsafe_agent_runtime_path_inline(project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }

    let designs = state
        .persistence
        .read_call_stack_designs(
            project_root,
            body.continuity_id.as_deref(),
            body.entry_name.as_deref(),
            100,
        )
        .map_err(|e| {
            rejection(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "status": "blocked",
                    "canonical": false,
                    "advisory": true,
                    "failure_class": "storage_unreadable",
                    "message": format!("read failed: {}", e),
                }),
            )
        })?;
    let design = if let Some(design_id) = body.design_id.as_deref() {
        designs.into_iter().find(|d| d.design_id == design_id)
    } else {
        designs.into_iter().last()
    };
    let Some(design) = design else {
        return Err(rejection(
            StatusCode::NOT_FOUND,
            json!({
                "status": "not_found",
                "canonical": false,
                "advisory": true,
                "failure_class": "call_stack_design_not_found",
                "design_id": body.design_id,
                "entry_name": body.entry_name,
            }),
        ));
    };

    let mut checks: Vec<Value> = Vec::new();
    let mut failures = 0usize;
    let mut warnings = 0usize;

    let scope_match = body
        .continuity_id
        .as_deref()
        .map(|cid| design.continuity_id.as_deref() == Some(cid))
        .unwrap_or(true);
    if scope_match {
        checks.push(call_stack_check("scope", "pass", "design scope matches requested project_root + continuity_id"));
    } else {
        failures += 1;
        checks.push(call_stack_check("scope", "fail", "design continuity_id does not match requested scope"));
    }

    if ENTRY_SURFACE_ALLOWED.contains(&design.entry_surface.as_str()) {
        checks.push(call_stack_check("entry_surface_allowed", "pass", "entry_surface is supported"));
    } else {
        failures += 1;
        checks.push(call_stack_check("entry_surface_allowed", "fail", "entry_surface is not supported"));
    }

    if entry_surface_exists(project_root, &design) {
        checks.push(call_stack_check("entry_surface_exists", "pass", "entry surface string found in bounded source search"));
    } else {
        failures += 1;
        checks.push(call_stack_check("entry_surface_exists", "fail", "entry surface not found in bounded source search"));
    }

    if design.handlers.is_empty() {
        failures += 1;
        checks.push(call_stack_check("handlers", "fail", "no handlers declared"));
    } else {
        checks.push(call_stack_check("handlers", "pass", "handlers declared"));
    }
    if design.services.is_empty() {
        failures += 1;
        checks.push(call_stack_check("services", "fail", "no services declared"));
    } else {
        checks.push(call_stack_check("services", "pass", "services declared"));
    }
    if design.adapters.is_empty() {
        failures += 1;
        checks.push(call_stack_check("adapters", "fail", "no adapters declared"));
    } else {
        checks.push(call_stack_check("adapters", "pass", "adapters declared"));
    }

    match design.output_envelope.as_deref() {
        Some("tool_result_v1") => checks.push(call_stack_check("output_envelope", "pass", "output envelope matches tool_result_v1")),
        Some(other) => {
            failures += 1;
            checks.push(call_stack_check("output_envelope", "fail", format!("unexpected output envelope: {}", other)));
        }
        None => {
            failures += 1;
            checks.push(call_stack_check("output_envelope", "fail", "output envelope missing"));
        }
    }

    if design.evidence_refs.is_empty() {
        checks.push(call_stack_check("evidence_refs", "pass", "no claimed evidence refs to verify"));
    } else {
        warnings += 1;
        checks.push(call_stack_check("evidence_refs", "warn", "claimed evidence refs require Workpoint/evidence verification"));
    }

    if design.workpoint_id.is_some() || design.attach_to_workpoint {
        checks.push(call_stack_check("workpoint_alignment", "pass", "design carries Workpoint attachment intent"));
    } else {
        warnings += 1;
        checks.push(call_stack_check("workpoint_alignment", "warn", "design is advisory and not attached to a Workpoint"));
    }

    let drift_status = if failures > 0 {
        "drifted"
    } else if warnings > 0 {
        "needs_review"
    } else {
        "aligned"
    };

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "failure_class": null,
        "scope_status": "matched",
        "drift_status": drift_status,
        "failures": failures,
        "warnings": warnings,
        "design_id": design.design_id,
        "entry_surface": design.entry_surface,
        "entry_name": design.entry_name,
        "checks": checks,
        "next_tools": [
            "focusa_call_stack_design",
            "focusa_workpoint_link_evidence",
            "focusa_trajectory_assess"
        ],
        "rehydrate_id": format!("call_stack_verify:{}", design.design_id),
    })))
}

fn is_unsafe_agent_runtime_path_inline(path: &str) -> bool {
    const BLOCKED: &[&str] = &[
        "/root/pi-mono",
        "/root/.pi",
        "/root/.claude",
        "/root/.opencode",
        "/root/.letta",
    ];
    BLOCKED
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{}/", p)))
}

fn sanitize_stub(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .chars()
        .take(40)
        .collect()
}

fn project_root_hash(project_root: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_stub_lowercases_and_replaces() {
        assert_eq!(sanitize_stub("/Home/User!"), "home_user");
        assert_eq!(
            sanitize_stub("Add Call Stack Design"),
            "add_call_stack_design"
        );
    }

    #[test]
    fn project_root_hash_is_deterministic() {
        assert_eq!(project_root_hash("/a/b"), project_root_hash("/a/b"));
        assert_ne!(project_root_hash("/a/b"), project_root_hash("/a/c"));
    }

    #[test]
    fn entry_surface_whitelist() {
        for surface in ENTRY_SURFACE_ALLOWED {
            assert!(ENTRY_SURFACE_ALLOWED.contains(surface));
        }
        assert!(!ENTRY_SURFACE_ALLOWED.contains(&"unknown"));
    }
}
