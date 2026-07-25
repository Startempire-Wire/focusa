//! Spec96 public surgical traversal facade.
//!
//! Read-only bounded traversal over large Focusa surfaces. This route is a
//! facade; individual domain routes remain authoritative for mutations and
//! deep/cold reads.

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, bounded_window, budgeted_default_limit,
    budgeted_hard_limit, budgeted_requested_limit, field_projection,
    full_payload_blocked_by_pressure, project_json_fields,
};
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::State;
use axum::{Json, Router, routing::post};
use focusa_core::types::{CltNodeType, FocusaState, FrameStatus};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct TraverseRequest {
    pub surface: String,
    pub selector: Option<String>,
    pub anchor: Option<String>,
    pub query: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub depth: Option<usize>,
    pub radius: Option<usize>,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub tags: Vec<Value>,
    pub tag_mode: Option<String>,
    #[serde(default, alias = "include_payload")]
    pub include_full_payload: bool,
    #[serde(default)]
    pub include_rehydrate_refs: bool,
    pub budget_tokens: Option<usize>,
    pub session_identity: Option<Value>,
    #[serde(default)]
    pub force_full_payload: bool,
}

fn default_limit(surface: &str) -> usize {
    match surface {
        "snapshots" => budgeted_default_limit("FOCUSA_TRAVERSE_SNAPSHOTS_DEFAULT_LIMIT", 10),
        "trajectory" => budgeted_default_limit("FOCUSA_TRAVERSE_TRAJECTORY_DEFAULT_LIMIT", 5),
        "workpoints" | "metacognition" | "predictions" => {
            budgeted_default_limit("FOCUSA_TRAVERSE_DEFAULT_LIMIT", 10)
        }
        _ => budgeted_default_limit("FOCUSA_TRAVERSE_DEFAULT_LIMIT", 25),
    }
}

fn full_limit(surface: &str, default: usize) -> usize {
    match surface {
        "lineage" | "ontology" | "telemetry" | "evidence" | "references" => {
            budgeted_hard_limit("FOCUSA_TRAVERSE_FULL_LIMIT", 200, default)
        }
        _ => budgeted_hard_limit("FOCUSA_TRAVERSE_FULL_LIMIT", 100, default),
    }
}

fn normalize_surface(surface: &str) -> String {
    surface.trim().to_ascii_lowercase().replace('-', "_")
}

fn selector(req: &TraverseRequest) -> String {
    req.selector
        .as_deref()
        .unwrap_or("window")
        .trim()
        .to_ascii_lowercase()
}

fn fields_csv(fields: &[String]) -> Option<String> {
    (!fields.is_empty()).then(|| fields.join(","))
}

fn bounded_json_items(
    items: Vec<Value>,
    req: &TraverseRequest,
    surface: &str,
    default_fields: &[&str],
    allowed_fields: &[&str],
) -> (Vec<Value>, Value, Value, bool) {
    let default_limit = default_limit(surface);
    let full_limit = full_limit(surface, default_limit);
    let full_blocked =
        full_payload_blocked_by_pressure(req.include_full_payload, req.force_full_payload);
    let include_full_payload = req.include_full_payload && !full_blocked;
    let ceiling = if include_full_payload {
        full_limit
    } else {
        default_limit
    };
    let limit = budgeted_requested_limit(req.limit, default_limit.min(ceiling), ceiling);
    let fields = fields_csv(&req.fields);
    let mut projection = field_projection(fields.as_deref(), default_fields, allowed_fields);
    let projection_fallback = !req.fields.is_empty() && projection.applied.is_empty();
    if projection_fallback {
        let defaults = field_projection(None, default_fields, allowed_fields);
        projection.applied = defaults.applied;
    }
    let projected = items
        .iter()
        .map(|item| project_json_fields(item, &projection))
        .collect::<Vec<_>>();
    let total = projected.len();
    let (window, cursor_window) = bounded_window(&projected, req.cursor.as_deref(), limit);
    let metadata = json!(bounded_metadata(
        total,
        window.len(),
        BoundedReadOptions {
            requested_limit: req.limit,
            include_full_payload,
            summary_only: !include_full_payload,
            cursor: req.cursor.clone(),
            next_cursor: cursor_window.next_cursor.clone(),
            default_limit,
            full_limit,
        },
    ));
    let mut projection_json = json!(projection);
    if let Some(object) = projection_json.as_object_mut() {
        object.insert(
            "fallback_to_defaults".to_string(),
            json!(projection_fallback),
        );
    }
    (window, metadata, projection_json, full_blocked)
}

fn value_id(value: &Value) -> String {
    value
        .get("node_id")
        .or_else(|| value.get("id"))
        .or_else(|| value.get("event_id"))
        .or_else(|| value.get("workpoint_id"))
        .or_else(|| value.get("primitive_id"))
        .or_else(|| value.get("frame_id"))
        .or_else(|| value.get("prediction_id"))
        .or_else(|| value.get("tool"))
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("item")
        .to_string()
}

fn digest_text(text: &str, len: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let hex = hex::encode(hasher.finalize());
    hex.chars().take(len.clamp(8, 64)).collect()
}

fn stable_value_digest(value: &Value) -> String {
    digest_text(&serde_json::to_string(value).unwrap_or_default(), 24)
}

const TRUST_BADGE_VOCABULARY: &[&str] = &[
    "canonical",
    "advisory",
    "projected",
    "stale",
    "degraded",
    "blocked",
    "spec_only",
    "partial",
    "verified",
    "unsafe_scope",
];

fn profile_selector_catalog() -> Vec<Value> {
    vec![
        json!({"id":"daily_driver","profile_id":"daily_driver","label":"Daily Driver","BLOATGAURD_PROFILE":"Daily Driver","CONTEXT_POSTURE":"balanced","FULL_PAYLOAD":"cold opt-in","availability":"implemented","authority":"render_policy_only","mutates":false,"trust_badges":["implemented","verified"]}),
        json!({"id":"beast_mode","profile_id":"beast_mode","label":"Beast Mode","BLOATGAURD_PROFILE":"Beast Mode","CONTEXT_POSTURE":"broad context, bounded handles","FULL_PAYLOAD":"cold opt-in required","availability":"partial","authority":"render_policy_only","mutates":false,"trust_badges":["partial","advisory"]}),
        json!({"id":"speedy","profile_id":"speedy","label":"Speedy","BLOATGAURD_PROFILE":"Speedy","CONTEXT_POSTURE":"low-token fast path","FULL_PAYLOAD":"off by default","availability":"implemented","authority":"render_policy_only","mutates":false,"trust_badges":["implemented","verified"]}),
        json!({"id":"neat_freak","profile_id":"neat_freak","label":"Neat Freak","BLOATGAURD_PROFILE":"Neat Freak","CONTEXT_POSTURE":"audit/cleanup","FULL_PAYLOAD":"cold opt-in","availability":"partial","authority":"render_policy_only","mutates":false,"trust_badges":["partial","advisory"]}),
        json!({"id":"tightwad","profile_id":"tightwad","label":"Tightwad","BLOATGAURD_PROFILE":"Tightwad","CONTEXT_POSTURE":"strict budget","FULL_PAYLOAD":"blocked unless explicit","availability":"partial","authority":"render_policy_only","mutates":false,"trust_badges":["partial","advisory"]}),
    ]
}

fn profile_selector_payload(items: &[Value]) -> Value {
    json!({"schema":"focusa.profile_selector.v1","profile_count":items.len(),"authority":"profiles change model-visible render/posture only; Workpoint/evidence remain authority"})
}

fn routine_commands_catalog() -> Vec<Value> {
    vec![
        json!({"id":"scout","routine_id":"scout","label":"The Scout","purpose":"choose route","command":"focusa routine scout","availability":"partial","requires_verified_scope":true,"mutates":false,"trust_badges":["partial","advisory"]}),
        json!({"id":"librarian","routine_id":"librarian","label":"The Librarian","purpose":"compile context","command":"focusa routine librarian","availability":"spec_only","requires_verified_scope":true,"mutates":false,"trust_badges":["spec_only","advisory"]}),
        json!({"id":"squeezer","routine_id":"squeezer","label":"The Squeezer","purpose":"compact tool history","command":"focusa routine squeezer","availability":"partial","requires_verified_scope":true,"mutates":false,"trust_badges":["partial","advisory"]}),
        json!({"id":"deep_dive","routine_id":"deep_dive","label":"The Deep Dive","purpose":"rehydrate exact proof","command":"focusa routine deep_dive","availability":"partial","requires_verified_scope":true,"mutates":false,"trust_badges":["partial","advisory"]}),
        json!({"id":"gatekeeper","routine_id":"gatekeeper","label":"The Gatekeeper","purpose":"strict check","command":"focusa routine gatekeeper","availability":"implemented","requires_verified_scope":true,"mutates":false,"trust_badges":["implemented","verified"]}),
    ]
}

fn routine_commands_payload(items: &[Value]) -> Value {
    json!({"schema":"focusa.routine_commands.v1","routine_count":items.len(),"policy":"routine commands are discovery affordances; automatic routines require verified project_root+continuity_id and do not delete/archive/rewrite code"})
}

fn spec_availability_registry() -> Vec<Value> {
    vec![
        json!({"id":"Spec100","spec_id":"Spec100","feature":"Context Cognition","availability":"spec_only","runtime_entrypoint":Value::Null,"docs_ref":"docs/100-context-cognition-spec.md","first_implementation_slice":"focusa-pm2b.24","trust_badges":["spec_only","advisory"],"SpecRuntimeAvailabilityLabel":"spec_only"}),
        json!({"id":"Spec101","spec_id":"Spec101","feature":"Focusa Bloatgaurd","availability":"partial","runtime_entrypoint":Value::Null,"docs_ref":"docs/101-focusa-bloatgaurd-spec.md","first_implementation_slice":"focusa-pm2b.24","trust_badges":["partial","advisory"],"SpecRuntimeAvailabilityLabel":"partial"}),
        json!({"id":"Spec102","spec_id":"Spec102","feature":"Agent UX composition and real-life repair backlog","availability":"implemented","runtime_entrypoint":"/v1/traverse + tests/spec102_*","docs_ref":"docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md","first_implementation_slice":"focusa-pm2b","trust_badges":["implemented","verified"],"SpecRuntimeAvailabilityLabel":"implemented"}),
        json!({"id":"deprecated-singleton-current","spec_id":"LegacyCurrentSingleton","feature":"Singleton current/active authority surfaces","availability":"deprecated","runtime_entrypoint":Value::Null,"docs_ref":"docs/102-focusa-agent-ux-composition-and-real-life-test-spec.md#13","first_implementation_slice":"Spec98/99 migration","trust_badges":["deprecated","advisory"],"SpecRuntimeAvailabilityLabel":"deprecated"}),
    ]
}

fn spec_availability_payload(items: &[Value]) -> Value {
    let mut counts = BTreeMap::<String, usize>::new();
    for item in items {
        if let Some(status) = item.get("availability").and_then(Value::as_str) {
            *counts.entry(status.to_string()).or_insert(0) += 1;
        }
    }
    json!({"schema":"focusa.spec_availability.v1","availability_counts":counts,"happy_path_rule":"implemented runtime features omit spec_only caveats","labels":["spec_only","partial","implemented","deprecated"]})
}

