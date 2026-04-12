//! Ontology inspection routes.
//!
//! Read-only projection of the typed software/work/mission/execution world.
//! This keeps ontology additive in implementation while making the bounded
//! working world inspectable at runtime.

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{routing::get, Json, Router};
use focusa_core::types::{FocusaState, FrameRecord, HandleKind};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const OBJECT_TYPES: &[&str] = &[
    "repo", "package", "module", "file", "symbol", "route", "endpoint", "schema", "migration", "dependency", "test", "environment",
    "task", "bug", "feature", "decision", "convention", "constraint", "risk", "milestone",
    "goal", "subgoal", "active_focus", "open_loop", "acceptance_criterion",
    "patch", "diff", "failure", "verification", "artifact",
];
const STATUS_VOCABULARY: &[&str] = &[
    "active", "speculative", "blocked", "verified", "stale", "deprecated", "canonical", "experimental",
];
const MEMBERSHIP_CLASSES: &[&str] = &[
    "pinned", "deterministic", "verified", "inferred", "provisional",
];
const PROVENANCE_CLASSES: &[&str] = &[
    "parser_derived", "tool_derived", "user_asserted", "model_inferred", "reducer_promoted", "verification_confirmed",
];
const LINK_TYPES: &[&str] = &[
    "imports", "calls", "renders", "persists_to", "depends_on", "configured_by", "tested_by", "implements", "violates", "blocks", "supersedes", "belongs_to_goal", "verifies", "derived_from",
    "contains", "declared_in", "targets_schema", "owned_by_repo",
];
const ACTION_TYPES: &[&str] = &[
    "refactor_module", "modify_schema", "add_route", "add_test", "verify_invariant", "promote_decision", "mark_blocked", "resolve_risk", "complete_task", "rollback_change",
];
const SLICE_TYPES: &[&str] = &[
    "active_mission", "debugging", "refactor", "regression", "architecture",
];
const MAX_DISCOVERED_PATHS: usize = 96;
const MAX_DISCOVERED_SYMBOLS: usize = 24;
const MAX_DISCOVERED_ENDPOINTS: usize = 16;

#[derive(Deserialize)]
struct OntologyWorldQuery {
    frame_id: Option<String>,
}

#[derive(Deserialize)]
struct SliceQuery {
    frame_id: Option<String>,
    #[serde(default = "default_slice_type")]
    slice_type: String,
}

fn default_slice_type() -> String {
    "active_mission".to_string()
}

fn slug(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn stable_id(prefix: &str, value: &str) -> String {
    format!("{}:{}", prefix, slug(value))
}

fn object_required_properties(object_type: &str) -> &'static [&'static str] {
    match object_type {
        "repo" => &["id", "name", "root_path", "vcs_type", "default_branch"],
        "package" => &["id", "repo_id", "name", "package_type", "path"],
        "module" => &["id", "package_id", "name", "path", "language"],
        "file" => &["id", "path", "file_type", "language"],
        "symbol" => &["id", "file_id", "symbol_name", "symbol_kind"],
        "route" => &["id", "path", "route_kind", "package_id"],
        "endpoint" => &["id", "path_or_signature", "method_or_transport", "package_id"],
        "schema" => &["id", "schema_name", "storage_kind"],
        "migration" => &["id", "path", "schema_targets"],
        "dependency" => &["id", "name", "version", "dependency_kind"],
        "test" => &["id", "path", "test_kind"],
        "environment" => &["id", "name", "environment_kind"],
        "task" => &["id", "title", "status", "priority"],
        "bug" => &["id", "title", "severity", "status"],
        "feature" => &["id", "title", "status"],
        "decision" => &["id", "statement", "decision_kind", "status"],
        "convention" => &["id", "rule_text", "convention_kind", "status"],
        "constraint" => &["id", "rule_text", "scope", "enforcement_level"],
        "risk" => &["id", "title", "severity", "status"],
        "milestone" => &["id", "title", "status"],
        "goal" => &["id", "title", "objective", "status"],
        "subgoal" => &["id", "title", "status"],
        "active_focus" => &["id", "title", "frame_id", "status"],
        "open_loop" => &["id", "statement", "urgency", "status"],
        "acceptance_criterion" => &["id", "text", "status"],
        "patch" => &["id", "patch_ref", "timestamp"],
        "diff" => &["id", "diff_ref", "timestamp"],
        "failure" => &["id", "failure_kind", "timestamp", "status"],
        "verification" => &["id", "method", "result", "timestamp"],
        "artifact" => &["id", "handle", "artifact_kind", "status"],
        _ => &["id", "status"],
    }
}

fn action_target_types(action_type: &str) -> &'static [&'static str] {
    match action_type {
        "refactor_module" => &["module", "file", "dependency"],
        "modify_schema" => &["schema", "migration"],
        "add_route" => &["route", "endpoint", "module"],
        "add_test" => &["test", "module", "file"],
        "verify_invariant" => &["verification", "test", "constraint"],
        "promote_decision" => &["decision"],
        "mark_blocked" => &["task", "goal", "risk", "failure"],
        "resolve_risk" => &["risk", "verification", "task"],
        "complete_task" => &["task", "goal", "milestone"],
        "rollback_change" => &["patch", "diff", "artifact"],
        _ => OBJECT_TYPES,
    }
}