fn verbosity_profile_catalog() -> Vec<Value> {
    vec![
        json!({"id":"operator","profile":"operator","compact_fields":["status","next_action","proof","blocker"],"detail_fields":["summary","trust_badges"],"hidden_by_default":["debug_payload","raw_payload","internal_scores"],"escalation_fields":["blocker","risk","operator_decision_needed"]}),
        json!({"id":"coding_agent","profile":"coding_agent","compact_fields":["status","next_action","target_files","tool"],"detail_fields":["workpoint_id","route_recommendation","tests_run","evidence_refs"],"hidden_by_default":["raw_payload","deep_lineage"],"escalation_fields":["failure_class","blocked_surface","safe_recovery"]}),
        json!({"id":"qa_agent","profile":"qa_agent","compact_fields":["status","tests_run","proof","residual_risk"],"detail_fields":["bead_review","clean_repair_checklist","evidence_diff"],"hidden_by_default":["raw_payload","debug_payload"],"escalation_fields":["missing_proof","regressions","residual_ui_risk"]}),
        json!({"id":"release_agent","profile":"release_agent","compact_fields":["status","readiness","risks","gates"],"detail_fields":["release_checks","rollback_card","change_feed"],"hidden_by_default":["debug_payload","raw_payload"],"escalation_fields":["blocked","unsafe_scope","regressions"]}),
        json!({"id":"debug_agent","profile":"debug_agent","compact_fields":["status","failure_class","route"],"detail_fields":["raw_payload","debug_payload","tool_result_v1","telemetry_refs"],"hidden_by_default":[],"escalation_fields":["failure_class","stack_trace","raw_payload"]}),
    ]
}

fn verbosity_profile_payload(items: &[Value]) -> Value {
    items.first().cloned().unwrap_or(Value::Null)
}

fn change_feed_items(state: &FocusaState, req: &TraverseRequest) -> Vec<Value> {
    let query = req
        .query
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    state
        .workpoint
        .records
        .iter()
        .filter(|record| {
            let haystack = [
                record.workpoint_id.to_string(),
                record.work_item_id.clone().unwrap_or_default(),
                record.session_id.clone().unwrap_or_default(),
                record.mission.clone().unwrap_or_default(),
                record.next_slice.clone().unwrap_or_default(),
                record.active_object_refs.join(" "),
                record
                    .verification_records
                    .iter()
                    .map(|verification| {
                        format!(
                            "{} {} {}",
                            verification.target_ref,
                            verification.result,
                            verification.evidence_ref.clone().unwrap_or_default()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            ]
            .join(" ")
            .to_ascii_lowercase();
            haystack.contains(&query)
        })
        .filter_map(|record| serde_json::to_value(record).ok())
        .collect()
}

fn change_feed_payload(items: &[Value], query: Option<&str>) -> Value {
    let mut files_changed = Vec::<String>::new();
    let mut beads_changed = Vec::<String>::new();
    let mut workpoints_changed = Vec::<String>::new();
    let mut evidence_changed = Vec::<String>::new();
    let mut agents_changed = Vec::<String>::new();
    for item in items {
        if let Some(id) = item.get("workpoint_id").and_then(Value::as_str) {
            workpoints_changed.push(id.to_string());
        }
        if let Some(bead) = item.get("work_item_id").and_then(Value::as_str) {
            beads_changed.push(bead.to_string());
        }
        if let Some(agent) = item.get("session_id").and_then(Value::as_str) {
            agents_changed.push(agent.to_string());
        }
        if let Some(files) = item.get("active_object_refs").and_then(Value::as_array) {
            for file in files.iter().filter_map(Value::as_str) {
                files_changed.push(file.to_string());
            }
        }
        if let Some(evidence) = item.get("verification_records").and_then(Value::as_array) {
            for proof in evidence {
                if let Some(evidence_ref) = proof.get("evidence_ref").and_then(Value::as_str) {
                    evidence_changed.push(evidence_ref.to_string());
                }
            }
        }
    }
    files_changed.sort();
    files_changed.dedup();
    beads_changed.sort();
    beads_changed.dedup();
    workpoints_changed.sort();
    workpoints_changed.dedup();
    evidence_changed.sort();
    evidence_changed.dedup();
    agents_changed.sort();
    agents_changed.dedup();
    let attention_required = !(files_changed.is_empty()
        && beads_changed.is_empty()
        && workpoints_changed.is_empty()
        && evidence_changed.is_empty()
        && agents_changed.is_empty());
    json!({
        "since": query,
        "files_changed": files_changed,
        "beads_changed": beads_changed,
        "workpoints_changed": workpoints_changed,
        "evidence_changed": evidence_changed,
        "predictions_changed": [],
        "agents_changed": agents_changed,
        "attention_required": attention_required,
        "summary": if attention_required { "changes changed; attention required" } else { "changes: none relevant" },
    })
}

fn command_palette_catalog() -> Vec<Value> {
    vec![
        json!({"id":"resume_work","label":"Resume work","tool":"focusa_workpoint_resume","args_preview":{"project_root":"/workspace/focusa-project","mode":"compact_prompt"},"when":"after compaction/resume or before choosing next action"}),
        json!({"id":"link_proof","label":"Link proof","tool":"focusa_evidence_capture","args_preview":{"target_ref":"file/test/api","result":"PASS summary","evidence_ref":"evidence:handle"},"when":"after test/API/file proof changes confidence"}),
        json!({"id":"start_next_bead","label":"Start next bead","tool":"focusa_workpoint_checkpoint","args_preview":{"work_item_id":"focusa-pm2b.N","mission":"next bead"},"when":"before durable implementation work"}),
        json!({"id":"explain_conflict","label":"Explain conflict","tool":"focusa_project_verify","args_preview":{"project_root":"/workspace/focusa-project"},"when":"scope or authority signals conflict"}),
        json!({"id":"make_repair_report","label":"Make repair report","tool":"scripts/spec102-repair-report","args_preview":{"epic":"focusa-pm2b"},"when":"after closing a repair bead"}),
        json!({"id":"run_clean_repair_check","label":"Run clean-repair check","tool":"tests/spec102_proof_matrix_enforcement_test.sh","args_preview":{},"when":"before closing Spec102 repair work"}),
    ]
}

fn command_palette_payload(items: &[Value], selector: &str) -> Value {
    json!({
        "mode": if selector == "full" { "full" } else { "top" },
        "commands": items,
        "full_palette_available": selector != "full",
    })
}

fn recovery_playbook_catalog() -> Vec<Value> {
    vec![
        json!({"id":"project_identity_mismatch","scenario":"project_identity_mismatch","symptoms":["project_root mismatch","saved scope differs from operator ask"],"first_safe_tool":"focusa_project_verify","next_tools":["focusa_project_identity","focusa_workpoint_checkpoint"],"proof_to_capture":"verified project_root/repo/continuity evidence","stop_conditions":["verified project_root and continuity match current ask"]}),
        json!({"id":"unsafe_broad_cwd","scenario":"unsafe_broad_cwd","symptoms":["cwd is /root or broad/unsafe"],"first_safe_tool":"focusa_project_identity","next_tools":["focusa_project_verify","focusa_workpoint_checkpoint"],"proof_to_capture":"explicit project identity with safe root","stop_conditions":["safe project_root verified"]}),
        json!({"id":"stale_trajectory","scenario":"stale_trajectory","symptoms":["trajectory provisional/stale/missing evidence"],"first_safe_tool":"focusa_trajectory_view","next_tools":["focusa_trajectory_assess","focusa_trajectory_define_goal"],"proof_to_capture":"trajectory state/gap evidence or operator-confirmed goal","stop_conditions":["trajectory has current state, desired state, active gap"]}),
        json!({"id":"wrong_workpoint_id","scenario":"wrong_workpoint_id","symptoms":["requested Workpoint missing or fallback risk"],"first_safe_tool":"focusa_workpoint_resume","next_tools":["focusa_traverse","focusa_workpoint_checkpoint"],"proof_to_capture":"canonical Workpoint id in verified project scope","stop_conditions":["requested id resolves or new scoped checkpoint exists"]}),
        json!({"id":"focus_state_blocked","scenario":"focus_state_blocked","symptoms":["Focus State write blocked or frame unavailable"],"first_safe_tool":"focusa_scratch","next_tools":["focusa_project_identity","focusa_workpoint_resume"],"proof_to_capture":"scratch fallback plus canonical Workpoint resume","stop_conditions":["frame/scope reloaded or note safely stored in scratch"]}),
        json!({"id":"evidence_index_lag","scenario":"evidence_index_lag","symptoms":["linked evidence pending or not visible"],"first_safe_tool":"focusa_workpoint_resume","next_tools":["focusa_traverse","focusa_resource_mode"],"proof_to_capture":"evidence_ref visible in Workpoint or artifact browser","stop_conditions":["evidence appears in scoped traversal"]}),
        json!({"id":"ontology_selector_empty","scenario":"ontology_selector_empty","symptoms":["ontology selector returns zero unexpectedly"],"first_safe_tool":"focusa_traverse","next_tools":["focusa_project_card","focusa_tool_doctor"],"proof_to_capture":"source_index/scope_key/count_semantics for selected ontology layer","stop_conditions":["correct selector/source explains zero or returns objects"]}),
        json!({"id":"doctor_ready_blocked_ambiguity","scenario":"doctor_ready_blocked_ambiguity","symptoms":["doctor ready vs blocked surfaces conflict"],"first_safe_tool":"focusa_tool_doctor","next_tools":["focusa_project_identity","focusa_workpoint_resume"],"proof_to_capture":"readiness plane and failing component evidence","stop_conditions":["runtime/project/workpoint/source planes separated"]}),
        json!({"id":"uiai_pressure","scenario":"uiai_pressure","symptoms":["UIAI pressure or diagnostics confusion"],"first_safe_tool":"uiai_health","next_tools":["uiai_browser_diagnostics","focusa_browser_diagnostics_intake"],"proof_to_capture":"health/diagnostics packet with current failures separated from history","stop_conditions":["current UIAI condition and next browser action clear"]}),
        json!({"id":"stuck_loop_no_confidence_change","scenario":"stuck_loop_no_confidence_change","symptoms":["repeated resume/checkpoint/search without proof delta"],"first_safe_tool":"focusa_traverse","next_tools":["focusa_evidence_capture","focusa_predict_record"],"proof_to_capture":"confidence-changing evidence or operator-chosen route change","stop_conditions":["new evidence changes confidence or route changes"]}),
    ]
}

fn recovery_playbook_payload(items: &[Value]) -> Value {
    items.first().cloned().unwrap_or(Value::Null)
}

fn evidence_diff_payload(surface: &str, items: &[Value]) -> Value {
    if !matches!(surface, "evidence" | "ecs" | "references") {
        return Value::Null;
    }
    if items.len() < 2 {
        return json!({
            "before_ref": items.first().and_then(|item| item.get("evidence_ref").or_else(|| item.get("id"))).cloned().unwrap_or(Value::Null),
            "after_ref": Value::Null,
            "changed_claims": [],
            "confidence_delta": "no_confidence_change",
            "regressions": [],
            "stale_refs_removed": [],
            "new_followups": ["capture or link the next proof that changes the current claim confidence"],
        });
    }
    let before = &items[0];
    let after = &items[items.len() - 1];
    let before_ref = before
        .get("evidence_ref")
        .or_else(|| before.get("id"))
        .cloned()
        .unwrap_or(Value::Null);
    let after_ref = after
        .get("evidence_ref")
        .or_else(|| after.get("id"))
        .cloned()
        .unwrap_or(Value::Null);
    let before_claim = before
        .get("summary")
        .or_else(|| before.get("result"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let after_claim = after
        .get("summary")
        .or_else(|| after.get("result"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let changed_claims = if before_claim != after_claim {
        vec![json!({"before": before_claim, "after": after_claim})]
    } else {
        Vec::new()
    };
    let confidence_delta = if changed_claims.is_empty() {
        "no_confidence_change"
    } else {
        "increased/proof_linked"
    };
    json!({
        "before_ref": before_ref,
        "after_ref": after_ref,
        "changed_claims": changed_claims,
        "confidence_delta": confidence_delta,
        "regressions": [],
        "stale_refs_removed": [],
        "new_followups": if confidence_delta == "no_confidence_change" { vec!["next proof should capture a different passing test/API/file result or remove stale evidence"] } else { Vec::<&str>::new() },
    })
}

fn stuck_loop_payload(surface: &str, items: &[Value]) -> Value {
    if !matches!(
        surface,
        "workpoints" | "workpoint" | "evidence" | "ecs" | "references"
    ) {
        return Value::Null;
    }
    let repeated_actions = items
        .iter()
        .filter_map(|item| {
            let mission = item.get("mission").and_then(Value::as_str).unwrap_or("");
            let next = item.get("next_slice").and_then(Value::as_str).unwrap_or("");
            let work_item = item
                .get("work_item_id")
                .and_then(Value::as_str)
                .unwrap_or("");
            if mission.is_empty() && next.is_empty() && work_item.is_empty() {
                None
            } else {
                Some(format!(
                    "work_item={work_item}; mission={mission}; next={next}"
                ))
            }
        })
        .collect::<Vec<_>>();
    let has_confidence_change = items.iter().any(|item| {
        item.get("confidence_delta")
            .and_then(Value::as_str)
            .is_some_and(|delta| delta != "none")
            || item
                .get("verified_evidence_refs")
                .and_then(Value::as_array)
                .is_some_and(|refs| !refs.is_empty())
            || item
                .get("evidence_ref")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    });
    let detected = repeated_actions.len() >= 3 && !has_confidence_change;
    if !detected {
        return json!({"detected": false});
    }
    json!({
        "detected": true,
        "repeated_actions": repeated_actions.into_iter().take(5).collect::<Vec<_>>(),
        "last_confidence_change": "none/no_confidence_change",
        "likely_cause": "repeated route/checkpoint cycle without linked proof or confidence-changing evidence",
        "break_glass_action": "link evidence that changes confidence, change route, or ask operator to choose a different action boundary",
    })
}

fn empty_state_payload(
    surface: &str,
    selector: &str,
    query: Option<&str>,
    returned: usize,
    supported: bool,
    full_payload_blocked: bool,
) -> Value {
    if returned > 0 {
        return Value::Null;
    }
    let empty_because = if !supported {
        "wrong_selector"
    } else if full_payload_blocked {
        "cold_path_disabled"
    } else if matches!(surface, "telemetry" | "turns" | "commands") {
        "not_checked"
    } else {
        "none_exist"
    };
    json!({
        "empty_because": empty_because,
        "scope": {"surface": surface, "query": query},
        "selector": selector,
        "next_selector": match empty_because {
            "wrong_selector" => "use a supported surface/selector such as workpoints, evidence, ontology, trajectory",
            "cold_path_disabled" => "retry with a narrower selector or resource-safe budget",
            "not_checked" => "run the relevant bounded status route first",
            _ => "adjust query/scope, or capture/link evidence if proof should exist",
        },
        "repair_or_retry": match empty_because {
            "wrong_selector" => "choose a supported surface/selector; this empty result is not proof of absence",
            "cold_path_disabled" => "reduce payload request or use focusa_resource_mode before retry",
            "not_checked" => "not checked in this bounded route; use a specific read/status tool",
            _ => "true empty for this current scope/query; verify selector/scope or capture/link evidence",
        },
        "vocabulary": ["none_exist", "wrong_selector", "wrong_scope", "index_unavailable", "permission_blocked", "cold_path_disabled", "not_checked"],
    })
}

fn route_recommendation_payload(surface: &str, selector: &str, degraded: bool) -> Value {
    json!({
        "recommended_tool": "focusa_traverse",
        "why": if degraded { "bounded retry or narrower selector is safer than broad/cold reads" } else { "bounded traversal answers the requested slice without full lineage or ontology graph" },
        "expected_output": match surface {
            "evidence" | "ecs" | "references" => "bounded evidence/artifact slice with count, source, freshness, and rehydrate refs",
            "workpoints" | "workpoint" => "bounded Workpoint slice with current ids and next actions",
            _ => "bounded items slice with metadata, cursor, and rehydrate refs",
        },
        "confidence": if degraded { "medium" } else { "high" },
        "alternatives": ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_active_object_resolve"],
        "avoid": ["full lineage tree", "full ontology graph", "full telemetry logs", "transcript tail as authority"],
        "selector": selector,
    })
}

fn trust_badges(
    canonical: bool,
    degraded: bool,
    blocked: bool,
    projected: bool,
    partial: bool,
    unsafe_scope: bool,
) -> Vec<&'static str> {
    let _ = TRUST_BADGE_VOCABULARY;
    if blocked {
        return vec!["blocked", "degraded"];
    }
    if unsafe_scope {
        return vec!["unsafe_scope", "degraded"];
    }
    if degraded {
        return vec!["degraded"];
    }
    if partial {
        return vec!["partial", "advisory"];
    }
    if projected {
        return vec!["projected", "advisory"];
    }
    if canonical {
        vec!["canonical", "verified"]
    } else {
        vec!["advisory"]
    }
}

fn tag_component(value: &str) -> String {
    let clean = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let clean = clean.trim_matches('_');
    if clean.is_empty() {
        "item".to_string()
    } else {
        clean.chars().take(96).collect()
    }
}

fn make_tag(surface: &str, selector: &str, mode: &str, anchor: &str, digest: &str) -> String {
    format!(
        "focusa://{}/{}/{}/{}/{}",
        tag_component(surface),
        tag_component(selector),
        tag_component(mode),
        tag_component(anchor),
        tag_component(digest)
    )
}

fn tag_record(
    surface: &str,
    selector: &str,
    mode: &str,
    anchor: &str,
    digest: &str,
    index: Option<usize>,
) -> Value {
    json!({
        "tag": make_tag(surface, selector, mode, anchor, digest),
        "tag_mode": mode,
        "surface": surface,
        "selector": selector,
        "anchor": anchor,
        "digest": digest,
        "index": index,
        "collision_policy": "sha256_24_hex_with_anchor; on collision request fields plus longer tag",
        "long_tag_policy": "stable 24-hex digest by default; clients may request full 64-hex verification via future tag_version",
    })
}

fn item_tags(surface: &str, selector: &str, items: &[Value]) -> Vec<Value> {
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let anchor = value_id(item);
            let digest = stable_value_digest(item);
            tag_record(surface, selector, "item", &anchor, &digest, Some(idx))
        })
        .collect()
}

fn aggregate_digest(items: &[Value]) -> String {
    let parts = items
        .iter()
        .map(|item| format!("{}:{}", value_id(item), stable_value_digest(item)))
        .collect::<Vec<_>>()
        .join("|");
    digest_text(&parts, 24)
}

fn aggregate_tags(surface: &str, selector: &str, items: &[Value], traversal: &Value) -> Vec<Value> {
    let cursor = traversal
        .get("cursor")
        .and_then(Value::as_str)
        .unwrap_or("0");
    let next_string = traversal
        .get("next_cursor")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            traversal
                .get("returned")
                .and_then(Value::as_u64)
                .map(|n| n.to_string())
        })
        .unwrap_or_else(|| "0".to_string());
    let next = next_string.as_str();
    let limit = traversal
        .get("limit")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| items.len().to_string());
    let total = traversal
        .get("total")
        .and_then(Value::as_u64)
        .map(|n| n.to_string())
        .unwrap_or_else(|| items.len().to_string());
    let digest = aggregate_digest(items);
    vec![
        tag_record(
            surface,
            selector,
            "range",
            &format!("{cursor}-{next}"),
            &digest,
            None,
        ),
        tag_record(
            surface,
            selector,
            "window",
            &format!("{cursor}:{limit}"),
            &digest,
            None,
        ),
        tag_record(
            surface,
            selector,
            "surface",
            &format!("{surface}:{total}"),
            &digest_text(&format!("{surface}:{total}:{digest}"), 24),
            None,
        ),
    ]
}

fn parse_tag(tag: &str) -> Option<(String, String, String, String, String)> {
    let rest = tag.strip_prefix("focusa://")?;
    let parts = rest.split('/').collect::<Vec<_>>();
    if parts.len() != 5 {
        return None;
    }
    Some((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
        parts[3].to_string(),
        parts[4].to_string(),
    ))
}

fn tag_index(records: &[Value]) -> BTreeMap<String, Value> {
    records
        .iter()
        .filter_map(|record| {
            record
                .get("tag")
                .and_then(Value::as_str)
                .map(|tag| (tag.to_string(), record.clone()))
        })
        .collect()
}

fn scope_from_item(item: &Value) -> Value {
    json!({
        "project_root": item.get("project_root").cloned().unwrap_or(Value::Null),
        "session_id": item.get("session_id").cloned().unwrap_or(Value::Null),
        "frame_id": item.get("frame_id").or_else(|| item.get("id")).cloned().unwrap_or(Value::Null),
        "workpoint_id": item.get("workpoint_id").cloned().unwrap_or(Value::Null),
    })
}

fn traversed_items(surface: &str, selector: &str, items: &[Value]) -> Vec<Value> {
    let item_records = item_tags(surface, selector, items);
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let tag = item_records
                .get(idx)
                .and_then(|record| record.get("tag"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            json!({
                "anchor": value_id(item),
                "ordinal": idx,
                "tag": tag,
                "surface_version": stable_value_digest(item),
                "freshness": "live",
                "scope": scope_from_item(item),
                "kind": item.get("kind").or_else(|| item.get("node_type")).or_else(|| item.get("event_type")).or_else(|| item.get("status")).or_else(|| item.get("type")).cloned().unwrap_or(Value::Null),
                "label": item.get("label").or_else(|| item.get("title")).or_else(|| item.get("work_item_id")).or_else(|| item.get("tool")).or_else(|| item.get("name")).or_else(|| item.get("event_type")).cloned().unwrap_or(Value::Null),
                "summary": item.get("summary").or_else(|| item.get("mission")).or_else(|| item.get("next_slice")).or_else(|| item.get("message")).or_else(|| item.get("result")).or_else(|| item.get("timestamp")).cloned().unwrap_or(Value::Null),
                "data": item,
            })
        })
        .collect()
}

fn requested_tag_strings(req: &TraverseRequest) -> Vec<String> {
    req.tags
        .iter()
        .filter_map(|tag| {
            tag.as_str()
                .map(str::to_string)
                .or_else(|| tag.get("tag").and_then(Value::as_str).map(str::to_string))
        })
        .collect()
}

fn adopt_verify_selector_from_requested_tags(req: &mut TraverseRequest) {
    if selector(req) != "tags_verify" {
        return;
    }
    if let Some(tag_selector) = requested_tag_strings(req)
        .into_iter()
        .find_map(|tag| parse_tag(&tag).map(|(_, selector, _, _, _)| selector))
        .filter(|selector| !selector.trim().is_empty())
    {
        req.selector = Some(tag_selector);
    }
}

fn verify_requested_tags(
    req: &TraverseRequest,
    items: &[Value],
    traversal: &Value,
) -> (Vec<Value>, Vec<Value>) {
    let surface = normalize_surface(&req.surface);
    let sel = selector(req);
    let mut current_records = item_tags(&surface, &sel, items);
    current_records.extend(aggregate_tags(&surface, &sel, items, traversal));
    let current = tag_index(&current_records);
    let mut verified = Vec::new();
    let mut stale = Vec::new();
    for tag in requested_tag_strings(req) {
        match parse_tag(&tag) {
            Some((tag_surface, tag_selector, mode, anchor, digest)) => {
                if let Some(record) = current.get(&tag) {
                    verified.push(json!({
                        "tag": tag,
                        "tag_mode": mode,
                        "surface": tag_surface,
                        "selector": tag_selector,
                        "anchor": anchor,
                        "digest": digest,
                        "verified": true,
                        "record": record,
                    }));
                } else {
                    stale.push(json!({
                        "tag": tag,
                        "tag_mode": mode,
                        "surface": tag_surface,
                        "selector": tag_selector,
                        "anchor": anchor,
                        "digest": digest,
                        "verified": false,
                        "reason": "tag digest, anchor, selector, or window no longer matches current bounded slice",
                    }));
                }
            }
            None => {
                stale.push(json!({"tag": tag, "verified": false, "reason": "invalid_tag_format"}))
            }
        }
    }
    (verified, stale)
}

fn active_frame_value(state: &FocusaState) -> Option<Value> {
    let frame = state
        .focus_stack
        .active_id
        .and_then(|id| state.focus_stack.frames.iter().find(|frame| frame.id == id))
        .or_else(|| state.focus_stack.frames.last())?;
    serde_json::to_value(frame).ok()
}

fn active_workpoint_value(state: &FocusaState) -> Option<Value> {
    let record = state
        .workpoint
        .active_workpoint_id
        .and_then(|id| {
            state
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id == id)
        })
        .or_else(|| state.workpoint.records.last())?;
    serde_json::to_value(record).ok()
}