fn action_contract(action_type: &str) -> Value {
    let (
        input_schema,
        output_schema,
        side_effects,
        failure_modes,
        idempotency,
        rollback,
        verification_hooks,
        expected_deltas,
        timeout_policy,
        retry_policy,
        degraded_fallback,
        tool_mappings,
    ) = match action_type {
        "refactor_module" => (
            json!({"type":"object","required":["module_id"],"properties":{"module_id":{"type":"string"},"scope":{"type":"string"},"reason":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","side_effect_summary","verification_result_or_next_step"]}),
            json!(["focus_frame_update", "command dispatch", "reducer-visible events"]),
            json!(["validation_failure", "dependency_failure", "execution_failure", "verification_failure", "timeout", "partial_success"]),
            json!("best_effort, repeatable with same target module"),
            json!({"available":true,"mechanism":"rollback_change / VCS revert"}),
            json!(["tests/tool_contract_test.sh", "tests/command_write_contract_test.sh"]),
            json!(["module updated", "verification queued", "artifact refs produced"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"manual retry after verification or dependency remediation","max_attempts":2}),
            json!({"behavior":"emit blocker/failure + preserve checkpoint-visible state"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"compact"},
                {"surface":"http","method":"POST","path":"/v1/focus/update","command":"focus.update"}
            ]),
        ),
        "modify_schema" => (
            json!({"type":"object","required":["schema_id"],"properties":{"schema_id":{"type":"string"},"migration_path":{"type":"string"},"reason":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","evidence_refs","verification_result_or_next_step"]}),
            json!(["migration proposal", "schema evidence", "verification hooks"]),
            json!(["validation_failure", "dependency_failure", "permission_failure", "execution_failure", "verification_failure", "rollback_failure"]),
            json!("non-idempotent without migration identity; requires explicit target"),
            json!({"available":true,"mechanism":"rollback_change / compensating migration"}),
            json!(["tests/tool_contract_test.sh", "tests/golden_tasks_eval.sh"]),
            json!(["schema target updated", "migration linked", "verification pending"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"no automatic retry on schema mutation without operator approval","max_attempts":1}),
            json!({"behavior":"emit blocker on missing dependency/permission"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"ascc.checkpoint"},
                {"surface":"http","method":"POST","path":"/v1/proposals","command":"proposal.submit"}
            ]),
        ),
        "add_route" => (
            json!({"type":"object","required":["path_or_signature"],"properties":{"path_or_signature":{"type":"string"},"method_or_transport":{"type":"string"},"package_id":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","side_effect_summary","ontology_delta_candidates"]}),
            json!(["route/endpoint projection", "verification hooks", "artifact refs"]),
            json!(["validation_failure", "execution_failure", "verification_failure", "timeout"]),
            json!("idempotent only when path+method pair already canonicalized"),
            json!({"available":true,"mechanism":"rollback_change / route removal"}),
            json!(["tests/ontology_world_contract_test.sh", "tests/tool_contract_test.sh"]),
            json!(["route added", "endpoint added", "test target expected"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry once after validation correction","max_attempts":2}),
            json!({"behavior":"fall back to proposal/intention when verification absent"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"},
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"focus.push_frame"}
            ]),
        ),
        "add_test" => (
            json!({"type":"object","required":["target_path"],"properties":{"target_path":{"type":"string"},"test_kind":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","verification_result_or_next_step"]}),
            json!(["test artifact creation", "verification linkage"]),
            json!(["validation_failure", "execution_failure", "partial_success", "timeout"]),
            json!("idempotent when target_path already contains canonical test"),
            json!({"available":true,"mechanism":"rollback_change / file revert"}),
            json!(["tests/ontology_world_contract_test.sh", "tests/golden_tasks_eval.sh"]),
            json!(["test object added", "tested_by link added", "verification target queued"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry once after dependency repair","max_attempts":2}),
            json!({"behavior":"mark as open loop if generation blocked"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/ontology/contracts","command":"ontology.contracts"}
            ]),
        ),
        "verify_invariant" => (
            json!({"type":"object","required":["verification_target"],"properties":{"verification_target":{"type":"string"},"method":{"type":"string"}}}),
            json!({"required":["result_status","verification_result_or_next_step","evidence_refs"]}),
            json!(["verification record emission", "telemetry trace"]),
            json!(["validation_failure", "execution_failure", "verification_failure", "timeout"]),
            json!("repeatable and expected to be idempotent over same target"),
            json!({"available":false,"mechanism":"n/a"}),
            json!(["tests/trace_dimensions_test.sh", "tests/golden_tasks_eval.sh"]),
            json!(["verification object updated", "verifies link added"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry after target stabilization","max_attempts":3}),
            json!({"behavior":"emit verification failure + blocker when mismatch persists"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/telemetry/trace","command":"telemetry.trace"},
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"}
            ]),
        ),
        "promote_decision" => (
            json!({"type":"object","required":["statement"],"properties":{"statement":{"type":"string"},"reason":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","ontology_delta_candidates"]}),
            json!(["decision distillation", "proposal scoring", "canonical mutation"]),
            json!(["validation_failure", "execution_failure", "verification_failure"]),
            json!("idempotent when same decision already canonical"),
            json!({"available":true,"mechanism":"superseding decision"}),
            json!(["tests/behavioral_alignment_test.sh", "tests/proposal_kind_enforcement_test.sh"]),
            json!(["decision object added", "belongs_to_goal link added"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry after improved evidence only","max_attempts":2}),
            json!({"behavior":"leave as proposal if not verified/canonical yet"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/proposals","command":"proposal.submit"},
                {"surface":"http","method":"POST","path":"/v1/focus/update","command":"focus.update"}
            ]),
        ),
        "mark_blocked" => (
            json!({"type":"object","required":["summary"],"properties":{"summary":{"type":"string"},"frame_context":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","side_effect_summary"]}),
            json!(["failure/blocker emission", "gate surfacing"]),
            json!(["validation_failure", "dependency_failure", "execution_failure"]),
            json!("repeatable; duplicates should converge on surfaced candidate state"),
            json!({"available":true,"mechanism":"resolve_risk / suppress candidate"}),
            json!(["tests/checkpoint_trigger_test.sh", "tests/behavioral_alignment_test.sh"]),
            json!(["failure object added", "blocks link added", "gate candidate surfaced"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry after context change only","max_attempts":2}),
            json!({"behavior":"persist blocker + checkpoint before risky continuation"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/focus-gate/ingest-signal","command":"gate.ingest_signal"},
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"gate.suppress"}
            ]),
        ),
        "resolve_risk" => (
            json!({"type":"object","required":["risk_id"],"properties":{"risk_id":{"type":"string"},"verification_target":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","verification_result_or_next_step"]}),
            json!(["risk status update", "verification record"]),
            json!(["validation_failure", "execution_failure", "verification_failure", "timeout"]),
            json!("repeatable while risk remains active"),
            json!({"available":true,"mechanism":"mark_blocked or supersede risk"}),
            json!(["tests/golden_tasks_eval.sh", "tests/trace_dimensions_test.sh"]),
            json!(["risk status changed", "verification added"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry after evidence refresh","max_attempts":2}),
            json!({"behavior":"degrade to blocker if verification unavailable"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/reflect/status","command":"reflect.status"}
            ]),
        ),
        "complete_task" => (
            json!({"type":"object","required":["task_id"],"properties":{"task_id":{"type":"string"},"completion_reason":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","side_effect_summary"]}),
            json!(["frame completion", "checkpoint persistence", "lineage update"]),
            json!(["validation_failure", "execution_failure", "partial_success"]),
            json!("idempotent when task already completed"),
            json!({"available":true,"mechanism":"supersede / reopen task"}),
            json!(["tests/fork_compact_recovery_test.sh", "tests/checkpoint_trigger_test.sh"]),
            json!(["task status changed", "goal/open-loop state updated"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"no blind retry after completion","max_attempts":1}),
            json!({"behavior":"record recent_result if completion only partially verified"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/focus/pop","command":"focus.pop_frame"},
                {"surface":"http","method":"POST","path":"/v1/session/close","command":"session.close"}
            ]),
        ),
        "rollback_change" => (
            json!({"type":"object","required":["artifact_ref"],"properties":{"artifact_ref":{"type":"string"},"reason":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","side_effect_summary","verification_result_or_next_step"]}),
            json!(["artifact rollback", "checkpoint refresh", "summary node"]),
            json!(["validation_failure", "permission_failure", "execution_failure", "rollback_failure", "timeout"]),
            json!("idempotent once rollback reaches canonical target state"),
            json!({"available":true,"mechanism":"VCS revert / compensating change"}),
            json!(["tests/fork_compact_recovery_test.sh", "tests/command_write_contract_test.sh"]),
            json!(["patch/diff status changed", "verification pending"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry only after permission/dependency remediation","max_attempts":2}),
            json!({"behavior":"emit failure + preserve prior checkpoint if rollback fails"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"micro-compact"},
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"compact"}
            ]),
        ),
        _ => (
            json!({"type":"object"}), json!({}), json!([]), json!([]), json!("unknown"), json!({}), json!([]), json!([]), json!({}), json!({}), json!({}), json!([]),
        ),
    };

    json!({
        "name": action_type,
        "target_types": action_target_types(action_type),
        "input_schema": input_schema,
        "output_schema": output_schema,
        "side_effects": side_effects,
        "failure_modes": failure_modes,
        "idempotency_expectations": idempotency,
        "rollback_availability": rollback,
        "verification_hooks": verification_hooks,
        "expected_ontology_deltas": expected_deltas,
        "timeout_policy": timeout_policy,
        "retry_policy": retry_policy,
        "degraded_fallback_behavior": degraded_fallback,
        "tool_mappings": tool_mappings,
    })
}

fn primitive_contracts() -> Json<Value> {
    let object_types: Vec<Value> = OBJECT_TYPES
        .iter()
        .map(|name| {
            json!({
                "type_name": name,
                "id_strategy": "stable_string_or_uuid",
                "required_properties": object_required_properties(name),
                "allowed_links": LINK_TYPES,
                "allowed_actions": ACTION_TYPES,
                "status_vocabulary": STATUS_VOCABULARY,
            })
        })
        .collect();

    let link_types: Vec<Value> = LINK_TYPES
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "source_types": OBJECT_TYPES,
                "target_types": OBJECT_TYPES,
                "multiplicity": "many",
                "directionality": "directed",
                "evidence_policy": "required",
                "promotion_policy": "reducer_only",
            })
        })
        .collect();

    let action_types: Vec<Value> = ACTION_TYPES.iter().map(|name| action_contract(name)).collect();

    Json(json!({
        "object_types": object_types,
        "link_types": link_types,
        "action_types": action_types,
        "status_vocabulary": STATUS_VOCABULARY,
        "membership_classes": MEMBERSHIP_CLASSES,
        "provenance_classes": PROVENANCE_CLASSES,
        "slice_policies": SLICE_TYPES.iter().map(|name| json!({
            "name": name,
            "max_object_count": 12,
            "max_artifact_handle_count": 5,
            "max_historical_delta_count": 3,
            "max_decision_constraint_count": 8,
        })).collect::<Vec<_>>(),
    }))
}

#[derive(Default)]
struct WorkspaceProjection {
    objects: Vec<Value>,
    links: Vec<Value>,
}

fn read_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn walk_workspace(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if out.len() >= MAX_DISCOVERED_PATHS {
            break;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') && name != ".git" {
                continue;
            }
            if path.is_dir() {
                if matches!(name, "target" | "node_modules" | "dist" | "build" | ".beads") {
                    continue;
                }
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
                if out.len() >= MAX_DISCOVERED_PATHS {
                    break;
                }
            }
        }
    }
    out
}

fn parse_cargo_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") && trimmed.contains('=') {
            return trimmed
                .split('=')
                .nth(1)
                .map(|v| v.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn parse_cargo_dependencies(content: &str) -> Vec<(String, String)> {
    let mut in_deps = false;
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]";
            continue;
        }
        if !in_deps || trimmed.is_empty() || trimmed.starts_with('#') || !trimmed.contains('=') {
            continue;
        }
        let mut parts = trimmed.splitn(2, '=');
        let name = parts.next().unwrap_or("").trim();
        let version = parts.next().unwrap_or("").trim().trim_matches('"');
        if !name.is_empty() {
            out.push((name.to_string(), version.to_string()));
        }
    }
    out
}

fn parse_package_json(content: &str) -> Option<(String, Vec<(String, String)>)> {
    let value: Value = serde_json::from_str(content).ok()?;
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("package")
        .to_string();
    let mut deps = Vec::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(map) = value.get(key).and_then(|v| v.as_object()) {
            for (dep, version) in map {
                deps.push((
                    dep.clone(),
                    version.as_str().unwrap_or("unknown").to_string(),
                ));
            }
        }
    }
    Some((name, deps))
}

fn classify_language(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "sql" => "sql",
        "json" => "json",
        "toml" => "toml",
        _ => "text",
    }
}

fn classify_file_type(path: &Path) -> &'static str {
    let rel = path.to_string_lossy();
    if rel.contains("/tests/")
        || rel.ends_with("_test.rs")
        || rel.ends_with(".test.ts")
        || rel.ends_with(".spec.ts")
    {
        "test"
    } else if rel.contains("migrations") || rel.ends_with(".sql") {
        "migration"
    } else if path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|n| n == "Cargo.toml" || n == "package.json")
        .unwrap_or(false)
    {
        "manifest"
    } else if rel.contains("route") {
        "route_source"
    } else {
        "source"
    }
}

fn parse_symbols(content: &str, language: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let parsed = match language {
            "rust" => {
                if let Some(rest) = trimmed
                    .strip_prefix("pub fn ")
                    .or_else(|| trimmed.strip_prefix("fn "))
                {
                    Some((rest.split('(').next().unwrap_or("symbol"), "function"))
                } else if let Some(rest) = trimmed
                    .strip_prefix("pub struct ")
                    .or_else(|| trimmed.strip_prefix("struct "))
                {
                    Some((rest.split_whitespace().next().unwrap_or("symbol"), "struct"))
                } else if let Some(rest) = trimmed
                    .strip_prefix("pub enum ")
                    .or_else(|| trimmed.strip_prefix("enum "))
                {
                    Some((rest.split_whitespace().next().unwrap_or("symbol"), "enum"))
                } else if let Some(rest) = trimmed
                    .strip_prefix("pub trait ")
                    .or_else(|| trimmed.strip_prefix("trait "))
                {
                    Some((rest.split_whitespace().next().unwrap_or("symbol"), "trait"))
                } else {
                    None
                }
            }
            "typescript" | "javascript" => {
                if let Some(rest) = trimmed
                    .strip_prefix("export function ")
                    .or_else(|| trimmed.strip_prefix("function "))
                {
                    Some((rest.split('(').next().unwrap_or("symbol"), "function"))
                } else if let Some(rest) = trimmed
                    .strip_prefix("export class ")
                    .or_else(|| trimmed.strip_prefix("class "))
                {
                    Some((rest.split_whitespace().next().unwrap_or("symbol"), "class"))
                } else if let Some(rest) = trimmed
                    .strip_prefix("export const ")
                    .or_else(|| trimmed.strip_prefix("const "))
                {
                    Some((rest.split('=').next().unwrap_or("symbol").trim(), "const"))
                } else if let Some(rest) = trimmed.strip_prefix("interface ") {
                    Some((
                        rest.split_whitespace().next().unwrap_or("symbol"),
                        "interface",
                    ))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some((name, kind)) = parsed {
            if !name.is_empty() {
                out.push((name.to_string(), kind.to_string()));
            }
        }
        if out.len() >= MAX_DISCOVERED_SYMBOLS {
            break;
        }
    }
    out
}

fn parse_endpoints(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let methods = [
            ("get(", "get"),
            ("post(", "post"),
            ("put(", "put"),
            ("patch(", "patch"),
            ("delete(", "delete"),
        ];
        for (needle, method) in methods {
            if let Some(idx) = trimmed.find(needle) {
                let start = idx + needle.len();
                let rest = &trimmed[start..];
                if let Some(first_quote) = rest.find('"') {
                    let after = &rest[first_quote + 1..];
                    if let Some(end_quote) = after.find('"') {
                        let route = &after[..end_quote];
                        if route.starts_with('/') {
                            out.push((method.to_string(), route.to_string()));
                        }
                    }
                }
            }
        }
        if let Some(idx) = trimmed.find(".route(\"") {
            let after = &trimmed[idx + 8..];
            if let Some(end_quote) = after.find('"') {
                let route = &after[..end_quote];
                let method = if trimmed.contains("post(") {
                    "post"
                } else if trimmed.contains("patch(") {
                    "patch"
                } else if trimmed.contains("put(") {
                    "put"
                } else if trimmed.contains("delete(") {
                    "delete"
                } else {
                    "get"
                };
                if route.starts_with('/') {
                    out.push((method.to_string(), route.to_string()));
                }
            }
        }
        if out.len() >= MAX_DISCOVERED_ENDPOINTS {
            break;
        }
    }
    out
}

fn workspace_projection(focusa: &FocusaState) -> WorkspaceProjection {
    let Some(workspace_id) = focusa.session.as_ref().and_then(|s| s.workspace_id.clone()) else {
        return WorkspaceProjection::default();
    };
    let root = PathBuf::from(&workspace_id);
    if !root.exists() || !root.is_dir() {
        return WorkspaceProjection::default();
    }

    let repo_id = stable_id("repo", &workspace_id);
    let mut objects = vec![json!({
        "id": repo_id,
        "object_type": "repo",
        "name": root.file_name().and_then(|s| s.to_str()).unwrap_or("workspace"),
        "root_path": workspace_id,
        "vcs_type": if root.join(".git").exists() { "git" } else { "unknown" },
        "default_branch": "main",
        "status": "canonical",
        "membership_class": "deterministic",
        "provenance_class": "parser_derived",
        "fresh": true,
    })];
    let mut links = Vec::new();

    let env_id = stable_id("environment", &format!("workspace:{}", root.display()));
    objects.push(json!({
        "id": env_id,
        "object_type": "environment",
        "name": root.file_name().and_then(|s| s.to_str()).unwrap_or("workspace"),
        "environment_kind": "workspace",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "parser_derived",
        "fresh": true,
    }));
    links.push(json!({
        "type": "configured_by",
        "source_id": repo_id,
        "target_id": env_id,
        "evidence": "session.workspace_id",
        "status": "verified",
    }));

    let mut package_ids: Vec<String> = Vec::new();
    let cargo_path = root.join("Cargo.toml");
    if let Some(content) = read_text(&cargo_path) {
        let pkg_name = parse_cargo_name(&content).unwrap_or_else(|| {
            root.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("workspace")
                .to_string()
        });
        let package_id = stable_id("package", &format!("cargo:{}", pkg_name));
        package_ids.push(package_id.clone());
        objects.push(json!({
            "id": package_id,
            "object_type": "package",
            "repo_id": repo_id,
            "name": pkg_name,
            "package_type": "cargo",
            "path": "Cargo.toml",
            "status": "canonical",
            "membership_class": "deterministic",
            "provenance_class": "parser_derived",
            "fresh": true,
        }));
        for (dep, version) in parse_cargo_dependencies(&content).into_iter().take(16) {
            let dep_id = stable_id("dependency", &format!("cargo:{}", dep));
            objects.push(json!({
                "id": dep_id,
                "object_type": "dependency",
                "name": dep,
                "version": version,
                "dependency_kind": "cargo",
                "status": "verified",
                "membership_class": "deterministic",
                "provenance_class": "parser_derived",
                "fresh": true,
            }));
            links.push(json!({
                "type": "depends_on",
                "source_id": package_id,
                "target_id": dep_id,
                "evidence": "Cargo.toml [dependencies]",
                "status": "verified",
            }));
        }
    }

    let package_json = root.join("package.json");
    if let Some(content) = read_text(&package_json) {
        if let Some((pkg_name, deps)) = parse_package_json(&content) {
            let package_id = stable_id("package", &format!("npm:{}", pkg_name));
            package_ids.push(package_id.clone());
            objects.push(json!({
                "id": package_id,
                "object_type": "package",
                "repo_id": repo_id,
                "name": pkg_name,
                "package_type": "npm",
                "path": "package.json",
                "status": "canonical",
                "membership_class": "deterministic",
                "provenance_class": "parser_derived",
                "fresh": true,
            }));
            for (dep, version) in deps.into_iter().take(16) {
                let dep_id = stable_id("dependency", &format!("npm:{}", dep));
                objects.push(json!({
                    "id": dep_id,
                    "object_type": "dependency",
                    "name": dep,
                    "version": version,
                    "dependency_kind": "npm",
                    "status": "verified",
                    "membership_class": "deterministic",
                    "provenance_class": "parser_derived",
                    "fresh": true,
                }));
                links.push(json!({
                    "type": "depends_on",
                    "source_id": package_id,
                    "target_id": dep_id,
                    "evidence": "package.json dependencies",
                    "status": "verified",
                }));
            }
        }
    }

    let package_id = package_ids
        .first()
        .cloned()
        .unwrap_or_else(|| stable_id("package", "workspace"));
    let mut module_ids = BTreeSet::new();
    let mut schema_ids = BTreeSet::new();
    let mut files_scanned = 0usize;

    for path in walk_workspace(&root) {
        if files_scanned >= MAX_DISCOVERED_PATHS {
            break;
        }
        let Ok(rel_path) = path.strip_prefix(&root) else {
            continue;
        };
        let rel = rel_path.to_string_lossy().to_string();
        let language = classify_language(&path);
        let file_type = classify_file_type(&path);
        let file_id = stable_id("file", &rel);
        objects.push(json!({
            "id": file_id,
            "object_type": "file",
            "path": rel,
            "file_type": file_type,
            "language": language,
            "status": "verified",
            "membership_class": "deterministic",
            "provenance_class": "parser_derived",
            "fresh": true,
        }));
        links.push(json!({
            "type": "owned_by_repo",
            "source_id": file_id,
            "target_id": repo_id,
            "evidence": "workspace scan",
            "status": "verified",
        }));
        files_scanned += 1;

        if let Some(parent) = rel_path.parent().and_then(|p| p.to_str()) {
            if !parent.is_empty() {
                let module_id = stable_id("module", parent);
                if module_ids.insert(module_id.clone()) {
                    objects.push(json!({
                        "id": module_id,
                        "object_type": "module",
                        "package_id": package_id,
                        "name": Path::new(parent).file_name().and_then(|s| s.to_str()).unwrap_or(parent),
                        "path": parent,
                        "language": language,
                        "status": "verified",
                        "membership_class": "deterministic",
                        "provenance_class": "parser_derived",
                        "fresh": true,
                    }));
                }
                links.push(json!({
                    "type": "contains",
                    "source_id": module_id,
                    "target_id": file_id,
                    "evidence": "filesystem parent path",
                    "status": "verified",
                }));
            }
        }

        if file_type == "test" {
            let test_id = stable_id("test", &rel);
            objects.push(json!({
                "id": test_id,
                "object_type": "test",
                "path": rel,
                "test_kind": if language == "rust" { "unit" } else { "integration" },
                "status": "verified",
                "membership_class": "deterministic",
                "provenance_class": "parser_derived",
                "fresh": true,
            }));
            links.push(json!({
                "type": "tested_by",
                "source_id": file_id,
                "target_id": test_id,
                "evidence": "filesystem test placement",
                "status": "verified",
            }));
        }

        if file_type == "migration" {
            let migration_id = stable_id("migration", &rel);
            let schema_id = stable_id("schema", "primary");
            if schema_ids.insert(schema_id.clone()) {
                objects.push(json!({
                    "id": schema_id,
                    "object_type": "schema",
                    "schema_name": "primary",
                    "storage_kind": "sql",
                    "status": "verified",
                    "membership_class": "deterministic",
                    "provenance_class": "parser_derived",
                    "fresh": true,
                }));
            }
            objects.push(json!({
                "id": migration_id,
                "object_type": "migration",
                "path": rel,
                "schema_targets": [schema_id.clone()],
                "status": "verified",
                "membership_class": "deterministic",
                "provenance_class": "parser_derived",
                "fresh": true,
            }));
            links.push(json!({
                "type": "persists_to",
                "source_id": migration_id,
                "target_id": schema_id,
                "evidence": "migration file path",
                "status": "verified",
            }));
        }

        if let Some(content) = read_text(&path) {
            for (name, kind) in parse_symbols(&content, language) {
                let symbol_id = stable_id("symbol", &format!("{}:{}", rel, name));
                objects.push(json!({
                    "id": symbol_id,
                    "object_type": "symbol",
                    "file_id": file_id,
                    "symbol_name": name,
                    "symbol_kind": kind,
                    "status": "verified",
                    "membership_class": "deterministic",
                    "provenance_class": "parser_derived",
                    "fresh": true,
                }));
                links.push(json!({
                    "type": "declared_in",
                    "source_id": symbol_id,
                    "target_id": file_id,
                    "evidence": "source scan",
                    "status": "verified",
                }));
            }

            if rel.contains("route") {
                let route_id = stable_id("route", &rel);
                objects.push(json!({
                    "id": route_id,
                    "object_type": "route",
                    "path": format!(
                        "/{}",
                        rel_path.file_stem().and_then(|s| s.to_str()).unwrap_or("route")
                    ),
                    "route_kind": "http",
                    "package_id": package_id,
                    "status": "verified",
                    "membership_class": "deterministic",
                    "provenance_class": "parser_derived",
                    "fresh": true,
                }));
                links.push(json!({
                    "type": "contains",
                    "source_id": route_id,
                    "target_id": file_id,
                    "evidence": "route source path",
                    "status": "verified",
                }));
                for (method, endpoint_path) in parse_endpoints(&content) {
                    let endpoint_id = stable_id(
                        "endpoint",
                        &format!("{}:{}", method, endpoint_path),
                    );
                    objects.push(json!({
                        "id": endpoint_id,
                        "object_type": "endpoint",
                        "path_or_signature": endpoint_path,
                        "method_or_transport": method,
                        "package_id": package_id,
                        "status": "verified",
                        "membership_class": "deterministic",
                        "provenance_class": "parser_derived",
                        "fresh": true,
                    }));
                    links.push(json!({
                        "type": "implements",
                        "source_id": endpoint_id,
                        "target_id": route_id,
                        "evidence": "route source scan",
                        "status": "verified",
                    }));
                }
            }
        }
    }

    WorkspaceProjection { objects, links }
}

fn selected_frame<'a>(focusa: &'a FocusaState, frame_id: Option<&str>) -> Option<&'a FrameRecord> {
    let selected = frame_id
        .and_then(|id| focusa.focus_stack.frames.iter().find(|f| f.id.to_string() == id));
    selected.or_else(|| {
        focusa
            .focus_stack
            .active_id
            .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid))
    })
}

fn mission_projection(focusa: &FocusaState, frame: Option<&FrameRecord>) -> WorkspaceProjection {
    let mut objects = Vec::new();
    let mut links = Vec::new();

    if let Some(frame) = frame {
        let goal_id = format!("goal:{}", frame.id);
        let focus_id = format!("active_focus:{}", frame.id);
        let task_id = format!("task:{}", frame.id);
        objects.push(json!({
            "id": goal_id,
            "object_type": "goal",
            "title": if frame.goal.is_empty() { frame.title.clone() } else { frame.goal.clone() },
            "objective": frame.goal,
            "status": format!("{:?}", frame.status).to_lowercase(),
            "membership_class": "deterministic",
            "provenance_class": "reducer_promoted",
            "fresh": true,
        }));
        objects.push(json!({
            "id": focus_id,
            "object_type": "active_focus",
            "title": frame.title,
            "frame_id": frame.id,
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "reducer_promoted",
            "fresh": true,
        }));
        objects.push(json!({
            "id": task_id,
            "object_type": "task",
            "title": frame.title,
            "status": format!("{:?}", frame.status).to_lowercase(),
            "priority": frame.priority_hint.clone().unwrap_or_else(|| "normal".to_string()),
            "membership_class": "deterministic",
            "provenance_class": "reducer_promoted",
            "fresh": true,
        }));
        links.push(json!({
            "type": "belongs_to_goal",
            "source_id": focus_id,
            "target_id": goal_id,
            "evidence": "focus_stack.active_frame",
            "status": "verified",
        }));
        links.push(json!({
            "type": "belongs_to_goal",
            "source_id": task_id,
            "target_id": goal_id,
            "evidence": "frame -> task",
            "status": "verified",
        }));

        for (idx, decision) in frame.focus_state.decisions.iter().take(4).enumerate() {
            let id = format!("decision:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "decision",
                "statement": decision,
                "decision_kind": "runtime",
                "status": "canonical",
                "membership_class": "verified",
                "provenance_class": "reducer_promoted",
                "fresh": true,
            }));
            links.push(json!({
                "type": "belongs_to_goal",
                "source_id": id,
                "target_id": format!("goal:{}", frame.id),
                "evidence": "focus_state.decisions",
                "status": "verified",
            }));
        }

        for (idx, constraint) in frame.focus_state.constraints.iter().take(4).enumerate() {
            let id = format!("constraint:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "constraint",
                "rule_text": constraint,
                "scope": "active_frame",
                "enforcement_level": "hard",
                "status": "active",
                "membership_class": "verified",
                "provenance_class": "reducer_promoted",
                "fresh": true,
            }));
            links.push(json!({
                "type": "configured_by",
                "source_id": format!("active_focus:{}", frame.id),
                "target_id": id,
                "evidence": "focus_state.constraints",
                "status": "verified",
            }));
        }

        for (idx, next_step) in frame.focus_state.next_steps.iter().take(3).enumerate() {
            let id = format!("open_loop:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "open_loop",
                "statement": next_step,
                "urgency": "normal",
                "status": "active",
                "membership_class": "provisional",
                "provenance_class": "reducer_promoted",
                "fresh": true,
            }));
        }

        for (idx, result) in frame.focus_state.recent_results.iter().take(3).enumerate() {
            let verify_id = format!("verification:{}:{}", frame.id, idx);
            let ac_id = format!("acceptance_criterion:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": verify_id,
                "object_type": "verification",
                "method": "recent_result",
                "result": result,
                "timestamp": frame.updated_at,
                "status": "verified",
                "membership_class": "verified",
                "provenance_class": "verification_confirmed",
                "fresh": true,
            }));
            objects.push(json!({
                "id": ac_id,
                "object_type": "acceptance_criterion",
                "text": result,
                "status": "verified",
                "membership_class": "verified",
                "provenance_class": "verification_confirmed",
                "fresh": true,
            }));
            links.push(json!({
                "type": "verifies",
                "source_id": verify_id,
                "target_id": format!("goal:{}", frame.id),
                "evidence": "focus_state.recent_results",
                "status": "verified",
            }));
        }

        for (idx, failure) in frame.focus_state.failures.iter().take(3).enumerate() {
            let failure_id = format!("failure:{}:{}", frame.id, idx);
            let risk_id = format!("risk:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": failure_id,
                "object_type": "failure",
                "failure_kind": "runtime",
                "timestamp": frame.updated_at,
                "status": "blocked",
                "summary": failure,
                "membership_class": "verified",
                "provenance_class": "verification_confirmed",
                "fresh": true,
            }));
            objects.push(json!({
                "id": risk_id,
                "object_type": "risk",
                "title": failure,
                "severity": "medium",
                "status": "active",
                "membership_class": "verified",
                "provenance_class": "verification_confirmed",
                "fresh": true,
            }));
            links.push(json!({
                "type": "blocks",
                "source_id": failure_id,
                "target_id": format!("goal:{}", frame.id),
                "evidence": "focus_state.failures",
                "status": "verified",
            }));
            links.push(json!({
                "type": "blocks",
                "source_id": risk_id,
                "target_id": format!("task:{}", frame.id),
                "evidence": "failure -> risk projection",
                "status": "verified",
            }));
        }
    }

    for rule in focusa.memory.procedural.iter().take(4) {
        let id = stable_id("convention", &rule.id);
        objects.push(json!({
            "id": id,
            "object_type": "convention",
            "rule_text": rule.rule,
            "convention_kind": "procedural_memory",
            "status": if rule.enabled { "active" } else { "stale" },
            "membership_class": "verified",
            "provenance_class": "reducer_promoted",
            "fresh": rule.enabled,
        }));
    }

    let session_id = focusa.session.as_ref().map(|s| s.session_id);
    for handle in focusa
        .reference_index
        .handles
        .iter()
        .filter(|h| h.session_id == session_id || h.pinned)
        .take(5)
    {
        let id = format!("artifact:{}", handle.id);
        objects.push(json!({
            "id": id,
            "object_type": "artifact",
            "handle": handle.id,
            "artifact_kind": match handle.kind {
                HandleKind::Text => "text",
                HandleKind::Diff => "diff",
                HandleKind::Log => "log",
                HandleKind::Json => "json",
                HandleKind::Url => "url",
                HandleKind::FileSnapshot => "file_snapshot",
                HandleKind::Other => "other",
            },
            "status": if handle.pinned { "canonical" } else { "verified" },
            "membership_class": if handle.pinned { "pinned" } else { "verified" },
            "provenance_class": "tool_derived",
            "fresh": true,
        }));
    }

    WorkspaceProjection { objects, links }
}

fn combined_projection(focusa: &FocusaState, frame_id: Option<&str>) -> WorkspaceProjection {
    let frame = selected_frame(focusa, frame_id);
    let mission = mission_projection(focusa, frame);
    let workspace = workspace_projection(focusa);
    WorkspaceProjection {
        objects: [mission.objects, workspace.objects].concat(),
        links: [mission.links, workspace.links].concat(),
    }
}

fn member(id: &str, object_type: &str, membership_class: &str, reason: &str) -> Value {
    json!({
        "id": id,
        "object_type": object_type,
        "membership_class": membership_class,
        "reason": reason,
    })
}

fn slice_members(objects: &[Value], slice_type: &str) -> Vec<Value> {
    let mut grouped: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for object in objects {
        if let Some(object_type) = object.get("object_type").and_then(|v| v.as_str()) {
            grouped.entry(object_type.to_string()).or_default().push(object);
        }
    }

    let take = |kind: &str, max: usize| -> Vec<Value> {
        grouped
            .get(kind)
            .into_iter()
            .flatten()
            .take(max)
            .map(|v| {
                member(
                    v.get("id").and_then(|x| x.as_str()).unwrap_or("unknown"),
                    kind,
                    v.get("membership_class")
                        .and_then(|x| x.as_str())
                        .unwrap_or("deterministic"),
                    match slice_type {
                        "debugging" => "debugging set relevance",
                        "refactor" => "refactor set relevance",
                        "regression" => "regression set relevance",
                        "architecture" => "architecture set relevance",
                        _ => "active mission relevance",
                    },
                )
            })
            .collect()
    };

    let mut members = match slice_type {
        "debugging" => [
            take("failure", 3),
            take("verification", 2),
            take("file", 3),
            take("test", 2),
            take("risk", 2),
        ]
        .concat(),
        "refactor" => [
            take("module", 3),
            take("file", 3),
            take("dependency", 3),
            take("convention", 2),
            take("test", 2),
            take("decision", 2),
            take("constraint", 2),
        ]
        .concat(),
        "regression" => [
            take("verification", 3),
            take("failure", 2),
            take("risk", 2),
            take("test", 3),
            take("route", 2),
            take("endpoint", 2),
        ]
        .concat(),
        "architecture" => [
            take("package", 3),
            take("module", 4),
            take("dependency", 4),
            take("route", 3),
            take("endpoint", 3),
            take("decision", 2),
            take("constraint", 2),
            take("convention", 2),
            take("risk", 2),
        ]
        .concat(),
        _ => [
            take("goal", 1),
            take("task", 1),
            take("active_focus", 1),
            take("decision", 3),
            take("constraint", 3),
            take("module", 2),
            take("file", 2),
            take("route", 1),
            take("test", 1),
            take("open_loop", 2),
            take("failure", 2),
        ]
        .concat(),
    };
    if members.len() > 12 {
        members.truncate(12);
    }
    members
}

fn slice_payload(focusa: &FocusaState, frame_id: Option<&str>, slice_type: &str) -> Value {
    let projection = combined_projection(focusa, frame_id);
    let members = slice_members(&projection.objects, slice_type);
    json!({
        "slice_type": slice_type,
        "source": "ontology_world_projection",
        "count": members.len(),
        "bounds": {
            "max_object_count": 12,
            "max_artifact_handle_count": 5,
            "max_historical_delta_count": 3,
            "max_decision_constraint_count": 8,
        },
        "refresh_triggers": [
            "active_frame_change",
            "goal_change",
            "accepted_ontology_delta",
            "failure_signal",
            "verification_result",
            "action_intent_completion",
            "user_pin_unpin",
            "session_resume",
            "explicit_refresh_request"
        ],
        "members": members,
    })
}

pub fn active_mission_slice_summary(
    focusa: &FocusaState,
    frame_id: Option<&str>,
) -> Option<String> {
    let payload = slice_payload(focusa, frame_id, "active_mission");
    let members = payload.get("members").and_then(|v| v.as_array())?;
    if members.is_empty() {
        return None;
    }
    let mut lines = Vec::new();
    for member in members.iter().take(6) {
        let object_type = member
            .get("object_type")
            .and_then(|v| v.as_str())
            .unwrap_or("object");
        let id = member.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let reason = member
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("relevant now");
        lines.push(format!("- {} :: {} ({})", object_type, id, reason));
    }
    Some(format!(
        "BOUNDED ONTOLOGY SLICE (active_mission):\n{}\nUse this slice before broad project recall when it is relevant.",
        lines.join("\n")
    ))
}

async fn primitives() -> Json<Value> {
    primitive_contracts()
}

async fn contracts() -> Json<Value> {
    Json(json!({
        "contracts": ACTION_TYPES.iter().map(|name| action_contract(name)).collect::<Vec<_>>()
    }))
}

async fn world(
    Query(query): Query<OntologyWorldQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let projection = combined_projection(&focusa, query.frame_id.as_deref());
    let action_catalog: Vec<Value> = ACTION_TYPES
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "constraint_checked": true,
                "reducer_visible": true,
                "verification_hooks": ["runtime_gate"],
            })
        })
        .collect();

    Json(json!({
        "object_count": projection.objects.len(),
        "link_count": projection.links.len(),
        "objects": projection.objects,
        "links": projection.links,
        "action_catalog": action_catalog,
        "working_sets": {
            "active_mission_set": slice_payload(&focusa, query.frame_id.as_deref(), "active_mission"),
            "debugging_set": slice_payload(&focusa, query.frame_id.as_deref(), "debugging"),
            "refactor_set": slice_payload(&focusa, query.frame_id.as_deref(), "refactor"),
            "regression_set": slice_payload(&focusa, query.frame_id.as_deref(), "regression"),
            "architecture_set": slice_payload(&focusa, query.frame_id.as_deref(), "architecture"),
        }
    }))
}

async fn slices(
    Query(query): Query<SliceQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(slice_payload(
        &focusa,
        query.frame_id.as_deref(),
        &query.slice_type,
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ontology/primitives", get(primitives))
        .route("/v1/ontology/contracts", get(contracts))
        .route("/v1/ontology/world", get(world))
        .route("/v1/ontology/slices", get(slices))
}