fn trajectory_items(state: &FocusaState) -> Vec<Value> {
    let frame = active_frame_value(state);
    let workpoint = active_workpoint_value(state);
    let ladder = state.trajectory_ladder_context();
    vec![json!({
        "id": "active_project_trajectory",
        "project_identity": {
            "project_root": frame.as_ref().and_then(|f| f.get("project_root")).cloned().unwrap_or(Value::Null),
            "continuity_id": frame.as_ref().and_then(|f| f.get("continuity_id")).cloned().unwrap_or(Value::Null),
            "workpoint_id": workpoint.as_ref().and_then(|w| w.get("workpoint_id")).cloned().unwrap_or(Value::Null),
        },
        "trajectory": {
            "long_term_goal": ladder
                .as_ref()
                .and_then(|ctx| ctx.hlt.clone())
                .map(Value::String)
                .or_else(|| frame.as_ref().and_then(|f| f.get("goal")).cloned())
                .unwrap_or(Value::Null),
            "mid_level_goal": ladder
                .as_ref()
                .and_then(|ctx| ctx.mlg.clone())
                .map(Value::String)
                .unwrap_or(Value::Null),
            "short_term_goal": ladder
                .as_ref()
                .and_then(|ctx| ctx.stg.clone())
                .map(Value::String)
                .unwrap_or(Value::Null),
            "waypoints": ladder
                .as_ref()
                .map(|ctx| json!(ctx.waypoints))
                .unwrap_or(Value::Null),
            "current_state": frame
                .as_ref()
                .and_then(|f| f.pointer("/focus_state/current_state"))
                .cloned()
                .unwrap_or(Value::Null),
            "active_gap": workpoint
                .as_ref()
                .and_then(|w| w.get("next_slice"))
                .cloned()
                .unwrap_or(Value::Null),
            "workpoint_candidate": workpoint,
            "trajectory_ladder": ladder,
        },
        "advisory_only": true,
    })]
}

fn metacognition_items(state: &FocusaState) -> Vec<Value> {
    let Some(frame) = active_frame_value(state) else {
        return Vec::new();
    };
    let focus_state = frame.get("focus_state").cloned().unwrap_or(Value::Null);
    let mut out = Vec::new();
    for (kind, pointer) in [
        ("decision", "/decisions"),
        ("constraint", "/constraints"),
        ("failure", "/failures"),
        ("recent_result", "/recent_results"),
        ("open_question", "/open_questions"),
    ] {
        if let Some(values) = focus_state.pointer(pointer).and_then(Value::as_array) {
            for (idx, value) in values.iter().enumerate() {
                out.push(json!({
                    "id": format!("{kind}:{idx}"),
                    "kind": kind,
                    "content": value,
                    "source": "focus_state_projection",
                }));
            }
        }
    }
    out
}

fn prediction_items(state: &FocusaState) -> Vec<Value> {
    vec![json!({
        "id": "prediction_stats_summary",
        "kind": "prediction_stats",
        "summary": "Use /v1/predictions/recent or focusa_predict_recent for persisted prediction records.",
        "telemetry_total_events": state.telemetry.total_events,
        "verification_result_events": state.telemetry.verification_result_events,
    })]
}

fn snapshot_items(state: &FocusaState) -> Vec<Value> {
    vec![json!({
        "id": "snapshot_current_head_summary",
        "kind": "snapshot_summary",
        "lineage_head": state.clt.head_id,
        "state_version": state.version,
        "summary": "Use focusa_tree_recent_snapshots for persisted snapshot records; traverse exposes current head metadata only by default.",
    })]
}

fn ownership_board_items(state: &FocusaState, req: &TraverseRequest) -> Vec<Value> {
    let query = req
        .query
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    state
        .workpoint
        .records
        .iter()
        .filter(|record| record.canonical)
        .filter(|record| {
            if query.is_empty() {
                return true;
            }
            let haystack = [
                record.workpoint_id.to_string(),
                record.work_item_id.clone().unwrap_or_default(),
                record.session_id.clone().unwrap_or_default(),
                record.project_root.clone().unwrap_or_default(),
                record.continuity_id.clone().unwrap_or_default(),
                record.mission.clone().unwrap_or_default(),
                record.next_slice.clone().unwrap_or_default(),
                record.active_object_refs.join(" "),
            ]
            .join(" ")
            .to_ascii_lowercase();
            haystack.contains(&query)
        })
        .map(|record| {
            let agent_id = record
                .session_id
                .clone()
                .or_else(|| {
                    record
                        .session_identity
                        .as_ref()
                        .and_then(|identity| identity.pi_session_id.clone())
                })
                .unwrap_or_else(|| format!("agent:workpoint:{}", record.workpoint_id));
            let touched_files = record.active_object_refs.clone();
            json!({
                "id": format!("ownership:{}", record.workpoint_id),
                "kind": "agent_ownership",
                "agent_id": agent_id,
                "owns": record.mission,
                "touched_files": touched_files,
                "last_activity": record.updated_at.or(record.created_at),
                "lease_status": if record.rejection_reason.is_none() { "active" } else { "released" },
                "workpoint_id": record.workpoint_id,
                "bead_id": record.work_item_id,
                "project_root": record.project_root,
                "continuity_id": record.continuity_id,
                "safe_next_action": "continue if ownership: clear; coordinate before edits when collision_risk is high",
            })
        })
        .collect()
}

fn artifact_group_by(sel: &str) -> String {
    match sel {
        "workpoint" | "bead" | "spec" | "file" | "test" | "confidence_change" => sel.to_string(),
        _ => "workpoint".to_string(),
    }
}

fn artifact_group_key(item: &Value, group_by: &str) -> Value {
    match group_by {
        "workpoint" => item.get("workpoint_id").cloned().unwrap_or(Value::Null),
        "bead" => item
            .get("bead_id")
            .or_else(|| item.get("work_item_id"))
            .cloned()
            .unwrap_or(Value::Null),
        "spec" => item
            .get("target_ref")
            .and_then(Value::as_str)
            .and_then(|target| target.split('#').next())
            .filter(|target| target.contains("spec") || target.contains("docs/"))
            .map(|target| json!(target))
            .unwrap_or(Value::Null),
        "file" => item.get("target_ref").cloned().unwrap_or(Value::Null),
        "test" => item
            .get("target_ref")
            .and_then(Value::as_str)
            .filter(|target| target.contains("test"))
            .map(|target| json!(target))
            .unwrap_or(Value::Null),
        "confidence_change" => item
            .get("confidence_delta")
            .cloned()
            .unwrap_or(json!("none")),
        _ => item.get("workpoint_id").cloned().unwrap_or(Value::Null),
    }
}

fn ownership_board_payload(items: &[Value]) -> Value {
    let active_agents = items
        .iter()
        .filter(|item| item.get("lease_status").and_then(Value::as_str) == Some("active"))
        .cloned()
        .collect::<Vec<_>>();
    let mut file_to_agents: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for agent in &active_agents {
        let agent_id = agent
            .get("agent_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown-agent")
            .to_string();
        if let Some(files) = agent.get("touched_files").and_then(Value::as_array) {
            for file in files.iter().filter_map(Value::as_str) {
                file_to_agents
                    .entry(file.to_string())
                    .or_default()
                    .push(agent_id.clone());
            }
        }
    }
    let collision_files = file_to_agents
        .iter()
        .filter(|(_, agents)| agents.len() > 1)
        .map(|(file, _)| file.clone())
        .collect::<Vec<_>>();
    let collision_risk = if collision_files.is_empty() {
        "none"
    } else {
        "high"
    };
    let status = if collision_risk == "none" && active_agents.len() <= 1 {
        "ownership: clear"
    } else if collision_risk == "none" {
        "ownership: shared_clear"
    } else {
        "ownership: collision"
    };
    let safe_next_action = if collision_risk == "none" {
        "continue with current Workpoint; no ownership collision detected"
    } else {
        "collision detected: coordinate owners, pause overlapping file edits, or handoff before mutation"
    };
    json!({
        "ownership_board": true,
        "status": status,
        "active_agents": active_agents,
        "collision_risk": collision_risk,
        "collision_files": collision_files,
        "safe_next_action": safe_next_action,
    })
}

fn reflex_primitive_items(req: &TraverseRequest, sel: &str) -> Vec<Value> {
    let registry: Value = serde_json::from_str(include_str!(
        "../../../../docs/current/focusa-reflex-primitives.json"
    ))
    .unwrap_or_else(|_| json!({"primitives": []}));
    let family = req
        .anchor
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let risk_or_object_query = req
        .query
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let primitives = registry
        .get("primitives")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    primitives
        .into_iter()
        .filter(|primitive| match sel {
            "family" | "children" => {
                family.is_empty()
                    || primitive
                        .get("family")
                        .and_then(Value::as_str)
                        .map(|value| value.eq_ignore_ascii_case(&family))
                        .unwrap_or(false)
            }
            _ => true,
        })
        .filter(|primitive| {
            risk_or_object_query.is_empty()
                || serde_json::to_string(primitive)
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&risk_or_object_query)
        })
        .map(|mut primitive| {
            if let Some(obj) = primitive.as_object_mut() {
                obj.insert(
                    "source".to_string(),
                    json!("spec97_reflex_primitive_registry"),
                );
                obj.insert("advisory_only".to_string(), json!(true));
            }
            primitive
        })
        .collect()
}

fn generic_filter_items(mut items: Vec<Value>, req: &TraverseRequest, sel: &str) -> Vec<Value> {
    let anchor = req.anchor.as_deref().unwrap_or_default();
    let query = req
        .query
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match sel {
        "current" | "by_id" => {
            if anchor.is_empty() {
                items.into_iter().take(1).collect()
            } else {
                items
                    .into_iter()
                    .filter(|item| {
                        value_id(item) == anchor
                            || serde_json::to_string(item)
                                .unwrap_or_default()
                                .contains(anchor)
                    })
                    .collect()
            }
        }
        "search" | "scenario" | "profile" | "confidence_change" | "window" | "workpoint"
        | "bead" | "spec" | "file" | "test" => {
            if query.is_empty() {
                items
            } else {
                items
                    .into_iter()
                    .filter(|item| {
                        serde_json::to_string(item)
                            .unwrap_or_default()
                            .to_ascii_lowercase()
                            .contains(&query)
                    })
                    .collect()
            }
        }
        "recent" => {
            items.reverse();
            items
        }
        _ => items,
    }
}

fn lineage_items(state: &FocusaState, req: &TraverseRequest, sel: &str) -> Vec<Value> {
    let anchor = req.anchor.as_deref();
    let head = state.clt.head_id.as_deref();
    let radius = req.radius.unwrap_or(1).clamp(1, 8);
    let nodes = match sel {
        "head" => state
            .clt
            .nodes
            .iter()
            .rev()
            .filter(|node| Some(node.node_id.as_str()) == head)
            .cloned()
            .collect::<Vec<_>>(),
        "children" => state
            .clt
            .nodes
            .iter()
            .filter(|node| node.parent_id.as_deref() == anchor.or(head))
            .cloned()
            .collect::<Vec<_>>(),
        "summaries" => state
            .clt
            .nodes
            .iter()
            .filter(|node| node.node_type == CltNodeType::Summary)
            .cloned()
            .collect::<Vec<_>>(),
        "path" => focusa_core::clt::lineage_path(&state.clt)
            .into_iter()
            .take(req.depth.unwrap_or(64).clamp(1, 64))
            .cloned()
            .collect::<Vec<_>>(),
        "neighborhood" => {
            let target = anchor.or(head).unwrap_or_default();
            let mut out = Vec::new();
            for node in &state.clt.nodes {
                if node.node_id == target || node.parent_id.as_deref() == Some(target) {
                    out.push(node.clone());
                }
            }
            out.into_iter()
                .take(radius.saturating_mul(8))
                .collect::<Vec<_>>()
        }
        _ => state.clt.nodes.clone(),
    };
    nodes
        .iter()
        .filter_map(|node| {
            let mut value = serde_json::to_value(node).ok()?;
            let (summary, content_ref) = lineage_node_summary_and_ref(&value);
            if let Some(obj) = value.as_object_mut() {
                if !summary.is_empty() {
                    obj.insert("summary".to_string(), json!(summary));
                }
                if let Some(content_ref) = content_ref {
                    obj.insert("content_ref".to_string(), json!(content_ref));
                }
            }
            Some(value)
        })
        .collect()
}

fn lineage_node_summary_and_ref(value: &Value) -> (String, Option<String>) {
    let payload = value.get("payload").unwrap_or(&Value::Null);
    let node_type = value
        .get("node_type")
        .and_then(Value::as_str)
        .unwrap_or("node");
    if let Some(content_ref) = payload.get("content_ref").and_then(Value::as_str) {
        return (content_ref.to_string(), Some(content_ref.to_string()));
    }
    if let Some(summary) = payload.get("summary").and_then(Value::as_str) {
        return (summary.to_string(), None);
    }
    if let Some(reason) = payload.get("reason").and_then(Value::as_str) {
        return (reason.to_string(), None);
    }
    let created = value
        .get("created_at")
        .and_then(Value::as_str)
        .unwrap_or("unknown_time");
    (format!("{node_type} at {created}"), None)
}

fn surface_items(
    state: &FocusaState,
    req: &TraverseRequest,
    surface: &str,
    sel: &str,
) -> Vec<Value> {
    let items = match surface {
        "trajectory" => trajectory_items(state),
        "lineage" | "tree" | "clt" => lineage_items(state, req, sel),
        "ontology" => match sel {
            "links" | "adjacency" | "neighborhood" => state.ontology.links.clone(),
            "proposals" => state
                .ontology
                .proposals
                .iter()
                .filter_map(|item| serde_json::to_value(item).ok())
                .collect(),
            _ => state.ontology.objects.clone(),
        },
        "focus_stack" | "frames" if sel == "current" => {
            active_frame_value(state).into_iter().collect()
        }
        "focus_stack" | "frames" => state
            .focus_stack
            .frames
            .iter()
            .filter_map(|frame| serde_json::to_value(frame).ok())
            .collect(),
        "workpoints" | "workpoint" if sel == "current" => {
            active_workpoint_value(state).into_iter().collect()
        }
        "workpoints" | "workpoint" => state
            .workpoint
            .records
            .iter()
            .filter_map(|record| serde_json::to_value(record).ok())
            .collect(),
        "ownership" | "ownership_board" | "agents" => ownership_board_items(state, req),
        "evidence" | "ecs" | "references" => {
            let mut evidence_items = state
                .reference_index
                .handles
                .iter()
                .filter_map(|handle| {
                    let mut value = serde_json::to_value(handle).ok()?;
                    if let Some(obj) = value.as_object_mut() {
                        obj.insert("source_index".to_string(), json!("reference_index.handles"));
                    }
                    Some(value)
                })
                .collect::<Vec<_>>();
            for record in &state.workpoint.records {
                for verification in &record.verification_records {
                    let evidence_id = verification.evidence_ref.clone().unwrap_or_else(|| {
                        format!(
                            "workpoint:{}:{}",
                            record.workpoint_id, verification.target_ref
                        )
                    });
                    let proof_kind = if verification.result.to_ascii_lowercase().contains("pass")
                        || verification.target_ref.contains("test")
                    {
                        "test"
                    } else if verification.target_ref.contains("http")
                        || verification.target_ref.contains("/v1/")
                    {
                        "api"
                    } else if verification.target_ref.contains(".md") {
                        "report"
                    } else {
                        "file"
                    };
                    evidence_items.push(json!({
                        "id": evidence_id,
                        "kind": "workpoint_verification",
                        "proof_kind": proof_kind,
                        "label": verification.evidence_ref.clone().unwrap_or_else(|| verification.target_ref.clone()),
                        "summary": verification.result,
                        "target_ref": verification.target_ref,
                        "result": verification.result,
                        "evidence_ref": verification.evidence_ref,
                        "workpoint_id": record.workpoint_id,
                        "work_item_id": record.work_item_id,
                        "bead_id": record.work_item_id,
                        "project_root": record.project_root,
                        "continuity_id": record.continuity_id,
                        "verified_at": verification.verified_at,
                        "source_index": "workpoint.verification_records",
                        "confidence_delta": if verification.result.trim().is_empty() { "none" } else { "proof_linked" },
                        "confidence_change": true,
                        "stale_refs": [],
                        "duplicate_cluster": Value::Null,
                        "rehydrate_ref": format!("workpoint:{}#evidence:{}", record.workpoint_id, evidence_id),
                    }));
                }
            }
            if sel == "confidence_change" {
                evidence_items.retain(|item| {
                    item.get("confidence_change").and_then(Value::as_bool) == Some(true)
                });
            }
            if matches!(
                sel,
                "workpoint" | "bead" | "spec" | "file" | "test" | "confidence_change"
            ) && let Some(query) = req
                .query
                .as_deref()
                .map(str::trim)
                .filter(|query| !query.is_empty())
            {
                let query = query.to_ascii_lowercase();
                evidence_items.retain(|item| {
                    serde_json::to_string(item)
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&query)
                });
            }
            evidence_items
        }
        "telemetry" | "turns" | "commands" => state.telemetry.trace_events.clone(),
        "metacognition" | "metacog" => metacognition_items(state),
        "predictions" | "prediction" => prediction_items(state),
        "snapshots" | "snapshot" => snapshot_items(state),
        "profile_selector" | "bloatgaurd_profiles" => profile_selector_catalog(),
        "routine_commands" | "bloatgaurd_routines" => routine_commands_catalog(),
        "spec_availability" | "spec_registry" | "specs" => spec_availability_registry(),
        "verbosity_profile" | "verbosity_profiles" | "profiles" => verbosity_profile_catalog(),
        "change_feed" | "changes" => change_feed_items(state, req),
        "command_palette" | "palette" => command_palette_catalog(),
        "recovery_playbooks" | "recovery_playbook" | "playbooks" => recovery_playbook_catalog(),
        "reflex" | "reflexes" | "reflex_primitives" => reflex_primitive_items(req, sel),
        "tool_registry" | "capabilities" => vec![json!({
            "id": "tool_registry_summary",
            "surface": "tool_registry",
            "summary": "Use /v1/ontology/tool-contracts or focusa_tool_doctor for the full bounded registry.",
            "next_tool": "focusa_tool_doctor"
        })],
        _ => Vec::new(),
    };
    generic_filter_items(items, req, sel)
}

fn surface_defaults(surface: &str) -> (&'static [&'static str], &'static [&'static str]) {
    match surface {
        "trajectory" => (
            &["id", "project_identity", "trajectory", "advisory_only"],
            &[
                "id",
                "project_identity",
                "trajectory",
                "advisory_only",
                "context_sufficiency",
            ],
        ),
        "lineage" | "tree" | "clt" => (
            &["node_id", "parent_id", "node_type", "summary", "created_at"],
            &[
                "node_id",
                "parent_id",
                "node_type",
                "payload",
                "summary",
                "created_at",
                "metadata",
            ],
        ),
        "focus_stack" | "frames" => (
            &["id", "title", "goal", "status", "continuity_id"],
            &[
                "id",
                "title",
                "goal",
                "status",
                "continuity_id",
                "project_root",
                "tags",
                "created_at",
            ],
        ),
        "telemetry" | "turns" | "commands" => (
            &[
                "event_id",
                "event_type",
                "timestamp",
                "session_id",
                "agent_id",
                "model_id",
                "schema_version",
            ],
            &[
                "event_id",
                "event_type",
                "timestamp",
                "session_id",
                "agent_id",
                "model_id",
                "clt_id",
                "focus_frame_id",
                "payload",
                "schema_version",
            ],
        ),
        "profile_selector" | "bloatgaurd_profiles" => (
            &[
                "profile_id",
                "label",
                "BLOATGAURD_PROFILE",
                "CONTEXT_POSTURE",
                "FULL_PAYLOAD",
                "availability",
                "authority",
                "mutates",
            ],
            &[
                "id",
                "profile_id",
                "label",
                "BLOATGAURD_PROFILE",
                "CONTEXT_POSTURE",
                "FULL_PAYLOAD",
                "availability",
                "authority",
                "mutates",
                "trust_badges",
            ],
        ),
        "routine_commands" | "bloatgaurd_routines" => (
            &[
                "routine_id",
                "label",
                "purpose",
                "command",
                "availability",
                "requires_verified_scope",
                "mutates",
            ],
            &[
                "id",
                "routine_id",
                "label",
                "purpose",
                "command",
                "availability",
                "requires_verified_scope",
                "mutates",
                "trust_badges",
            ],
        ),
        "spec_availability" | "spec_registry" | "specs" => (
            &[
                "spec_id",
                "feature",
                "availability",
                "runtime_entrypoint",
                "docs_ref",
                "trust_badges",
            ],
            &[
                "id",
                "spec_id",
                "feature",
                "availability",
                "runtime_entrypoint",
                "docs_ref",
                "first_implementation_slice",
                "trust_badges",
                "SpecRuntimeAvailabilityLabel",
            ],
        ),
        "verbosity_profile" | "verbosity_profiles" | "profiles" => (
            &[
                "profile",
                "compact_fields",
                "detail_fields",
                "hidden_by_default",
                "escalation_fields",
            ],
            &[
                "id",
                "profile",
                "compact_fields",
                "detail_fields",
                "hidden_by_default",
                "escalation_fields",
            ],
        ),
        "change_feed" | "changes" => (
            &[
                "workpoint_id",
                "work_item_id",
                "session_id",
                "active_object_refs",
                "verification_records",
            ],
            &[
                "workpoint_id",
                "work_item_id",
                "session_id",
                "active_object_refs",
                "verification_records",
                "updated_at",
                "mission",
                "next_slice",
            ],
        ),
        "command_palette" | "palette" => (
            &["label", "tool", "args_preview", "when"],
            &["id", "label", "tool", "args_preview", "when"],
        ),
        "recovery_playbooks" | "recovery_playbook" | "playbooks" => (
            &[
                "scenario",
                "first_safe_tool",
                "next_tools",
                "proof_to_capture",
                "stop_conditions",
            ],
            &[
                "id",
                "scenario",
                "symptoms",
                "first_safe_tool",
                "next_tools",
                "proof_to_capture",
                "stop_conditions",
            ],
        ),
        "reflex" | "reflexes" | "reflex_primitives" => (
            &[
                "primitive_id",
                "family",
                "trigger",
                "reflex_action",
                "advisory_only",
            ],
            &[
                "primitive_id",
                "family",
                "trigger",
                "context_inputs",
                "reflex_action",
                "evidence_output",
                "escalation_boundary",
                "authority_boundary",
                "hot_path_budget",
                "failure_envelope",
                "implementation_status",
                "source",
                "advisory_only",
            ],
        ),
        "workpoints" | "workpoint" => (
            &[
                "workpoint_id",
                "work_item_id",
                "status",
                "mission",
                "next_slice",
                "updated_at",
            ],
            &[
                "workpoint_id",
                "work_item_id",
                "status",
                "mission",
                "next_slice",
                "canonical",
                "project_root",
                "continuity_id",
                "updated_at",
            ],
        ),
        "ownership" | "ownership_board" | "agents" => (
            &[
                "agent_id",
                "owns",
                "touched_files",
                "last_activity",
                "lease_status",
                "workpoint_id",
                "bead_id",
            ],
            &[
                "id",
                "kind",
                "agent_id",
                "owns",
                "touched_files",
                "last_activity",
                "lease_status",
                "workpoint_id",
                "bead_id",
                "project_root",
                "continuity_id",
                "safe_next_action",
            ],
        ),
        "evidence" | "ecs" | "references" => (
            &[
                "id",
                "kind",
                "label",
                "summary",
                "trajectory",
                "evidence_ref",
                "target_ref",
                "workpoint_id",
                "work_item_id",
                "bead_id",
                "project_root",
                "source_index",
                "confidence_delta",
                "rehydrate_ref",
            ],
            &[
                "id",
                "kind",
                "proof_kind",
                "label",
                "summary",
                "trajectory",
                "created_at",
                "pinned",
                "session_id",
                "size",
                "sha256",
                "target_ref",
                "result",
                "evidence_ref",
                "workpoint_id",
                "work_item_id",
                "bead_id",
                "project_root",
                "continuity_id",
                "verified_at",
                "source_index",
                "confidence_delta",
                "confidence_change",
                "stale_refs",
                "duplicate_cluster",
                "rehydrate_ref",
            ],
        ),
        _ => (
            &["id", "label", "summary", "status"],
            &[
                "id",
                "label",
                "summary",
                "status",
                "payload",
                "created_at",
                "updated_at",
            ],
        ),
    }
}

fn traverse_response(state: &FocusaState, req: TraverseRequest, verify_only: bool) -> Value {
    let surface = normalize_surface(&req.surface);
    let sel = if verify_only {
        "tags_verify".to_string()
    } else {
        selector(&req)
    };
    let supported = matches!(
        surface.as_str(),
        "trajectory"
            | "lineage"
            | "tree"
            | "clt"
            | "ontology"
            | "focus_stack"
            | "frames"
            | "workpoints"
            | "workpoint"
            | "ownership"
            | "ownership_board"
            | "agents"
            | "evidence"
            | "ecs"
            | "references"
            | "metacognition"
            | "metacog"
            | "predictions"
            | "prediction"
            | "telemetry"
            | "turns"
            | "commands"
            | "snapshots"
            | "snapshot"
            | "profile_selector"
            | "bloatgaurd_profiles"
            | "routine_commands"
            | "bloatgaurd_routines"
            | "spec_availability"
            | "spec_registry"
            | "specs"
            | "verbosity_profile"
            | "verbosity_profiles"
            | "profiles"
            | "change_feed"
            | "changes"
            | "command_palette"
            | "palette"
            | "recovery_playbooks"
            | "recovery_playbook"
            | "playbooks"
            | "reflex"
            | "reflexes"
            | "reflex_primitives"
            | "tool_registry"
            | "capabilities"
    );
    if !supported {
        let reflex_suggestions =
            crate::routes::reflex::reflex_suggestions_for_failure("validation_rejected");
        return json!({
            "status": "validation_rejected",
            "canonical": false,
            "degraded": true,
            "trust_badges": trust_badges(false, true, true, false, false, false),
            "route_recommendation": route_recommendation_payload(&surface, &sel, true),
            "failure_class": "validation_rejected",
            "items": [],
            "summary": "unsupported traversal surface or selector",
            "do_not_use": ["unsupported_surface"],
            "traversal": {
                "surface": surface,
                "selector": sel,
                "returned": 0,
                "total": 0,
                "truncated": false,
                "empty_state": empty_state_payload(&surface, &sel, req.query.as_deref(), 0, false, false),
                "caps": {"limit": 0, "depth": 0, "radius": 0, "payload_bytes": 0, "budget_tokens": req.budget_tokens},
                "omitted": ["unsupported_surface"],
                "rehydrate_refs": [],
                "stale_tags": [],
                "verified_tags": []
            },
            "empty_state": empty_state_payload(&surface, &sel, req.query.as_deref(), 0, false, false),
            "tag_scheme": {
                "version": "focusa-traverse-tag-v1",
                "algorithm": "opaque_version",
                "length": 24,
                "includes_anchor": true,
                "includes_surface_version": true,
                "collision_policy": "retry_with_longer_tag"
            },
            "next_tools": ["focusa_tool_doctor"],
            "reflex_suggestions": reflex_suggestions,
            "details": {"tool_result_v1": {"ok": false, "status": "validation_rejected", "failure_class": "validation_rejected", "canonical": false, "degraded": true, "reflex_suggestions": reflex_suggestions}}
        });
    }

    let raw_items = surface_items(state, &req, &surface, &selector(&req));
    let (default_fields, allowed_fields) = surface_defaults(&surface);
    let (items, metadata, field_projection, full_payload_blocked) =
        bounded_json_items(raw_items, &req, &surface, default_fields, allowed_fields);
    let returned = items.len();
    let total = metadata
        .get("total")
        .and_then(Value::as_u64)
        .unwrap_or(returned as u64);
    let truncated = metadata
        .get("truncated")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let limit_value = metadata.get("limit").cloned().unwrap_or(Value::Null);
    let next_cursor_value = metadata.get("next_cursor").cloned().unwrap_or(Value::Null);
    let more_available = truncated && !next_cursor_value.is_null();
    let pagination_guidance = if more_available {
        format!(
            "More {surface}/{sel} items are available. Re-run focusa_traverse with cursor={} and the same surface/selector/query/limit to fetch the next bounded page.",
            next_cursor_value.as_str().unwrap_or("<next_cursor>")
        )
    } else if truncated {
        "Traversal was truncated but no next_cursor is available; narrow selector/query or lower payload scope before retrying.".to_string()
    } else {
        "No additional page is available for this bounded traversal window.".to_string()
    };
    let omitted_count = metadata.get("omitted").and_then(Value::as_u64).unwrap_or(0);
    let omitted = if omitted_count > 0 {
        vec![format!("items_omitted:{omitted_count}")]
    } else {
        Vec::<String>::new()
    };
    let rehydrate_refs = if req.include_rehydrate_refs || truncated || full_payload_blocked {
        vec![format!(
            "focusa://traverse/{}/{}?cursor={}",
            surface,
            sel,
            metadata
                .get("next_cursor")
                .and_then(Value::as_str)
                .unwrap_or("0")
        )]
    } else {
        Vec::<String>::new()
    };
    let (source_index, scope_key, freshness, count_semantics, why_zero_if_empty, next_selector) =
        if surface == "ontology" {
            (
                match sel.as_str() {
                    "links" | "adjacency" | "neighborhood" => "runtime_ontology_links",
                    "proposals" => "runtime_ontology_proposals",
                    _ => "runtime_ontology_objects",
                },
                req.session_identity
                    .as_ref()
                    .and_then(|value| value.pointer("/project_identity/project_root"))
                    .and_then(Value::as_str)
                    .unwrap_or("global_runtime_ontology"),
                "live_state_snapshot",
                match sel.as_str() {
                    "links" | "adjacency" | "neighborhood" => {
                        "total is runtime ontology links for the requested selector; compare project-card counts.runtime_links for parity"
                    }
                    "proposals" => "total is runtime ontology proposals for selector=proposals",
                    _ => {
                        "total is runtime ontology objects only; compare project-card counts.runtime_objects, not effective_project_card_objects"
                    }
                },
                match sel.as_str() {
                    "links" | "adjacency" | "neighborhood" => {
                        "zero means no runtime ontology links currently match this selector"
                    }
                    "proposals" => {
                        "zero means no runtime ontology proposals currently match this selector"
                    }
                    _ => {
                        "zero means no runtime ontology objects currently match this selector; project-card derived objects may still exist outside this runtime index"
                    }
                },
                match sel.as_str() {
                    "links" | "adjacency" | "neighborhood" => {
                        "focusa_traverse surface=ontology selector=window for objects, or focusa_project_card for effective derived counts"
                    }
                    "proposals" => {
                        "focusa_traverse surface=ontology selector=window for objects, or selector=links for links"
                    }
                    _ => {
                        "focusa_project_card for effective derived counts; focusa_traverse surface=workpoints selector=window or focusa_trajectory_view for derived context"
                    }
                },
            )
        } else {
            (
                surface.as_str(),
                "surface_default",
                "live_state_snapshot",
                "total is the bounded traversal source count for this surface and selector",
                "zero means no items matched this surface/selector/query in the current bounded source",
                "adjust selector/query/anchor or use the listed next_tools",
            )
        };
    let index_health = if matches!(surface.as_str(), "evidence" | "ecs" | "references") {
        json!({
            "status": "healthy",
            "index_lag": false,
            "source_index": "reference_index.handles_plus_workpoint.verification_records",
            "freshness": "live_state_snapshot",
            "count_semantics": "search matches reference handles and linked Workpoint verification records by target_ref, result text, evidence_ref, id, and label",
            "why_zero_if_empty": "zero means no current evidence handle or linked Workpoint verification matched the selector/query",
            "exact_handle_alternatives": ["query by exact evidence_ref", "query by exact target_ref", "focusa_workpoint_resume for active Workpoint verification records"],
        })
    } else {
        Value::Null
    };
    let artifact_browser = if matches!(surface.as_str(), "evidence" | "ecs" | "references") {
        let group_by = artifact_group_by(&sel);
        json!({
            "group_by": group_by,
            "filters": {"selector": sel, "query": req.query, "workpoint": req.anchor},
            "stale_refs": [],
            "duplicate_clusters": [],
            "artifacts": items.iter().take(20).map(|item| json!({
                "evidence_ref": item.get("evidence_ref").or_else(|| item.get("id")).cloned().unwrap_or(Value::Null),
                "target_ref": item.get("target_ref").cloned().unwrap_or(Value::Null),
                "workpoint_id": item.get("workpoint_id").cloned().unwrap_or(Value::Null),
                "bead_id": item.get("bead_id").or_else(|| item.get("work_item_id")).cloned().unwrap_or(Value::Null),
                "project_root": item.get("project_root").cloned().unwrap_or(Value::Null),
                "kind": item.get("proof_kind").or_else(|| item.get("kind")).cloned().unwrap_or(Value::Null),
                "confidence_delta": item.get("confidence_delta").cloned().unwrap_or(json!("none")),
                "freshness": item.get("verified_at").or_else(|| item.get("created_at")).cloned().unwrap_or(json!("live_state_snapshot")),
                "rehydrate_ref": item.get("rehydrate_ref").cloned().unwrap_or_else(|| item.get("id").and_then(Value::as_str).map(|id| json!(format!("evidence:{id}"))).unwrap_or(Value::Null)),
                "group_key": artifact_group_key(item, &group_by),
            })).collect::<Vec<_>>()
        })
    } else {
        Value::Null
    };
    let ownership_board = if matches!(surface.as_str(), "ownership" | "ownership_board" | "agents")
    {
        ownership_board_payload(&items)
    } else {
        Value::Null
    };
    let traversal_meta = json!({
        "surface": surface,
        "selector": sel,
        "source_index": source_index,
        "scope_key": scope_key,
        "freshness": freshness,
        "count_semantics": count_semantics,
        "why_zero_if_empty": why_zero_if_empty,
        "next_selector": next_selector,
        "index_health": index_health,
        "artifact_browser": artifact_browser,
        "ownership_board": ownership_board,
        "route_recommendation": route_recommendation_payload(&surface, &sel, false),
        "stuck_loop": stuck_loop_payload(&surface, &items),
        "evidence_diff": evidence_diff_payload(&surface, &items),
        "profile_selector": if matches!(surface.as_str(), "profile_selector" | "bloatgaurd_profiles") { profile_selector_payload(&items) } else { Value::Null },
        "routine_commands": if matches!(surface.as_str(), "routine_commands" | "bloatgaurd_routines") { routine_commands_payload(&items) } else { Value::Null },
        "spec_availability": if matches!(surface.as_str(), "spec_availability" | "spec_registry" | "specs") { spec_availability_payload(&items) } else { Value::Null },
        "verbosity_profile": if matches!(surface.as_str(), "verbosity_profile" | "verbosity_profiles" | "profiles") { verbosity_profile_payload(&items) } else { Value::Null },
        "change_feed": if matches!(surface.as_str(), "change_feed" | "changes") { change_feed_payload(&items, req.query.as_deref()) } else { Value::Null },
        "command_palette": if matches!(surface.as_str(), "command_palette" | "palette") { command_palette_payload(&items, &sel) } else { Value::Null },
        "recovery_playbook": if matches!(surface.as_str(), "recovery_playbooks" | "recovery_playbook" | "playbooks") { recovery_playbook_payload(&items) } else { Value::Null },
        "empty_state": empty_state_payload(&surface, &sel, req.query.as_deref(), returned, true, full_payload_blocked),
        "anchor": req.anchor,
        "query": req.query,
        "cursor": metadata.get("cursor").cloned().unwrap_or(Value::Null),
        "next_cursor": next_cursor_value.clone(),
        "more_available": more_available,
        "pagination_guidance": pagination_guidance.clone(),
        "returned": returned,
        "total": total,
        "total_known": total,
        "truncated": truncated,
        "limit": limit_value,
        "caps": {
            "limit": metadata.get("limit").and_then(Value::as_u64).unwrap_or(0),
            "depth": req.depth.unwrap_or(1).clamp(1, 64),
            "radius": req.radius.unwrap_or(1).clamp(1, 8),
            "payload_bytes": metadata.get("payload_bytes").cloned().unwrap_or(Value::Null),
            "budget_tokens": req.budget_tokens,
        },
        "depth": req.depth.unwrap_or(1).clamp(1, 64),
        "radius": req.radius.unwrap_or(1).clamp(1, 8),
        "fields": field_projection,
        "metadata": metadata,
        "omitted": omitted,
        "rehydrate_refs": rehydrate_refs,
    });
    let mut tags = item_tags(&surface, &sel, &items);
    tags.extend(aggregate_tags(&surface, &sel, &items, &traversal_meta));
    let (verified_tags, stale_tags) = verify_requested_tags(&req, &items, &traversal_meta);
    let mut traversal_meta = traversal_meta;
    if let Some(obj) = traversal_meta.as_object_mut() {
        obj.insert(
            "verified_tags".to_string(),
            Value::Array(verified_tags.clone()),
        );
        obj.insert("stale_tags".to_string(), Value::Array(stale_tags.clone()));
    }
    let response_items = if verify_only {
        Vec::<Value>::new()
    } else {
        traversed_items(&surface, &sel, &items)
    };
    let degraded = full_payload_blocked || !stale_tags.is_empty();
    let failure_class = if full_payload_blocked {
        json!("resource_exhausted")
    } else if !stale_tags.is_empty() {
        json!("read_model_lag")
    } else {
        Value::Null
    };
    json!({
        "status": if degraded { "degraded" } else { "completed" },
        "canonical": !degraded,
        "degraded": degraded,
        "trust_badges": trust_badges(!degraded, degraded, false, false, false, false),
        "route_recommendation": route_recommendation_payload(&surface, &sel, degraded),
        "empty_state": empty_state_payload(&surface, &sel, req.query.as_deref(), returned, true, full_payload_blocked),
        "recovery_playbook": if matches!(surface.as_str(), "recovery_playbooks" | "recovery_playbook" | "playbooks") { recovery_playbook_payload(&items) } else { Value::Null },
        "failure_class": failure_class,
        "surface": surface,
        "selector": sel,
        "anchor": req.anchor,
        "project_identity": req.session_identity.as_ref().and_then(|value| value.get("project_identity")).cloned().unwrap_or(Value::Null),
        "items": response_items,
        "summary": format!("traverse surface={} selector={} returned={} truncated={} more_available={}", surface, sel, returned, truncated, more_available),
        "more_available": more_available,
        "pagination_guidance": pagination_guidance,
        "do_not_use": if full_payload_blocked { vec!["full_payload_without_budget"] } else { Vec::<&str>::new() },
        "verified_tags": verified_tags,
        "stale_tags": stale_tags,
        "traversal": traversal_meta,
        "tag_scheme": {
            "version": "focusa-traverse-tag-v1",
            "algorithm": "opaque_version",
            "length": 24,
            "includes_anchor": true,
            "includes_surface_version": true,
            "collision_policy": "retry_with_longer_tag",
            "modes": ["item", "range", "window", "surface"],
            "requested_tag_mode": req.tag_mode.as_deref().unwrap_or("mixed"),
            "item_tag_format": "focusa://{surface}/{selector}/item/{anchor}/{sha256_24}",
            "range_tag_format": "focusa://{surface}/{selector}/range/{start-end}/{sha256_24}",
            "window_tag_format": "focusa://{surface}/{selector}/window/{cursor-limit}/{sha256_24}",
            "surface_tag_format": "focusa://{surface}/{selector}/surface/{surface-total}/{sha256_24}",
            "long_tag_policy": "stable 24-hex digest by default; future versions may use full 64-hex digest",
            "tags_verify_endpoint": "/v1/traverse/verify-tags"
        },
        "tags": tags,
        "next_tools": ["focusa_traverse", "focusa_trajectory_view", "focusa_workpoint_resume"],
        "reflex_suggestions": if full_payload_blocked { crate::routes::reflex::reflex_suggestions_for_failure("resource_exhausted") } else { Vec::new() },
        "details": {"tool_result_v1": {"ok": !degraded, "status": if degraded { "degraded" } else { "completed" }, "failure_class": failure_class, "canonical": !degraded, "degraded": degraded, "reflex_suggestions": if full_payload_blocked { crate::routes::reflex::reflex_suggestions_for_failure("resource_exhausted") } else { Vec::new() }}}
    })
}

fn scoped_traverse_state(state: &FocusaState, scope: &ScopeContext) -> FocusaState {
    let mut scoped = state.clone();
    scoped.clt = crate::routes::clt::scoped_clt_state(&state.clt, scope);
    let project_root = scope
        .project_root
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let continuity_id = scope
        .continuity_id
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    scoped.focus_stack.frames.retain(|frame| {
        !project_root.is_empty()
            && !continuity_id.is_empty()
            && frame.project_root.as_deref().map(str::trim) == Some(project_root)
            && frame.continuity_id.as_deref().map(str::trim) == Some(continuity_id)
    });
    scoped.focus_stack.active_id = scoped
        .focus_stack
        .frames
        .iter()
        .rev()
        .find(|frame| frame.status == FrameStatus::Active)
        .map(|frame| frame.id);
    scoped.focus_stack.root_id = scoped.focus_stack.frames.iter().find_map(|frame| {
        frame
            .parent_id
            .is_none_or(|parent_id| {
                !scoped
                    .focus_stack
                    .frames
                    .iter()
                    .any(|item| item.id == parent_id)
            })
            .then_some(frame.id)
    });
    scoped.focus_stack.stack_path_cache.retain(|id| {
        scoped
            .focus_stack
            .frames
            .iter()
            .any(|frame| frame.id == *id)
    });
    scoped
}

async fn traverse(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(req): Json<TraverseRequest>,
) -> Json<Value> {
    let s = state.focusa.read().await;
    let scoped = scoped_traverse_state(&s, &scope);
    Json(traverse_response(&scoped, req, false))
}

async fn verify_tags(
    scope: ScopeContext,
    State(state): State<Arc<AppState>>,
    Json(mut req): Json<TraverseRequest>,
) -> Json<Value> {
    adopt_verify_selector_from_requested_tags(&mut req);
    let s = state.focusa.read().await;
    let scoped = scoped_traverse_state(&s, &scope);
    Json(traverse_response(&scoped, req, true))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/traverse", post(traverse))
        .route("/v1/traverse/verify-tags", post(verify_tags))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_trajectory() -> FocusaState {
        let mut state = FocusaState::new();
        state.trajectory.active_trajectory_id = Some("traj-test".to_string());
        state
            .trajectory
            .records
            .push(focusa_core::types::TrajectoryProjectionRecord {
                trajectory_id: "traj-test".to_string(),
                project_root: Some("/tmp/focusa-test".to_string()),
                continuity_id: Some("cont-test".to_string()),
                long_term_goal: "High-level target".to_string(),
                mid_level_goal: Some("Mid-level target".to_string()),
                short_term_goal: Some("Short-term target".to_string()),
                waypoints: ["Waypoint A", "Waypoint B"]
                    .into_iter()
                    .enumerate()
                    .map(
                        |(index, title)| focusa_core::types::TrajectoryWaypointRecord {
                            waypoint_id: format!("waypoint:{}", index + 1),
                            title: title.to_string(),
                            ..focusa_core::types::TrajectoryWaypointRecord::default()
                        },
                    )
                    .collect(),
                ..focusa_core::types::TrajectoryProjectionRecord::default()
            });
        state
    }

    #[test]
    fn telemetry_surface_preserves_event_identity_in_default_projection() {
        let mut state = FocusaState::new();
        state.telemetry.trace_events.push(json!({
            "event_id": "evt-low-frequency-1",
            "event_type": "tool_call",
            "timestamp": "2026-07-14T22:00:00Z",
            "session_id": "session-test",
            "agent_id": "pi-test",
            "model_id": "openai-codex/gpt-5.3-codex-spark",
            "schema_version": "focusa.telemetry_event.v1",
            "payload": {"tool": "focusa_traverse", "status": "completed"}
        }));

        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "telemetry".to_string(),
                selector: Some("summaries".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );

        assert_eq!(
            res.pointer("/items/0/anchor").and_then(Value::as_str),
            Some("evt-low-frequency-1")
        );
        assert_eq!(
            res.pointer("/items/0/kind").and_then(Value::as_str),
            Some("tool_call")
        );
        assert_eq!(
            res.pointer("/items/0/data/event_type")
                .and_then(Value::as_str),
            Some("tool_call")
        );
        assert_eq!(
            res.pointer("/items/0/summary").and_then(Value::as_str),
            Some("2026-07-14T22:00:00Z")
        );
    }

    #[test]
    fn invalid_requested_fields_fall_back_to_surface_defaults() {
        let state = FocusaState::new();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "tool_registry".to_string(),
                selector: Some("summaries".to_string()),
                fields: vec!["name".to_string(), "family".to_string()],
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );

        assert_eq!(
            res.pointer("/items/0/anchor").and_then(Value::as_str),
            Some("tool_registry_summary")
        );
        assert!(
            res.pointer("/items/0/summary")
                .and_then(Value::as_str)
                .is_some_and(|summary| summary.contains("focusa_tool_doctor"))
        );
        assert_eq!(
            res.pointer("/traversal/fields/fallback_to_defaults")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            res.pointer("/traversal/fields/omitted")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn trajectory_surface_projects_ladder_context() {
        let state = state_with_trajectory();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "trajectory".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let trajectory = res
            .pointer("/items/0/data/trajectory")
            .expect("trajectory projection");
        assert_eq!(
            trajectory.get("long_term_goal").and_then(Value::as_str),
            Some("High-level target")
        );
        assert_eq!(
            trajectory.get("mid_level_goal").and_then(Value::as_str),
            Some("Mid-level target")
        );
        assert_eq!(
            trajectory
                .pointer("/trajectory_ladder/trajectory_id")
                .and_then(Value::as_str),
            Some("traj-test")
        );
        assert_eq!(
            trajectory
                .get("waypoints")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
    }

    #[test]
    fn evidence_surface_default_projection_includes_trajectory_context() {
        let mut state = state_with_trajectory();
        let trajectory = state.trajectory_ladder_context();
        state
            .reference_index
            .handles
            .push(focusa_core::types::HandleRef {
                id: uuid::Uuid::now_v7(),
                kind: focusa_core::types::HandleKind::Text,
                label: "proof-handle".to_string(),
                size: 123,
                sha256: "deadbeef".to_string(),
                created_at: chrono::Utc::now(),
                session_id: None,
                project_root: None,
                continuity_id: None,
                pinned: false,
                trajectory,
            });
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "evidence".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let item = res.pointer("/items/0/data").expect("evidence item");
        assert_eq!(
            item.get("label").and_then(Value::as_str),
            Some("proof-handle")
        );
        assert_eq!(
            item.pointer("/trajectory/trajectory_id")
                .and_then(Value::as_str),
            Some("traj-test")
        );
        assert!(
            item.get("sha256").is_none(),
            "sha256 stays out of default projection"
        );
    }

    #[test]
    fn unsupported_surface_returns_blocked_tool_envelope() {
        let state = FocusaState::new();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "unknown".to_string(),
                ..TraverseRequest::default()
            },
            false,
        );
        assert_eq!(
            res.get("status").and_then(Value::as_str),
            Some("validation_rejected")
        );
        assert_eq!(
            res.pointer("/details/tool_result_v1/failure_class")
                .and_then(Value::as_str),
            Some("validation_rejected")
        );
    }

    #[test]
    fn tag_verify_preserves_item_tag_after_unrelated_change() {
        let mut state = FocusaState::new();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n1".to_string(),
            parent_id: None,
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "user".to_string(),
                content_ref: Some("hello".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        state.clt.head_id = Some("n1".to_string());
        let first = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let item_tag = first
            .get("tags")
            .and_then(Value::as_array)
            .and_then(|tags| {
                tags.iter()
                    .find(|tag| tag.get("tag_mode").and_then(Value::as_str) == Some("item"))
            })
            .and_then(|tag| tag.get("tag"))
            .and_then(Value::as_str)
            .expect("item tag")
            .to_string();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n2".to_string(),
            parent_id: Some("n1".to_string()),
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "assistant".to_string(),
                content_ref: Some("unrelated".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        let verified = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                tags: vec![json!(item_tag)],
                ..TraverseRequest::default()
            },
            true,
        );
        assert_eq!(
            verified
                .get("items")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            verified
                .get("verified_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            verified
                .get("stale_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn major_surface_adapters_return_bounded_items() {
        let state = FocusaState::new();
        for surface in [
            "trajectory",
            "lineage",
            "ontology",
            "focus_stack",
            "workpoints",
            "evidence",
            "metacognition",
            "predictions",
            "telemetry",
            "snapshots",
            "reflex_primitives",
            "tool_registry",
        ] {
            let res = traverse_response(
                &state,
                TraverseRequest {
                    surface: surface.to_string(),
                    selector: Some("window".to_string()),
                    limit: Some(5),
                    ..TraverseRequest::default()
                },
                false,
            );
            assert_eq!(
                res.get("status").and_then(Value::as_str),
                Some("completed"),
                "surface={surface}"
            );
            assert!(
                res.get("traversal").and_then(Value::as_object).is_some(),
                "surface={surface}"
            );
        }
    }

    #[test]
    fn reflex_primitive_surface_returns_registry_backed_family_items() {
        let state = FocusaState::new();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "reflex_primitives".to_string(),
                selector: Some("family".to_string()),
                anchor: Some("recovery".to_string()),
                fields: vec![
                    "primitive_id".to_string(),
                    "family".to_string(),
                    "reflex_action".to_string(),
                ],
                limit: Some(8),
                ..TraverseRequest::default()
            },
            false,
        );
        assert_eq!(res.get("status").and_then(Value::as_str), Some("completed"));
        let items = res.get("items").and_then(Value::as_array).unwrap();
        assert!(items.iter().any(|item| {
            item.get("data")
                .and_then(|payload| payload.get("primitive_id"))
                .and_then(Value::as_str)
                == Some("route_noncanonical_result")
        }));
        assert!(items.iter().all(|item| {
            item.get("data")
                .and_then(|payload| payload.get("family"))
                .and_then(Value::as_str)
                == Some("recovery")
        }));
    }

    #[test]
    fn tag_verify_endpoint_adopts_selector_from_requested_tag() {
        let mut state = FocusaState::new();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n1".to_string(),
            parent_id: None,
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "user".to_string(),
                content_ref: Some("hello".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        state.clt.head_id = Some("n1".to_string());
        let first = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        let item_tag = first
            .get("tags")
            .and_then(Value::as_array)
            .and_then(|tags| {
                tags.iter()
                    .find(|tag| tag.get("tag_mode").and_then(Value::as_str) == Some("item"))
            })
            .and_then(|tag| tag.get("tag"))
            .and_then(Value::as_str)
            .expect("item tag")
            .to_string();
        let mut req = TraverseRequest {
            surface: "lineage".to_string(),
            selector: Some("tags_verify".to_string()),
            limit: Some(1),
            tags: vec![json!({"tag": item_tag})],
            ..TraverseRequest::default()
        };
        adopt_verify_selector_from_requested_tags(&mut req);
        assert_eq!(selector(&req), "window");
        let verified = traverse_response(&state, req, true);
        assert_eq!(
            verified
                .get("verified_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            verified
                .get("stale_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn tag_verify_reports_stale_tags() {
        let state = FocusaState::new();
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                tags: vec![json!(
                    "focusa://lineage/window/item/missing/deadbeefdeadbeefdeadbeef"
                )],
                ..TraverseRequest::default()
            },
            true,
        );
        assert_eq!(
            res.get("verified_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            res.get("stale_tags")
                .and_then(Value::as_array)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn traverse_truncated_window_explains_pagination() {
        let mut state = FocusaState::new();
        for id in ["n1", "n2"] {
            state.clt.nodes.push(focusa_core::types::CltNode {
                node_id: id.to_string(),
                parent_id: None,
                node_type: CltNodeType::Interaction,
                created_at: chrono::Utc::now(),
                session_id: None,
                payload: focusa_core::types::CltPayload::Interaction {
                    role: "user".to_string(),
                    content_ref: Some(id.to_string()),
                },
                metadata: focusa_core::types::CltMetadata::default(),
            });
        }
        state.clt.head_id = Some("n2".to_string());
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        assert_eq!(
            res.pointer("/traversal/more_available")
                .and_then(Value::as_bool),
            Some(true)
        );
        assert!(
            res.pointer("/traversal/pagination_guidance")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("Re-run focusa_traverse with cursor=")
        );
    }

    #[test]
    fn lineage_window_response_has_traversal_metadata_and_tags() {
        let mut state = FocusaState::new();
        state.clt.nodes.push(focusa_core::types::CltNode {
            node_id: "n1".to_string(),
            parent_id: None,
            node_type: CltNodeType::Interaction,
            created_at: chrono::Utc::now(),
            session_id: None,
            payload: focusa_core::types::CltPayload::Interaction {
                role: "user".to_string(),
                content_ref: Some("hello".to_string()),
            },
            metadata: focusa_core::types::CltMetadata::default(),
        });
        state.clt.head_id = Some("n1".to_string());
        let res = traverse_response(
            &state,
            TraverseRequest {
                surface: "lineage".to_string(),
                selector: Some("window".to_string()),
                limit: Some(1),
                ..TraverseRequest::default()
            },
            false,
        );
        assert_eq!(res.get("status").and_then(Value::as_str), Some("completed"));
        assert_eq!(
            res.pointer("/traversal/returned").and_then(Value::as_u64),
            Some(1)
        );
        assert!(res.get("tags").and_then(Value::as_array).unwrap().len() >= 4);
        assert_eq!(
            res.pointer("/tag_scheme/version").and_then(Value::as_str),
            Some("focusa-traverse-tag-v1")
        );
    }
}
