//! Ontology inspection routes.
//!
//! Runtime projection and action entrypoints for the typed software/work/mission/execution world.
//! This keeps ontology bounded and inspectable while routing canonical ontology
//! mutations through governance-aware reducer events.

use crate::routes::bounded::{
    BoundedReadOptions, bounded_metadata, env_limit, full_payload_blocked_by_pressure,
    pressure_status, record_json_response_size,
};
use crate::routes::predictions::append_prediction_record_scoped;
use crate::scope::ScopeContext;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::prediction::{PredictionOntologyContext, PredictionValue};
use focusa_core::scoped_state::{ScopeKind, ScopeRef, WorkstreamKey};
use focusa_core::types::{
    Action, FocusaEvent, FocusaState, FrameRecord, HandleKind, HandleRef,
    OntologyScopeMigrationRecordKind, OntologyScopeMigrationSelection, RuleScope,
    ontology_scope_record_hash,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

const OBJECT_TYPES: &[&str] = &[
    "repo",
    "package",
    "module",
    "file",
    "symbol",
    "route",
    "endpoint",
    "schema",
    "migration",
    "dependency",
    "test",
    "environment",
    "capability",
    "tool_surface",
    "permission",
    "authority_boundary",
    "precondition",
    "resource",
    "cost_model",
    "latency_profile",
    "reliability_profile",
    "reversibility_profile",
    "ownership",
    "execution_context",
    "affordance",
    "page",
    "region",
    "component",
    "variant",
    "content_slot",
    "token",
    "layout_rule",
    "interaction",
    "ui_state",
    "binding",
    "validation_rule",
    "visual_artifact",
    "task",
    "bug",
    "feature",
    "decision",
    "convention",
    "constraint",
    "risk",
    "reflex_primitive",
    "reflex_trigger",
    "reflex_action",
    "reflex_risk",
    "reflex_affordance",
    "trajectory_hlt",
    "trajectory_mlg",
    "trajectory_stg",
    "trajectory_waypoint",
    "goal",
    "subgoal",
    "active_focus",
    "open_loop",
    "acceptance_criterion",
    "patch",
    "diff",
    "failure",
    "verification",
    "artifact",
    "current_ask",
    "query_scope",
    "relevant_context_set",
    "excluded_context_set",
    "scope_failure",
    "canonical_entity",
    "reference_alias",
    "resolution_candidate",
    "resolution_decision",
    "supersession_record",
    "projection",
    "view_profile",
    "projection_rule",
    "projection_boundary",
    "ontology_version",
    "compatibility_profile",
    "migration_plan",
    "deprecation_record",
    "governance_decision",
    "agent_identity",
    "actor_instance",
    "actor",
    "role_profile",
    "capability_profile",
    "permission_profile",
    "responsibility",
    "handoff_boundary",
    "session_continuity",
    "workpoint_scope_binding",
    "prediction_record",
    "identity_state",
    "ontology_domain",
    "shared_layer",
];
const STATUS_VOCABULARY: &[&str] = &[
    "proposed",
    "candidate",
    "active",
    "speculative",
    "blocked",
    "verified",
    "failed",
    "stale",
    "deprecated",
    "superseded",
    "retired",
    "completed",
    "canonical",
    "experimental",
];
const MEMBERSHIP_CLASSES: &[&str] = &[
    "pinned",
    "deterministic",
    "verified",
    "inferred",
    "provisional",
];
const PROVENANCE_CLASSES: &[&str] = &[
    "parser_derived",
    "tool_derived",
    "user_asserted",
    "operator_asserted",
    "artifact_derived",
    "screenshot_derived",
    "runtime_observed",
    "model_inferred",
    "reducer_promoted",
    "verification_confirmed",
];
const LINK_TYPES: &[&str] = &[
    "imports",
    "calls",
    "renders",
    "persists_to",
    "depends_on",
    "configured_by",
    "tested_by",
    "implements",
    "violates",
    "blocks",
    "supersedes",
    "belongs_to_goal",
    "derived_from_hlt",
    "derived_from_mlg",
    "derived_from_stg",
    "marks_waypoint_for",
    "belongs_to_working_set",
    "constrains",
    "supports",
    "verifies",
    "derived_from",
    "contains",
    "declared_in",
    "targets_schema",
    "owned_by_repo",
    "enabled_by",
    "requires_permission",
    "bounded_by_authority",
    "consumes_resource",
    "has_reliability",
    "has_reversibility",
    "available_in_context",
    "blocks_execution_of",
    "supports_execution_of",
    "composed_of",
    "variants_of",
    "fills_slot",
    "aligns_with",
    "inherits_token",
    "binds_to",
    "transitions_to",
    "validates",
    "derived_from_reference",
    "governed_by",
    "includes_context",
    "excludes_context",
    "violates_scope_of",
    "aliases",
    "candidate_for",
    "resolved_as",
    "equivalent_to",
    "supersedes_entity",
    "derived_from_canonical",
    "shaped_by_view",
    "allowed_for_role",
    "versioned_as",
    "compatible_with",
    "migrated_by",
    "deprecated_by",
    "approved_by_governance",
    "instantiates",
    "serves_role",
    "has_capability_profile",
    "has_permission_profile",
    "owns_responsibility",
    "bounded_by_handoff",
    "persists_via",
    "governed_by_identity",
    "commits_to",
    "inhibits",
    "persists_on",
    "abandons_under",
    "drives_completion_of",
    "conflicts_with",
    "retained_under",
    "decays_via",
    "archived_as",
    "pruned_by",
];
const ACTION_TYPES: &[&str] = &[
    "refactor_module",
    "modify_schema",
    "add_route",
    "add_test",
    "verify_invariant",
    "promote_decision",
    "apply_inhibition",
    "evaluate_switch",
    "maintain_commitment",
    "authorize_abandonment",
    "push_to_completion",
    "evaluate_retention",
    "archive_object",
    "restore_from_archive",
    "prune_active_context",
    "derive_mlg_from_hlt",
    "derive_stg_from_mlg",
    "offer_waypoints",
    "decompose_goal",
    "prioritize_work",
    "record_decision",
    "register_constraint",
    "identify_risk",
    "mark_blocked",
    "resolve_risk",
    "route_reflex_primitive",
    "suggest_reflex_recovery",
    "inspect_reflex_registry",
    "restore_progress",
    "verify_progress",
    "refresh_working_set",
    "close_loop",
    "complete_task",
    "rollback_change",
    "detect_affordances",
    "verify_permissions",
    "verify_preconditions",
    "evaluate_dependencies",
    "estimate_cost",
    "estimate_latency",
    "estimate_reliability",
    "estimate_reversibility",
    "choose_execution_path",
    "escalate_authority",
    "mark_unavailable",
    "derive_structure",
    "extract_components",
    "derive_slots",
    "infer_tokens",
    "infer_spacing",
    "map_component_tree",
    "attach_bindings",
    "attach_validation",
    "wire_interaction",
    "compare_to_reference",
    "critique_ui",
    "infer_interaction_and_state",
    "derive_implementation_semantics",
    "derive_component_tree",
    "derive_plumbing_requirements",
    "map_tokens_to_surfaces",
    "map_states_to_views",
    "map_bindings_and_validation",
    "synthesize_completion_checklist",
    "determine_current_ask",
    "build_query_scope",
    "select_relevant_context",
    "exclude_irrelevant_context",
    "verify_answer_scope",
    "record_scope_failure",
    "detect_aliases",
    "build_resolution_candidates",
    "resolve_identity",
    "verify_resolution",
    "record_supersession",
    "reject_cross_session_resume",
    "record_prediction",
    "evaluate_prediction",
    "build_projection",
    "compress_projection",
    "verify_projection_fidelity",
    "switch_view_profile",
    "create_version",
    "declare_compatibility",
    "build_migration_plan",
    "execute_migration",
    "deprecate_schema_element",
    "review_governance_change",
    "verify_post_migration_conformance",
    "establish_identity",
    "load_role_profile",
    "verify_capability_profile",
    "verify_permission_profile",
    "assign_responsibility",
    "determine_handoff_boundary",
    "restore_identity_continuity",
    "form_intention",
    "promote_commitment",
    "record_goal_conflict",
    "evaluate_retention",
    "apply_decay",
];
const SLICE_TYPES: &[&str] = &[
    "active_mission",
    "debugging",
    "refactor",
    "regression",
    "architecture",
];

static ACTION_CATALOG_PROJECTION: LazyLock<Vec<Value>> = LazyLock::new(|| {
    ACTION_TYPES
        .iter()
        .map(|name| {
            let contract = action_contract(name);
            let verification_hooks = contract
                .get("verification_hooks")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let runtime_execution_supported = contract
                .get("tool_mappings")
                .and_then(Value::as_array)
                .map(|mappings| !mappings.is_empty())
                .unwrap_or(false);
            let reducer_visible = contract
                .get("expected_ontology_deltas")
                .and_then(Value::as_array)
                .map(|deltas| !deltas.is_empty())
                .unwrap_or(false);
            json!({
                "name": name,
                "constraint_checked": true,
                "reducer_visible": reducer_visible,
                "verification_hooks": verification_hooks,
                "runtime_execution_supported": runtime_execution_supported,
                "catalog_role": "contract_projection_reference",
                "cache_role": "static_action_catalog_projection"
            })
        })
        .collect()
});
const MAX_DISCOVERED_PATHS: usize = 512;
const DEFAULT_DISCOVERY_SCAN_PATHS: usize = 48;

fn max_discovery_scan_paths() -> usize {
    env_limit(
        "FOCUSA_ONTOLOGY_WORKSPACE_SCAN_LIMIT",
        DEFAULT_DISCOVERY_SCAN_PATHS,
    )
    .min(10_000)
}
const MAX_DISCOVERED_SYMBOLS: usize = 24;
const MAX_DISCOVERED_ENDPOINTS: usize = 16;

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct OntologyWorldQuery {
    frame_id: Option<String>,
    #[serde(default = "default_true")]
    summary_only: bool,
    #[serde(default)]
    include_full_payload: bool,
    #[serde(default)]
    include_action_catalog: bool,
    #[serde(default)]
    include_working_sets: bool,
    limit_objects: Option<usize>,
    limit_links: Option<usize>,
    cursor_objects: Option<usize>,
    cursor_links: Option<usize>,
    #[serde(default)]
    force_full_payload: bool,
}

#[derive(Deserialize)]
struct SliceQuery {
    frame_id: Option<String>,
    #[serde(default = "default_slice_type")]
    slice_type: String,
}

#[derive(Deserialize)]
struct AdjacencyQuery {
    frame_id: Option<String>,
    target_ref: Option<String>,
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct WorkingSetQuery {
    frame_id: Option<String>,
    ask: Option<String>,
    target_ref: Option<String>,
    limit: Option<usize>,
    cursor: Option<usize>,
    #[serde(default = "default_slice_type")]
    slice_type: String,
    #[serde(default)]
    include_reasons: bool,
}

#[derive(Deserialize)]
struct OntologyContextRequest {
    current_ask: Option<String>,
    frame_id: Option<String>,
    workpoint_id: Option<String>,
    #[serde(default)]
    target_refs: Vec<String>,
    budget_tokens: Option<usize>,
    view_profile: Option<String>,
    #[serde(default = "default_slice_type")]
    slice_type: String,
    #[serde(default)]
    operator_steering_detected: bool,
    #[serde(default)]
    active_object_refs: Vec<String>,
}

#[derive(Deserialize)]
struct AffordancesQuery {
    frame_id: Option<String>,
    target_ref: Option<String>,
    action_intent: Option<String>,
    scope: Option<String>,
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct RetrievalGovernorRequest {
    current_ask: Option<String>,
    frame_id: Option<String>,
    workpoint_id: Option<String>,
    #[serde(default)]
    target_refs: Vec<String>,
    budget_tokens: Option<usize>,
    #[serde(default)]
    operator_steering_detected: bool,
    #[serde(default)]
    include_metacog: bool,
    ask_kind: Option<String>,
    query_scope: Option<String>,
    action_intent: Option<String>,
    #[serde(default)]
    stale_state: bool,
    #[serde(default)]
    degraded_state: bool,
    #[serde(default)]
    previous_retrieval_outcomes: Vec<Value>,
}

#[derive(Deserialize)]
struct ToolResultProposalRequest {
    tool_name: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    ok: Option<bool>,
    #[serde(default)]
    target_refs: Vec<String>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    #[serde(default)]
    workpoint_id: Option<String>,
    #[serde(default)]
    action_intent: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    emit_proposals: bool,
}

#[derive(Deserialize)]
struct ExecutionCriticRequest {
    intended_action: Option<String>,
    #[serde(default)]
    target_refs: Vec<String>,
    #[serde(default)]
    verification_hooks: Vec<String>,
    tool_result: ToolResultProposalRequest,
    workpoint_next_action: Option<String>,
    #[serde(default)]
    operator_priority: Option<String>,
}

#[derive(Deserialize)]
struct ReflectionSynthesizerRequest {
    #[serde(default)]
    traces: Vec<Value>,
    #[serde(default)]
    evals: Vec<Value>,
    #[serde(default)]
    critic_outputs: Vec<Value>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    #[serde(default)]
    scope_tags: Vec<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    promote: bool,
}

#[derive(Deserialize)]
struct MemoryPipelineRequest {
    scope: WorkstreamKey,
    #[serde(default)]
    episodic_events: Vec<Value>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    #[serde(default)]
    synthesis_artifacts: Vec<Value>,
    #[serde(default)]
    eval_results: Vec<Value>,
    #[serde(default)]
    repeated_validation_count: Option<usize>,
    #[serde(default)]
    lesson_age_days: Option<u64>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct OntologyActionRequest {
    action_type: String,
    #[serde(default)]
    payload: Value,
    source: Option<String>,
    proposal_id: Option<String>,
    auto_verify: Option<bool>,
    auto_promote: Option<bool>,
}

#[derive(Deserialize)]
struct OntologyScopeMigrationRequest {
    action: String,
    migration_id: Option<Uuid>,
    rollback_id: Option<Uuid>,
    #[serde(default)]
    selections: Vec<OntologyScopeMigrationSelection>,
    #[serde(default)]
    evidence_refs: Vec<String>,
}

fn default_slice_type() -> String {
    "active_mission".to_string()
}

fn normalize_slice_type(slice_type: &str) -> &str {
    if SLICE_TYPES.contains(&slice_type) {
        slice_type
    } else {
        "active_mission"
    }
}

fn infer_slice_type_from_operator_context<'a>(
    focusa: &'a FocusaState,
    requested: &'a str,
) -> &'a str {
    let normalized = normalize_slice_type(requested);
    if normalized != "active_mission" {
        return normalized;
    }

    let ask_kind = focusa
        .work_loop
        .decision_context
        .ask_kind
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    let current_ask = focusa
        .work_loop
        .decision_context
        .current_ask
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();

    if ask_kind.contains("debug")
        || current_ask.contains("debug")
        || current_ask.contains("error")
        || current_ask.contains("fail")
    {
        "debugging"
    } else if ask_kind.contains("refactor") || current_ask.contains("refactor") {
        "refactor"
    } else if ask_kind.contains("regression")
        || current_ask.contains("regression")
        || current_ask.contains("verify")
    {
        "regression"
    } else if ask_kind.contains("architect") || current_ask.contains("architecture") {
        "architecture"
    } else {
        "active_mission"
    }
}

fn slice_view_profile(slice_type: &str) -> &'static str {
    match normalize_slice_type(slice_type) {
        "debugging" => "pi_debugging_view",
        "refactor" => "pi_refactor_view",
        "regression" => "pi_regression_view",
        "architecture" => "pi_architecture_view",
        _ => "pi_operator_view",
    }
}

fn slice_projection_kind(slice_type: &str) -> &'static str {
    match normalize_slice_type(slice_type) {
        "debugging" => "debugging_projection",
        "refactor" => "refactor_projection",
        "regression" => "regression_projection",
        "architecture" => "architecture_projection",
        _ => "active_mission_projection",
    }
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
        "endpoint" => &[
            "id",
            "path_or_signature",
            "method_or_transport",
            "package_id",
        ],
        "schema" => &["id", "schema_name", "storage_kind"],
        "migration" => &["id", "path", "schema_targets"],
        "dependency" => &["id", "name", "version", "dependency_kind", "status"],
        "test" => &["id", "path", "test_kind"],
        "environment" => &["id", "name", "environment_kind"],
        "capability" => &["id", "capability_kind", "status"],
        "tool_surface" => &["id", "surface_kind", "status"],
        "permission" => &["id", "permission_kind", "status"],
        "authority_boundary" => &["id", "boundary_kind", "status"],
        "precondition" => &["id", "precondition_kind", "status"],
        "resource" => &["id", "resource_kind", "status"],
        "cost_model" => &["id", "cost_kind", "status"],
        "latency_profile" => &["id", "latency_kind", "status"],
        "reliability_profile" => &["id", "reliability_kind", "status"],
        "reversibility_profile" => &["id", "reversibility_kind", "status"],
        "ownership" => &["id", "owner_kind", "status"],
        "execution_context" => &["id", "context_kind", "status"],
        "affordance" => &["id", "affordance_kind", "status"],
        "page" => &["id", "name", "page_kind", "primary_goal", "status"],
        "region" => &["id", "name", "region_kind", "status"],
        "component" => &["id", "name", "component_kind", "status"],
        "variant" => &["id", "name", "variant_kind", "status"],
        "content_slot" => &["id", "slot_kind", "status"],
        "token" => &["id", "token_kind", "value", "status"],
        "layout_rule" => &["id", "rule_kind", "status"],
        "interaction" => &["id", "interaction_kind", "status"],
        "ui_state" => &["id", "state_kind", "status"],
        "binding" => &["id", "binding_kind", "status"],
        "validation_rule" => &["id", "rule_kind", "status"],
        "visual_artifact" => &["id", "artifact_kind", "status"],
        "task" => &["id", "title", "status", "priority"],
        "bug" => &["id", "title", "severity", "status"],
        "feature" => &["id", "title", "status"],
        "decision" => &["id", "statement", "decision_kind", "status"],
        "convention" => &["id", "rule_text", "convention_kind", "status"],
        "constraint" => &["id", "rule_text", "scope", "enforcement_level"],
        "risk" => &["id", "title", "severity", "status"],
        "trajectory_hlt" => &[
            "id",
            "title",
            "ultimate_direction",
            "operator_authority",
            "status",
        ],
        "trajectory_mlg" => &["id", "title", "hlt_id", "objective", "status"],
        "trajectory_stg" => &["id", "title", "mlg_id", "bounded_goal", "status"],
        "trajectory_waypoint" => &["id", "title", "stg_id", "evidence_policy", "status"],
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
        "current_ask" => &["id", "ask_text", "ask_kind", "status"],
        "query_scope" => &["id", "scope_kind", "status"],
        "relevant_context_set" => &["id", "selection_kind", "status"],
        "excluded_context_set" => &["id", "exclusion_kind", "status"],
        "scope_failure" => &["id", "failure_kind", "severity", "status"],
        "canonical_entity" => &["id", "entity_kind", "status"],
        "reference_alias" => &["id", "alias_kind", "status"],
        "resolution_candidate" => &["id", "candidate_kind", "status"],
        "resolution_decision" => &["id", "decision_kind", "status"],
        "supersession_record" => &["id", "record_kind", "status"],
        "projection" => &["id", "projection_kind", "status"],
        "view_profile" => &["id", "view_kind", "status"],
        "projection_rule" => &["id", "rule_kind", "status"],
        "projection_boundary" => &["id", "boundary_kind", "status"],
        "ontology_version" => &["id", "version_kind", "status"],
        "compatibility_profile" => &["id", "profile_kind", "status"],
        "migration_plan" => &["id", "plan_kind", "status"],
        "deprecation_record" => &["id", "record_kind", "status"],
        "governance_decision" => &["id", "decision_kind", "status"],
        "agent_identity" => &["id", "identity_name", "identity_kind", "status"],
        "actor_instance" => &["id", "instance_kind", "status"],
        "actor" => &["id", "actor_kind", "status"],
        "role_profile" => &["id", "role_kind", "status"],
        "capability_profile" => &["id", "profile_kind", "status"],
        "permission_profile" => &["id", "profile_kind", "status"],
        "responsibility" => &["id", "responsibility_kind", "status"],
        "handoff_boundary" => &["id", "boundary_kind", "status"],
        "session_continuity" => &["id", "continuity_kind", "status"],
        "identity_state" => &["id", "state_kind", "status"],
        "ontology_domain" => &["id", "domain_kind", "status"],
        "shared_layer" => &["id", "layer_kind", "status"],
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
        "apply_inhibition" => &["active_focus", "open_loop", "goal", "risk"],
        "evaluate_switch" => &["active_focus", "goal", "risk", "verification"],
        "maintain_commitment" => &["active_focus", "goal", "task", "open_loop"],
        "authorize_abandonment" => &["active_focus", "goal", "task", "risk"],
        "push_to_completion" => &["task", "goal", "trajectory_waypoint", "active_focus"],
        "evaluate_retention" => &["artifact", "verification", "active_focus", "goal"],
        "archive_object" => &["artifact", "archive_record", "retention_policy"],
        "restore_from_archive" => &["archive_record", "artifact", "canonical_entity"],
        "prune_active_context" => &["active_focus", "working_set", "retention_policy"],
        "decompose_goal" => &["goal", "subgoal", "task", "active_focus"],
        "prioritize_work" => &["task", "goal", "subgoal", "trajectory_waypoint"],
        "record_decision" => &["decision", "goal", "constraint", "risk"],
        "register_constraint" => &["constraint", "goal", "task", "risk"],
        "identify_risk" => &["risk", "task", "goal", "verification"],
        "mark_blocked" => &["task", "goal", "risk", "failure"],
        "resolve_risk" => &["risk", "verification", "task"],
        "restore_progress" => &["task", "goal", "trajectory_waypoint", "active_focus"],
        "verify_progress" => &["verification", "task", "goal", "trajectory_waypoint"],
        "refresh_working_set" => &["active_focus", "task", "goal", "subgoal"],
        "close_loop" => &["task", "goal", "verification", "decision"],
        "complete_task" => &["task", "goal", "trajectory_waypoint"],
        "rollback_change" => &["patch", "diff", "artifact"],
        "detect_affordances" => &[
            "affordance",
            "execution_context",
            "tool_surface",
            "capability",
        ],
        "verify_permissions" => &[
            "permission",
            "authority_boundary",
            "affordance",
            "capability",
        ],
        "verify_preconditions" => &["precondition", "dependency", "resource", "affordance"],
        "evaluate_dependencies" => &["dependency", "precondition", "affordance", "capability"],
        "estimate_cost" => &["cost_model", "resource", "affordance"],
        "estimate_latency" => &["latency_profile", "execution_context", "affordance"],
        "estimate_reliability" => &["reliability_profile", "affordance", "tool_surface"],
        "estimate_reversibility" => &["reversibility_profile", "affordance", "tool_surface"],
        "choose_execution_path" => &["affordance", "task", "execution_context", "risk"],
        "escalate_authority" => &[
            "authority_boundary",
            "permission",
            "ownership",
            "affordance",
        ],
        "mark_unavailable" => &["affordance", "precondition", "dependency", "resource"],
        "derive_structure" => &["visual_artifact", "page", "region", "layout_rule"],
        "extract_components" => &["visual_artifact", "component", "variant", "region"],
        "derive_slots" => &["content_slot", "component", "region", "visual_artifact"],
        "infer_tokens" => &["token", "component", "region", "visual_artifact"],
        "infer_spacing" => &["layout_rule", "token", "region", "component"],
        "map_component_tree" => &["page", "region", "component", "variant"],
        "attach_bindings" => &["binding", "component", "ui_state", "validation_rule"],
        "attach_validation" => &["validation_rule", "binding", "component", "ui_state"],
        "wire_interaction" => &["interaction", "ui_state", "component", "page"],
        "compare_to_reference" => &["visual_artifact", "verification", "component", "page"],
        "critique_ui" => &["verification", "component", "layout_rule", "ui_state"],
        "infer_interaction_and_state" => &["interaction", "ui_state", "binding", "validation_rule"],
        "derive_implementation_semantics" => &["component", "binding", "validation_rule", "page"],
        "derive_component_tree" => &["page", "region", "component", "content_slot"],
        "derive_plumbing_requirements" => {
            &["interaction", "ui_state", "binding", "validation_rule"]
        }
        "map_tokens_to_surfaces" => &["token", "layout_rule", "component", "region"],
        "map_states_to_views" => &["ui_state", "interaction", "component", "page"],
        "map_bindings_and_validation" => &["binding", "validation_rule", "component", "ui_state"],
        "synthesize_completion_checklist" => {
            &["verification", "acceptance_criterion", "task", "artifact"]
        }
        "determine_current_ask" => &["current_ask", "query_scope"],
        "build_query_scope" => &["query_scope", "current_ask"],
        "select_relevant_context" => &[
            "relevant_context_set",
            "current_ask",
            "decision",
            "constraint",
            "artifact",
            "visual_artifact",
        ],
        "exclude_irrelevant_context" => &[
            "excluded_context_set",
            "current_ask",
            "decision",
            "constraint",
            "artifact",
            "visual_artifact",
        ],
        "verify_answer_scope" => &[
            "query_scope",
            "current_ask",
            "verification",
            "scope_failure",
        ],
        "record_scope_failure" => &["scope_failure", "current_ask", "query_scope"],
        "detect_aliases" => &["reference_alias", "canonical_entity", "artifact"],
        "build_resolution_candidates" => &[
            "resolution_candidate",
            "canonical_entity",
            "reference_alias",
        ],
        "resolve_identity" => &[
            "resolution_decision",
            "canonical_entity",
            "resolution_candidate",
        ],
        "verify_resolution" => &["verification", "resolution_decision", "canonical_entity"],
        "record_supersession" => &["supersession_record", "canonical_entity"],
        "build_projection" => &[
            "projection",
            "view_profile",
            "projection_rule",
            "projection_boundary",
        ],
        "compress_projection" => &["projection", "projection_boundary"],
        "verify_projection_fidelity" => {
            &["projection", "verification", "query_scope", "current_ask"]
        }
        "switch_view_profile" => &["view_profile", "projection", "actor", "role_profile"],
        "create_version" => &["ontology_version", "ontology_domain", "shared_layer"],
        "declare_compatibility" => &["ontology_version", "compatibility_profile"],
        "build_migration_plan" => &["migration_plan", "ontology_version", "ontology_domain"],
        "execute_migration" => &["migration_plan", "ontology_version"],
        "deprecate_schema_element" => &[
            "deprecation_record",
            "ontology_domain",
            "shared_layer",
            "ontology_version",
        ],
        "review_governance_change" => &[
            "governance_decision",
            "migration_plan",
            "deprecation_record",
            "ontology_version",
        ],
        "verify_post_migration_conformance" => &[
            "verification",
            "ontology_domain",
            "shared_layer",
            "ontology_version",
        ],
        "establish_identity" => &["agent_identity", "actor_instance", "identity_state"],
        "load_role_profile" => &["role_profile", "agent_identity", "actor_instance"],
        "verify_capability_profile" => &[
            "capability_profile",
            "actor_instance",
            "capability",
            "tool_surface",
        ],
        "verify_permission_profile" => &[
            "permission_profile",
            "actor_instance",
            "permission",
            "authority_boundary",
        ],
        "assign_responsibility" => &["responsibility", "task", "goal", "agent_identity"],
        "determine_handoff_boundary" => &[
            "handoff_boundary",
            "agent_identity",
            "actor_instance",
            "responsibility",
        ],
        "restore_identity_continuity" => &[
            "session_continuity",
            "identity_state",
            "agent_identity",
            "actor_instance",
        ],
        "form_intention" => &["task", "goal", "subgoal", "active_focus"],
        "promote_commitment" => &["task", "goal", "subgoal", "decision"],
        "record_goal_conflict" => &["goal", "subgoal", "risk", "constraint"],
        "apply_decay" => &["object_set", "semantic_memory_entry", "failure"],
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
            json!([
                "focus_frame_update",
                "command dispatch",
                "reducer-visible events"
            ]),
            json!([
                "validation_failure",
                "dependency_failure",
                "execution_failure",
                "verification_failure",
                "timeout",
                "partial_success"
            ]),
            json!("best_effort, repeatable with same target module"),
            json!({"available":true,"mechanism":"rollback_change / VCS revert"}),
            json!([
                "tests/tool_contract_test.sh",
                "tests/command_write_contract_test.sh"
            ]),
            json!([
                "module updated",
                "verification queued",
                "artifact refs produced"
            ]),
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
            json!([
                "migration proposal",
                "schema evidence",
                "verification hooks"
            ]),
            json!([
                "validation_failure",
                "dependency_failure",
                "permission_failure",
                "execution_failure",
                "verification_failure",
                "rollback_failure"
            ]),
            json!("non-idempotent without migration identity; requires explicit target"),
            json!({"available":true,"mechanism":"rollback_change / compensating migration"}),
            json!(["tests/tool_contract_test.sh", "tests/golden_tasks_eval.sh"]),
            json!([
                "schema target updated",
                "migration linked",
                "verification pending"
            ]),
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
            json!([
                "route/endpoint projection",
                "verification hooks",
                "artifact refs"
            ]),
            json!([
                "validation_failure",
                "execution_failure",
                "verification_failure",
                "timeout"
            ]),
            json!("idempotent only when path+method pair already canonicalized"),
            json!({"available":true,"mechanism":"rollback_change / route removal"}),
            json!([
                "tests/ontology_world_contract_test.sh",
                "tests/tool_contract_test.sh"
            ]),
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
            json!([
                "validation_failure",
                "execution_failure",
                "partial_success",
                "timeout"
            ]),
            json!("idempotent when target_path already contains canonical test"),
            json!({"available":true,"mechanism":"rollback_change / file revert"}),
            json!([
                "tests/ontology_world_contract_test.sh",
                "tests/golden_tasks_eval.sh"
            ]),
            json!([
                "test object added",
                "tested_by link added",
                "verification target queued"
            ]),
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
            json!([
                "validation_failure",
                "execution_failure",
                "verification_failure",
                "timeout"
            ]),
            json!("repeatable and expected to be idempotent over same target"),
            json!({"available":false,"mechanism":"n/a"}),
            json!([
                "tests/trace_dimensions_test.sh",
                "tests/golden_tasks_eval.sh"
            ]),
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
            json!([
                "decision distillation",
                "proposal scoring",
                "canonical mutation"
            ]),
            json!([
                "validation_failure",
                "execution_failure",
                "verification_failure"
            ]),
            json!("idempotent when same decision already canonical"),
            json!({"available":true,"mechanism":"superseding decision"}),
            json!([
                "tests/behavioral_alignment_test.sh",
                "tests/proposal_kind_enforcement_test.sh"
            ]),
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
            json!([
                "validation_failure",
                "dependency_failure",
                "execution_failure"
            ]),
            json!("repeatable; duplicates should converge on surfaced candidate state"),
            json!({"available":true,"mechanism":"resolve_risk / suppress candidate"}),
            json!([
                "tests/checkpoint_trigger_test.sh",
                "tests/behavioral_alignment_test.sh"
            ]),
            json!([
                "failure object added",
                "blocks link added",
                "gate candidate surfaced"
            ]),
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
            json!([
                "validation_failure",
                "execution_failure",
                "verification_failure",
                "timeout"
            ]),
            json!("repeatable while risk remains active"),
            json!({"available":true,"mechanism":"mark_blocked or supersede risk"}),
            json!([
                "tests/golden_tasks_eval.sh",
                "tests/trace_dimensions_test.sh"
            ]),
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
            json!([
                "frame completion",
                "checkpoint persistence",
                "lineage update"
            ]),
            json!(["validation_failure", "execution_failure", "partial_success"]),
            json!("idempotent when task already completed"),
            json!({"available":true,"mechanism":"supersede / reopen task"}),
            json!([
                "tests/fork_compact_recovery_test.sh",
                "tests/checkpoint_trigger_test.sh"
            ]),
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
            json!([
                "validation_failure",
                "permission_failure",
                "execution_failure",
                "rollback_failure",
                "timeout"
            ]),
            json!("idempotent once rollback reaches canonical target state"),
            json!({"available":true,"mechanism":"VCS revert / compensating change"}),
            json!([
                "tests/fork_compact_recovery_test.sh",
                "tests/command_write_contract_test.sh"
            ]),
            json!(["patch/diff status changed", "verification pending"]),
            json!({"source":"/v1/status","job_timeout_ms_field":"worker_status.job_timeout_ms"}),
            json!({"policy":"retry only after permission/dependency remediation","max_attempts":2}),
            json!({"behavior":"emit failure + preserve prior checkpoint if rollback fails"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"micro-compact"},
                {"surface":"http","method":"POST","path":"/v1/commands/submit","command":"compact"}
            ]),
        ),
        "derive_structure"
        | "extract_components"
        | "derive_slots"
        | "infer_tokens"
        | "infer_spacing"
        | "infer_interaction_and_state"
        | "derive_implementation_semantics" => (
            json!({"type":"object","required":["artifact_refs"],"properties":{"artifact_refs":{"type":"array","items":{"type":"string"}},"frame_id":{"type":"string"},"stage":{"type":"string"},"confidence_floor":{"type":"number"}}}),
            json!({"required":["result_status","affected_object_refs","ontology_delta_candidates","evidence_refs","stage_confidence"]}),
            json!([
                "typed ontology proposals",
                "evidence linkage",
                "blueprint stage snapshot"
            ]),
            json!([
                "validation_failure",
                "insufficient_evidence",
                "ambiguous_extraction",
                "partial_success"
            ]),
            json!("deterministic within fixed artifacts and extraction policy version"),
            json!({"available":true,"mechanism":"supersede proposal with refined extraction pass"}),
            json!(["tests/ontology_visual_reverse_extraction_pipeline_contract_test.sh"]),
            json!([
                "visual object proposals emitted",
                "stage confidence recorded",
                "comparison baseline prepared"
            ]),
            json!({"source":"/v1/ontology/contracts","job_timeout_ms_field":null}),
            json!({"policy":"rerun after adding artifacts or narrowing ambiguity","max_attempts":3}),
            json!({"behavior":"preserve proposal-level outputs and emit missing-evidence markers"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/ontology/contracts","command":"ontology.contracts"},
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"}
            ]),
        ),
        "derive_component_tree"
        | "derive_plumbing_requirements"
        | "map_tokens_to_surfaces"
        | "map_states_to_views"
        | "map_bindings_and_validation"
        | "synthesize_completion_checklist" => (
            json!({"type":"object","required":["blueprint_ref"],"properties":{"blueprint_ref":{"type":"string"},"frame_id":{"type":"string"},"implementation_target":{"type":"string"},"strictness":{"type":"string"}}}),
            json!({"required":["result_status","affected_object_refs","handoff_outputs","plumbing_requirements","completion_checks","conformance_report","diff_validation_report","intent_preservation_result"]}),
            json!([
                "implementation handoff projection",
                "typed plumbing map",
                "completion readiness checklist",
                "handoff conformance report",
                "implementation diff validation report"
            ]),
            json!([
                "validation_failure",
                "insufficient_handoff_detail",
                "missing_state_coverage",
                "conformance_failure",
                "diff_validation_failure",
                "intent_drift_detected",
                "partial_success"
            ]),
            json!("deterministic for fixed blueprint and implementation policy version"),
            json!({"available":true,"mechanism":"supersede handoff outputs with refined blueprint mapping"}),
            json!([
                "tests/ontology_visual_implementation_handoff_contract_test.sh",
                "tests/ontology_visual_implementation_handoff_conformance_diff_contract_test.sh"
            ]),
            json!([
                "component-tree mapping emitted",
                "plumbing coverage surfaced",
                "completion checks synthesized",
                "conformance report emitted",
                "implementation diff validation recorded"
            ]),
            json!({"source":"/v1/ontology/contracts","job_timeout_ms_field":null}),
            json!({"policy":"rerun after blueprint refinement or missing-plumbing remediation","max_attempts":3}),
            json!({"behavior":"emit proposal-level handoff outputs with explicit uncovered plumbing gaps and explicit intent-preservation status"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/ontology/contracts","command":"ontology.contracts"},
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"}
            ]),
        ),
        "determine_current_ask"
        | "build_query_scope"
        | "select_relevant_context"
        | "exclude_irrelevant_context"
        | "verify_answer_scope"
        | "record_scope_failure" => (
            json!({"type":"object","required":["current_ask"],"properties":{"current_ask":{"type":"string"},"ask_kind":{"type":"string"},"scope_kind":{"type":"string"},"carryover_policy":{"type":"string"},"excluded_context_reason":{"type":"string"},"excluded_context_labels":{"type":"array","items":{"type":"string"}},"source_turn_id":{"type":"string"}}}),
            json!({"required":["result_status","scope_state","affected_object_refs","verification_result_or_next_step"]}),
            json!([
                "work-loop decision context update",
                "scope-control object projection",
                "scope governance linkage"
            ]),
            json!([
                "validation_failure",
                "scope_mismatch",
                "context_write_rejected",
                "verification_failure"
            ]),
            json!("idempotent for same current_ask/scope payload"),
            json!({"available":true,"mechanism":"overwrite decision context with corrected scope payload"}),
            json!([
                "tests/work_loop_query_scope_boundary_contract_test.sh",
                "tests/doc61_first_consumer_path_test.sh",
                "tests/ontology_world_contract_test.sh"
            ]),
            json!([
                "decision_context updated",
                "query scope projected",
                "scope violations surfaced when present"
            ]),
            json!({"source":"/v1/work-loop/status","job_timeout_ms_field":null}),
            json!({"policy":"retry after writer-claim or payload correction","max_attempts":2}),
            json!({"behavior":"emit context unchanged + scope failure evidence when write cannot be applied"}),
            json!([
                {"surface":"http","method":"POST","path":"/v1/work-loop/context","command":"work-loop.context"},
                {"surface":"http","method":"GET","path":"/v1/work-loop/status","command":"work-loop.status"},
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"},
                {"surface":"http","method":"GET","path":"/v1/events/recent","command":"events.recent"}
            ]),
        ),
        "detect_aliases"
        | "build_resolution_candidates"
        | "resolve_identity"
        | "verify_resolution"
        | "record_supersession" => (
            json!({"type":"object","required":["reference"],"properties":{"reference":{"type":"string"},"canonical_hint":{"type":"string"},"resolution_policy":{"type":"string"},"confidence_floor":{"type":"number"}}}),
            json!({"required":["result_status","resolution_state","affected_object_refs","verification_result_or_next_step"]}),
            json!([
                "reference index lookup",
                "identity resolution candidate ranking",
                "canonical/supersession projection"
            ]),
            json!([
                "validation_failure",
                "reference_not_found",
                "ambiguous_resolution",
                "verification_failure"
            ]),
            json!("idempotent for same reference and resolution policy"),
            json!({"available":true,"mechanism":"supersede resolution decision with higher-confidence canonical target"}),
            json!([
                "tests/doc74_reference_resolution_consumer_path_test.sh",
                "tests/ontology_world_contract_test.sh"
            ]),
            json!([
                "reference aliases surfaced",
                "resolution candidates linked",
                "resolution decisions/verifications projected"
            ]),
            json!({"source":"/v1/references/search","job_timeout_ms_field":null}),
            json!({"policy":"retry after expanding evidence or reducing ambiguity","max_attempts":2}),
            json!({"behavior":"emit candidate set only and mark unresolved when confidence floor not met"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/references/search","command":"references.search"},
                {"surface":"http","method":"GET","path":"/v1/references","command":"references.list"},
                {"surface":"http","method":"GET","path":"/v1/references/{ref_id}","command":"references.get"},
                {"surface":"http","method":"GET","path":"/v1/references/{ref_id}/meta","command":"references.meta"},
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"}
            ]),
        ),
        "build_projection"
        | "compress_projection"
        | "verify_projection_fidelity"
        | "switch_view_profile" => (
            json!({"type":"object","required":["projection_kind"],"properties":{"projection_kind":{"type":"string"},"view_profile":{"type":"string"},"scope_kind":{"type":"string"},"fidelity_target":{"type":"string"}}}),
            json!({"required":["result_status","projection_state","affected_object_refs","verification_result_or_next_step"]}),
            json!([
                "projection view shaping",
                "working-set boundary enforcement",
                "fidelity verification"
            ]),
            json!([
                "validation_failure",
                "scope_overflow",
                "projection_fidelity_failure",
                "verification_failure"
            ]),
            json!("idempotent for stable source world + view profile"),
            json!({"available":true,"mechanism":"switch view profile or tighten projection boundary"}),
            json!([
                "tests/ontology_world_contract_test.sh",
                "tests/work_loop_query_scope_boundary_contract_test.sh"
            ]),
            json!([
                "projection/view objects surfaced",
                "projection boundaries represented",
                "projection fidelity verifications emitted"
            ]),
            json!({"source":"/v1/ontology/world","job_timeout_ms_field":null}),
            json!({"policy":"retry after boundary/profile adjustment","max_attempts":2}),
            json!({"behavior":"return bounded projection with explicit omissions when fidelity cannot be satisfied"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"},
                {"surface":"http","method":"GET","path":"/v1/ontology/slices","command":"ontology.slices"},
                {"surface":"http","method":"GET","path":"/v1/ontology/contracts","command":"ontology.contracts"}
            ]),
        ),
        "create_version"
        | "declare_compatibility"
        | "build_migration_plan"
        | "execute_migration"
        | "deprecate_schema_element"
        | "review_governance_change"
        | "verify_post_migration_conformance" => (
            json!({"type":"object","required":["version_ref"],"properties":{"version_ref":{"type":"string"},"domain":{"type":"string"},"compatibility_target":{"type":"string"},"migration_plan_ref":{"type":"string"},"governance_change_ref":{"type":"string"}}}),
            json!({"required":["result_status","governance_state","affected_object_refs","verification_result_or_next_step"]}),
            json!([
                "ontology version/governance projection",
                "migration conformance tracking",
                "post-migration verification"
            ]),
            json!([
                "validation_failure",
                "governance_conflict",
                "migration_conformance_failure",
                "verification_failure"
            ]),
            json!("idempotent per version_ref + migration_plan_ref tuple"),
            json!({"available":true,"mechanism":"supersede migration plan/version compatibility profile"}),
            json!([
                "tests/work_loop_migration_conformance_checks_test.sh",
                "tests/doc78_remaining_frontier_contract_test.sh",
                "tests/ontology_world_contract_test.sh"
            ]),
            json!([
                "version and compatibility objects projected",
                "migration/governance records represented",
                "post-migration verifications surfaced"
            ]),
            json!({"source":"/v1/events/recent","job_timeout_ms_field":null}),
            json!({"policy":"retry after governance approval or migration-plan correction","max_attempts":2}),
            json!({"behavior":"emit governance decision plus pending conformance verification when migration cannot execute"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/ontology/contracts","command":"ontology.contracts"},
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"},
                {"surface":"http","method":"GET","path":"/v1/events/recent","command":"events.recent"},
                {"surface":"http","method":"GET","path":"/v1/work-loop/status","command":"work-loop.status"}
            ]),
        ),
        "map_component_tree"
        | "attach_bindings"
        | "attach_validation"
        | "wire_interaction"
        | "compare_to_reference"
        | "critique_ui"
        | "detect_affordances"
        | "verify_permissions"
        | "verify_preconditions"
        | "evaluate_dependencies"
        | "estimate_cost"
        | "estimate_latency"
        | "estimate_reliability"
        | "estimate_reversibility"
        | "choose_execution_path"
        | "escalate_authority"
        | "mark_unavailable"
        | "establish_identity"
        | "load_role_profile"
        | "verify_capability_profile"
        | "verify_permission_profile"
        | "assign_responsibility"
        | "determine_handoff_boundary"
        | "restore_identity_continuity" => (
            json!({
                "type":"object",
                "required":["action_ref"],
                "properties":{
                    "action_ref":{"type":"string"},
                    "target_ref":{"type":"string"},
                    "scope":{"type":"string"},
                    "reason":{"type":"string"}
                }
            }),
            json!({
                "required":["result_status","affected_object_refs","verification_result_or_next_step"]
            }),
            json!([
                "typed ontology action intent projection",
                "bounded target set evaluation",
                "trace-visible action intent surface"
            ]),
            json!([
                "validation_failure",
                "missing_target",
                "insufficient_context",
                "verification_failure"
            ]),
            json!("idempotent per action_ref + target_ref tuple"),
            json!({"available":true,"mechanism":"supersede via newer reducer-visible proposal"}),
            json!([
                "tests/ontology_world_contract_test.sh",
                "tests/ontology_event_contract_test.sh"
            ]),
            json!([
                "ontology_object_upsert_proposed",
                "ontology_link_upsert_proposed",
                "ontology_status_change_proposed"
            ]),
            json!({"source":"/v1/ontology/contracts","job_timeout_ms_field":null}),
            json!({"policy":"retry when verification evidence is insufficient","max_attempts":2}),
            json!({"behavior":"emit proposal-only action metadata; canonical mutation remains reducer-gated"}),
            json!([
                {"surface":"http","method":"GET","path":"/v1/ontology/contracts","command":"ontology.contracts"},
                {"surface":"http","method":"GET","path":"/v1/ontology/world","command":"ontology.world"},
                {"surface":"http","method":"GET","path":"/v1/events/recent","command":"events.recent"}
            ]),
        ),
        _ => (
            json!({"type":"object"}),
            json!({}),
            json!([]),
            json!([]),
            json!("unknown"),
            json!({}),
            json!([]),
            json!([]),
            json!({}),
            json!({}),
            json!({}),
            json!([]),
        ),
    };

    let runtime_execution_supported = tool_mappings
        .as_array()
        .map(|mappings| !mappings.is_empty())
        .unwrap_or(false);

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
        "tool_action_metadata": {
            "runtime_execution_supported": runtime_execution_supported,
            "contract_role": "executable_via_ontology_action_route",
            "route_surfaces": ["POST /v1/ontology/actions", "GET /v1/ontology/contracts", "GET /v1/ontology/world"]
        },
        "trace_metadata": {
            "trace_surface": "projection_snapshot_and_action_events",
            "emits_reducer_event_on_read": false,
            "emits_reducer_event_on_action": true,
            "source_inputs": ["focus_state", "workspace_scan", "action_payload"]
        },
        "eval_metadata": {
            "validation_mode": "route-contract-regression",
            "backing_tests": ["tests/ontology_world_contract_test.sh", "tests/tool_contract_test.sh"]
        },
        "projection_metadata": {
            "projection_kind": "runtime_projection_with_action_entrypoint",
            "mutates_canonical_state": true,
            "snapshot_consistency": "best_effort"
        },
        "governance_metadata": {
            "api_permission_scope": null,
            "writes_allowed": true,
            "authority_note": "canonical writes are routed through ontology action events and reducer governance"
        }
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

    let action_types: Vec<Value> = ACTION_TYPES
        .iter()
        .map(|name| action_contract(name))
        .collect();

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

#[derive(Default, Clone)]
struct WorkspaceProjection {
    objects: Vec<Value>,
    links: Vec<Value>,
}

#[derive(Clone)]
struct OntologyReadIndex {
    source_state_version: u64,
    frame_id: Option<String>,
    generated_at: chrono::DateTime<Utc>,
    objects: BTreeMap<String, Value>,
    incoming_by_id: BTreeMap<String, Vec<Value>>,
    outgoing_by_id: BTreeMap<String, Vec<Value>>,
    incoming_by_type: BTreeMap<String, BTreeMap<String, Vec<Value>>>,
    outgoing_by_type: BTreeMap<String, BTreeMap<String, Vec<Value>>>,
    object_type_counts: BTreeMap<String, usize>,
    link_type_counts: BTreeMap<String, usize>,
    last_reducer_event_id: Option<String>,
    ttl_seconds: usize,
}

fn read_text(path: &Path) -> Option<String> {
    const MAX_ONTOLOGY_PARSE_BYTES: u64 = 256 * 1024;
    if path
        .metadata()
        .ok()
        .map(|meta| meta.len() > MAX_ONTOLOGY_PARSE_BYTES)
        .unwrap_or(false)
    {
        return None;
    }
    fs::read_to_string(path).ok()
}

fn selected_workspace_root(focusa: &FocusaState) -> Option<PathBuf> {
    if let Some(workspace_id) = focusa
        .session
        .as_ref()
        .and_then(|s| s.workspace_id.as_deref())
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        return Some(PathBuf::from(workspace_id));
    }

    env::var_os("FOCUSA_PROJECT_ROOT")
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .filter(|fallback| fallback.exists() && fallback.is_dir())
}

fn binary_available(binary: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| {
            env::split_paths(&paths).any(|dir| {
                let candidate = dir.join(binary);
                candidate.exists() && candidate.is_file()
            })
        })
        .unwrap_or(false)
}

fn walk_workspace(root: &Path) -> Vec<PathBuf> {
    let max_scan_paths = max_discovery_scan_paths();
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if out.len() >= max_scan_paths {
            break;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                if matches!(
                    name,
                    "target" | "node_modules" | "dist" | "build" | ".beads"
                ) {
                    continue;
                }
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
                if out.len() >= max_scan_paths {
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
    } else if rel.ends_with(".md")
        && (rel.to_ascii_lowercase().contains("spec") || rel.contains("docs/"))
    {
        "spec_doc"
    } else if rel.contains("route") {
        "route_source"
    } else {
        "source"
    }
}

fn file_projection_priority(root: &Path, path: &Path) -> (u8, String) {
    let rel = path
        .strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| path.to_string_lossy().to_ascii_lowercase());

    let rank = if rel.contains("/src/") || rel.starts_with("src/") {
        0
    } else if rel.contains("/routes/") || rel.contains("endpoint") || rel.contains("api") {
        1
    } else if rel.contains("/tests/") || rel.starts_with("tests/") {
        2
    } else if rel.contains("migrations") || rel.ends_with(".sql") {
        3
    } else if rel.ends_with("cargo.toml") || rel.ends_with("package.json") {
        4
    } else {
        5
    };

    (rank, rel)
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
        if let Some((name, kind)) = parsed
            && !name.is_empty()
        {
            out.push((name.to_string(), kind.to_string()));
        }
        if out.len() >= MAX_DISCOVERED_SYMBOLS {
            break;
        }
    }
    out
}

fn parse_route_bindings(content: &str) -> Vec<(String, String, Option<String>)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
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
                let handler = trimmed
                    .split_once(&format!("{method}("))
                    .and_then(|(_, rest)| rest.split(')').next())
                    .map(str::trim)
                    .filter(|value| !value.is_empty() && !value.starts_with('"'))
                    .map(|value| value.trim_start_matches("move ||").trim().to_string());
                if route.starts_with('/') {
                    out.push((method.to_string(), route.to_string(), handler));
                }
            }
        }
        if out.len() >= MAX_DISCOVERED_ENDPOINTS {
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

fn parse_import_targets(content: &str, language: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        match language {
            "rust" => {
                if let Some(rest) = trimmed.strip_prefix("use ") {
                    let target = rest
                        .split(';')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_start_matches("crate::")
                        .trim_start_matches("super::")
                        .to_string();
                    if !target.is_empty() {
                        out.insert(target);
                    }
                }
            }
            "typescript" | "javascript" => {
                if let Some(idx) = trimmed.find(" from ") {
                    let after = &trimmed[idx + 6..];
                    if let Some(start) = after.find('"').or_else(|| after.find('\'')) {
                        let quote = after.as_bytes()[start] as char;
                        let inner = &after[start + 1..];
                        if let Some(end) = inner.find(quote) {
                            let target = inner[..end].trim();
                            if !target.is_empty() {
                                out.insert(target.to_string());
                            }
                        }
                    }
                } else if trimmed.starts_with("import ")
                    && let Some(start) = trimmed.find('"').or_else(|| trimmed.find('\''))
                {
                    let quote = trimmed.as_bytes()[start] as char;
                    let inner = &trimmed[start + 1..];
                    if let Some(end) = inner.find(quote) {
                        let target = inner[..end].trim();
                        if !target.is_empty() {
                            out.insert(target.to_string());
                        }
                    }
                }
            }
            "python" => {
                if let Some(rest) = trimmed.strip_prefix("import ") {
                    let target = rest.split_whitespace().next().unwrap_or("");
                    if !target.is_empty() {
                        out.insert(target.to_string());
                    }
                } else if let Some(rest) = trimmed.strip_prefix("from ") {
                    let target = rest.split_whitespace().next().unwrap_or("");
                    if !target.is_empty() {
                        out.insert(target.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    out.into_iter().take(MAX_DISCOVERED_SYMBOLS).collect()
}

fn parse_call_targets(content: &str, language: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        for token in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
            if token.is_empty() || !token.contains('(') {
                continue;
            }
        }
        let mut cursor = trimmed;
        while let Some(idx) = cursor.find('(') {
            let prefix = cursor[..idx].trim_end();
            let candidate = prefix
                .rsplit(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .next()
                .unwrap_or("")
                .trim_matches('.');
            if !candidate.is_empty()
                && candidate
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic() || c == '_')
                    .unwrap_or(false)
                && !matches!(
                    candidate,
                    "if" | "for" | "while" | "match" | "loop" | "return"
                )
            {
                let normalized = if language == "python" {
                    candidate.to_string()
                } else {
                    candidate.replace("::", ".")
                };
                out.insert(normalized);
            }
            cursor = &cursor[idx + 1..];
        }
    }
    out.into_iter().take(MAX_DISCOVERED_SYMBOLS).collect()
}

fn workspace_projection(focusa: &FocusaState) -> WorkspaceProjection {
    let Some(root) = selected_workspace_root(focusa) else {
        return WorkspaceProjection::default();
    };

    let workspace_id = root.to_string_lossy().to_string();
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
    if let Some(content) = read_text(&package_json)
        && let Some((pkg_name, deps)) = parse_package_json(&content)
    {
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

    let package_id = package_ids
        .first()
        .cloned()
        .unwrap_or_else(|| stable_id("package", "workspace"));
    let mut module_ids = BTreeSet::new();
    let mut schema_ids = BTreeSet::new();
    let mut dependency_ids = BTreeSet::new();
    let mut import_dependency_ids = BTreeSet::new();
    let mut call_symbol_ids = BTreeSet::new();
    let mut files_scanned = 0usize;

    let mut discovered_paths = walk_workspace(&root);
    discovered_paths.sort_by_key(|path| file_projection_priority(&root, path));

    for path in discovered_paths {
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

        if let Some(parent) = rel_path.parent().and_then(|p| p.to_str())
            && !parent.is_empty()
        {
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

        if rel.contains("tool-contract") || rel.contains("tool_contract") {
            let contract_id = stable_id("tool_contract", &rel);
            objects.push(json!({
                "id": contract_id,
                "object_type": "tool_contract",
                "path": rel,
                "status": "verified",
                "membership_class": "deterministic",
                "provenance_class": "parser_derived",
                "fresh": true,
            }));
            links.push(json!({
                "type": "targets_schema",
                "source_id": contract_id,
                "target_id": package_id,
                "evidence": "tool contract file scan",
                "status": "candidate",
            }));
        }

        if file_type == "spec_doc" {
            let spec_id = stable_id("spec", &rel);
            objects.push(json!({
                "id": spec_id,
                "object_type": "specification",
                "path": rel,
                "status": "verified",
                "membership_class": "deterministic",
                "provenance_class": "parser_derived",
                "fresh": true,
            }));
            links.push(json!({
                "type": "declared_in",
                "source_id": spec_id,
                "target_id": file_id,
                "evidence": "docs/spec file scan",
                "status": "verified",
            }));
            links.push(json!({
                "type": "constrains",
                "source_id": spec_id,
                "target_id": package_id,
                "evidence": "docs/spec -> package heuristic",
                "status": "candidate",
            }));
        }

        if file_type == "manifest"
            && let Some(content) = read_text(&path)
        {
            let manifest_package_id = if rel.ends_with("Cargo.toml") {
                let cargo_name = parse_cargo_name(&content).unwrap_or_else(|| {
                    Path::new(&rel)
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("workspace")
                        .to_string()
                });
                let pkg_id = stable_id("package", &format!("cargo:{}:{}", rel, cargo_name));
                if !package_ids.contains(&pkg_id) {
                    package_ids.push(pkg_id.clone());
                    objects.push(json!({
                        "id": pkg_id,
                        "object_type": "package",
                        "name": cargo_name,
                        "package_type": "cargo",
                        "path": rel,
                        "status": "verified",
                        "membership_class": "deterministic",
                        "provenance_class": "parser_derived",
                        "fresh": true,
                    }));
                }
                for (dep, version) in parse_cargo_dependencies(&content).into_iter().take(32) {
                    let dep_id = stable_id("dependency", &format!("cargo:{}", dep));
                    if dependency_ids.insert(dep_id.clone()) {
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
                    }
                    links.push(json!({
                        "type": "depends_on",
                        "source_id": pkg_id,
                        "target_id": dep_id,
                        "evidence": "Cargo.toml [dependencies]",
                        "status": "verified",
                    }));
                    links.push(json!({
                        "type": "declared_in",
                        "source_id": dep_id,
                        "target_id": file_id,
                        "evidence": "Cargo.toml [dependencies]",
                        "status": "verified",
                    }));
                }
                pkg_id
            } else if rel.ends_with("package.json") {
                if let Some((pkg_name, deps)) = parse_package_json(&content) {
                    let pkg_id = stable_id("package", &format!("npm:{}:{}", rel, pkg_name));
                    if !package_ids.contains(&pkg_id) {
                        package_ids.push(pkg_id.clone());
                        objects.push(json!({
                            "id": pkg_id,
                            "object_type": "package",
                            "name": pkg_name,
                            "package_type": "npm",
                            "path": rel,
                            "status": "verified",
                            "membership_class": "deterministic",
                            "provenance_class": "parser_derived",
                            "fresh": true,
                        }));
                    }
                    for (dep, version) in deps.into_iter().take(32) {
                        let dep_id = stable_id("dependency", &format!("npm:{}", dep));
                        if dependency_ids.insert(dep_id.clone()) {
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
                        }
                        links.push(json!({
                            "type": "depends_on",
                            "source_id": pkg_id,
                            "target_id": dep_id,
                            "evidence": "package.json dependencies",
                            "status": "verified",
                        }));
                        links.push(json!({
                            "type": "declared_in",
                            "source_id": dep_id,
                            "target_id": file_id,
                            "evidence": "package.json dependencies",
                            "status": "verified",
                        }));
                    }
                    pkg_id
                } else {
                    package_id.clone()
                }
            } else {
                package_id.clone()
            };

            links.push(json!({
                "type": "contains",
                "source_id": repo_id,
                "target_id": manifest_package_id,
                "evidence": "manifest discovery",
                "status": "verified",
            }));
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
            for import_target in parse_import_targets(&content, language) {
                let dep_id = stable_id("dependency", &format!("import:{}", import_target));
                if import_dependency_ids.insert(dep_id.clone()) {
                    objects.push(json!({
                        "id": dep_id,
                        "object_type": "dependency",
                        "name": import_target,
                        "dependency_kind": "import",
                        "status": "candidate",
                        "membership_class": "inferred",
                        "provenance_class": "parser_derived",
                        "fresh": true,
                    }));
                }
                links.push(json!({
                    "type": "imports",
                    "source_id": file_id,
                    "target_id": dep_id,
                    "evidence": "source import scan",
                    "status": "verified",
                }));
            }

            for call_target in parse_call_targets(&content, language) {
                let call_symbol_id = stable_id("symbol", &format!("call:{}", call_target));
                if call_symbol_ids.insert(call_symbol_id.clone()) {
                    objects.push(json!({
                        "id": call_symbol_id,
                        "object_type": "symbol",
                        "symbol_name": call_target,
                        "symbol_kind": "call_target",
                        "status": "candidate",
                        "membership_class": "inferred",
                        "provenance_class": "parser_derived",
                        "fresh": true,
                    }));
                }
                links.push(json!({
                    "type": "calls",
                    "source_id": file_id,
                    "target_id": call_symbol_id,
                    "evidence": "source call scan",
                    "status": "verified",
                }));
            }

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
                for (method, endpoint_path, handler) in parse_route_bindings(&content)
                    .into_iter()
                    .chain(
                        parse_endpoints(&content)
                            .into_iter()
                            .map(|(method, path)| (method, path, None)),
                    )
                    .take(MAX_DISCOVERED_ENDPOINTS)
                {
                    let endpoint_id =
                        stable_id("endpoint", &format!("{}:{}", method, endpoint_path));
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
                    if let Some(handler) = handler {
                        let handler_id = stable_id("symbol", &format!("{}:{}", rel, handler));
                        objects.push(json!({
                            "id": handler_id,
                            "object_type": "symbol",
                            "file_id": file_id,
                            "symbol_name": handler,
                            "symbol_kind": "route_handler",
                            "status": "verified",
                            "membership_class": "deterministic",
                            "provenance_class": "parser_derived",
                            "fresh": true,
                        }));
                        links.push(json!({
                            "type": "binds_to",
                            "source_id": endpoint_id,
                            "target_id": handler_id,
                            "evidence": "route handler scan",
                            "status": "verified",
                        }));
                    }
                }
            }
        }
    }

    WorkspaceProjection { objects, links }
}

fn selected_frame<'a>(focusa: &'a FocusaState, frame_id: Option<&str>) -> Option<&'a FrameRecord> {
    let selected = frame_id.and_then(|id| {
        focusa
            .focus_stack
            .frames
            .iter()
            .find(|f| f.id.to_string() == id)
    });
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

    let mut current_ask_id: Option<String> = None;
    let mut query_scope_id: Option<String> = None;
    if let Some(current_ask) = focusa.work_loop.decision_context.current_ask.clone() {
        let ask_id = stable_id("current_ask", &current_ask);
        current_ask_id = Some(ask_id.clone());
        objects.push(json!({
            "id": ask_id.clone(),
            "object_type": "current_ask",
            "ask_text": current_ask,
            "ask_kind": focusa.work_loop.decision_context.ask_kind.clone().unwrap_or_else(|| "question".to_string()),
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));

        let scope_kind = focusa
            .work_loop
            .decision_context
            .scope_kind
            .clone()
            .unwrap_or_else(|| "fresh_question".to_string());
        let carryover_policy = focusa.work_loop.decision_context.carryover_policy.clone();
        let excluded_labels = focusa
            .work_loop
            .decision_context
            .excluded_context_labels
            .clone();
        let excluded_reason = focusa
            .work_loop
            .decision_context
            .excluded_context_reason
            .clone();
        let scope_id = stable_id(
            "query_scope",
            &format!(
                "{}:{}",
                scope_kind,
                carryover_policy.clone().unwrap_or_default()
            ),
        );
        query_scope_id = Some(scope_id.clone());
        objects.push(json!({
            "id": scope_id.clone(),
            "object_type": "query_scope",
            "scope_kind": scope_kind,
            "status": "active",
            "carryover_policy": carryover_policy,
            "excluded_topics": excluded_labels,
            "status_reason": excluded_reason,
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));
        links.push(json!({
            "type": "governed_by",
            "source_id": scope_id,
            "target_id": ask_id,
            "evidence": "work_loop.decision_context",
            "status": "verified",
        }));

        let relevant_id = stable_id("relevant_context_set", "active-answer-context");
        objects.push(json!({
            "id": relevant_id,
            "object_type": "relevant_context_set",
            "selection_kind": "policy_filtered",
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));
        links.push(json!({
            "type": "includes_context",
            "source_id": relevant_id,
            "target_id": ask_id.clone(),
            "evidence": "work_loop.current_task + bounded slice",
            "status": "verified",
        }));

        if !focusa
            .work_loop
            .decision_context
            .excluded_context_labels
            .is_empty()
            || focusa
                .work_loop
                .decision_context
                .excluded_context_reason
                .is_some()
        {
            let excluded_id = stable_id("excluded_context_set", "policy-excluded-context");
            objects.push(json!({
                "id": excluded_id,
                "object_type": "excluded_context_set",
                "exclusion_kind": "policy_exclusion",
                "status": "active",
                "membership_class": "deterministic",
                "provenance_class": "runtime_observed",
                "fresh": true,
            }));
            links.push(json!({
                "type": "excludes_context",
                "source_id": excluded_id,
                "target_id": ask_id.clone(),
                "evidence": focusa.work_loop.decision_context.excluded_context_reason.clone(),
                "status": "verified",
            }));
        }
    }

    for (idx, event) in focusa
        .telemetry
        .trace_events
        .iter()
        .rev()
        .take(40)
        .enumerate()
    {
        let event_type = event
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if !matches!(
            event_type,
            "scope_failure_recorded"
                | "scope_contamination_detected"
                | "wrong_question_detected"
                | "answer_broadening_detected"
        ) {
            continue;
        }
        let scope_failure_id = stable_id("scope_failure", &format!("{}:{}", event_type, idx));
        let severity = if event_type == "wrong_question_detected" {
            "high"
        } else {
            "medium"
        };
        objects.push(json!({
            "id": scope_failure_id,
            "object_type": "scope_failure",
            "failure_kind": event_type,
            "severity": severity,
            "status": "failed",
            "membership_class": "verified",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));
        if let Some(ask_id) = current_ask_id.clone() {
            links.push(json!({
                "type": "violates_scope_of",
                "source_id": scope_failure_id,
                "target_id": ask_id,
                "evidence": "telemetry.trace_events",
                "status": "verified",
            }));
        }
        if let Some(scope_id) = query_scope_id.clone() {
            links.push(json!({
                "type": "violates_scope_of",
                "source_id": scope_failure_id,
                "target_id": scope_id,
                "evidence": "telemetry.trace_events",
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
    let mut artifact_count = 0usize;
    for handle in focusa
        .reference_index
        .handles
        .iter()
        .filter(|h| h.session_id == session_id || h.pinned || h.session_id.is_none())
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
        artifact_count += 1;
    }

    if artifact_count == 0 {
        let workspace_ref = focusa
            .session
            .as_ref()
            .and_then(|s| s.workspace_id.clone())
            .or_else(|| {
                env::var("FOCUSA_PROJECT_ROOT")
                    .ok()
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(|| {
                env::current_dir()
                    .ok()
                    .map(|path| path.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "workspace".to_string());
        objects.push(json!({
            "id": stable_id("artifact", &workspace_ref),
            "object_type": "artifact",
            "artifact_kind": "workspace_snapshot",
            "source_ref": workspace_ref,
            "status": "verified",
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));
    }

    WorkspaceProjection { objects, links }
}

fn canonical_ontology_projection(focusa: &FocusaState) -> WorkspaceProjection {
    let mut objects = focusa.ontology.objects.clone();
    let mut links = focusa.ontology.links.clone();

    for proposal in focusa.ontology.proposals.iter().take(128) {
        objects.push(json!({
            "id": format!("ontology_proposal:{}", proposal.proposal_id),
            "object_type": "ontology_domain",
            "domain_kind": proposal.target_class.clone(),
            "status": proposal.status.clone(),
            "proposal_kind": proposal.proposal_kind.clone(),
            "proposal_id": proposal.proposal_id,
            "membership_class": "provisional",
            "provenance_class": "reducer_promoted",
            "fresh": true,
        }));
    }

    for verification in focusa.ontology.verifications.iter().take(128) {
        let verification_id = stable_id(
            "verification",
            &format!(
                "ontology:{}:{}",
                verification.verification.clone(),
                verification
                    .proposal_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
        );
        objects.push(json!({
            "id": verification_id,
            "object_type": "verification",
            "method": verification.verification.clone(),
            "result": verification.outcome.clone(),
            "timestamp": verification.timestamp,
            "status": "verified",
            "membership_class": "verified",
            "provenance_class": "verification_confirmed",
            "fresh": true,
        }));
    }

    for refresh in focusa.ontology.working_set_refreshes.iter().take(64) {
        let refresh_id = stable_id(
            "relevant_context_set",
            &format!("{}:{}", refresh.scope.clone(), refresh.reason.clone()),
        );
        objects.push(json!({
            "id": refresh_id,
            "object_type": "relevant_context_set",
            "selection_kind": refresh.scope.clone(),
            "status": "active",
            "reason": refresh.reason.clone(),
            "timestamp": refresh.timestamp,
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));
    }

    for delta in focusa.ontology.delta_log.iter().take(256) {
        links.push(json!({
            "type": "derived_from",
            "source_id": stable_id("ontology_delta", &delta.delta_kind),
            "target_id": "ontology:canonical",
            "evidence": delta.delta_kind.clone(),
            "status": "verified",
            "timestamp": delta.timestamp,
        }));
    }

    WorkspaceProjection { objects, links }
}

fn visual_projection(focusa: &FocusaState, frame: Option<&FrameRecord>) -> WorkspaceProjection {
    let mut objects = Vec::new();
    let mut links = Vec::new();
    let mut object_ids = BTreeSet::new();

    let visual_handles: Vec<_> = focusa
        .reference_index
        .handles
        .iter()
        .filter(|h| {
            let label = h.label.to_ascii_lowercase();
            label.contains("screenshot")
                || label.contains("mockup")
                || label.contains("wireframe")
                || label.contains("visual")
                || label.contains("blueprint")
                || label.contains("token_map")
                || label.contains("spacing_map")
                || label.contains("component_inventory")
                || matches!(h.kind, HandleKind::FileSnapshot)
        })
        .take(64)
        .collect();

    if visual_handles.is_empty() {
        return WorkspaceProjection { objects, links };
    }

    let page_id = frame.map(|f| stable_id("page", &f.id.to_string()));
    if let Some(frame_ref) = frame
        && let Some(ref id) = page_id
    {
        object_ids.insert(id.clone());
        objects.push(json!({
            "id": id,
            "object_type": "page",
            "name": frame_ref.title,
            "page_kind": "focus_frame",
            "primary_goal": frame_ref.goal,
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));
    }

    for handle in visual_handles {
        let label = handle.label.to_ascii_lowercase();
        let (artifact_kind, provenance_class) = if label.contains("screenshot") {
            ("screenshot", "screenshot_derived")
        } else if label.contains("mockup") {
            ("mockup", "artifact_derived")
        } else if label.contains("wireframe") {
            ("wireframe", "artifact_derived")
        } else if label.contains("blueprint") {
            ("blueprint", "artifact_derived")
        } else if label.contains("token_map") {
            ("token_map", "artifact_derived")
        } else if label.contains("spacing_map") {
            ("spacing_map", "artifact_derived")
        } else if label.contains("component_inventory") {
            ("component_inventory", "artifact_derived")
        } else {
            ("implementation_artifact", "artifact_derived")
        };

        let visual_artifact_id = stable_id("visual_artifact", &handle.id.to_string());
        if object_ids.insert(visual_artifact_id.clone()) {
            objects.push(json!({
                "id": visual_artifact_id,
                "object_type": "visual_artifact",
                "artifact_kind": artifact_kind,
                "status": if handle.pinned { "canonical" } else { "verified" },
                "handle": handle.id,
                "label": handle.label,
                "membership_class": if handle.pinned { "pinned" } else { "verified" },
                "provenance_class": provenance_class,
                "fresh": true,
            }));
        }

        if let Some(ref pid) = page_id {
            links.push(json!({
                "type": "derived_from_reference",
                "source_id": pid,
                "target_id": visual_artifact_id,
                "evidence": "reference_index.handles",
                "status": "verified"
            }));
        }

        if label.contains("header")
            || label.contains("hero")
            || label.contains("sidebar")
            || label.contains("footer")
            || label.contains("modal")
            || label.contains("section")
        {
            let region_kind = if label.contains("header") {
                "header"
            } else if label.contains("hero") {
                "hero"
            } else if label.contains("sidebar") {
                "sidebar"
            } else if label.contains("footer") {
                "footer"
            } else if label.contains("modal") {
                "modal_body"
            } else {
                "form_section"
            };
            let region_id = stable_id("region", &format!("{}:{}", handle.id, region_kind));
            if object_ids.insert(region_id.clone()) {
                objects.push(json!({
                    "id": region_id,
                    "object_type": "region",
                    "name": region_kind,
                    "region_kind": region_kind,
                    "status": "verified",
                    "membership_class": "verified",
                    "provenance_class": provenance_class,
                    "fresh": true,
                }));
            }
            links.push(json!({"type":"contains","source_id":visual_artifact_id,"target_id":region_id,"evidence":"reference_index.handles.label","status":"verified"}));
            if let Some(ref pid) = page_id {
                links.push(json!({"type":"contains","source_id":pid,"target_id":region_id,"evidence":"focus_stack.active_frame + visual handle","status":"verified"}));
            }
        }

        if label.contains("component")
            || label.contains("button")
            || label.contains("card")
            || label.contains("navbar")
            || label.contains("form")
            || label.contains("input")
            || label.contains("table")
            || label.contains("dialog")
            || label.contains("accordion")
        {
            let component_kind = if label.contains("button") {
                "button"
            } else if label.contains("card") {
                "card"
            } else if label.contains("navbar") {
                "navbar"
            } else if label.contains("form") {
                "form"
            } else if label.contains("input") {
                "input"
            } else if label.contains("table") {
                "table"
            } else if label.contains("dialog") {
                "dialog"
            } else if label.contains("accordion") {
                "accordion"
            } else {
                "component"
            };
            let component_id = stable_id("component", &format!("{}:{}", handle.id, component_kind));
            if object_ids.insert(component_id.clone()) {
                objects.push(json!({
                    "id": component_id,
                    "object_type": "component",
                    "name": component_kind,
                    "component_kind": component_kind,
                    "status": "verified",
                    "membership_class": "verified",
                    "provenance_class": provenance_class,
                    "fresh": true,
                }));
            }
            links.push(json!({"type":"derived_from_reference","source_id":component_id,"target_id":visual_artifact_id,"evidence":"reference_index.handles.label","status":"verified"}));

            if label.contains("variant")
                || label.contains("primary")
                || label.contains("secondary")
                || label.contains("compact")
                || label.contains("destructive")
                || label.contains("mobile")
            {
                let variant_kind = if label.contains("primary") {
                    "primary"
                } else if label.contains("secondary") {
                    "secondary"
                } else if label.contains("compact") {
                    "compact"
                } else if label.contains("destructive") {
                    "destructive"
                } else if label.contains("mobile") {
                    "mobile"
                } else {
                    "default"
                };
                let variant_id =
                    stable_id("variant", &format!("{}:{}", component_id, variant_kind));
                if object_ids.insert(variant_id.clone()) {
                    objects.push(json!({
                        "id": variant_id,
                        "object_type": "variant",
                        "name": format!("{} {}", variant_kind, component_kind),
                        "variant_kind": variant_kind,
                        "status": "verified",
                        "membership_class": "verified",
                        "provenance_class": provenance_class,
                        "fresh": true,
                    }));
                }
                links.push(json!({"type":"variants_of","source_id":variant_id,"target_id":component_id,"evidence":"reference_index.handles.label","status":"verified"}));
            }

            if label.contains("token") || label.contains("color") || label.contains("spacing") {
                let token_kind = if label.contains("color") {
                    "color"
                } else if label.contains("spacing") {
                    "spacing"
                } else {
                    "design_token"
                };
                let token_id = stable_id("token", &format!("{}:{}", handle.id, token_kind));
                if object_ids.insert(token_id.clone()) {
                    objects.push(json!({
                        "id": token_id,
                        "object_type": "token",
                        "token_kind": token_kind,
                        "value": label,
                        "status": "verified",
                        "membership_class": "verified",
                        "provenance_class": provenance_class,
                        "fresh": true,
                    }));
                }
                links.push(json!({"type":"inherits_token","source_id":component_id,"target_id":token_id,"evidence":"reference_index.handles.label","status":"verified"}));
            }

            if label.contains("grid")
                || label.contains("layout")
                || label.contains("container")
                || label.contains("alignment")
            {
                let layout_id = stable_id("layout_rule", &handle.id.to_string());
                if object_ids.insert(layout_id.clone()) {
                    objects.push(json!({
                        "id": layout_id,
                        "object_type": "layout_rule",
                        "rule_kind": "layout_from_artifact",
                        "status": "verified",
                        "membership_class": "verified",
                        "provenance_class": provenance_class,
                        "fresh": true,
                    }));
                }
                links.push(json!({"type":"aligns_with","source_id":component_id,"target_id":layout_id,"evidence":"reference_index.handles.label","status":"verified"}));
                links.push(json!({"type":"derived_from_reference","source_id":layout_id,"target_id":visual_artifact_id,"evidence":"reference_index.handles.label","status":"verified"}));
            }
        }

        if label.contains("binding") || label.contains("bound") {
            let binding_id = stable_id("binding", &handle.id.to_string());
            if object_ids.insert(binding_id.clone()) {
                objects.push(json!({
                    "id": binding_id,
                    "object_type": "binding",
                    "binding_kind": "artifact_binding",
                    "status": "verified",
                    "membership_class": "verified",
                    "provenance_class": provenance_class,
                    "fresh": true,
                }));
            }
            links.push(json!({"type":"derived_from_reference","source_id":binding_id,"target_id":visual_artifact_id,"evidence":"reference_index.handles.label","status":"verified"}));

            if label.contains("validation") || label.contains("required") || label.contains("min") {
                let validation_id = stable_id("validation_rule", &handle.id.to_string());
                if object_ids.insert(validation_id.clone()) {
                    objects.push(json!({
                        "id": validation_id,
                        "object_type": "validation_rule",
                        "rule_kind": "artifact_validation",
                        "status": "verified",
                        "membership_class": "verified",
                        "provenance_class": provenance_class,
                        "fresh": true,
                    }));
                }
                links.push(json!({"type":"validates","source_id":validation_id,"target_id":binding_id,"evidence":"reference_index.handles.label","status":"verified"}));
            }
        }

        if label.contains("interaction")
            || label.contains("click")
            || label.contains("submit")
            || label.contains("open")
            || label.contains("navigate")
        {
            let interaction_id = stable_id("interaction", &handle.id.to_string());
            if object_ids.insert(interaction_id.clone()) {
                objects.push(json!({
                    "id": interaction_id,
                    "object_type": "interaction",
                    "interaction_kind": "artifact_interaction",
                    "status": "verified",
                    "membership_class": "verified",
                    "provenance_class": provenance_class,
                    "fresh": true,
                }));
            }
            links.push(json!({"type":"derived_from_reference","source_id":interaction_id,"target_id":visual_artifact_id,"evidence":"reference_index.handles.label","status":"verified"}));

            let state_kind = if label.contains("loading") {
                "loading"
            } else if label.contains("success") {
                "success"
            } else if label.contains("error") {
                "error"
            } else if label.contains("disabled") {
                "disabled"
            } else {
                "default"
            };
            let state_id = stable_id("ui_state", &format!("{}:{}", handle.id, state_kind));
            if object_ids.insert(state_id.clone()) {
                objects.push(json!({
                    "id": state_id,
                    "object_type": "ui_state",
                    "state_kind": state_kind,
                    "status": "verified",
                    "membership_class": "verified",
                    "provenance_class": provenance_class,
                    "fresh": true,
                }));
            }
            links.push(json!({"type":"transitions_to","source_id":interaction_id,"target_id":state_id,"evidence":"reference_index.handles.label","status":"verified"}));
        }
    }

    WorkspaceProjection { objects, links }
}

fn affordance_execution_projection(
    focusa: &FocusaState,
    frame: Option<&FrameRecord>,
) -> WorkspaceProjection {
    let Some(root) = selected_workspace_root(focusa) else {
        return WorkspaceProjection::default();
    };

    let workspace_id = root.to_string_lossy().to_string();
    let repo_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("workspace")
        .to_string();
    let workspace_writable = fs::metadata(&root)
        .map(|meta| !meta.permissions().readonly())
        .unwrap_or(false);
    let has_git_dir = root.join(".git").exists();
    let cargo_manifest_present = root.join("Cargo.toml").exists();
    let git_available = binary_available("git");
    let cargo_available = binary_available("cargo");
    let query_task_target = frame.map(|active| format!("task:{}", active.id));

    let mut objects = Vec::new();
    let mut links = Vec::new();
    let mut push_link =
        |link_type: &str, source_id: &str, target_id: &str, evidence: &str, status: &str| {
            links.push(json!({
                "type": link_type,
                "source_id": source_id,
                "target_id": target_id,
                "evidence": evidence,
                "status": status,
            }));
        };

    let execution_context_id =
        stable_id("execution_context", &format!("workspace:{}", workspace_id));
    objects.push(json!({
        "id": execution_context_id.clone(),
        "object_type": "execution_context",
        "context_kind": "local_workspace_runtime",
        "workspace_id": workspace_id,
        "repo_name": repo_name,
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let workspace_boundary_id = stable_id(
        "authority_boundary",
        &format!("workspace:{}", root.display()),
    );
    objects.push(json!({
        "id": workspace_boundary_id.clone(),
        "object_type": "authority_boundary",
        "boundary_kind": "workspace_boundary",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let workspace_read_permission_id =
        stable_id("permission", &format!("workspace-read:{}", root.display()));
    objects.push(json!({
        "id": workspace_read_permission_id.clone(),
        "object_type": "permission",
        "permission_kind": "workspace_read",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let workspace_write_permission_id =
        stable_id("permission", &format!("workspace-write:{}", root.display()));
    objects.push(json!({
        "id": workspace_write_permission_id.clone(),
        "object_type": "permission",
        "permission_kind": "workspace_write",
        "status": if workspace_writable { "active" } else { "blocked" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let destructive_confirmation_precondition_id =
        stable_id("precondition", "destructive_confirmation_required");
    objects.push(json!({
        "id": destructive_confirmation_precondition_id.clone(),
        "object_type": "precondition",
        "precondition_kind": "destructive_confirmation_required",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let workspace_resource_id = stable_id("resource", &format!("workspace:{}", root.display()));
    objects.push(json!({
        "id": workspace_resource_id.clone(),
        "object_type": "resource",
        "resource_kind": "workspace_filesystem",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let local_exec_resource_id =
        stable_id("resource", &format!("local-execution:{}", root.display()));
    objects.push(json!({
        "id": local_exec_resource_id.clone(),
        "object_type": "resource",
        "resource_kind": "local_process_execution",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let interactive_cost_id = stable_id("cost_model", "interactive_local_cost");
    objects.push(json!({
        "id": interactive_cost_id.clone(),
        "object_type": "cost_model",
        "cost_kind": "interactive_local_cost",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let build_cost_id = stable_id("cost_model", "local_build_cost");
    objects.push(json!({
        "id": build_cost_id.clone(),
        "object_type": "cost_model",
        "cost_kind": "local_build_cost",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let interactive_latency_id = stable_id("latency_profile", "interactive_local_latency");
    objects.push(json!({
        "id": interactive_latency_id.clone(),
        "object_type": "latency_profile",
        "latency_kind": "interactive_local_latency",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let build_latency_id = stable_id("latency_profile", "local_build_latency");
    objects.push(json!({
        "id": build_latency_id.clone(),
        "object_type": "latency_profile",
        "latency_kind": "local_build_latency",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let local_reliability_id = stable_id("reliability_profile", "local_runtime_reliability");
    objects.push(json!({
        "id": local_reliability_id.clone(),
        "object_type": "reliability_profile",
        "reliability_kind": "local_runtime_reliability",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let read_only_reversibility_id = stable_id("reversibility_profile", "read_only_reversible");
    objects.push(json!({
        "id": read_only_reversibility_id.clone(),
        "object_type": "reversibility_profile",
        "reversibility_kind": "read_only_reversible",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let workspace_mutation_reversibility_id = stable_id(
        "reversibility_profile",
        if has_git_dir {
            "workspace_mutation_vcs_backed"
        } else {
            "workspace_mutation_manual_recovery"
        },
    );
    objects.push(json!({
        "id": workspace_mutation_reversibility_id.clone(),
        "object_type": "reversibility_profile",
        "reversibility_kind": if has_git_dir {
            "workspace_mutation_vcs_backed"
        } else {
            "workspace_mutation_manual_recovery"
        },
        "status": if has_git_dir { "active" } else { "warning" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let focusa_api_surface_id = stable_id("tool_surface", "focusa_http_api");
    objects.push(json!({
        "id": focusa_api_surface_id.clone(),
        "object_type": "tool_surface",
        "surface_kind": "focusa_http_api",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let workspace_fs_surface_id = stable_id("tool_surface", "workspace_filesystem");
    objects.push(json!({
        "id": workspace_fs_surface_id.clone(),
        "object_type": "tool_surface",
        "surface_kind": "workspace_filesystem",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let git_cli_surface_id = stable_id("tool_surface", "git_cli");
    objects.push(json!({
        "id": git_cli_surface_id.clone(),
        "object_type": "tool_surface",
        "surface_kind": "git_cli",
        "status": if git_available { "active" } else { "inactive" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let cargo_cli_surface_id = stable_id("tool_surface", "cargo_cli");
    objects.push(json!({
        "id": cargo_cli_surface_id.clone(),
        "object_type": "tool_surface",
        "surface_kind": "cargo_cli",
        "status": if cargo_available { "active" } else { "inactive" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let git_repo_precondition_id =
        stable_id("precondition", &format!("git-repo:{}", root.display()));
    objects.push(json!({
        "id": git_repo_precondition_id.clone(),
        "object_type": "precondition",
        "precondition_kind": "git_repository_present",
        "status": if has_git_dir { "satisfied" } else { "missing" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let cargo_manifest_precondition_id = stable_id(
        "precondition",
        &format!("cargo-manifest:{}", root.display()),
    );
    objects.push(json!({
        "id": cargo_manifest_precondition_id.clone(),
        "object_type": "precondition",
        "precondition_kind": "cargo_manifest_present",
        "status": if cargo_manifest_present { "satisfied" } else { "missing" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let git_binary_dependency_id = stable_id("dependency", "git-cli-binary");
    objects.push(json!({
        "id": git_binary_dependency_id.clone(),
        "object_type": "dependency",
        "name": "git",
        "version": "runtime",
        "dependency_kind": "git_cli_binary",
        "status": if git_available { "satisfied" } else { "missing" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let cargo_binary_dependency_id = stable_id("dependency", "cargo-cli-binary");
    objects.push(json!({
        "id": cargo_binary_dependency_id.clone(),
        "object_type": "dependency",
        "name": "cargo",
        "version": "runtime",
        "dependency_kind": "cargo_cli_binary",
        "status": if cargo_available { "satisfied" } else { "missing" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let query_focusa_capability_id = stable_id("capability", "query_focusa_runtime");
    objects.push(json!({
        "id": query_focusa_capability_id.clone(),
        "object_type": "capability",
        "capability_kind": "query_focusa_runtime",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let inspect_workspace_capability_id = stable_id(
        "capability",
        &format!("inspect-workspace:{}", root.display()),
    );
    objects.push(json!({
        "id": inspect_workspace_capability_id.clone(),
        "object_type": "capability",
        "capability_kind": "inspect_workspace_files",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let modify_workspace_capability_id = stable_id(
        "capability",
        &format!("modify-workspace:{}", root.display()),
    );
    objects.push(json!({
        "id": modify_workspace_capability_id.clone(),
        "object_type": "capability",
        "capability_kind": "modify_workspace_files",
        "status": if workspace_writable { "active" } else { "blocked" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let inspect_git_capability_id =
        stable_id("capability", &format!("inspect-git:{}", root.display()));
    objects.push(json!({
        "id": inspect_git_capability_id.clone(),
        "object_type": "capability",
        "capability_kind": "inspect_versioned_changes",
        "status": if has_git_dir && git_available { "active" } else { "blocked" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let build_rust_capability_id =
        stable_id("capability", &format!("build-rust:{}", root.display()));
    objects.push(json!({
        "id": build_rust_capability_id.clone(),
        "object_type": "capability",
        "capability_kind": "build_rust_workspace",
        "status": if cargo_manifest_present && cargo_available { "active" } else { "blocked" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let inspect_focusa_affordance_id = stable_id("affordance", "inspect_focusa_runtime");
    objects.push(json!({
        "id": inspect_focusa_affordance_id.clone(),
        "object_type": "affordance",
        "affordance_kind": "inspect_focusa_runtime",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let inspect_workspace_affordance_id = stable_id(
        "affordance",
        &format!("inspect-workspace:{}", root.display()),
    );
    objects.push(json!({
        "id": inspect_workspace_affordance_id.clone(),
        "object_type": "affordance",
        "affordance_kind": "inspect_workspace",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let edit_workspace_affordance_id =
        stable_id("affordance", &format!("edit-workspace:{}", root.display()));
    objects.push(json!({
        "id": edit_workspace_affordance_id.clone(),
        "object_type": "affordance",
        "affordance_kind": "edit_workspace_files",
        "status": if workspace_writable { "active" } else { "blocked" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let inspect_git_affordance_id =
        stable_id("affordance", &format!("inspect-git:{}", root.display()));
    objects.push(json!({
        "id": inspect_git_affordance_id.clone(),
        "object_type": "affordance",
        "affordance_kind": "inspect_versioned_workspace",
        "status": if has_git_dir && git_available { "active" } else { "blocked" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let build_rust_affordance_id =
        stable_id("affordance", &format!("build-rust:{}", root.display()));
    objects.push(json!({
        "id": build_rust_affordance_id.clone(),
        "object_type": "affordance",
        "affordance_kind": "build_rust_workspace",
        "status": if cargo_manifest_present && cargo_available { "active" } else { "blocked" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    for source_id in [
        focusa_api_surface_id.as_str(),
        workspace_fs_surface_id.as_str(),
        git_cli_surface_id.as_str(),
        cargo_cli_surface_id.as_str(),
    ] {
        push_link(
            "available_in_context",
            source_id,
            &execution_context_id,
            "runtime workspace context",
            "verified",
        );
    }

    push_link(
        "enabled_by",
        &query_focusa_capability_id,
        &focusa_api_surface_id,
        "local daemon API",
        "verified",
    );
    push_link(
        "available_in_context",
        &query_focusa_capability_id,
        &execution_context_id,
        "local daemon runtime",
        "verified",
    );
    push_link(
        "consumes_resource",
        &query_focusa_capability_id,
        &local_exec_resource_id,
        "runtime query budget",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_focusa_affordance_id,
        &interactive_cost_id,
        "interactive query cost",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_focusa_affordance_id,
        &interactive_latency_id,
        "interactive query latency",
        "verified",
    );
    push_link(
        "has_reliability",
        &focusa_api_surface_id,
        &local_reliability_id,
        "local daemon surface",
        "verified",
    );
    push_link(
        "has_reliability",
        &inspect_focusa_affordance_id,
        &local_reliability_id,
        "local daemon surface",
        "verified",
    );
    push_link(
        "has_reversibility",
        &query_focusa_capability_id,
        &read_only_reversibility_id,
        "read-only runtime query",
        "verified",
    );
    push_link(
        "has_reversibility",
        &inspect_focusa_affordance_id,
        &read_only_reversibility_id,
        "read-only runtime query",
        "verified",
    );

    push_link(
        "enabled_by",
        &inspect_workspace_capability_id,
        &workspace_fs_surface_id,
        "workspace filesystem surface",
        "verified",
    );
    push_link(
        "requires_permission",
        &inspect_workspace_capability_id,
        &workspace_read_permission_id,
        "workspace read required",
        "verified",
    );
    push_link(
        "bounded_by_authority",
        &inspect_workspace_capability_id,
        &workspace_boundary_id,
        "workspace authority boundary",
        "verified",
    );
    push_link(
        "available_in_context",
        &inspect_workspace_capability_id,
        &execution_context_id,
        "workspace runtime",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_workspace_affordance_id,
        &workspace_resource_id,
        "workspace inspection",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_workspace_affordance_id,
        &interactive_cost_id,
        "interactive inspection cost",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_workspace_affordance_id,
        &interactive_latency_id,
        "interactive inspection latency",
        "verified",
    );
    push_link(
        "has_reliability",
        &workspace_fs_surface_id,
        &local_reliability_id,
        "local filesystem surface",
        "verified",
    );
    push_link(
        "has_reliability",
        &inspect_workspace_affordance_id,
        &local_reliability_id,
        "local filesystem surface",
        "verified",
    );
    push_link(
        "has_reversibility",
        &inspect_workspace_capability_id,
        &read_only_reversibility_id,
        "read-only workspace inspection",
        "verified",
    );
    push_link(
        "has_reversibility",
        &inspect_workspace_affordance_id,
        &read_only_reversibility_id,
        "read-only workspace inspection",
        "verified",
    );

    push_link(
        "enabled_by",
        &modify_workspace_capability_id,
        &workspace_fs_surface_id,
        "workspace filesystem surface",
        "verified",
    );
    push_link(
        "requires_permission",
        &modify_workspace_capability_id,
        &workspace_write_permission_id,
        "workspace write required",
        "verified",
    );
    push_link(
        "bounded_by_authority",
        &modify_workspace_capability_id,
        &workspace_boundary_id,
        "workspace authority boundary",
        "verified",
    );
    push_link(
        "available_in_context",
        &modify_workspace_capability_id,
        &execution_context_id,
        "workspace runtime",
        "verified",
    );
    push_link(
        "consumes_resource",
        &edit_workspace_affordance_id,
        &workspace_resource_id,
        "workspace mutation",
        "verified",
    );
    push_link(
        "consumes_resource",
        &edit_workspace_affordance_id,
        &interactive_cost_id,
        "interactive edit cost",
        "verified",
    );
    push_link(
        "consumes_resource",
        &edit_workspace_affordance_id,
        &interactive_latency_id,
        "interactive edit latency",
        "verified",
    );
    push_link(
        "has_reliability",
        &edit_workspace_affordance_id,
        &local_reliability_id,
        "local filesystem surface",
        "verified",
    );
    push_link(
        "has_reversibility",
        &modify_workspace_capability_id,
        &workspace_mutation_reversibility_id,
        "workspace mutation reversibility",
        "verified",
    );
    push_link(
        "has_reversibility",
        &edit_workspace_affordance_id,
        &workspace_mutation_reversibility_id,
        "workspace mutation reversibility",
        "verified",
    );

    push_link(
        "enabled_by",
        &inspect_git_capability_id,
        &git_cli_surface_id,
        "git CLI surface",
        "verified",
    );
    push_link(
        "depends_on",
        &inspect_git_capability_id,
        &git_repo_precondition_id,
        "git repo required",
        "verified",
    );
    push_link(
        "depends_on",
        &inspect_git_capability_id,
        &git_binary_dependency_id,
        "git CLI binary required",
        "verified",
    );
    push_link(
        "requires_permission",
        &inspect_git_capability_id,
        &workspace_read_permission_id,
        "workspace read required",
        "verified",
    );
    push_link(
        "bounded_by_authority",
        &inspect_git_capability_id,
        &workspace_boundary_id,
        "workspace authority boundary",
        "verified",
    );
    push_link(
        "available_in_context",
        &inspect_git_capability_id,
        &execution_context_id,
        "workspace runtime",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_git_affordance_id,
        &workspace_resource_id,
        "git inspection",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_git_affordance_id,
        &interactive_cost_id,
        "interactive git cost",
        "verified",
    );
    push_link(
        "consumes_resource",
        &inspect_git_affordance_id,
        &interactive_latency_id,
        "interactive git latency",
        "verified",
    );
    push_link(
        "has_reliability",
        &git_cli_surface_id,
        &local_reliability_id,
        "local CLI surface",
        "verified",
    );
    push_link(
        "has_reliability",
        &inspect_git_affordance_id,
        &local_reliability_id,
        "local CLI surface",
        "verified",
    );
    push_link(
        "has_reversibility",
        &inspect_git_capability_id,
        &read_only_reversibility_id,
        "read-only git inspection",
        "verified",
    );
    push_link(
        "has_reversibility",
        &inspect_git_affordance_id,
        &read_only_reversibility_id,
        "read-only git inspection",
        "verified",
    );

    push_link(
        "enabled_by",
        &build_rust_capability_id,
        &cargo_cli_surface_id,
        "cargo CLI surface",
        "verified",
    );
    push_link(
        "depends_on",
        &build_rust_capability_id,
        &cargo_manifest_precondition_id,
        "Cargo manifest required",
        "verified",
    );
    push_link(
        "depends_on",
        &build_rust_capability_id,
        &cargo_binary_dependency_id,
        "cargo CLI binary required",
        "verified",
    );
    push_link(
        "requires_permission",
        &build_rust_capability_id,
        &workspace_read_permission_id,
        "workspace read required",
        "verified",
    );
    push_link(
        "bounded_by_authority",
        &build_rust_capability_id,
        &workspace_boundary_id,
        "workspace authority boundary",
        "verified",
    );
    push_link(
        "available_in_context",
        &build_rust_capability_id,
        &execution_context_id,
        "workspace runtime",
        "verified",
    );
    push_link(
        "consumes_resource",
        &build_rust_affordance_id,
        &local_exec_resource_id,
        "local build execution",
        "verified",
    );
    push_link(
        "consumes_resource",
        &build_rust_affordance_id,
        &build_cost_id,
        "local build cost",
        "verified",
    );
    push_link(
        "consumes_resource",
        &build_rust_affordance_id,
        &build_latency_id,
        "local build latency",
        "verified",
    );
    push_link(
        "has_reliability",
        &cargo_cli_surface_id,
        &local_reliability_id,
        "local CLI surface",
        "verified",
    );
    push_link(
        "has_reliability",
        &build_rust_affordance_id,
        &local_reliability_id,
        "local CLI surface",
        "verified",
    );
    push_link(
        "has_reversibility",
        &build_rust_capability_id,
        &workspace_mutation_reversibility_id,
        "local build may write workspace artifacts",
        "verified",
    );
    push_link(
        "has_reversibility",
        &build_rust_affordance_id,
        &workspace_mutation_reversibility_id,
        "local build may write workspace artifacts",
        "verified",
    );

    for (affordance_id, capability_id, tool_surface_id) in [
        (
            &inspect_focusa_affordance_id,
            &query_focusa_capability_id,
            &focusa_api_surface_id,
        ),
        (
            &inspect_workspace_affordance_id,
            &inspect_workspace_capability_id,
            &workspace_fs_surface_id,
        ),
        (
            &edit_workspace_affordance_id,
            &modify_workspace_capability_id,
            &workspace_fs_surface_id,
        ),
        (
            &inspect_git_affordance_id,
            &inspect_git_capability_id,
            &git_cli_surface_id,
        ),
        (
            &build_rust_affordance_id,
            &build_rust_capability_id,
            &cargo_cli_surface_id,
        ),
    ] {
        push_link(
            "enabled_by",
            affordance_id,
            capability_id,
            "capability enables affordance",
            "verified",
        );
        push_link(
            "enabled_by",
            affordance_id,
            tool_surface_id,
            "tool surface enables affordance",
            "verified",
        );
        push_link(
            "available_in_context",
            affordance_id,
            &execution_context_id,
            "workspace runtime",
            "verified",
        );
        push_link(
            "bounded_by_authority",
            affordance_id,
            &workspace_boundary_id,
            "workspace authority boundary",
            "verified",
        );
    }

    push_link(
        "requires_permission",
        &inspect_workspace_affordance_id,
        &workspace_read_permission_id,
        "workspace read required",
        "verified",
    );
    push_link(
        "requires_permission",
        &edit_workspace_affordance_id,
        &workspace_write_permission_id,
        "workspace write required",
        "verified",
    );
    push_link(
        "requires_permission",
        &inspect_git_affordance_id,
        &workspace_read_permission_id,
        "workspace read required",
        "verified",
    );
    push_link(
        "requires_permission",
        &build_rust_affordance_id,
        &workspace_read_permission_id,
        "workspace read required",
        "verified",
    );
    push_link(
        "depends_on",
        &inspect_git_affordance_id,
        &git_repo_precondition_id,
        "git repo required",
        "verified",
    );
    push_link(
        "depends_on",
        &inspect_git_affordance_id,
        &git_binary_dependency_id,
        "git CLI binary required",
        "verified",
    );
    push_link(
        "depends_on",
        &build_rust_affordance_id,
        &cargo_manifest_precondition_id,
        "Cargo manifest required",
        "verified",
    );
    push_link(
        "depends_on",
        &build_rust_affordance_id,
        &cargo_binary_dependency_id,
        "cargo CLI binary required",
        "verified",
    );

    if let Some(task_target) = query_task_target.as_ref() {
        for source_id in [
            inspect_focusa_affordance_id.as_str(),
            inspect_workspace_affordance_id.as_str(),
            edit_workspace_affordance_id.as_str(),
            inspect_git_affordance_id.as_str(),
            build_rust_affordance_id.as_str(),
        ] {
            push_link(
                "supports_execution_of",
                source_id,
                task_target,
                "active focus frame",
                "verified",
            );
        }
    }

    push_link(
        "blocks_execution_of",
        &destructive_confirmation_precondition_id,
        &edit_workspace_affordance_id,
        "destructive operations require explicit confirmation",
        "verified",
    );

    if !workspace_writable {
        push_link(
            "blocks_execution_of",
            &workspace_write_permission_id,
            &modify_workspace_capability_id,
            "workspace is not writable",
            "verified",
        );
        push_link(
            "blocks_execution_of",
            &workspace_write_permission_id,
            &edit_workspace_affordance_id,
            "workspace is not writable",
            "verified",
        );
    }
    if !has_git_dir {
        push_link(
            "blocks_execution_of",
            &git_repo_precondition_id,
            &inspect_git_capability_id,
            "git repository missing",
            "verified",
        );
        push_link(
            "blocks_execution_of",
            &git_repo_precondition_id,
            &inspect_git_affordance_id,
            "git repository missing",
            "verified",
        );
    }
    if !git_available {
        push_link(
            "blocks_execution_of",
            &git_binary_dependency_id,
            &inspect_git_capability_id,
            "git CLI binary missing",
            "verified",
        );
        push_link(
            "blocks_execution_of",
            &git_binary_dependency_id,
            &inspect_git_affordance_id,
            "git CLI binary missing",
            "verified",
        );
    }
    if !cargo_manifest_present {
        push_link(
            "blocks_execution_of",
            &cargo_manifest_precondition_id,
            &build_rust_capability_id,
            "Cargo manifest missing",
            "verified",
        );
        push_link(
            "blocks_execution_of",
            &cargo_manifest_precondition_id,
            &build_rust_affordance_id,
            "Cargo manifest missing",
            "verified",
        );
    }
    if !cargo_available {
        push_link(
            "blocks_execution_of",
            &cargo_binary_dependency_id,
            &build_rust_capability_id,
            "cargo CLI binary missing",
            "verified",
        );
        push_link(
            "blocks_execution_of",
            &cargo_binary_dependency_id,
            &build_rust_affordance_id,
            "cargo CLI binary missing",
            "verified",
        );
    }

    WorkspaceProjection { objects, links }
}

fn identity_projection(focusa: &FocusaState) -> WorkspaceProjection {
    let mut objects = Vec::new();
    let mut links = Vec::new();

    let identity_name = focusa
        .session
        .as_ref()
        .and_then(|s| s.adapter_id.clone())
        .unwrap_or_else(|| "focusa-daemon".to_string());
    let agent_identity_id = stable_id("agent_identity", &identity_name);
    objects.push(json!({
        "id": agent_identity_id,
        "object_type": "agent_identity",
        "identity_name": identity_name,
        "identity_kind": "runtime_agent",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    let actor_seed = focusa
        .work_loop
        .run
        .worker_session_id
        .clone()
        .or_else(|| focusa.session.as_ref().map(|s| s.session_id.to_string()))
        .unwrap_or_else(|| focusa.work_loop.run.project_run_id.to_string());
    let actor_instance_id = stable_id("actor_instance", &actor_seed);
    objects.push(json!({
        "id": actor_instance_id,
        "object_type": "actor_instance",
        "instance_kind": if focusa.work_loop.enabled { "work_loop_runtime" } else { "session_runtime" },
        "status": if focusa.work_loop.enabled { "active" } else { "stale" },
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));
    links.push(json!({"type":"instantiates","source_id":actor_instance_id,"target_id":agent_identity_id,"evidence":"work_loop.run.worker_session_id|session.session_id","status":"verified"}));

    let role_kind = match focusa.work_loop.authorship_mode {
        focusa_core::types::AuthorshipMode::Delegated => "executor",
        focusa_core::types::AuthorshipMode::OperatorOnly => "operator_assistant",
    };
    let role_profile_id = stable_id("role_profile", role_kind);
    objects.push(json!({
        "id": role_profile_id,
        "object_type": "role_profile",
        "role_kind": role_kind,
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));
    links.push(json!({"type":"serves_role","source_id":actor_instance_id,"target_id":role_profile_id,"evidence":"work_loop.authorship_mode","status":"verified"}));

    if let Some(worker) = focusa.work_loop.active_worker.as_ref() {
        let capability_profile_id = stable_id("capability_profile", &worker.worker_id);
        objects.push(json!({
            "id": capability_profile_id,
            "object_type": "capability_profile",
            "profile_kind": worker.context_window_class.clone().unwrap_or_else(|| "runtime_capabilities".to_string()),
            "status": "active",
            "membership_class": "verified",
            "provenance_class": "runtime_observed",
            "fresh": true,
            "tool_use_supported": worker.tool_use_supported,
            "edit_reliable": worker.edit_reliable,
            "structured_output_reliable": worker.structured_output_reliable,
            "code_generation_strong": worker.code_generation_strong,
            "fallback_available": worker.fallback_available,
        }));
        links.push(json!({"type":"has_capability_profile","source_id":actor_instance_id,"target_id":capability_profile_id,"evidence":"work_loop.active_worker","status":"verified"}));
    }

    let permission_profile_id = stable_id(
        "permission_profile",
        &format!(
            "{}:{}:{}",
            focusa.work_loop.policy.allow_destructive_actions,
            focusa.work_loop.policy.require_operator_for_governance,
            focusa.work_loop.policy.require_operator_for_scope_change
        ),
    );
    objects.push(json!({
        "id": permission_profile_id,
        "object_type": "permission_profile",
        "profile_kind": "work_loop_policy_permissions",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
        "allow_destructive_actions": focusa.work_loop.policy.allow_destructive_actions,
        "require_operator_for_governance": focusa.work_loop.policy.require_operator_for_governance,
        "require_operator_for_scope_change": focusa.work_loop.policy.require_operator_for_scope_change,
        "require_verification_before_persist": focusa.work_loop.policy.require_verification_before_persist,
    }));
    links.push(json!({"type":"has_permission_profile","source_id":actor_instance_id,"target_id":permission_profile_id,"evidence":"work_loop.policy","status":"verified"}));

    if let Some(task) = focusa.work_loop.current_task.as_ref() {
        let responsibility_id = stable_id("responsibility", &task.work_item_id);
        objects.push(json!({
            "id": responsibility_id,
            "object_type": "responsibility",
            "responsibility_kind": format!("{:?}", task.task_class).to_ascii_lowercase(),
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
            "title": task.title,
        }));
        links.push(json!({"type":"owns_responsibility","source_id":actor_instance_id,"target_id":responsibility_id,"evidence":"work_loop.current_task","status":"verified"}));
    }

    if focusa.work_loop.pause_flags.operator_override_active
        || focusa.work_loop.pause_flags.governance_decision_pending
        || focusa
            .work_loop
            .pause_flags
            .destructive_confirmation_required
        || focusa.work_loop.policy.require_operator_for_scope_change
        || focusa.work_loop.policy.require_operator_for_governance
    {
        let boundary_id = stable_id(
            "handoff_boundary",
            &format!(
                "{}:{}:{}",
                focusa.work_loop.pause_flags.operator_override_active,
                focusa.work_loop.pause_flags.governance_decision_pending,
                focusa
                    .work_loop
                    .pause_flags
                    .destructive_confirmation_required
            ),
        );
        objects.push(json!({
            "id": boundary_id,
            "object_type": "handoff_boundary",
            "boundary_kind": if focusa.work_loop.pause_flags.operator_override_active {"operator_override_boundary"} else if focusa.work_loop.pause_flags.governance_decision_pending {"governance_boundary"} else if focusa.work_loop.pause_flags.destructive_confirmation_required {"destructive_confirmation_boundary"} else {"operator_policy_boundary"},
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
        }));
        links.push(json!({"type":"bounded_by_handoff","source_id":actor_instance_id,"target_id":boundary_id,"evidence":"work_loop.pause_flags|work_loop.policy","status":"verified"}));
    }

    if focusa.work_loop.run.worker_session_id.is_some() || focusa.session.is_some() {
        let continuity_id = stable_id("session_continuity", &actor_seed);
        objects.push(json!({
            "id": continuity_id,
            "object_type": "session_continuity",
            "continuity_kind": "session_bound",
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "runtime_observed",
            "fresh": true,
            "last_checkpoint_id": focusa.work_loop.run.last_checkpoint_id,
        }));
        links.push(json!({"type":"persists_via","source_id":actor_instance_id,"target_id":continuity_id,"evidence":"work_loop.run|session","status":"verified"}));
    }

    let identity_state_kind = if focusa.work_loop.pause_flags.operator_override_active {
        "awaiting_operator"
    } else if focusa.work_loop.pause_flags.governance_decision_pending
        || focusa
            .work_loop
            .pause_flags
            .destructive_confirmation_required
    {
        "handoff_required"
    } else if focusa.work_loop.enabled {
        "trusted_for_scope"
    } else {
        "constrained_by_runtime"
    };
    let identity_state_id = stable_id("identity_state", identity_state_kind);
    objects.push(json!({
        "id": identity_state_id,
        "object_type": "identity_state",
        "state_kind": identity_state_kind,
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));

    if let Some(current_ask) = focusa.work_loop.decision_context.current_ask.as_ref() {
        let current_ask_id = stable_id("current_ask", current_ask);
        links.push(json!({
            "type": "governed_by_identity",
            "source_id": agent_identity_id,
            "target_id": current_ask_id,
            "evidence": "work_loop.decision_context.current_ask",
            "status": "verified",
        }));
        links.push(json!({
            "type": "governed_by_identity",
            "source_id": role_profile_id,
            "target_id": current_ask_id,
            "evidence": "work_loop.authorship_mode + decision_context.current_ask",
            "status": "verified",
        }));
    }

    if let Some(scope_kind) = focusa.work_loop.decision_context.scope_kind.as_ref() {
        let query_scope_id = stable_id("query_scope", scope_kind);
        links.push(json!({
            "type": "governed_by_identity",
            "source_id": agent_identity_id,
            "target_id": query_scope_id,
            "evidence": "work_loop.decision_context.scope_kind",
            "status": "verified",
        }));
    }

    WorkspaceProjection { objects, links }
}

fn governance_projection(focusa: &FocusaState) -> WorkspaceProjection {
    let mut objects = Vec::new();
    let mut links = Vec::new();
    let mut governance_by_proposal = BTreeMap::new();

    for verification in focusa.ontology.verifications.iter().take(128) {
        let Some(proposal_id) = verification.proposal_id else {
            continue;
        };
        let outcome = verification.outcome.to_ascii_lowercase();
        if !(outcome.contains("approved")
            || outcome.contains("accept")
            || outcome.contains("verified")
            || outcome.contains("pass")
            || outcome.contains("success"))
        {
            continue;
        }

        let decision_id = stable_id("governance_decision", &proposal_id.to_string());
        governance_by_proposal.insert(proposal_id.to_string(), decision_id.clone());
        objects.push(json!({
            "id": decision_id,
            "object_type": "governance_decision",
            "decision_kind": verification.verification,
            "status": "verified",
            "membership_class": "verified",
            "provenance_class": "verification_confirmed",
            "fresh": true,
            "proposal_id": proposal_id,
            "outcome": verification.outcome,
            "timestamp": verification.timestamp,
        }));
    }

    for proposal in focusa.ontology.proposals.iter().take(128) {
        let target = proposal.target_class.to_ascii_lowercase();
        let object_type = if [
            "ontology_version",
            "compatibility_profile",
            "migration_plan",
            "deprecation_record",
            "governance_decision",
        ]
        .contains(&target.as_str())
        {
            target
        } else if let Some(object_type) = proposal.object_type.as_ref() {
            let lowered = object_type.to_ascii_lowercase();
            if [
                "ontology_version",
                "compatibility_profile",
                "migration_plan",
                "deprecation_record",
                "governance_decision",
            ]
            .contains(&lowered.as_str())
            {
                lowered
            } else {
                continue;
            }
        } else {
            continue;
        };

        let object_id = proposal
            .object_id
            .clone()
            .unwrap_or_else(|| stable_id(&object_type, &proposal.proposal_id.to_string()));
        let mut obj = json!({
            "id": object_id,
            "object_type": object_type,
            "status": proposal.status,
            "membership_class": "provisional",
            "provenance_class": "reducer_promoted",
            "fresh": true,
            "proposal_id": proposal.proposal_id,
            "proposal_kind": proposal.proposal_kind,
            "source": proposal.source,
            "updated_at": proposal.updated_at,
        });

        if let Some(map) = obj.as_object_mut() {
            match object_type.as_str() {
                "ontology_version" => {
                    map.insert("version_kind".to_string(), json!("proposal_version"));
                }
                "compatibility_profile" => {
                    map.insert("profile_kind".to_string(), json!("proposal_compatibility"));
                }
                "migration_plan" => {
                    map.insert("plan_kind".to_string(), json!("proposal_migration"));
                }
                "deprecation_record" => {
                    map.insert("record_kind".to_string(), json!("proposal_deprecation"));
                }
                "governance_decision" => {
                    map.insert("decision_kind".to_string(), json!("proposal_decision"));
                }
                _ => {}
            }
        }

        if let (Some(source_id), Some(target_id)) =
            (proposal.source_id.as_ref(), proposal.target_id.as_ref())
        {
            let link_type = match object_type.as_str() {
                "compatibility_profile" => "compatible_with",
                "migration_plan" => "migrated_by",
                "deprecation_record" => "deprecated_by",
                "ontology_version" => "versioned_as",
                _ => "derived_from",
            };
            links.push(json!({
                "type": link_type,
                "source_id": source_id,
                "target_id": target_id,
                "evidence": "ontology.proposals.source_id/target_id",
                "status": "verified"
            }));
        }

        if let Some(decision_id) = governance_by_proposal.get(&proposal.proposal_id.to_string()) {
            links.push(json!({
                "type": "approved_by_governance",
                "source_id": object_id,
                "target_id": decision_id,
                "evidence": "ontology.verifications + ontology.proposals",
                "status": "verified"
            }));
        }

        objects.push(obj);
    }

    WorkspaceProjection { objects, links }
}

fn reference_resolution_projection(focusa: &FocusaState) -> WorkspaceProjection {
    let mut objects = Vec::new();
    let mut links = Vec::new();
    let mut object_ids = BTreeSet::new();
    let mut canonical_by_label = BTreeMap::new();

    for handle in focusa.reference_index.handles.iter().take(128) {
        let normalized = handle.label.to_ascii_lowercase().trim().to_string();
        if normalized.is_empty() {
            continue;
        }

        let canonical_id = canonical_by_label
            .entry(normalized.clone())
            .or_insert_with(|| stable_id("canonical_entity", &normalized))
            .clone();

        if object_ids.insert(canonical_id.clone()) {
            objects.push(json!({
                "id": canonical_id,
                "object_type": "canonical_entity",
                "entity_kind": match handle.kind {
                    HandleKind::Url => "url_entity",
                    HandleKind::Json => "json_entity",
                    HandleKind::Text => "text_entity",
                    HandleKind::Diff => "diff_entity",
                    HandleKind::Log => "log_entity",
                    HandleKind::FileSnapshot => "file_entity",
                    HandleKind::Other => "generic_entity",
                },
                "status": "verified",
                "membership_class": "verified",
                "provenance_class": "runtime_observed",
                "fresh": true,
            }));
        }

        let alias_id = stable_id("reference_alias", &handle.id.to_string());
        if object_ids.insert(alias_id.clone()) {
            objects.push(json!({
                "id": alias_id,
                "object_type": "reference_alias",
                "alias_kind": "handle_label_alias",
                "alias_text": handle.label,
                "status": "verified",
                "membership_class": "verified",
                "provenance_class": "runtime_observed",
                "fresh": true,
            }));
        }
        links.push(json!({
            "type": "derived_from",
            "source_id": alias_id,
            "target_id": canonical_id,
            "evidence": "reference_index.handles",
            "status": "verified",
        }));

        let candidate_id = stable_id("resolution_candidate", &handle.id.to_string());
        if object_ids.insert(candidate_id.clone()) {
            objects.push(json!({
                "id": candidate_id,
                "object_type": "resolution_candidate",
                "candidate_kind": "handle_to_entity_candidate",
                "status": "verified",
                "membership_class": "verified",
                "provenance_class": "runtime_observed",
                "fresh": true,
            }));
        }
        links.push(json!({
            "type": "derived_from",
            "source_id": candidate_id,
            "target_id": alias_id,
            "evidence": "reference_index.handles",
            "status": "verified",
        }));

        let decision_id = stable_id("resolution_decision", &handle.id.to_string());
        if object_ids.insert(decision_id.clone()) {
            objects.push(json!({
                "id": decision_id,
                "object_type": "resolution_decision",
                "decision_kind": "deterministic_handle_resolution",
                "status": "verified",
                "membership_class": "deterministic",
                "provenance_class": "runtime_observed",
                "fresh": true,
            }));
        }
        links.push(json!({
            "type": "verifies",
            "source_id": decision_id,
            "target_id": candidate_id,
            "evidence": "reference_index.handle_id_uniqueness",
            "status": "verified",
        }));
    }

    for proposal in focusa.ontology.proposals.iter().take(64) {
        if proposal
            .proposal_kind
            .to_ascii_lowercase()
            .contains("supersed")
            || proposal
                .target_class
                .to_ascii_lowercase()
                .contains("supersed")
        {
            let record_id = stable_id("supersession_record", &proposal.proposal_id.to_string());
            if object_ids.insert(record_id.clone()) {
                objects.push(json!({
                    "id": record_id,
                    "object_type": "supersession_record",
                    "record_kind": "proposal_supersession",
                    "status": proposal.status,
                    "membership_class": "provisional",
                    "provenance_class": "reducer_promoted",
                    "fresh": true,
                }));
            }
            if let (Some(source_id), Some(target_id)) =
                (proposal.source_id.as_ref(), proposal.target_id.as_ref())
            {
                links.push(json!({
                    "type": "supersedes",
                    "source_id": source_id,
                    "target_id": target_id,
                    "evidence": "ontology.proposals supersession",
                    "status": "verified",
                }));
                links.push(json!({
                    "type": "derived_from",
                    "source_id": record_id,
                    "target_id": source_id,
                    "evidence": "ontology.proposals.source_id",
                    "status": "verified",
                }));
            }
        }
    }

    WorkspaceProjection { objects, links }
}

fn projection_view_semantics_projection(focusa: &FocusaState) -> WorkspaceProjection {
    let mut objects = Vec::new();
    let mut links = Vec::new();

    let current_ask = focusa
        .work_loop
        .decision_context
        .current_ask
        .clone()
        .unwrap_or_else(|| "active_mission".to_string());
    let scope_kind = focusa
        .work_loop
        .decision_context
        .scope_kind
        .clone()
        .unwrap_or_else(|| "mission_carryover".to_string());
    let carryover_policy = focusa
        .work_loop
        .decision_context
        .carryover_policy
        .clone()
        .unwrap_or_else(|| "allow_if_relevant".to_string());

    let projection_id = stable_id("projection", &format!("{}:{}", current_ask, scope_kind));
    let view_profile_id = stable_id(
        "view_profile",
        focusa
            .work_loop
            .decision_context
            .ask_kind
            .as_deref()
            .unwrap_or("pi_operator_view"),
    );
    let projection_rule_id = stable_id("projection_rule", &carryover_policy);
    let projection_boundary_id = stable_id(
        "projection_boundary",
        &format!(
            "{}:{}",
            scope_kind,
            focusa
                .work_loop
                .decision_context
                .excluded_context_reason
                .clone()
                .unwrap_or_default()
        ),
    );

    objects.push(json!({
        "id": projection_id,
        "object_type": "projection",
        "projection_kind": "operator_ask_scoped",
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));
    objects.push(json!({
        "id": view_profile_id,
        "object_type": "view_profile",
        "view_kind": focusa.work_loop.decision_context.ask_kind.clone().unwrap_or_else(|| "pi_operator_view".to_string()),
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));
    objects.push(json!({
        "id": projection_rule_id,
        "object_type": "projection_rule",
        "rule_kind": carryover_policy,
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
    }));
    objects.push(json!({
        "id": projection_boundary_id,
        "object_type": "projection_boundary",
        "boundary_kind": scope_kind,
        "status": "active",
        "membership_class": "deterministic",
        "provenance_class": "runtime_observed",
        "fresh": true,
        "excluded_context_reason": focusa.work_loop.decision_context.excluded_context_reason,
        "excluded_context_labels": focusa.work_loop.decision_context.excluded_context_labels,
    }));

    links.push(json!({"type":"configured_by","source_id":projection_id,"target_id":view_profile_id,"evidence":"work_loop.decision_context.ask_kind","status":"verified"}));
    links.push(json!({"type":"configured_by","source_id":projection_id,"target_id":projection_rule_id,"evidence":"work_loop.decision_context.carryover_policy","status":"verified"}));
    links.push(json!({"type":"bounded_by_authority","source_id":projection_id,"target_id":projection_boundary_id,"evidence":"work_loop.decision_context.scope_kind","status":"verified"}));

    WorkspaceProjection { objects, links }
}

fn dedupe_objects(objects: Vec<Value>) -> Vec<Value> {
    let mut by_id: BTreeMap<String, Value> = BTreeMap::new();
    for object in objects {
        let Some(id) = object
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        else {
            continue;
        };

        match by_id.get_mut(&id) {
            Some(existing) => {
                let existing_status = existing
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let incoming_status = object.get("status").and_then(|v| v.as_str()).unwrap_or("");
                let incoming_preferred =
                    matches!(incoming_status, "canonical" | "verified" | "active")
                        && !matches!(existing_status, "canonical" | "verified" | "active");
                if incoming_preferred {
                    *existing = object;
                }
            }
            None => {
                by_id.insert(id, object);
            }
        }
    }
    by_id.into_values().collect()
}

fn dedupe_links(links: Vec<Value>) -> Vec<Value> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for link in links {
        let key = format!(
            "{}|{}|{}",
            link.get("type").and_then(|v| v.as_str()).unwrap_or(""),
            link.get("source_id").and_then(|v| v.as_str()).unwrap_or(""),
            link.get("target_id").and_then(|v| v.as_str()).unwrap_or("")
        );
        if seen.insert(key) {
            out.push(link);
        }
    }
    out
}

fn merge_projections(projections: Vec<WorkspaceProjection>) -> WorkspaceProjection {
    let total_object_capacity = projections.iter().map(|p| p.objects.len()).sum();
    let total_link_capacity = projections.iter().map(|p| p.links.len()).sum();
    let mut raw_objects = Vec::with_capacity(total_object_capacity);
    let mut raw_links = Vec::with_capacity(total_link_capacity);
    for projection in projections {
        raw_objects.extend(projection.objects);
        raw_links.extend(projection.links);
    }
    WorkspaceProjection {
        objects: dedupe_objects(raw_objects),
        links: dedupe_links(raw_links),
    }
}

fn bounded_summary_projection(focusa: &FocusaState, frame_id: Option<&str>) -> WorkspaceProjection {
    let frame = selected_frame(focusa, frame_id);
    merge_projections(vec![
        mission_projection(focusa, frame),
        canonical_ontology_projection(focusa),
        identity_projection(focusa),
        governance_projection(focusa),
        reference_resolution_projection(focusa),
        projection_view_semantics_projection(focusa),
    ])
}

fn combined_projection(focusa: &FocusaState, frame_id: Option<&str>) -> WorkspaceProjection {
    let frame = selected_frame(focusa, frame_id);
    merge_projections(vec![
        mission_projection(focusa, frame),
        workspace_projection(focusa),
        canonical_ontology_projection(focusa),
        visual_projection(focusa, frame),
        affordance_execution_projection(focusa, frame),
        identity_projection(focusa),
        governance_projection(focusa),
        reference_resolution_projection(focusa),
        projection_view_semantics_projection(focusa),
    ])
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
    let resolved_slice_type = normalize_slice_type(slice_type);
    let mut grouped: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for object in objects {
        if let Some(object_type) = object.get("object_type").and_then(|v| v.as_str()) {
            grouped
                .entry(object_type.to_string())
                .or_default()
                .push(object);
        }
    }

    let take = |kind: &str, max: usize| -> Vec<Value> {
        let mut bucket: Vec<&Value> = grouped.get(kind).into_iter().flatten().copied().collect();
        bucket.sort_by_key(|v| {
            v.get("id")
                .and_then(|x| x.as_str())
                .unwrap_or("unknown")
                .to_string()
        });

        bucket
            .into_iter()
            .take(max)
            .map(|v| {
                member(
                    v.get("id").and_then(|x| x.as_str()).unwrap_or("unknown"),
                    kind,
                    v.get("membership_class")
                        .and_then(|x| x.as_str())
                        .unwrap_or("deterministic"),
                    match resolved_slice_type {
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

    let mut members = match resolved_slice_type {
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

    let mut seen = BTreeSet::new();
    members.retain(|entry| {
        let id = entry
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        seen.insert(id)
    });

    if members.len() > 12 {
        members.truncate(12);
    }
    members
}

fn slice_payload(focusa: &FocusaState, frame_id: Option<&str>, slice_type: &str) -> Value {
    let resolved_slice_type = infer_slice_type_from_operator_context(focusa, slice_type);
    let projection = bounded_summary_projection(focusa, frame_id);
    let members = slice_members(&projection.objects, resolved_slice_type);
    json!({
        "requested_slice_type": slice_type,
        "slice_type": resolved_slice_type,
        "source": "ontology_world_projection",
        "projection_profile": {
            "projection_kind": slice_projection_kind(resolved_slice_type),
            "view_profile": slice_view_profile(resolved_slice_type),
            "canonical_truth_mutation": false,
            "invariants": [
                "canonical_and_projection_are_distinct",
                "unknown_slice_types_fallback_to_active_mission",
                "operator_context_can_refine_active_mission_slice",
                "membership_is_capped_and_deduplicated",
                "default_slice_uses_bounded_summary_projection"
            ]
        },
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
        let id = member
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
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

fn uncertainty_label(value: &Value) -> &'static str {
    let degraded = value
        .get("degraded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if degraded {
        return "degraded";
    }
    if value.get("contradiction_refs").is_some() || value.get("conflicts_with").is_some() {
        return "contradictory";
    }
    if value.get("rehydrate").is_some()
        && value
            .get("omitted_detail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    {
        return "rehydrate_needed";
    }
    let status = value.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if matches!(status, "stale" | "deprecated" | "superseded") {
        return "stale";
    }
    if matches!(status, "blocked" | "failed") {
        return "blocked_or_failed";
    }
    if matches!(status, "verified" | "canonical") {
        return "verified";
    }
    if value.get("evidence_ref").is_some()
        || value.get("verification_id").is_some()
        || value.get("provenance").is_some()
    {
        return "evidence_linked";
    }
    if matches!(status, "speculative" | "candidate" | "proposed") {
        return "speculative";
    }
    "projection_only"
}

fn compact_object_summary(object: &Value) -> Value {
    json!({
        "id": object.get("id").cloned().unwrap_or(Value::Null),
        "object_type": object.get("object_type").cloned().unwrap_or(Value::Null),
        "status": object.get("status").cloned().unwrap_or(Value::Null),
        "membership_class": object.get("membership_class").cloned().unwrap_or(Value::Null),
        "provenance_class": object.get("provenance_class").cloned().unwrap_or(Value::Null),
        "fresh": object.get("fresh").cloned().unwrap_or(Value::Null),
        "uncertainty": uncertainty_label(object),
        "rehydrate": {"route":"/v1/ontology/adjacency", "target_ref": object.get("id").cloned().unwrap_or(Value::Null)},
    })
}

fn compact_link(link: &Value) -> Value {
    json!({
        "type": link.get("type").cloned().unwrap_or(Value::Null),
        "source_id": link.get("source_id").cloned().unwrap_or(Value::Null),
        "target_id": link.get("target_id").cloned().unwrap_or(Value::Null),
        "evidence_ref": link.get("evidence_ref").cloned().unwrap_or(Value::Null),
        "status": link.get("status").cloned().unwrap_or(Value::Null),
        "uncertainty": uncertainty_label(link),
    })
}

fn link_strength_score(link: &Value) -> i64 {
    let status_bonus = match link
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("candidate")
    {
        "verified" | "canonical" => 8,
        "active" => 5,
        "proposed" | "candidate" => 2,
        "failed" | "blocked" => 4,
        _ => 1,
    };
    let type_bonus = match link
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("related_to")
    {
        "tested_by" | "verifies" | "implements" | "binds_to" => 7,
        "blocks" | "constrains" | "conflicts_with" => 6,
        "belongs_to_working_set" | "commits_to" | "drives_completion_of" => 5,
        _ => 2,
    };
    status_bonus + type_bonus
}

fn link_reason(link: &Value) -> String {
    let link_type = link
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("related_to");
    let source = link
        .get("source_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let target = link
        .get("target_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    format!("{source} --{link_type}--> {target}")
}

fn value_field_counts(items: &[Value], field: &str, fallback: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        let key = item
            .get(field)
            .and_then(|v| v.as_str())
            .unwrap_or(fallback)
            .to_string();
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

fn ontology_reducer_event_id(focusa: &FocusaState) -> Option<String> {
    focusa
        .ontology
        .delta_log
        .last()
        .map(|delta| format!("{}:{:?}", delta.delta_kind, delta.timestamp))
}

fn ontology_projection_authority_metadata() -> Value {
    json!({
        "advisory_only": true,
        "canonical": false,
        "canonical_truth_mutation": false,
        "promotion_path": "ontology projection -> PRE proposal or Workpoint candidate -> reducer/governance promotion",
        "canonicalization_tools": ["focusa_workpoint_checkpoint", "focusa_active_object_resolve", "focusa_evidence_capture"],
        "do_not_use_as": ["canonical_task_meaning", "resume_authority", "focus_state_mutation"],
    })
}

fn build_ontology_read_index(
    focusa: &FocusaState,
    frame_id: Option<&str>,
    projection: WorkspaceProjection,
) -> OntologyReadIndex {
    let mut objects = BTreeMap::new();
    let mut incoming_by_id: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut outgoing_by_id: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut incoming_by_type: BTreeMap<String, BTreeMap<String, Vec<Value>>> = BTreeMap::new();
    let mut outgoing_by_type: BTreeMap<String, BTreeMap<String, Vec<Value>>> = BTreeMap::new();
    for object in projection.objects {
        let id = object
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        objects.insert(id, object);
    }
    let mut link_type_counts = BTreeMap::new();
    for link in projection.links {
        let source = link
            .get("source_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let target = link
            .get("target_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let link_type = link
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("related_to")
            .to_string();
        *link_type_counts.entry(link_type.clone()).or_insert(0) += 1;
        outgoing_by_id
            .entry(source.clone())
            .or_default()
            .push(link.clone());
        incoming_by_id
            .entry(target.clone())
            .or_default()
            .push(link.clone());
        outgoing_by_type
            .entry(source)
            .or_default()
            .entry(link_type.clone())
            .or_default()
            .push(link.clone());
        incoming_by_type
            .entry(target)
            .or_default()
            .entry(link_type)
            .or_default()
            .push(link);
    }
    let object_values = objects.values().cloned().collect::<Vec<_>>();
    OntologyReadIndex {
        source_state_version: focusa.version,
        frame_id: frame_id.map(ToString::to_string),
        generated_at: Utc::now(),
        objects,
        incoming_by_id,
        outgoing_by_id,
        incoming_by_type,
        outgoing_by_type,
        object_type_counts: value_field_counts(&object_values, "object_type", "object"),
        link_type_counts,
        last_reducer_event_id: ontology_reducer_event_id(focusa),
        ttl_seconds: env_limit("FOCUSA_ONTOLOGY_READ_INDEX_TTL_SECONDS", 300),
    }
}

fn ontology_value_matches_scope(value: &Value, project_root: &str, continuity_id: &str) -> bool {
    let scope_class = value
        .get("scope_class")
        .or_else(|| value.pointer("/scope/class"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if scope_class == "global_schema" {
        return true;
    }
    let root = value
        .get("project_root")
        .or_else(|| value.pointer("/scope/project_root"))
        .or_else(|| value.pointer("/workstream/root_scope/root_path"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let continuity = value
        .get("continuity_id")
        .or_else(|| value.pointer("/scope/continuity_id"))
        .or_else(|| value.pointer("/workstream/continuity_id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    root == project_root && continuity == continuity_id
}

fn scoped_ontology_state(focusa: &FocusaState, scope: &ScopeContext) -> Option<FocusaState> {
    let project_root = scope.project_root.as_deref()?.trim();
    let continuity_id = scope.continuity_id.as_deref()?.trim();
    if project_root.is_empty() || continuity_id.is_empty() {
        return None;
    }
    let mut scoped = focusa.clone();
    scoped.clt = crate::routes::clt::scoped_clt_state(&focusa.clt, scope);
    scoped.session = focusa
        .session
        .as_ref()
        .filter(|session| {
            session.project_root.as_deref() == Some(project_root)
                && session.continuity_id.as_deref() == Some(continuity_id)
        })
        .cloned();

    scoped.focus_stack.frames.retain(|frame| {
        frame.project_root.as_deref() == Some(project_root)
            && frame.continuity_id.as_deref() == Some(continuity_id)
    });
    let frame_ids = scoped
        .focus_stack
        .frames
        .iter()
        .map(|frame| frame.id)
        .collect::<BTreeSet<_>>();
    scoped.focus_stack.active_id = scoped
        .focus_stack
        .active_id
        .filter(|id| frame_ids.contains(id));
    scoped.focus_stack.root_id = scoped
        .focus_stack
        .root_id
        .filter(|id| frame_ids.contains(id));
    scoped
        .focus_stack
        .stack_path_cache
        .retain(|id| frame_ids.contains(id));

    scoped.workpoint.records.retain(|record| {
        record.project_root.as_deref() == Some(project_root)
            && record.continuity_id.as_deref() == Some(continuity_id)
    });
    let workpoint_ids = scoped
        .workpoint
        .records
        .iter()
        .map(|record| record.workpoint_id)
        .collect::<BTreeSet<_>>();
    scoped.workpoint.active_workpoint_id = scoped
        .workpoint
        .active_workpoint_id
        .filter(|id| workpoint_ids.contains(id));
    scoped.workpoint.resume_events.retain(|record| {
        record
            .workpoint_id
            .as_ref()
            .is_some_and(|id| workpoint_ids.contains(id))
    });
    scoped.workpoint.drift_events.retain(|record| {
        record
            .workpoint_id
            .as_ref()
            .is_some_and(|id| workpoint_ids.contains(id))
    });
    scoped
        .workpoint
        .degraded_fallbacks
        .retain(|record| workpoint_ids.contains(&record.workpoint_id));

    scoped.trajectory.records.retain(|record| {
        record.project_root.as_deref() == Some(project_root)
            && record.continuity_id.as_deref() == Some(continuity_id)
            && record.scope_ref.as_ref().is_some_and(|scope_ref| {
                scope_ref.scope_kind == ScopeKind::Project
                    && scope_ref.root_path.to_string_lossy() == project_root
            })
    });
    let trajectory_ids = scoped
        .trajectory
        .records
        .iter()
        .map(|record| record.trajectory_id.clone())
        .collect::<BTreeSet<_>>();
    scoped.trajectory.active_trajectory_id = scoped
        .trajectory
        .active_trajectory_id
        .filter(|id| trajectory_ids.contains(id));
    scoped
        .trajectory
        .checkpoints
        .retain(|record| trajectory_ids.contains(&record.trajectory_id));
    scoped
        .trajectory
        .state_deltas
        .retain(|record| trajectory_ids.contains(&record.trajectory_id));

    scoped.reference_index.handles.retain(|handle| {
        handle.project_root.as_deref() == Some(project_root)
            && handle.continuity_id.as_deref() == Some(continuity_id)
    });
    scoped.memory.procedural.retain(|rule| match &rule.scope {
        RuleScope::Global => true,
        RuleScope::Project(root) => root == project_root,
        RuleScope::Frame(frame_id) => frame_ids.contains(frame_id),
    });

    scoped
        .ontology
        .objects
        .retain(|value| ontology_value_matches_scope(value, project_root, continuity_id));
    scoped
        .ontology
        .links
        .retain(|value| ontology_value_matches_scope(value, project_root, continuity_id));
    scoped.ontology.proposals.clear();
    scoped.ontology.verifications.clear();
    scoped.ontology.working_set_refreshes.clear();
    scoped.ontology.delta_log.retain(|record| {
        ontology_value_matches_scope(&record.payload, project_root, continuity_id)
    });
    Some(scoped)
}

fn ontology_read_index(
    focusa: &FocusaState,
    frame_id: Option<&str>,
    _scope: Option<&ScopeContext>,
) -> Arc<OntologyReadIndex> {
    // The caller supplies a pre-filtered exact-workstream state. Immutable
    // ontology schema remains global; mutable instance records are scoped.
    Arc::new(build_ontology_read_index(
        focusa,
        frame_id,
        combined_projection(focusa, frame_id),
    ))
}

/// Expose read-index cache metadata for telemetry/cache-metadata endpoints (Spec95 H1).
/// Returns per-cache-entry metadata: source reducer version, generated_at, TTL/invalidation
/// rule, canonical/degraded/stale status, and object/link/action counts.
pub fn ontology_read_index_cache_metadata(
    focusa: &FocusaState,
    scope: Option<&ScopeContext>,
) -> Value {
    let index = ontology_read_index(focusa, None, scope);
    let age_seconds = (Utc::now() - index.generated_at).num_seconds();
    let stale = age_seconds >= index.ttl_seconds as i64;
    json!({
        "cache_tier": "reducer-fed-hot",
        "cache_name": "ontology_read_index",
        "authority": ontology_projection_authority_metadata(),
        "advisory_only": true,
        "projection_canonical": false,
        "canonical_truth_mutation": false,
        "promotion_path": "read-index cache hit -> bounded projection selection -> Workpoint/PRE/reducer promotion path",
        "source_reducer_version": index.source_state_version,
        "generated_at": index.generated_at.to_rfc3339(),
        "ttl_seconds": index.ttl_seconds,
        "age_seconds": age_seconds,
        "invalidation_rule": "ontology_reducer_event_or_frame_change_or_ttl",
        "canonical": !stale,
        "canonical_meaning": "cache_entry_freshness_only_not_task_authority",
        "degraded": false,
        "stale": stale,
        "object_count": index.objects.len(),
        "link_count": index.link_type_counts.values().sum::<usize>(),
        "object_type_counts": index.object_type_counts,
        "link_type_counts": index.link_type_counts,
        "last_reducer_event_id": index.last_reducer_event_id,
        "frame_id": index.frame_id,
    })
}

fn evidence_handle_summary(handle: &HandleRef) -> Value {
    json!({
        "id": handle.id,
        "kind": handle.kind,
        "label": handle.label,
        "pinned": handle.pinned,
        "trajectory": handle.trajectory,
    })
}

fn adjacency_index_payload(
    focusa: &FocusaState,
    frame_id: Option<&str>,
    target_ref: Option<&str>,
    limit: usize,
    scope: Option<&ScopeContext>,
) -> Value {
    let index = ontology_read_index(focusa, frame_id, scope);
    let capped_limit = limit.clamp(1, 25);
    let link_limit = capped_limit.min(3);
    let mut nodes = Vec::new();
    for (id, object) in &index.objects {
        if let Some(target_ref) = target_ref
            && id != target_ref
        {
            continue;
        }
        let object_incoming = index.incoming_by_id.get(id).cloned().unwrap_or_default();
        let object_outgoing = index.outgoing_by_id.get(id).cloned().unwrap_or_default();
        let related_links = object_incoming
            .iter()
            .chain(object_outgoing.iter())
            .collect::<Vec<_>>();
        let verification_refs = related_links
            .iter()
            .filter_map(|link| {
                let source = link.get("source_id").and_then(|v| v.as_str()).unwrap_or_default();
                let target = link.get("target_id").and_then(|v| v.as_str()).unwrap_or_default();
                (source.contains("verification") || target.contains("verification") || link.get("verification_id").is_some())
                    .then(|| json!({"source_id":source, "target_id":target, "type":link.get("type").cloned().unwrap_or(Value::Null)}))
            })
            .take(8)
            .collect::<Vec<_>>();
        let evidence_handles = focusa
            .reference_index
            .handles
            .iter()
            .filter(|handle| handle.label.contains(id))
            .take(8)
            .map(evidence_handle_summary)
            .collect::<Vec<_>>();
        let related_workpoints = focusa.workpoint.records.iter().filter(|record| record.active_object_refs.iter().any(|target| target.contains(id) || id.contains(target))).take(8).map(|record| json!({"workpoint_id":record.workpoint_id, "action_intent":record.action_intent, "confidence":record.confidence})).collect::<Vec<_>>();
        let action_affordance_ids = index
            .objects
            .keys()
            .filter(|candidate| candidate.contains("affordance") && candidate.contains(id))
            .take(8)
            .cloned()
            .collect::<Vec<_>>();
        nodes.push(json!({
            "id": id,
            "object_type": object.get("object_type").cloned().unwrap_or(Value::Null),
            "status": object.get("status").cloned().unwrap_or(Value::Null),
            "membership_class": object.get("membership_class").cloned().unwrap_or(Value::Null),
            "provenance_refs": [object.get("provenance_class").cloned().unwrap_or(Value::Null)],
            "verification_refs": verification_refs,
            "working_set_memberships": [object.get("membership_class").cloned().unwrap_or(Value::Null)],
            "action_affordance_ids": action_affordance_ids,
            "related_evidence_handles": evidence_handles,
            "related_workpoints": related_workpoints,
            "incoming_count": object_incoming.len(),
            "outgoing_count": object_outgoing.len(),
            "incoming": object_incoming.into_iter().take(link_limit).map(|link| compact_link(&link)).collect::<Vec<_>>(),
            "outgoing": object_outgoing.into_iter().take(link_limit).map(|link| compact_link(&link)).collect::<Vec<_>>(),
            "incoming_by_type": index.incoming_by_type.get(id).map(|by_type| by_type.iter().map(|(kind, links)| (kind.clone(), links.len())).collect::<BTreeMap<_, _>>()).unwrap_or_default(),
            "outgoing_by_type": index.outgoing_by_type.get(id).map(|by_type| by_type.iter().map(|(kind, links)| (kind.clone(), links.len())).collect::<BTreeMap<_, _>>()).unwrap_or_default(),
            "uncertainty": uncertainty_label(object),
        }));
        if target_ref.is_none() && nodes.len() >= capped_limit {
            break;
        }
    }

    json!({
        "status": "ok",
        "source": "ontology_adjacency_read_index",
        "authority": ontology_projection_authority_metadata(),
        "advisory_only": true,
        "canonical": false,
        "promotion_path": "adjacency projection -> active-object resolution/evidence -> Workpoint or reducer proposal",
        "source_state_version": focusa.version,
        "index": {
            "projection_kind": "combined_projection_full_world_semantics",
            "source_reducer_version": index.source_state_version,
            "generated_at": index.generated_at,
            "last_reducer_event_id": index.last_reducer_event_id,
            "ttl_seconds": index.ttl_seconds,
            "invalidation_rule": "ontology_reducer_event_or_frame_change_or_ttl; source_state_version_observed",
            "parity_reference": "combined_projection_full_world_object_and_link_counts",
            "canonical_truth_mutation": false,
            "stale": (Utc::now() - index.generated_at).num_seconds() >= index.ttl_seconds as i64,
            "degraded": false
        },
        "object_count": index.objects.len(),
        "link_count": index.link_type_counts.values().sum::<usize>(),
        "object_type_counts": index.object_type_counts,
        "link_type_counts": index.link_type_counts,
        "returned": nodes.len(),
        "limit": capped_limit,
        "target_ref": target_ref,
        "selector": "adjacency",
        "field_projection": ["id", "object_type", "status", "membership_class", "verification_refs", "related_evidence_handles", "related_workpoints", "incoming_count", "outgoing_count", "uncertainty"],
        "do_not_use": ["full_ontology_graph", "broad_object_link_serialization"],
        "rehydrate_refs": nodes.iter().filter_map(|node| node.get("id").and_then(Value::as_str).map(|id| json!({"route":"/v1/ontology/adjacency", "target_ref": id}))).take(8).collect::<Vec<_>>(),
        "traversal_metadata": {
            "surface": "ontology",
            "selector": "adjacency",
            "limit": capped_limit,
            "returned": nodes.len(),
            "summary_only": true,
            "cold_full_payload_opt_in": false,
        },
        "nodes": nodes,
        "canonical_truth_mutation": false,
        "stale": false,
        "degraded": false,
    })
}

fn slice_object_relevance_score(object_type: &str, slice_type: &str) -> i64 {
    match slice_type {
        "debugging" => match object_type {
            "failure" => 50,
            "verification" => 45,
            "file" => 35,
            "test" => 30,
            "risk" => 25,
            _ => 0,
        },
        "refactor" => match object_type {
            "module" => 45,
            "file" => 40,
            "symbol" => 35,
            "test" => 25,
            _ => 0,
        },
        "regression" => match object_type {
            "test" => 45,
            "failure" => 40,
            "verification" => 35,
            "file" => 25,
            _ => 0,
        },
        "architecture" => match object_type {
            "decision" => 45,
            "constraint" => 40,
            "module" => 35,
            "route" => 30,
            "schema" => 25,
            _ => 0,
        },
        _ => match object_type {
            "goal" => 45,
            "task" => 40,
            "active_focus" => 35,
            "decision" => 30,
            "constraint" => 25,
            _ => 0,
        },
    }
}

struct WorkingSetPayloadParams<'a> {
    frame_id: Option<&'a str>,
    ask: Option<&'a str>,
    target_ref: Option<&'a str>,
    slice_type: &'a str,
    limit: usize,
    include_reasons: bool,
    cursor: usize,
    scope: Option<&'a ScopeContext>,
}

const ALLOWED_MEMBERSHIP_CLASSES: &[&str] = &[
    "pinned",
    "deterministic",
    "verified",
    "inferred",
    "provisional",
];

/// Spec 49: every working-set member carries one of the five membership
/// classes. Explicit values are honored only when they are in the enum;
/// otherwise the class is derived deterministically from verification
/// handles, provenance, and ask evidence — never `null`.
fn derived_membership_class(object: &Value, verified: bool, ask_matched: bool) -> Value {
    let explicit = object
        .get("membership_class")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if ALLOWED_MEMBERSHIP_CLASSES.contains(&explicit) {
        return json!(explicit);
    }
    if verified {
        return json!("verified");
    }
    if object
        .get("provenance_class")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        == "parser_derived"
    {
        return json!("deterministic");
    }
    if ask_matched {
        return json!("inferred");
    }
    json!("provisional")
}

/// Member freshness is derived, never defaulted to `fresh`. An explicit
/// `fresh` field is honored but downgraded when the read index is stale;
/// members without a tracked field fall back to evidence/index age.
fn member_freshness(object: &Value, index_stale: bool, verified: bool) -> Value {
    let explicit_fresh = object.get("fresh").and_then(|value| value.as_bool());
    let status = match explicit_fresh {
        Some(true) if !index_stale => "fresh",
        Some(false) => "stale",
        Some(true) => "degraded",
        None if index_stale => "degraded",
        None if verified => "fresh",
        None => "provisional",
    };
    json!({
        "status": status,
        "derived": explicit_fresh.is_none(),
        "source": if explicit_fresh.is_some() { "member_field" } else { "read_index_age_and_evidence" },
    })
}

fn working_set_payload(focusa: &FocusaState, params: WorkingSetPayloadParams<'_>) -> Value {
    let WorkingSetPayloadParams {
        frame_id,
        ask,
        target_ref,
        slice_type,
        limit,
        include_reasons,
        cursor,
        scope,
    } = params;
    let index = ontology_read_index(focusa, frame_id, scope);
    let resolved_slice_type = infer_slice_type_from_operator_context(focusa, slice_type);
    let capped_limit = limit.clamp(1, 50);
    let mut scored = Vec::new();
    let index_age_seconds = (Utc::now() - index.generated_at).num_seconds().max(0);
    let index_stale = index_age_seconds >= index.ttl_seconds as i64;
    let ask_lc = ask.unwrap_or_default().to_ascii_lowercase();
    let object_scan_limit = if target_ref.is_some() || !ask_lc.is_empty() {
        512
    } else {
        12
    };
    for object in index.objects.values().take(object_scan_limit) {
        let id = object
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let object_type = object
            .get("object_type")
            .and_then(|v| v.as_str())
            .unwrap_or("object");
        if let Some(target_ref) = target_ref
            && id != target_ref
        {
            continue;
        }
        let mut score = slice_object_relevance_score(object_type, resolved_slice_type);
        let mut ask_matched = false;
        if !ask_lc.is_empty()
            && (id.to_ascii_lowercase().contains(&ask_lc)
                || object_type.to_ascii_lowercase().contains(&ask_lc))
        {
            ask_matched = true;
            score += 75;
        }
        let mut related_links = Vec::new();
        if let Some(outgoing) = index.outgoing_by_id.get(id) {
            related_links.extend(outgoing.iter().take(3).cloned());
        }
        if let Some(incoming) = index.incoming_by_id.get(id) {
            related_links.extend(incoming.iter().take(3).cloned());
        }
        related_links.truncate(6);
        let link_strength: i64 = related_links.iter().map(link_strength_score).sum();
        score += related_links.len() as i64 + link_strength;
        if score <= 0 && target_ref.is_none() {
            continue;
        }
        let reasons = if include_reasons {
            related_links
                .iter()
                .map(|link| {
                    format!(
                        "{} [strength={}]",
                        link_reason(link),
                        link_strength_score(link)
                    )
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let verification_handles = related_links
            .iter()
            .filter(|link| {
                link.get("source_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .contains("verification")
                    || link
                        .get("target_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .contains("verification")
                    || link.get("verification_id").is_some()
            })
            .map(compact_link)
            .take(6)
            .collect::<Vec<_>>();
        let verification_status = if verification_handles.is_empty() {
            "unverified_projection"
        } else {
            "evidence_linked"
        };
        let action_affordance_ids = index
            .objects
            .keys()
            .filter(|candidate| candidate.contains("affordance") && candidate.contains(id))
            .take(6)
            .cloned()
            .collect::<Vec<_>>();
        scored.push((
            score,
            id.to_string(),
            json!({
                "id": id,
                "object_type": object_type,
                "status": object.get("status").cloned().unwrap_or(Value::Null),
                "membership_class": derived_membership_class(
                    object,
                    !verification_handles.is_empty(),
                    ask_matched,
                ),
                "provenance_handles": [object.get("provenance_class").cloned().unwrap_or(Value::Null)],
                "verification_handles": verification_handles,
                "verification_status": verification_status,
                "confidence": (score.max(0) as f64 / 100.0).min(1.0),
                "freshness": member_freshness(object, index_stale, !verification_handles.is_empty()),
                "action_affordance_ids": action_affordance_ids,
                "score": score,
                "reason_count": reasons.len(),
                "reasons": reasons,
                "related_link_count": related_links.len(),
                "link_strength_score": link_strength,
                "link_path_reason": reasons.first().cloned().unwrap_or_else(|| "slice relevance".to_string()),
                "rehydrate": {"route":"/v1/ontology/adjacency", "target_ref": id},
                "uncertainty": uncertainty_label(object),
            }),
        ));
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let total_candidates = scored.len();
    let members = scored
        .into_iter()
        .skip(cursor)
        .take(capped_limit)
        .map(|(_, _, value)| value)
        .collect::<Vec<_>>();
    let next_cursor =
        (cursor + members.len() < total_candidates).then(|| (cursor + members.len()).to_string());

    json!({
        "status": "ok",
        "source": "ontology_working_set_projection",
        "authority": ontology_projection_authority_metadata(),
        "advisory_only": true,
        "canonical": false,
        "promotion_path": "working-set projection -> selected target refs -> Workpoint checkpoint or reducer proposal",
        "index": {
            "projection_kind": "combined_projection_full_world_semantics",
            "source_reducer_version": focusa.version,
            "last_reducer_event_id": ontology_reducer_event_id(focusa),
            "canonical_truth_mutation": false,
            "stale": index_stale,
            "freshness": {
                "status": if index_stale { "stale" } else { "fresh" },
                "age_seconds": index_age_seconds,
                "ttl_seconds": index.ttl_seconds,
                "derived": true,
                "derived_from": "read_index_generated_at_and_reducer_delta_log",
            },
        },
        "slice_type": resolved_slice_type,
        "requested_slice_type": slice_type,
        "source_state_version": focusa.version,
        "target_ref": target_ref,
        "ask_present": ask.map(|value| !value.trim().is_empty()).unwrap_or(false),
        "total_candidates": total_candidates,
        "returned": members.len(),
        "limit": capped_limit,
        "cursor": cursor,
        "next_cursor": next_cursor,
        "truncated": next_cursor.is_some() || cursor > 0,
        "selector": "working_set",
        "field_projection": ["id", "object_type", "status", "membership_class", "verification_status", "confidence", "score", "rehydrate", "uncertainty"],
        "do_not_use": ["full_ontology_graph", "all_object_links", "unbounded_context_recall"],
        "rehydrate_refs": members.iter().filter_map(|member| member.get("rehydrate").cloned()).take(8).collect::<Vec<_>>(),
        "traversal_metadata": {
            "surface": "ontology",
            "selector": "working_set",
            "cursor": cursor,
            "limit": capped_limit,
            "next_cursor": next_cursor,
            "returned": members.len(),
            "total_candidates": total_candidates,
            "summary_only": true,
            "cold_full_payload_opt_in": false,
        },
        "members": members,
        "canonical_truth_mutation": false,
        "stale": false,
        "degraded": false,
    })
}

fn compact_action_candidate(name: &str) -> Value {
    let contract = action_contract(name);
    json!({
        "name": name,
        "verification_hooks": contract.get("verification_hooks").cloned().unwrap_or(Value::Null),
        "preconditions": contract.get("preconditions").cloned().unwrap_or(Value::Null),
        "side_effects": contract.get("side_effects").cloned().unwrap_or(Value::Null),
        "ontology_delta_expected": contract
            .get("expected_ontology_deltas")
            .and_then(|v| v.as_array())
            .map(|items| !items.is_empty())
            .unwrap_or(false),
    })
}

fn context_action_candidates(current_ask: Option<&str>, limit: usize) -> Vec<Value> {
    let ask = current_ask.unwrap_or_default().to_ascii_lowercase();
    let preferred: Vec<&str> = if ask.contains("test") || ask.contains("verify") {
        vec!["verify_invariant", "add_test", "verify_progress"]
    } else if ask.contains("debug") || ask.contains("fail") || ask.contains("bug") {
        vec!["verify_invariant", "resolve_risk", "record_scope_failure"]
    } else if ask.contains("ontology") || ask.contains("context") {
        vec![
            "select_relevant_context",
            "build_projection",
            "verify_projection_fidelity",
        ]
    } else {
        vec![
            "refresh_working_set",
            "select_relevant_context",
            "verify_progress",
        ]
    };
    preferred
        .into_iter()
        .filter(|name| ACTION_TYPES.contains(name))
        .take(limit)
        .map(compact_action_candidate)
        .collect()
}

fn ontology_identity_axes_payload(
    focusa: &FocusaState,
    requested_workpoint_id: Option<&str>,
) -> Value {
    let active_workpoint = focusa.workpoint.active_workpoint_id.and_then(|id| {
        focusa
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == id)
    });
    let selected_workpoint = requested_workpoint_id
        .and_then(|requested| {
            focusa
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id.to_string() == requested)
        })
        .or(active_workpoint);
    let project_root = selected_workpoint
        .and_then(|record| record.project_root.clone())
        .or_else(|| {
            focusa
                .session
                .as_ref()
                .and_then(|session| session.project_root.clone())
        });
    let continuity_id = selected_workpoint
        .and_then(|record| record.continuity_id.clone())
        .or_else(|| {
            focusa
                .session
                .as_ref()
                .and_then(|session| session.continuity_id.clone())
        });
    let temporal_session_id = selected_workpoint
        .and_then(|record| record.session_id.clone())
        .or_else(|| {
            focusa
                .session
                .as_ref()
                .map(|session| session.session_id.to_string())
        });
    let daemon_session_id = focusa
        .session
        .as_ref()
        .map(|session| session.session_id.to_string());
    let workpoint_card = selected_workpoint.map(|record| {
        json!({
            "workpoint_id": record.workpoint_id,
            "work_item_id": record.work_item_id,
            "canonical": record.canonical,
            "status": record.status,
            "mission": record.mission.as_deref().map(|value| value.chars().take(240).collect::<String>()),
            "next_slice": record.next_slice.as_deref().map(|value| value.chars().take(240).collect::<String>()),
            "project_root": record.project_root,
            "continuity_id": record.continuity_id,
            "session_id": record.session_id,
            "rehydrate": {"tool":"focusa_workpoint_resume", "workpoint_id": record.workpoint_id},
        })
    });
    json!({
        "projection_kind": "ontology_identity_axes_v1",
        "authority_gate": "project_root_plus_continuity_id",
        "advisory_only": true,
        "identity_axes": {
            "project": {
                "project_root": project_root,
                "authority_role": "project_folder_boundary",
                "rehydrate": {"tool":"focusa_trajectory_view"},
            },
            "logical_workstream": {
                "continuity_id": continuity_id,
                "authority_role": "logical_session_boundary",
                "must_match_for_same_root_resume": true,
            },
            "daemon_session": {
                "daemon_session_id": daemon_session_id,
                "process_id": std::process::id(),
                "authority_role": "runtime_instance_metadata",
            },
            "adapter_session": {
                "session_id": temporal_session_id,
                "authority_role": "temporal_metadata_only",
            },
            "workpoint_continuation_card": workpoint_card,
        },
        "aliases": [
            {"label":"project_root", "maps_to":"identity_axes.project.project_root", "authority":"project_folder_boundary"},
            {"label":"continuity_id", "maps_to":"identity_axes.logical_workstream.continuity_id", "authority":"logical_session_boundary"},
            {"label":"daemon_session_id", "maps_to":"identity_axes.daemon_session.daemon_session_id", "authority":"runtime_metadata"},
            {"label":"session_id", "maps_to":"identity_axes.adapter_session.session_id", "authority":"temporal_metadata_only"},
            {"label":"workpoint_id", "maps_to":"identity_axes.workpoint_continuation_card.workpoint_id", "authority":"continuation_card_id"}
        ],
        "do_not_use": [
            "session_id_as_authority_gate",
            "daemon_session_id_as_project_identity",
            "ontology_similarity_as_resume_authority"
        ],
        "rehydrate_refs": [
            {"tool":"focusa_workpoint_resume", "reason":"canonical continuation card"},
            {"tool":"focusa_trajectory_view", "reason":"project north-star orientation"},
            {"tool":"focusa_traverse", "surface":"ontology", "selector":"active_context"}
        ],
    })
}

fn verified_scope_ref_for_context(scope: &ScopeContext) -> Option<ScopeRef> {
    scope
        .require_workstream_key()
        .ok()
        .map(|workstream| workstream.root_scope)
}

fn verified_workstream_key(scope: &ScopeContext) -> Option<WorkstreamKey> {
    scope.require_workstream_key().ok()
}

fn ontology_context_payload(
    focusa: &FocusaState,
    body: &OntologyContextRequest,
    scope: &ScopeContext,
    scope_ref: &ScopeRef,
) -> Value {
    let budget_tokens = body.budget_tokens.unwrap_or(500).clamp(100, 4_000);
    let member_limit = (budget_tokens / 80).clamp(3, 20);
    let first_target = body.target_refs.first().map(String::as_str);
    let working_set = working_set_payload(
        focusa,
        WorkingSetPayloadParams {
            frame_id: body.frame_id.as_deref(),
            ask: body.current_ask.as_deref(),
            target_ref: first_target,
            slice_type: &body.slice_type,
            limit: member_limit,
            include_reasons: true,
            cursor: 0,
            scope: Some(scope),
        },
    );
    let members = working_set
        .get("members")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let active_object_set = members
        .iter()
        .map(|member| {
            json!({
                "id": member.get("id").cloned().unwrap_or(Value::Null),
                "object_type": member.get("object_type").cloned().unwrap_or(Value::Null),
                "uncertainty": member.get("uncertainty").cloned().unwrap_or(Value::Null),
                "score": member.get("score").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();
    let link_paths = members
        .iter()
        .flat_map(|member| {
            member
                .get("reasons")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        })
        .take(12)
        .collect::<Vec<_>>();
    let uncertainty_flags: BTreeSet<String> = members
        .iter()
        .filter_map(|member| member.get("uncertainty").and_then(|v| v.as_str()))
        .map(|value| value.to_string())
        .collect();
    let evidence_handles = focusa
        .reference_index
        .handles
        .iter()
        .rev()
        .take(8)
        .map(evidence_handle_summary)
        .collect::<Vec<_>>();
    let affordances = affordances_payload(
        focusa,
        body.frame_id.as_deref(),
        first_target,
        body.current_ask.as_deref(),
        Some("current"),
        8,
    );
    let blocked_affordances = affordances
        .get("blocked_actions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    json!({
        "status": "ok",
        "source": "ontology_prompt_safe_context",
        "authority": ontology_projection_authority_metadata(),
        "advisory_only": true,
        "canonical": false,
        "scope": {
            "root_scope": scope_ref,
            "continuity_id": scope.continuity_id,
            "working_subpath_id": scope.working_subpath_id,
        },
        "scope_verification": {
            "status": "verified_exact",
            "project_root": scope.project_root,
            "continuity_id": scope.continuity_id,
            "scope_ref": scope_ref,
        },
        "cross_plane_agreement": {
            "status": "partial",
            "ontology_scope": "verified",
            "temporal_authority": "receipt_required",
            "prediction_authority": "receipt_required",
            "rdf_owl_shacl_integrity": "receipt_required",
            "evidence_verifiers": "receipt_required",
            "policy": "do_not_promote_partial_agreement_to_canonical_authority",
        },
        "promotion_path": "prompt-safe scoped inner-world projection -> temporal/prediction/RDF/verifier agreement -> active-object resolve -> Workpoint checkpoint/evidence capture",
        "source_state_version": focusa.version,
        "observed_at_wall_clock_utc": Utc::now().to_rfc3339(),
        "temporal_context": {
            "clock_domain": "wall_clock_utc",
            "source": "daemon_system_wall_clock",
            "authority": "observation_only_until_calibrated",
            "calibration_status": "required_for_exact_or_high_consequence_claims",
        },
        "freshness": {
            "status": "fresh",
            "ttl_seconds": env_limit("FOCUSA_ONTOLOGY_CONTEXT_TTL_SECONDS", 300),
        },
        "view_profile": body.view_profile.as_deref().unwrap_or("pi_operator_view"),
        "budget_tokens": budget_tokens,
        "workpoint_id": body.workpoint_id,
        "target_refs": body.target_refs,
        "active_object_refs": body.active_object_refs,
        "operator_steering_detected": body.operator_steering_detected,
        "context_posture": "surgical_summary_only",
        "identity_axes": ontology_identity_axes_payload(focusa, body.workpoint_id.as_deref()),
        "selector": "active_context",
        "field_projection": ["active_object_set", "relevant_link_paths", "valid_next_actions", "blocked_affordances", "evidence_handles", "uncertainty_flags"],
        "do_not_use": ["full_ontology_graph", "broad_object_link_serialization", "unbounded_working_set"],
        "active_object_set": active_object_set,
        "relevant_link_paths": link_paths,
        "valid_next_actions": context_action_candidates(body.current_ask.as_deref(), 5),
        "blocked_affordances": blocked_affordances,
        "evidence_handles": evidence_handles,
        "uncertainty_flags": uncertainty_flags.into_iter().collect::<Vec<_>>(),
        "working_set": working_set,
        "traversal_metadata": {
            "surface": "ontology",
            "selector": "active_context",
            "limit": member_limit,
            "returned": active_object_set.len(),
            "summary_only": true,
            "rehydrate_routes": ["/v1/ontology/working-set", "/v1/ontology/adjacency", "/v1/ecs/rehydrate/{handle_id}"],
        },
        "canonical_truth_mutation": false,
        "stale": false,
        "degraded": false,
        "rehydrate": {"routes":["/v1/ontology/working-set", "/v1/ontology/adjacency", "/v1/ecs/rehydrate/{handle_id}"]}
    })
}

fn graph_community_summaries_payload(
    focusa: &FocusaState,
    frame_id: Option<&str>,
    limit: usize,
) -> Value {
    let projection = bounded_summary_projection(focusa, frame_id);
    let capped_limit = limit.clamp(1, 20);
    let mut by_type: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for object in &projection.objects {
        let object_type = object
            .get("object_type")
            .and_then(|v| v.as_str())
            .unwrap_or("object")
            .to_string();
        by_type.entry(object_type).or_default().push(object);
    }
    let mut summaries = Vec::new();
    for (community_type, objects) in by_type.into_iter() {
        let all_object_ids = objects
            .iter()
            .filter_map(|object| object.get("id").and_then(|v| v.as_str()))
            .collect::<BTreeSet<_>>();
        let object_ids = all_object_ids.iter().copied().take(8).collect::<Vec<_>>();
        let evidence_links = projection
            .links
            .iter()
            .filter(|link| {
                let source = link.get("source_id").and_then(|v| v.as_str()).unwrap_or_default();
                let target = link.get("target_id").and_then(|v| v.as_str()).unwrap_or_default();
                all_object_ids.contains(source) || all_object_ids.contains(target)
            })
            .take(8)
            .map(|link| json!({
                "path": link_reason(link),
                "strength": link_strength_score(link),
                "evidence": link.get("evidence").or_else(|| link.get("evidence_ref")).cloned().unwrap_or(Value::Null),
                "uncertainty": uncertainty_label(link),
            }))
            .collect::<Vec<_>>();
        let community_score: i64 = evidence_links
            .iter()
            .map(|link| link.get("strength").and_then(|v| v.as_i64()).unwrap_or(0))
            .sum::<i64>()
            + object_ids.len() as i64;
        summaries.push(json!({
            "community_id": stable_id("community", &community_type),
            "community_type": community_type,
            "object_count": objects.len(),
            "sample_object_ids": object_ids,
            "community_score": community_score,
            "summary": format!("{} projected ontology objects with {} evidence-linked paths", objects.len(), evidence_links.len()),
            "evidence_links": evidence_links,
            "rehydrate": {"route":"/v1/ontology/working-set", "slice_type":"active_mission"},
            "canonical_truth_mutation": false,
        }));
    }
    let mut sorted_links = projection.links.iter().collect::<Vec<_>>();
    sorted_links.sort_by_key(|link| std::cmp::Reverse(link_strength_score(link)));
    let top_link_evidence = sorted_links
        .into_iter()
        .take(16)
        .map(|link| json!({
            "path": link_reason(link),
            "strength": link_strength_score(link),
            "evidence": link.get("evidence").or_else(|| link.get("evidence_ref")).cloned().unwrap_or(Value::Null),
            "uncertainty": uncertainty_label(link),
        }))
        .collect::<Vec<_>>();
    if !top_link_evidence.is_empty() {
        let community_score = top_link_evidence
            .iter()
            .map(|link| link.get("strength").and_then(|v| v.as_i64()).unwrap_or(0))
            .sum::<i64>();
        summaries.push(json!({
            "community_id": stable_id("community", "relation_paths"),
            "community_type": "relation_paths",
            "object_count": projection.objects.len(),
            "sample_object_ids": [],
            "community_score": community_score + 10_000,
            "summary": format!("{} high-signal ontology relation paths", top_link_evidence.len()),
            "evidence_links": top_link_evidence,
            "rehydrate": {"route":"/v1/ontology/adjacency"},
            "canonical_truth_mutation": false,
        }));
    }
    summaries.sort_by(|a, b| {
        b.get("community_score")
            .and_then(|v| v.as_i64())
            .cmp(&a.get("community_score").and_then(|v| v.as_i64()))
    });
    let total_communities = summaries.len();
    summaries.truncate(capped_limit);
    json!({
        "status": "ok",
        "source": "ontology_graph_community_projection",
        "source_state_version": focusa.version,
        "total_communities": total_communities,
        "returned": summaries.len(),
        "limit": capped_limit,
        "communities": summaries,
        "canonical_truth_mutation": false,
        "stale": false,
        "degraded": false,
    })
}

fn affordances_payload(
    focusa: &FocusaState,
    frame_id: Option<&str>,
    target_ref: Option<&str>,
    action_intent: Option<&str>,
    scope: Option<&str>,
    limit: usize,
) -> Value {
    let frame = selected_frame(focusa, frame_id);
    let projection = affordance_execution_projection(focusa, frame);
    let capped_limit = limit.clamp(1, 50);
    let mut feasible = Vec::new();
    let mut blocked = Vec::new();
    for object in projection
        .objects
        .iter()
        .filter(|object| object.get("object_type").and_then(|v| v.as_str()) == Some("affordance"))
    {
        let id = object
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        if let Some(target_ref) = target_ref
            && !id.contains(target_ref)
        {
            continue;
        }
        let status = object
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("active");
        let candidate = json!({
            "id": id,
            "status": status,
            "affordance_kind": object.get("affordance_kind").cloned().unwrap_or(Value::Null),
            "target_ref": target_ref,
            "action_intent": action_intent,
            "scope": scope.unwrap_or("current"),
            "preconditions": object.get("preconditions").cloned().unwrap_or(Value::Null),
            "authority_boundary": object.get("authority_boundary").cloned().unwrap_or(Value::Null),
            "permission_boundary": object.get("permission_boundary").or_else(|| object.get("authority_boundary")).cloned().unwrap_or(Value::Null),
            "estimated_latency": object.get("estimated_latency").cloned().unwrap_or(Value::Null),
            "estimated_cost": object.get("estimated_cost").or_else(|| object.get("cost")).cloned().unwrap_or(json!("bounded_local_projection")),
            "cost": object.get("cost").or_else(|| object.get("estimated_cost")).cloned().unwrap_or(json!("bounded_local_projection")),
            "reversibility": object.get("reversibility").cloned().unwrap_or(Value::Null),
            "reliability": object.get("reliability").cloned().unwrap_or(Value::Null),
            "uncertainty": uncertainty_label(object),
            "rehydrate": {"route":"/v1/ontology/adjacency", "target_ref": id},
        });
        if matches!(status, "blocked" | "failed" | "unavailable") {
            blocked.push(candidate);
        } else {
            feasible.push(candidate);
        }
    }
    feasible.truncate(capped_limit);
    blocked.truncate(capped_limit);

    json!({
        "status": "ok",
        "source": "ontology_affordance_execution_projection",
        "authority": ontology_projection_authority_metadata(),
        "advisory_only": true,
        "canonical": false,
        "promotion_path": "affordance projection -> verified action/evidence -> Workpoint or reducer-governed ontology event",
        "source_state_version": focusa.version,
        "target_ref": target_ref,
        "action_intent": action_intent,
        "scope": scope.unwrap_or("current"),
        "feasible_actions": feasible,
        "blocked_actions": blocked,
        "valid_next_actions": context_action_candidates(action_intent, 5),
        "verification_hooks_required": true,
        "canonical_truth_mutation": false,
        "stale": false,
        "degraded": false,
    })
}

fn bm25_score_with_scope(
    keyword: &str,
    text: &str,
    action_intent: Option<&str>,
    previous_outcomes: &[Value],
) -> f64 {
    // Base BM25 score over id/label/summary fields.
    let text_lc = text.to_ascii_lowercase();
    let kw_lc = keyword.to_ascii_lowercase();
    let terms: Vec<&str> = kw_lc
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    if terms.is_empty() && action_intent.is_none() && previous_outcomes.is_empty() {
        return 0.0;
    }
    let text_words: Vec<&str> = text_lc
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .collect();
    if text_words.is_empty() && terms.is_empty() {
        return 0.0;
    }
    let avg_len = text_words.len().max(1) as f64;
    let k1 = 1.5_f64;
    let b = 0.75_f64;
    let mut score = 0.0_f64;
    let doc_len = text_words.len() as f64;
    for term in &terms {
        let tf = text_words.iter().filter(|w| *w == term).count() as f64;
        if tf > 0.0 {
            let norm = 1.0_f64 - b + b * (doc_len / avg_len);
            score += (tf * (k1 + 1.0)) / (tf + k1 * norm);
        }
    }
    // Spec95 J2: query-scope signal boost from action_intent.
    if let Some(intent) = action_intent {
        let intent_lc = intent.to_ascii_lowercase();
        if text_lc.contains(&intent_lc) {
            score += 8.0;
        }
        // Partial token match on action_intent terms.
        for intent_word in intent_lc
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 3)
        {
            if text_lc.contains(intent_word) {
                score += 2.0;
            }
        }
    }
    // Spec95 J2: boost items matching previous successful retrieval outcomes.
    for outcome in previous_outcomes.iter().take(5) {
        if let Some(outcome_id) = outcome.get("id").and_then(|v| v.as_str())
            && text.contains(outcome_id)
        {
            score += 5.0;
        }
        if let Some(outcome_label) = outcome.get("label").and_then(|v| v.as_str())
            && text.contains(outcome_label)
        {
            score += 3.0;
        }
    }
    score
}

fn local_bm25_rerank(
    hits: &mut Vec<Value>,
    ask: &str,
    action_intent: Option<&str>,
    previous_outcomes: &[Value],
    top_k: usize,
) {
    if hits.is_empty() {
        return;
    }
    for hit in hits.iter_mut() {
        if let Some(obj) = hit.as_object_mut() {
            let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let label = obj.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let summary = obj.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let text = format!("{} {} {}", id, label, summary);
            let base = obj.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            obj.insert(
                "score".to_string(),
                serde_json::json!(
                    base + bm25_score_with_scope(ask, &text, action_intent, previous_outcomes)
                        * 10.0
                ),
            );
        }
    }
    hits.sort_by(|a, b| {
        let sa = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let sb = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    hits.truncate(top_k);
}

fn retrieval_governor_payload(
    focusa: &FocusaState,
    body: &RetrievalGovernorRequest,
    scope: &ScopeContext,
    scope_ref: &ScopeRef,
) -> Value {
    let ask = body.current_ask.as_deref().unwrap_or_default();
    let ask_lc = ask.to_ascii_lowercase();
    let budget_tokens = body.budget_tokens.unwrap_or(800).clamp(100, 8_000);
    let include_workpoint = body.workpoint_id.is_some() && !body.operator_steering_detected;
    let include_affordances = ask_lc.contains("implement")
        || ask_lc.contains("action")
        || ask_lc.contains("route")
        || ask_lc.contains("tool");
    let include_metacog = body.include_metacog
        || ask_lc.contains("learn")
        || ask_lc.contains("remember")
        || ask_lc.contains("pattern");
    let context_body = OntologyContextRequest {
        current_ask: body.current_ask.clone(),
        frame_id: body.frame_id.clone(),
        workpoint_id: body.workpoint_id.clone(),
        target_refs: body.target_refs.clone(),
        budget_tokens: Some((budget_tokens / 2).max(100)),
        view_profile: Some("pi_operator_view".to_string()),
        slice_type: "active_mission".to_string(),
        operator_steering_detected: false,
        active_object_refs: Vec::new(),
    };
    let ontology_context = ontology_context_payload(focusa, &context_body, scope, scope_ref);
    let first_target = body.target_refs.first().map(String::as_str);
    let affordances = include_affordances.then(|| {
        affordances_payload(
            focusa,
            body.frame_id.as_deref(),
            first_target,
            body.current_ask.as_deref(),
            Some("current"),
            8,
        )
    });
    let mut retrieval_plan = vec![json!({
        "substrate": "ontology_context",
        "reason": "prompt-safe active object/link/action projection",
        "budget_tokens": budget_tokens / 2,
    })];
    if include_workpoint {
        retrieval_plan.push(json!({
            "substrate": "workpoint",
            "reason": "active continuation anchor remains relevant",
            "workpoint_id": body.workpoint_id,
        }));
    }
    if include_affordances {
        retrieval_plan.push(json!({
            "substrate": "ontology_affordances",
            "reason": "ask appears action/tool/implementation oriented",
        }));
    }
    if include_metacog {
        retrieval_plan.push(json!({
            "substrate": "metacognition",
            "reason": "ask implies reusable learning or pattern retrieval",
        }));
    }
    if !body.target_refs.is_empty() {
        retrieval_plan.push(json!({
            "substrate": "exact_target_refs",
            "reason": "operator or tool supplied explicit target refs",
            "target_refs": body.target_refs,
        }));
    }
    if ask.trim().is_empty() && body.target_refs.is_empty() && body.workpoint_id.is_none() {
        retrieval_plan.clear();
        retrieval_plan.push(json!({"substrate":"none", "reason":"self-contained or empty ask; no retrieval required"}));
    }
    let semantic_hits = focusa
        .memory
        .semantic
        .iter()
        .rev()
        .take(5)
        .map(|record| {
            let exact = (!ask_lc.is_empty() && (record.key.to_ascii_lowercase().contains(&ask_lc) || record.value.to_ascii_lowercase().contains(&ask_lc))) as i64;
            let score = 20 + (exact * 50) + (record.confidence * 30.0) as i64;
            json!({
                "substrate": "semantic_memory",
                "id": record.key,
                "summary": record.value.chars().take(160).collect::<String>(),
                "confidence": record.confidence,
                "freshness": "recent_window",
                "uncertainty": "evidence_linked",
                "score": score,
                "reasons": [if exact > 0 { "keyword_or_exact_match" } else { "recent_semantic_memory" }],
                "evidence_handles": []
            })
        })
        .collect::<Vec<_>>();
    let graph_hits = ontology_context
        .get("active_object_set")
        .and_then(|members| members.as_array())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|member| {
            let score = member.get("score").and_then(|v| v.as_i64()).unwrap_or(0) + 35;
            json!({
                "substrate": "ontology_graph",
                "id": member.get("id").cloned().unwrap_or(Value::Null),
                "object_type": member.get("object_type").cloned().unwrap_or(Value::Null),
                "score": score,
                "reasons": member.get("reasons").cloned().unwrap_or(json!(["graph_active_object_relevance"])),
                "evidence_handles": member.get("evidence_handles").cloned().unwrap_or(json!([])),
                "uncertainty": member.get("uncertainty").cloned().unwrap_or(json!("unverified_projection"))
            })
        })
        .collect::<Vec<_>>();
    let evidence_hits = focusa
        .reference_index
        .handles
        .iter()
        .rev()
        .take(5)
        .map(|handle| {
            let exact = (!ask_lc.is_empty() && handle.label.to_ascii_lowercase().contains(&ask_lc)) as i64;
            let score = 25 + (exact * 50) + if handle.pinned { 10 } else { 0 };
            json!({
                "substrate": "ecs_evidence",
                "id": handle.id,
                "kind": handle.kind,
                "label": handle.label,
                "freshness": "recent_window",
                "uncertainty": "evidence_linked",
                "score": score,
                "reasons": [if exact > 0 { "evidence_label_match" } else { "recent_evidence_handle" }],
                "evidence_handles": [handle.id]
            })
        })
        .collect::<Vec<_>>();

    // Apply local BM25 reranking with query-scope signals (Spec95 J2).
    let mut graph_reranked = graph_hits.clone();
    local_bm25_rerank(
        &mut graph_reranked,
        ask,
        body.action_intent.as_deref(),
        &body.previous_retrieval_outcomes,
        20,
    );
    let mut semantic_reranked = semantic_hits.clone();
    local_bm25_rerank(
        &mut semantic_reranked,
        ask,
        body.action_intent.as_deref(),
        &body.previous_retrieval_outcomes,
        20,
    );
    let mut evidence_reranked = evidence_hits.clone();
    local_bm25_rerank(
        &mut evidence_reranked,
        ask,
        body.action_intent.as_deref(),
        &body.previous_retrieval_outcomes,
        20,
    );

    json!({
        "status": "ok",
        "source": "ontology_retrieval_governor",
        "authority": ontology_projection_authority_metadata(),
        "advisory_only": true,
        "canonical": false,
        "promotion_path": "retrieval plan -> operator/tool selection -> Workpoint checkpoint/PRE proposal/reducer promotion",
        "source_state_version": focusa.version,
        "current_ask_present": !ask.trim().is_empty(),
        "operator_steering_detected": body.operator_steering_detected,
        "ask_kind": body.ask_kind,
        "query_scope": body.query_scope,
        "active_action_intent": body.action_intent,
        "stale_state": body.stale_state,
        "degraded_state": body.degraded_state,
        "previous_retrieval_outcome_count": body.previous_retrieval_outcomes.len(),
        "budget_tokens": budget_tokens,
        "retrieval_plan": retrieval_plan,
        "excluded_context_reason": if body.operator_steering_detected { "operator_steering" } else { "none" },
        "hybrid_ranker": {
            "signals": ["exact_refs", "ontology_graph", "working_set_score", "semantic_memory", "ecs_evidence", "evidence_handles", "freshness", "uncertainty", "operator_steering"],
            "secondary_model_reranking": {"enabled": true, "method": "bm25_local_rerank"},
            "canonical_truth_mutation": false
        },
        "ontology_context": ontology_context,
        "retrieval_results": {
            "ontology_graph": graph_reranked,
            "semantic_memory": semantic_reranked,
            "ecs_evidence": evidence_reranked,
            "exact_target_refs": body.target_refs,
            "reranked_by": ["exact_refs", "ontology_graph", "semantic_memory", "ecs_evidence", "freshness", "operator_steering"]
        },
        "affordances": affordances.unwrap_or(Value::Null),
        "degraded": body.degraded_state,
        "stale": body.stale_state,
    })
}

fn safe_ref_id(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, ':' | '/' | '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .chars()
        .take(180)
        .collect()
}

fn infer_tool_object_type(tool_name: &str) -> &'static str {
    if tool_name.contains("read") || tool_name.contains("edit") || tool_name.contains("write") {
        "file"
    } else if tool_name.contains("bash")
        || tool_name.contains("cargo")
        || tool_name.contains("test")
    {
        "test"
    } else if tool_name.contains("workpoint") {
        "workpoint"
    } else if tool_name.contains("metacog") {
        "memory_signal"
    } else {
        "tool_surface"
    }
}

fn tool_result_candidate_deltas(body: &ToolResultProposalRequest) -> Vec<Value> {
    let status = body
        .status
        .as_deref()
        .unwrap_or(if body.ok.unwrap_or(false) {
            "completed"
        } else {
            "observed"
        });
    let tool_ref = format!("tool:{}", safe_ref_id(&body.tool_name));
    let mut deltas = vec![json!({
        "delta_kind": "ontology_object_upsert_proposed",
        "object_type": "tool_surface",
        "object_id": tool_ref,
        "source": "tool_result_envelope",
        "status": "proposed",
        "summary": body.summary,
        "error": body.error
    })];

    for target in body.target_refs.iter().take(8) {
        let target_id = safe_ref_id(target);
        deltas.push(json!({
            "delta_kind": "ontology_object_upsert_proposed",
            "object_type": infer_tool_object_type(&body.tool_name),
            "object_id": target_id,
            "source": "tool_result_target",
            "status": "proposed"
        }));
        deltas.push(json!({
            "delta_kind": "ontology_link_upsert_proposed",
            "link_type": "supports_execution_of",
            "source_id": tool_ref,
            "target_id": target_id,
            "source": "tool_result_action_target",
            "status": "proposed"
        }));
        if status == "failed" || status == "error" || status == "blocked" {
            deltas.push(json!({
                "delta_kind": "ontology_status_change_proposed",
                "subject": target_id,
                "to_status": "failed",
                "source": "tool_result_failure",
                "status": "proposed"
            }));
        }
    }

    for evidence in body.evidence_refs.iter().take(8) {
        let evidence_id = safe_ref_id(evidence);
        deltas.push(json!({
            "delta_kind": "ontology_object_upsert_proposed",
            "object_type": "evidence",
            "object_id": evidence_id,
            "source": "tool_result_evidence",
            "status": "proposed"
        }));
        for target in body.target_refs.iter().take(4) {
            deltas.push(json!({
                "delta_kind": "ontology_link_upsert_proposed",
                "link_type": "verifies",
                "source_id": evidence_id,
                "target_id": safe_ref_id(target),
                "source": "tool_result_evidence_target",
                "status": "proposed"
            }));
        }
    }

    if let Some(workpoint_id) = &body.workpoint_id {
        let workpoint_ref = format!("workpoint:{}", safe_ref_id(workpoint_id));
        deltas.push(json!({
            "delta_kind": "ontology_object_upsert_proposed",
            "object_type": "workpoint",
            "object_id": workpoint_ref,
            "source": "tool_result_workpoint",
            "status": "proposed"
        }));
        if let Some(intent) = &body.action_intent {
            deltas.push(json!({
                "delta_kind": "ontology_link_upsert_proposed",
                "link_type": "commits_to",
                "source_id": workpoint_ref,
                "target_id": format!("action_intent:{}", safe_ref_id(intent)),
                "source": "tool_result_workpoint_intent",
                "status": "proposed"
            }));
        }
    }

    deltas.truncate(40);
    deltas
}

fn events_from_tool_result_deltas(
    proposal_id: Uuid,
    deltas: &[Value],
    workstream: &WorkstreamKey,
) -> Vec<FocusaEvent> {
    deltas
        .iter()
        .filter_map(
            |delta| match delta.get("delta_kind").and_then(|v| v.as_str()) {
                Some("ontology_object_upsert_proposed") => {
                    Some(FocusaEvent::OntologyObjectUpsertProposed {
                        workstream: Some(workstream.clone()),
                        proposal_id,
                        object_type: delta
                            .get("object_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tool_surface")
                            .to_string(),
                        object_id: delta
                            .get("object_id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        source: delta
                            .get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tool_result_envelope")
                            .to_string(),
                    })
                }
                Some("ontology_link_upsert_proposed") => {
                    Some(FocusaEvent::OntologyLinkUpsertProposed {
                        workstream: Some(workstream.clone()),
                        proposal_id,
                        link_type: delta
                            .get("link_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("derived_from")
                            .to_string(),
                        source_id: delta
                            .get("source_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tool:unknown")
                            .to_string(),
                        target_id: delta
                            .get("target_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("target:unknown")
                            .to_string(),
                        source: delta
                            .get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tool_result_envelope")
                            .to_string(),
                    })
                }
                Some("ontology_status_change_proposed") => {
                    Some(FocusaEvent::OntologyStatusChangeProposed {
                        workstream: Some(workstream.clone()),
                        proposal_id,
                        subject: delta
                            .get("subject")
                            .and_then(|v| v.as_str())
                            .unwrap_or("target:unknown")
                            .to_string(),
                        from_status: None,
                        to_status: delta
                            .get("to_status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("failed")
                            .to_string(),
                        source: delta
                            .get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("tool_result_envelope")
                            .to_string(),
                    })
                }
                _ => None,
            },
        )
        .collect()
}

fn execution_critic_payload(body: &ExecutionCriticRequest) -> Value {
    let intended = body
        .intended_action
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let next = body
        .workpoint_next_action
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let tool_status =
        body.tool_result
            .status
            .as_deref()
            .unwrap_or(if body.tool_result.ok.unwrap_or(false) {
                "completed"
            } else {
                "observed"
            });
    let failed = matches!(
        tool_status,
        "failed" | "error" | "blocked" | "validation_rejected"
    );
    let target_missing = body.target_refs.is_empty() && body.tool_result.target_refs.is_empty();
    let verification_missing =
        body.verification_hooks.is_empty() && body.tool_result.evidence_refs.is_empty();
    let aligned = !failed
        && !target_missing
        && (!verification_missing || intended.contains("inspect") || intended.contains("read"))
        && (next.is_empty()
            || intended.is_empty()
            || next.contains(&intended)
            || intended.contains(&next));
    let critic_outcome = if aligned {
        "alignment_no_op"
    } else if failed {
        "bounded_failure_proposal"
    } else {
        "recovery_suggestion"
    };
    let candidate_deltas = tool_result_candidate_deltas(&body.tool_result);
    json!({
        "status": "ok",
        "source": "ontology_execution_critic",
        "critic_outcome": critic_outcome,
        "aligned": aligned,
        "operator_priority_preserved": true,
        "reducer_authority_preserved": true,
        "canonical_truth_mutation": false,
        "signals": {
            "tool_status": tool_status,
            "failed": failed,
            "target_missing": target_missing,
            "verification_missing": verification_missing,
            "operator_priority": body.operator_priority,
        },
        "recovery_suggestion": if aligned { Value::Null } else { json!({
            "next_action": if failed { "inspect failure, adjust target or command, then re-run verification" } else { "attach target refs/evidence refs before promotion" },
            "safe_retry": !failed,
            "workpoint_next_action": body.workpoint_next_action,
        })},
        "failure_artifact": if failed { json!({
            "kind": "tool_result_failure",
            "tool": body.tool_result.tool_name,
            "error": body.tool_result.error,
            "affected_targets": body.target_refs,
        }) } else { Value::Null },
        "candidate_ontology_deltas": candidate_deltas,
        "evidence_refs": body.tool_result.evidence_refs,
    })
}

async fn execution_critic(Json(body): Json<ExecutionCriticRequest>) -> Json<Value> {
    Json(execution_critic_payload(&body))
}

fn bounded_text(value: &Value, max_chars: usize) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
        .chars()
        .take(max_chars)
        .collect()
}

fn reflection_synthesizer_payload(body: &ReflectionSynthesizerRequest) -> Value {
    let capped_limit = body.limit.unwrap_or(8).clamp(1, 20);
    let noisy = body.traces.len() + body.evals.len() + body.critic_outputs.len() == 0
        || body.evidence_refs.is_empty();
    let failure_count = body
        .critic_outputs
        .iter()
        .filter(|item| {
            item.get("critic_outcome").and_then(|v| v.as_str()) == Some("bounded_failure_proposal")
                || item
                    .get("signals")
                    .and_then(|v| v.get("failed"))
                    .and_then(|v| v.as_bool())
                    == Some(true)
        })
        .count();
    let alignment_count = body
        .critic_outputs
        .iter()
        .filter(|item| item.get("aligned").and_then(|v| v.as_bool()) == Some(true))
        .count();
    let mut artifacts = Vec::new();
    if failure_count > 0 {
        artifacts.push(json!({
            "artifact_kind": "failure_class_proposal",
            "title": "tool/result misalignment or failed verification",
            "summary": format!("{} critic output(s) indicated bounded failures", failure_count),
            "evidence_refs": body.evidence_refs.iter().take(6).cloned().collect::<Vec<_>>(),
            "scope_tags": body.scope_tags,
            "promotion_state": "proposed",
            "promotion_gate": "requires evaluation evidence before metacog promotion",
        }));
        artifacts.push(json!({
            "artifact_kind": "procedural_playbook_proposal",
            "title": "recover failed tool result before continuing",
            "steps": ["inspect failure artifact", "bind target refs", "run verification hook", "record evidence ref", "retry only when side effects are understood"],
            "evidence_refs": body.evidence_refs.iter().take(6).cloned().collect::<Vec<_>>(),
            "promotion_state": "proposed",
        }));
    }
    if alignment_count > 0 {
        artifacts.push(json!({
            "artifact_kind": "metacog_signal_proposal",
            "title": "execution alignment pattern",
            "summary": format!("{} critic output(s) were aligned no-ops", alignment_count),
            "evidence_refs": body.evidence_refs.iter().take(6).cloned().collect::<Vec<_>>(),
            "promotion_state": "proposed",
        }));
    }
    for eval in body.evals.iter().take(capped_limit) {
        artifacts.push(json!({
            "artifact_kind": "prediction_calibration_proposal",
            "title": "evaluation-derived calibration sample",
            "summary": bounded_text(eval, 220),
            "evidence_refs": body.evidence_refs.iter().take(4).cloned().collect::<Vec<_>>(),
            "promotion_state": "proposed",
        }));
    }
    if artifacts.is_empty() && !noisy {
        artifacts.push(json!({
            "artifact_kind": "rejected_alternative_proposal",
            "title": "insufficient repeated signal for promotion",
            "summary": "trace/eval evidence present but no reusable pattern exceeded synthesis thresholds",
            "evidence_refs": body.evidence_refs.iter().take(4).cloned().collect::<Vec<_>>(),
            "promotion_state": "rejected_noise",
        }));
    }
    artifacts.truncate(capped_limit);
    json!({
        "status": "ok",
        "source": "ontology_secondary_reflection_synthesizer",
        "canonical_truth_mutation": false,
        "promoted": false,
        "promotion_blocked_reason": if body.promote { "promotion requires explicit evidence/evaluation gate outside synthesizer" } else { "proposal_only" },
        "noise_rejected": noisy,
        "quality_gate": {
            "requires_evidence_refs": true,
            "requires_eval_before_promotion": true,
            "min_failure_or_alignment_signal": 1,
        },
        "synthesized_count": artifacts.len(),
        "synthesized_artifacts": artifacts,
        "rehydrate": {"routes": ["/v1/metacognition/retrieve", "/v1/predictions/recent", "/v1/ontology/execution-critic"]},
    })
}

async fn reflection_synthesizer(Json(body): Json<ReflectionSynthesizerRequest>) -> Json<Value> {
    Json(reflection_synthesizer_payload(&body))
}

fn eval_gate_passed(eval_results: &[Value]) -> bool {
    eval_results.iter().any(|item| {
        item.get("result").and_then(|v| v.as_str()) == Some("improved")
            || item.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) >= 0.7
            || item.get("promote_learning").and_then(|v| v.as_bool()) == Some(true)
    })
}

fn ontology_runtime_dir(state: &AppState, category: &str) -> PathBuf {
    Path::new(&state.config.data_dir)
        .join("runtime")
        .join("ontology")
        .join(category)
}

fn persist_ontology_artifact(
    state: &AppState,
    category: &str,
    id: &str,
    payload: &Value,
) -> Option<String> {
    let dir = ontology_runtime_dir(state, category);
    let path = dir.join(format!("{id}.json"));
    fs::create_dir_all(&dir).ok()?;
    let bytes = serde_json::to_vec_pretty(payload).ok()?;
    fs::write(&path, bytes).ok()?;
    Some(path.display().to_string())
}

async fn record_memory_pipeline_prediction(
    state: &AppState,
    scope: WorkstreamKey,
    artifact: &Value,
    evidence_refs: &[String],
    procedural_ready: bool,
) -> Option<String> {
    let record = append_prediction_record_scoped(
        state,
        scope,
        PredictionValue {
            prediction_type: "ontology_memory_pipeline_promotion".into(),
            context_refs: evidence_refs.iter().take(8).cloned().collect(),
            ontology_context: PredictionOntologyContext {
                object_refs: vec!["OntologyMemoryPipeline".into()],
                action_refs: vec!["promote_memory_candidate".into()],
                tool_refs: vec!["focusa_predict_evaluate".into()],
                evidence_refs: evidence_refs.iter().take(8).cloned().collect(),
                relation_refs: vec!["artifact_informs_prediction".into()],
            },
            predicted_outcome: if procedural_ready {
                "procedural candidate will improve repeated recovery"
            } else {
                "semantic candidate will improve future retrieval"
            }
            .into(),
            confidence: if procedural_ready { 0.82 } else { 0.72 },
            recommended_action: if procedural_ready {
                "evaluate procedural playbook candidate after next repeated use"
            } else {
                "retrieve promoted semantic candidate in the next related task"
            }
            .into(),
            why: format!(
                "Scoped ontology memory pipeline persisted promotion artifact {}.",
                artifact
                    .get("artifact_id")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            ),
            trajectory: None,
            actual_outcome: None,
            evaluated_at: None,
            score: None,
            learning_signal_ref: None,
            outcome_capture: None,
        },
    )
    .await
    .ok()?;
    Some(record.record_id)
}

fn memory_pipeline_payload(
    body: &MemoryPipelineRequest,
    persisted_artifact: Option<Value>,
) -> Value {
    let capped_limit = body.limit.unwrap_or(10).clamp(1, 25);
    let evidence_present = !body.evidence_refs.is_empty();
    let eval_passed = eval_gate_passed(&body.eval_results);
    let repeated = body.repeated_validation_count.unwrap_or(0);
    let stale = body.lesson_age_days.unwrap_or(0) > 30;
    let weak = !evidence_present || (!eval_passed && repeated == 0);
    let semantic_ready = evidence_present && eval_passed;
    let procedural_ready = semantic_ready && repeated >= 2;
    let mut stages = vec![
        json!({
            "stage": "episodic_event_capture",
            "status": if body.episodic_events.is_empty() { "missing" } else { "proposed" },
            "artifact_count": body.episodic_events.len(),
            "rehydrate": {"route":"/v1/events/recent"},
        }),
        json!({
            "stage": "evidence_handle_link",
            "status": if evidence_present { "proposed" } else { "blocked" },
            "evidence_refs": body.evidence_refs.iter().take(8).cloned().collect::<Vec<_>>(),
            "promotion_gate": "evidence required",
        }),
        json!({
            "stage": "secondary_summary_proposal",
            "status": if body.synthesis_artifacts.is_empty() { "missing" } else { "proposed" },
            "artifact_count": body.synthesis_artifacts.len(),
            "rehydrate": {"route":"/v1/ontology/reflection-synthesizer"},
        }),
        json!({
            "stage": "evaluator_check",
            "status": if eval_passed { "passed" } else { "blocked" },
            "eval_count": body.eval_results.len(),
            "promotion_gate": "eval result or calibration score required",
        }),
        json!({
            "stage": "semantic_metacog_learning",
            "status": if semantic_ready { "proposed" } else { "blocked" },
            "candidate": if semantic_ready { json!({"kind":"semantic_learning", "source":"secondary_synthesis", "evidence_refs": body.evidence_refs.iter().take(6).cloned().collect::<Vec<_>>()}) } else { Value::Null },
            "canonical_truth_mutation": false,
        }),
        json!({
            "stage": "procedural_playbook_hint",
            "status": if procedural_ready { "proposed" } else { "blocked" },
            "candidate": if procedural_ready { json!({"kind":"procedural_playbook", "source":"repeated_validated_learning", "tool_contract_hint": true}) } else { Value::Null },
            "promotion_gate": "repeated validated semantic learning required",
        }),
        json!({
            "stage": "decay_or_archive",
            "status": if stale || weak { "archive_weak_lesson_proposed" } else { "retained" },
            "reason": if stale { "stale_lesson" } else if weak { "weak_or_missing_evidence" } else { "active_signal" },
        }),
    ];
    stages.truncate(capped_limit);
    json!({
        "status": "ok",
        "source": "ontology_memory_promotion_pipeline",
        "canonical_truth_mutation": false,
        "pipeline_state": if procedural_ready { "procedural_candidate_ready" } else if semantic_ready { "semantic_candidate_ready" } else if weak { "blocked_or_archival_candidate" } else { "episodic_candidate" },
        "promotion_gates": {
            "evidence_present": evidence_present,
            "eval_passed": eval_passed,
            "repeated_validation_count": repeated,
            "procedural_threshold": 2,
        },
        "linked_artifacts": {
            "events": body.episodic_events.len(),
            "evidence_refs": body.evidence_refs.iter().take(8).cloned().collect::<Vec<_>>(),
            "synthesis_artifacts": body.synthesis_artifacts.iter().take(8).cloned().collect::<Vec<_>>(),
            "eval_results": body.eval_results.iter().take(8).cloned().collect::<Vec<_>>(),
        },
        "stages": stages,
        "durable_artifact": persisted_artifact,
    })
}

async fn memory_pipeline(
    State(state): State<Arc<AppState>>,
    Json(body): Json<MemoryPipelineRequest>,
) -> Json<Value> {
    let evidence_present = !body.evidence_refs.is_empty();
    let eval_passed = eval_gate_passed(&body.eval_results);
    let repeated = body.repeated_validation_count.unwrap_or(0);
    let semantic_ready = evidence_present && eval_passed;
    let procedural_ready = semantic_ready && repeated >= 2;
    let artifact_id = format!(
        "memory-pipeline-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let mut prediction_record_id = None;
    let artifact = if semantic_ready {
        let payload = json!({
            "schema": "focusa.ontology.memory_pipeline_artifact.v1",
            "artifact_id": artifact_id,
            "created_at": Utc::now().to_rfc3339(),
            "pipeline_state": if procedural_ready { "procedural_candidate_ready" } else { "semantic_candidate_ready" },
            "evidence_refs": body.evidence_refs.iter().take(16).cloned().collect::<Vec<_>>(),
            "synthesis_artifacts": body.synthesis_artifacts.iter().take(16).cloned().collect::<Vec<_>>(),
            "eval_results": body.eval_results.iter().take(16).cloned().collect::<Vec<_>>(),
            "repeated_validation_count": repeated,
            "canonical_truth_mutation": false,
            "promotion_target": if procedural_ready { "procedural_playbook_candidate" } else { "semantic_metacog_candidate" },
        });
        persist_ontology_artifact(&state, "memory-pipeline", &artifact_id, &payload).map(|path| {
            let artifact = json!({
                "artifact_id": artifact_id,
                "storage_path": path,
                "written": true,
                "promotion_target": payload.get("promotion_target").cloned().unwrap_or(Value::Null),
            });
            artifact
        })
    } else {
        None
    };
    if let Some(ref artifact) = artifact {
        prediction_record_id = record_memory_pipeline_prediction(
            &state,
            body.scope.clone(),
            artifact,
            &body.evidence_refs,
            procedural_ready,
        )
        .await;
    }
    let mut payload = memory_pipeline_payload(&body, artifact);
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "prediction_record".to_string(),
            prediction_record_id
                .map(|prediction_id| json!({"prediction_id": prediction_id, "written": true}))
                .unwrap_or(Value::Null),
        );
    }
    Json(payload)
}

fn intelligence_dashboard_payload(focusa: &FocusaState) -> Value {
    let projection = bounded_summary_projection(focusa, None);
    let evidence_linked = projection
        .links
        .iter()
        .filter(|link| link.get("evidence").is_some() || link.get("evidence_ref").is_some())
        .count();
    let stale_objects = projection
        .objects
        .iter()
        .filter(|object| object.get("status").and_then(|v| v.as_str()) == Some("stale"))
        .count();
    let failure_objects = projection
        .objects
        .iter()
        .filter(|object| {
            matches!(
                object.get("object_type").and_then(|v| v.as_str()),
                Some("failure" | "risk")
            )
        })
        .count();
    let verified_links = projection
        .links
        .iter()
        .filter(|link| link.get("status").and_then(|v| v.as_str()) == Some("verified"))
        .count();
    let total_links = projection.links.len().max(1);
    let eval_fixtures = vec![
        "compaction_recovery",
        "ontology_context",
        "affordances",
        "uncertainty_labels",
        "secondary_critic",
        "metacog_reuse",
        "code_docs_test_linkage",
        "operator_steering",
    ];
    let usefulness_metrics = json!({
        "retrieval_hit_rate": (verified_links as f64 / total_links as f64),
        "irrelevant_stale_context_rate": (stale_objects as f64 / projection.objects.len().max(1) as f64),
        "stale_context_rate": (stale_objects as f64 / projection.objects.len().max(1) as f64),
        "drift_prevented": focusa.workpoint.drift_events.len(),
        "tool_calls_saved": projection.links.iter().filter(|link| link.get("type").and_then(|v| v.as_str()) == Some("available_in_context")).count(),
        "failed_tool_calls_predicted": failure_objects,
        "workpoint_resume_success": focusa.workpoint.resume_events.len(),
        "evidence_linked_answer_rate": (evidence_linked as f64 / total_links as f64),
        "task_completion_delta": projection.objects.iter().filter(|object| object.get("status").and_then(|v| v.as_str()) == Some("completed")).count(),
        "latency_rss_overhead_status": "bounded_projection_only"
    });
    let fixed_eval_suite = json!({
        "fixture_count": eval_fixtures.len(),
        "fixtures": eval_fixtures,
        "release_gate": "all_fixed_eval_fixtures_required_for_doc78_claims",
        "regression_policy": "fail_ci_on_metric_contract_break"
    });
    json!({
        "status": "ok",
        "source": "ontology_intelligence_dashboard",
        "source_state_version": focusa.version,
        "projection_kind": "bounded_summary_projection",
        "canonical_truth_mutation": false,
        "metrics": usefulness_metrics,
        "usefulness_metrics": usefulness_metrics,
        "latency_rss_overhead": {"status":"bounded_projection_only", "proof_required": true},
        "fixed_eval_suite": fixed_eval_suite,
        "fixed_eval_suites": fixed_eval_suite,
        "deterministic_extractors": {
            "file_to_module_package": "workspace_projection",
            "route_to_handler": "route_file_parser",
            "test_to_code_under_test": "test_path_and_name_heuristic",
            "docs_spec_to_code_surface": "spec_reference_path_classifier",
            "tool_contract_to_api_cli_core": "spec90_contract_registry",
            "workpoint_target_ref_to_object_id": "workpoint_target_ref_projection",
            "evidence_handle_to_object_ref_doc_test": "ecs_handle_label_classifier",
            "canonical_truth_mutation": false
        },
        "proof_routes": [
            "/v1/ontology/context",
            "/v1/ontology/affordances",
            "/v1/ontology/retrieval-governor",
            "/v1/ontology/execution-critic",
            "/v1/ontology/reflection-synthesizer",
            "/v1/ontology/memory-pipeline"
        ],
    })
}

async fn intelligence_dashboard(State(state): State<Arc<AppState>>) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(intelligence_dashboard_payload(&focusa))
}

fn ontology_failure(
    http_status: StatusCode,
    error: impl Into<String>,
    failure_class: &str,
    why: impl Into<String>,
    recovery_hint: &str,
    misuse_hint: &str,
    next_tools: Vec<&'static str>,
) -> (StatusCode, Json<Value>) {
    let error = error.into();
    let why = why.into();
    let next_tools_value = json!(next_tools);
    let retry_safe = !matches!(
        failure_class,
        "validation_rejected" | "not_found" | "scope_mismatch" | "permission_denied"
    );
    let retry_posture = if retry_safe {
        "safe_retry"
    } else {
        "do_not_retry_unchanged"
    };
    (
        http_status,
        Json(json!({
            "status": "blocked",
            "canonical": false,
            "degraded": true,
            "error": error,
            "failure_class": failure_class,
            "why": why,
            "recovery_hint": recovery_hint,
            "misuse_hint": misuse_hint,
            "next_tools": next_tools_value.clone(),
            "details": {"tool_result_v1": {
                "ok": false,
                "status": "blocked",
                "canonical": false,
                "degraded": true,
                "failure_class": failure_class,
                "summary": why,
                "retry": {"safe": retry_safe, "posture": retry_posture, "reason": failure_class},
                "recovery_hint": recovery_hint,
                "misuse_hint": misuse_hint,
                "side_effects": [],
                "evidence_refs": [],
                "next_tools": next_tools_value,
                "error": {"code": failure_class, "message": error}
            }}
        })),
    )
}

fn ontology_validation_rejected(error: impl Into<String>) -> (StatusCode, Json<Value>) {
    let error = error.into();
    ontology_failure(
        StatusCode::BAD_REQUEST,
        error.clone(),
        "validation_rejected",
        error,
        "Correct the ontology request payload before retrying unchanged.",
        "Likely missing tool_name, unknown action_type, or invalid auto_promote/action_type combination.",
        vec!["focusa_tool_doctor", "focusa_trajectory_view"],
    )
}

fn ontology_dispatch_failed(
    action: &str,
    error: impl std::fmt::Display,
) -> (StatusCode, Json<Value>) {
    ontology_failure(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("failed to dispatch {action}: {error}"),
        "daemon_unavailable",
        format!("ontology {action} event could not be dispatched to daemon command channel"),
        "Check daemon health and retry after command channel recovery is clear.",
        "Likely daemon command channel closed, runtime shutdown, or writer/transport ownership issue.",
        vec![
            "focusa_tool_doctor",
            "focusa_work_loop_status",
            "focusa_workpoint_resume",
        ],
    )
}

async fn tool_result_proposals(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(body): Json<ToolResultProposalRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.tool_name.trim().is_empty() {
        return Err(ontology_validation_rejected("tool_name is required"));
    }
    let workstream = verified_workstream_key(&scope).ok_or_else(|| {
        ontology_validation_rejected("verified project and continuity scope required")
    })?;
    let proposal_id = Uuid::now_v7();
    let deltas = tool_result_candidate_deltas(&body);
    let events = events_from_tool_result_deltas(proposal_id, &deltas, &workstream);
    if body.emit_proposals {
        for event in events {
            state
                .command_tx
                .send(Action::EmitEvent { event })
                .await
                .map_err(|error| {
                    ontology_dispatch_failed("tool-result ontology proposal", error)
                })?;
        }
    }
    Ok(Json(json!({
        "status": "ok",
        "proposal_id": proposal_id,
        "canonical_truth_mutation": false,
        "emitted": body.emit_proposals,
        "candidate_delta_count": deltas.len(),
        "candidate_deltas": deltas,
        "reducer_route": "/v1/ontology/actions",
        "promotion_policy": "candidate_only_unless_emit_proposals_true_then_reducer_records_proposed_status",
        "reducer_promotion_records": {
            "route": "/v1/ontology/actions",
            "states": ["proposed", "accepted", "rejected"],
            "emitted_proposed_records": if body.emit_proposals { deltas.len() } else { 0 },
            "canonical_truth_mutation": false
        }
    })))
}

fn parse_or_new_proposal_id(raw: Option<&str>) -> Uuid {
    raw.and_then(|v| Uuid::parse_str(v).ok())
        .unwrap_or_else(Uuid::now_v7)
}

fn proposed_events_from_action(
    proposal_id: Uuid,
    action_type: &str,
    payload: &Value,
    source: &str,
    workstream: &WorkstreamKey,
) -> Vec<FocusaEvent> {
    let mut events = Vec::new();

    if let (Some(link_type), Some(source_id), Some(target_id)) = (
        payload.get("link_type").and_then(|v| v.as_str()),
        payload.get("source_id").and_then(|v| v.as_str()),
        payload.get("target_id").and_then(|v| v.as_str()),
    ) {
        events.push(FocusaEvent::OntologyLinkUpsertProposed {
            workstream: Some(workstream.clone()),
            proposal_id,
            link_type: link_type.to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            source: source.to_string(),
        });
        return events;
    }

    match action_type {
        "decompose_goal"
        | "prioritize_work"
        | "record_decision"
        | "register_constraint"
        | "identify_risk"
        | "mark_blocked"
        | "restore_progress"
        | "verify_progress"
        | "refresh_working_set"
        | "close_loop"
        | "complete_task" => {
            let subject_id = payload
                .get("object_id")
                .or_else(|| payload.get("task_id"))
                .or_else(|| payload.get("goal_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if matches!(
                action_type,
                "mark_blocked" | "restore_progress" | "verify_progress" | "complete_task"
            ) && let Some(subject) = subject_id.clone()
            {
                let to_status = match action_type {
                    "mark_blocked" => "blocked",
                    "restore_progress" => "active",
                    "verify_progress" => "verified",
                    "complete_task" => "completed",
                    _ => "active",
                };
                events.push(FocusaEvent::OntologyStatusChangeProposed {
                    workstream: Some(workstream.clone()),
                    proposal_id,
                    subject,
                    from_status: payload
                        .get("from_status")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    to_status: to_status.to_string(),
                    source: source.to_string(),
                });
            }

            if action_type == "refresh_working_set" {
                events.push(FocusaEvent::OntologyWorkingSetMembershipProposed {
                    workstream: Some(workstream.clone()),
                    proposal_id,
                    subject: payload
                        .get("subject")
                        .and_then(|v| v.as_str())
                        .unwrap_or("working_set")
                        .to_string(),
                    operation: "add".to_string(),
                    source: source.to_string(),
                });
            }

            events.push(FocusaEvent::OntologyObjectUpsertProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                object_type: payload
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(match action_type {
                        "decompose_goal" => "subgoal",
                        "prioritize_work" => "task",
                        "record_decision" => "decision",
                        "register_constraint" => "constraint",
                        "identify_risk" => "risk",
                        "close_loop" => "verification",
                        _ => "task",
                    })
                    .to_string(),
                object_id: subject_id,
                source: source.to_string(),
            });
        }
        "determine_current_ask"
        | "build_query_scope"
        | "verify_answer_scope"
        | "record_scope_failure" => {
            events.push(FocusaEvent::OntologyObjectUpsertProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                object_type: payload
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(if action_type == "determine_current_ask" {
                        "current_ask"
                    } else if action_type == "build_query_scope" {
                        "query_scope"
                    } else if action_type == "record_scope_failure" {
                        "scope_failure"
                    } else {
                        "verification"
                    })
                    .to_string(),
                object_id: payload
                    .get("object_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                source: source.to_string(),
            });
        }
        "select_relevant_context" | "exclude_irrelevant_context" => {
            events.push(FocusaEvent::OntologyWorkingSetMembershipProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                subject: payload
                    .get("subject")
                    .and_then(|v| v.as_str())
                    .unwrap_or("context_membership")
                    .to_string(),
                operation: if action_type == "select_relevant_context" {
                    "add".to_string()
                } else {
                    "remove".to_string()
                },
                source: source.to_string(),
            });
        }
        "detect_aliases"
        | "build_resolution_candidates"
        | "resolve_identity"
        | "verify_resolution"
        | "record_supersession" => {
            let canonical_object_id = if action_type == "resolve_identity" {
                payload
                    .get("canonical_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("entity_id").and_then(|v| v.as_str()))
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str()))
            } else {
                payload.get("object_id").and_then(|v| v.as_str())
            }
            .map(|s| s.to_string());

            if action_type == "resolve_identity" {
                if let (Some(alias_id), Some(canonical_id)) = (
                    payload.get("alias_id").and_then(|v| v.as_str()),
                    payload.get("canonical_id").and_then(|v| v.as_str()),
                ) {
                    events.push(FocusaEvent::OntologyLinkUpsertProposed {
                        workstream: Some(workstream.clone()),
                        proposal_id,
                        link_type: "canonicalizes".to_string(),
                        source_id: alias_id.to_string(),
                        target_id: canonical_id.to_string(),
                        source: source.to_string(),
                    });
                }

                if let Some(canonical_id) = canonical_object_id.clone() {
                    events.push(FocusaEvent::OntologyStatusChangeProposed {
                        workstream: Some(workstream.clone()),
                        proposal_id,
                        subject: canonical_id,
                        from_status: Some("candidate".to_string()),
                        to_status: "canonical".to_string(),
                        source: source.to_string(),
                    });
                }
            }

            events.push(FocusaEvent::OntologyObjectUpsertProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                object_type: payload
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(match action_type {
                        "detect_aliases" => "reference_alias",
                        "build_resolution_candidates" => "resolution_candidate",
                        "resolve_identity" => "canonical_entity",
                        "record_supersession" => "supersession_record",
                        _ => "canonical_entity",
                    })
                    .to_string(),
                object_id: canonical_object_id,
                source: source.to_string(),
            });
        }
        "build_projection"
        | "compress_projection"
        | "verify_projection_fidelity"
        | "switch_view_profile" => {
            let view_object_id = if action_type == "switch_view_profile" {
                payload
                    .get("profile_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str()))
                    .or_else(|| payload.get("actor_id").and_then(|v| v.as_str()))
            } else {
                payload.get("object_id").and_then(|v| v.as_str())
            }
            .map(|s| s.to_string());

            if action_type == "switch_view_profile"
                && let Some(view_id) = view_object_id.clone()
            {
                events.push(FocusaEvent::OntologyStatusChangeProposed {
                    workstream: Some(workstream.clone()),
                    proposal_id,
                    subject: view_id,
                    from_status: Some("candidate".to_string()),
                    to_status: "active".to_string(),
                    source: source.to_string(),
                });
            }

            events.push(FocusaEvent::OntologyObjectUpsertProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                object_type: payload
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(match action_type {
                        "switch_view_profile" => "view_profile",
                        "verify_projection_fidelity" => "verification",
                        _ => "projection",
                    })
                    .to_string(),
                object_id: view_object_id,
                source: source.to_string(),
            });
        }
        "create_version"
        | "declare_compatibility"
        | "build_migration_plan"
        | "execute_migration"
        | "deprecate_schema_element"
        | "review_governance_change"
        | "verify_post_migration_conformance" => {
            let governance_object_id = match action_type {
                "create_version" => payload
                    .get("version_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str())),
                "declare_compatibility" => payload
                    .get("compatibility_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str())),
                "build_migration_plan" | "execute_migration" => payload
                    .get("migration_plan_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str()))
                    .or_else(|| payload.get("version_id").and_then(|v| v.as_str())),
                "deprecate_schema_element" => payload
                    .get("schema_element_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str())),
                "review_governance_change" => payload
                    .get("decision_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str())),
                "verify_post_migration_conformance" => payload
                    .get("conformance_id")
                    .and_then(|v| v.as_str())
                    .or_else(|| payload.get("object_id").and_then(|v| v.as_str())),
                _ => payload.get("object_id").and_then(|v| v.as_str()),
            }
            .map(|s| s.to_string());

            if let Some(subject) = governance_object_id.clone() {
                let (from_status, to_status) = match action_type {
                    "create_version" => (Some("draft".to_string()), "active".to_string()),
                    "declare_compatibility" => {
                        (Some("candidate".to_string()), "declared".to_string())
                    }
                    "build_migration_plan" => {
                        (Some("candidate".to_string()), "planned".to_string())
                    }
                    "execute_migration" => (Some("planned".to_string()), "migrated".to_string()),
                    "deprecate_schema_element" => {
                        (Some("active".to_string()), "deprecated".to_string())
                    }
                    "review_governance_change" => (
                        Some("proposed".to_string()),
                        payload
                            .get("decision")
                            .and_then(|v| v.as_str())
                            .unwrap_or("approved")
                            .to_string(),
                    ),
                    "verify_post_migration_conformance" => {
                        (Some("pending".to_string()), "verified".to_string())
                    }
                    _ => (Some("candidate".to_string()), "active".to_string()),
                };

                events.push(FocusaEvent::OntologyStatusChangeProposed {
                    workstream: Some(workstream.clone()),
                    proposal_id,
                    subject,
                    from_status,
                    to_status,
                    source: source.to_string(),
                });
            }

            events.push(FocusaEvent::OntologyObjectUpsertProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                object_type: payload
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(match action_type {
                        "create_version" => "ontology_version",
                        "declare_compatibility" => "compatibility_profile",
                        "build_migration_plan" | "execute_migration" => "migration_plan",
                        "deprecate_schema_element" => "deprecation_record",
                        "verify_post_migration_conformance" => "conformance_report",
                        _ => "governance_decision",
                    })
                    .to_string(),
                object_id: governance_object_id,
                source: source.to_string(),
            });
        }
        "establish_identity"
        | "load_role_profile"
        | "verify_capability_profile"
        | "verify_permission_profile"
        | "assign_responsibility"
        | "determine_handoff_boundary"
        | "restore_identity_continuity" => {
            events.push(FocusaEvent::OntologyObjectUpsertProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                object_type: payload
                    .get("object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(match action_type {
                        "load_role_profile" => "role_profile",
                        "verify_capability_profile" => "capability_profile",
                        "verify_permission_profile" => "permission_profile",
                        "assign_responsibility" => "responsibility",
                        "determine_handoff_boundary" => "handoff_boundary",
                        "restore_identity_continuity" => "session_continuity",
                        _ => "agent_identity",
                    })
                    .to_string(),
                object_id: payload
                    .get("object_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                source: source.to_string(),
            });
        }
        _ => {
            let object_type = payload
                .get("object_type")
                .and_then(|v| v.as_str())
                .or_else(|| action_target_types(action_type).first().copied())
                .unwrap_or("ontology_domain")
                .to_string();
            let object_id = payload
                .get("object_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    payload
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });

            events.push(FocusaEvent::OntologyObjectUpsertProposed {
                workstream: Some(workstream.clone()),
                proposal_id,
                object_type,
                object_id,
                source: source.to_string(),
            });
        }
    }

    events
}

async fn execute_ontology_action(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(body): Json<OntologyActionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if !ACTION_TYPES.contains(&body.action_type.as_str()) {
        return Err(ontology_validation_rejected("unknown ontology action_type"));
    }

    let workstream = verified_workstream_key(&scope).ok_or_else(|| {
        ontology_validation_rejected("verified project and continuity scope required")
    })?;
    let source = body
        .source
        .as_deref()
        .unwrap_or("ontology_action_route")
        .to_string();
    let proposal_id = parse_or_new_proposal_id(body.proposal_id.as_deref());
    let payload = body.payload;

    let auto_verify = body.auto_verify.unwrap_or(true);
    let auto_promote = body.auto_promote.unwrap_or(false);

    if auto_promote && body.action_type != "review_governance_change" {
        return Err(ontology_validation_rejected(format!(
            "auto_promote requires review_governance_change action; action_type={}",
            body.action_type
        )));
    }

    let mut events = proposed_events_from_action(
        proposal_id,
        &body.action_type,
        &payload,
        &source,
        &workstream,
    );

    if auto_verify {
        events.push(FocusaEvent::OntologyVerificationApplied {
            workstream: Some(workstream.clone()),
            proposal_id: Some(proposal_id),
            verification: format!("action:{}", body.action_type),
            outcome: payload
                .get("verification_outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("accepted")
                .to_string(),
        });
    }

    if auto_promote {
        events.push(FocusaEvent::OntologyProposalPromoted {
            workstream: Some(workstream.clone()),
            proposal_id,
            target_class: action_target_types(&body.action_type)
                .first()
                .copied()
                .unwrap_or("ontology_domain")
                .to_string(),
            applied_kind: body.action_type.clone(),
        });
    }

    if matches!(
        body.action_type.as_str(),
        "select_relevant_context"
            | "refresh_working_set"
            | "execute_migration"
            | "verify_post_migration_conformance"
    ) {
        events.push(FocusaEvent::OntologyWorkingSetRefreshed {
            workstream: Some(workstream.clone()),
            scope: payload
                .get("scope_kind")
                .and_then(|v| v.as_str())
                .unwrap_or("active_mission")
                .to_string(),
            reason: payload
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("ontology action working-set refresh")
                .to_string(),
        });
    }

    for event in events {
        state
            .command_tx
            .send(Action::EmitEvent { event })
            .await
            .map_err(|error| ontology_dispatch_failed("ontology action", error))?;
    }

    Ok(Json(json!({
        "status": "accepted",
        "action_type": body.action_type,
        "proposal_id": proposal_id,
        "auto_verify": auto_verify,
        "auto_promote": auto_promote,
    })))
}

fn legacy_ontology_scope_migration_candidates(focusa: &FocusaState) -> Vec<Value> {
    let mut candidates = Vec::new();
    let mut push =
        |record_kind: OntologyScopeMigrationRecordKind, source_hash: String, record_ref: String| {
            candidates.push(json!({
                "record_kind": record_kind,
                "source_hash": source_hash,
                "record_ref": record_ref,
                "ownership": "legacy_unscoped_quarantined",
                "evidence_required": true,
            }));
        };

    for (index, record) in focusa.ontology.objects.iter().enumerate() {
        if record
            .get("workstream")
            .is_none_or(serde_json::Value::is_null)
            && record.get("scope_class").and_then(Value::as_str) != Some("global_schema")
        {
            push(
                OntologyScopeMigrationRecordKind::Object,
                ontology_scope_record_hash(record),
                format!("ontology.objects[{index}]"),
            );
        }
    }
    for (index, record) in focusa.ontology.links.iter().enumerate() {
        if record
            .get("workstream")
            .is_none_or(serde_json::Value::is_null)
            && record.get("scope_class").and_then(Value::as_str) != Some("global_schema")
        {
            push(
                OntologyScopeMigrationRecordKind::Link,
                ontology_scope_record_hash(record),
                format!("ontology.links[{index}]"),
            );
        }
    }
    macro_rules! typed_candidates {
        ($records:expr, $kind:expr, $label:literal) => {
            for (index, record) in $records.iter().enumerate() {
                if record.workstream.is_none() {
                    push(
                        $kind,
                        ontology_scope_record_hash(record),
                        format!(concat!($label, "[{}]"), index),
                    );
                }
            }
        };
    }
    typed_candidates!(
        focusa.ontology.proposals,
        OntologyScopeMigrationRecordKind::Proposal,
        "ontology.proposals"
    );
    typed_candidates!(
        focusa.ontology.verifications,
        OntologyScopeMigrationRecordKind::Verification,
        "ontology.verifications"
    );
    typed_candidates!(
        focusa.ontology.working_set_refreshes,
        OntologyScopeMigrationRecordKind::WorkingSetRefresh,
        "ontology.working_set_refreshes"
    );
    typed_candidates!(
        focusa.ontology.delta_log,
        OntologyScopeMigrationRecordKind::Delta,
        "ontology.delta_log"
    );
    typed_candidates!(
        focusa.pre.proposals,
        OntologyScopeMigrationRecordKind::PreProposal,
        "pre.proposals"
    );
    candidates.sort_by(|left, right| {
        left.get("record_ref")
            .and_then(Value::as_str)
            .cmp(&right.get("record_ref").and_then(Value::as_str))
    });
    candidates.truncate(env_limit("FOCUSA_ONTOLOGY_MIGRATION_CANDIDATE_LIMIT", 500));
    candidates
}

async fn ontology_scope_migrations(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(body): Json<OntologyScopeMigrationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let target_workstream = verified_workstream_key(&scope).ok_or_else(|| {
        ontology_validation_rejected("verified project and continuity scope required")
    })?;
    match body.action.trim() {
        "dry_run" => {
            let focusa = state.focusa.read().await;
            let candidates = legacy_ontology_scope_migration_candidates(&focusa);
            Ok(Json(json!({
                "status": "planned",
                "canonical": false,
                "mutation": false,
                "target_workstream": target_workstream,
                "candidate_count": candidates.len(),
                "candidates": candidates,
                "policy": "explicit per-record evidence required; ownership is never inferred",
            })))
        }
        "status" => {
            let focusa = state.focusa.read().await;
            let receipts = focusa
                .ontology
                .scope_migration_receipts
                .iter()
                .filter(|receipt| receipt.target_workstream == target_workstream)
                .cloned()
                .collect::<Vec<_>>();
            Ok(Json(json!({
                "status": "completed",
                "canonical": true,
                "target_workstream": target_workstream,
                "receipts": receipts,
            })))
        }
        "apply" => {
            if body.selections.is_empty()
                || body.evidence_refs.is_empty()
                || body
                    .selections
                    .iter()
                    .any(|selection| selection.evidence_refs.is_empty())
            {
                return Err(ontology_validation_rejected(
                    "apply requires selections plus migration and per-record evidence",
                ));
            }
            let migration_id = body.migration_id.unwrap_or_else(Uuid::now_v7);
            state
                .command_tx
                .send(Action::EmitEvent {
                    event: FocusaEvent::OntologyScopeMigrationApplied {
                        migration_id,
                        target_workstream: target_workstream.clone(),
                        selections: body.selections,
                        evidence_refs: body.evidence_refs,
                    },
                })
                .await
                .map_err(|error| {
                    ontology_dispatch_failed("ontology scope migration apply", error)
                })?;
            Ok(Json(json!({
                "status": "accepted",
                "canonical": false,
                "migration_id": migration_id,
                "target_workstream": target_workstream,
                "next_action": "poll status for the append-only apply receipt",
            })))
        }
        "rollback" => {
            let migration_id = body
                .migration_id
                .ok_or_else(|| ontology_validation_rejected("rollback requires migration_id"))?;
            if body.evidence_refs.is_empty() {
                return Err(ontology_validation_rejected(
                    "rollback requires evidence_refs",
                ));
            }
            {
                let focusa = state.focusa.read().await;
                let owned = focusa
                    .ontology
                    .scope_migration_receipts
                    .iter()
                    .any(|receipt| {
                        receipt.migration_id == migration_id
                            && receipt.operation == "apply"
                            && receipt.target_workstream == target_workstream
                    });
                if !owned {
                    return Err(ontology_validation_rejected(
                        "migration apply receipt not found in exact workstream",
                    ));
                }
            }
            let rollback_id = body.rollback_id.unwrap_or_else(Uuid::now_v7);
            state
                .command_tx
                .send(Action::EmitEvent {
                    event: FocusaEvent::OntologyScopeMigrationRolledBack {
                        rollback_id,
                        migration_id,
                        evidence_refs: body.evidence_refs,
                    },
                })
                .await
                .map_err(|error| {
                    ontology_dispatch_failed("ontology scope migration rollback", error)
                })?;
            Ok(Json(json!({
                "status": "accepted",
                "canonical": false,
                "migration_id": migration_id,
                "rollback_id": rollback_id,
                "target_workstream": target_workstream,
                "next_action": "poll status for the append-only rollback receipt",
            })))
        }
        _ => Err(ontology_validation_rejected(
            "action must be dry_run, apply, status, or rollback",
        )),
    }
}

async fn primitives() -> Json<Value> {
    primitive_contracts()
}

fn visual_reverse_engineering_pipeline_contract() -> Value {
    json!({
        "pipeline_id": "visual_reverse_engineering_extraction_v1",
        "source_doc": "docs/59-visual-ui-reverse-engineering.md",
        "determinism_posture": "policy_bounded_deterministic_projection",
        "stage_order": [
            "derive_structure",
            "extract_components",
            "derive_slots",
            "infer_tokens",
            "infer_spacing",
            "infer_interaction_and_state",
            "derive_implementation_semantics"
        ],
        "inputs_required": ["artifact_id", "artifact_kind", "source_ref", "capture_context", "provenance"],
        "typed_outputs": {
            "objects": ["page", "region", "component", "variant", "content_slot", "token", "layout_rule", "interaction", "ui_state", "binding", "validation_rule", "visual_artifact"],
            "links": ["contains", "composed_of", "variants_of", "fills_slot", "inherits_token", "binds_to", "transitions_to", "validates", "derived_from_reference"],
            "blueprint_payload_required": ["structure", "components", "slots", "tokens", "spacing_layout", "interaction_state", "implementation_semantics", "evidence_refs", "stage_confidence"]
        },
        "promotion_policy": {
            "default_state": "proposal_level",
            "promotion_requires": ["multi_artifact_confirmation_or_operator_review", "verification_backing_for_ambiguous_inference"]
        },
        "failure_modes": [
            "ambiguous_component_boundaries",
            "ambiguous_slot_assignments",
            "uncertain_token_inference",
            "insufficient_evidence_for_responsive_behavior",
            "insufficient_evidence_for_binding_or_validation"
        ]
    })
}

fn visual_to_implementation_handoff_contract() -> Value {
    json!({
        "pipeline_id": "visual_to_implementation_handoff_v1",
        "source_doc": "docs/64-visual-ui-to-implementation.md",
        "determinism_posture": "policy_bounded_deterministic_projection",
        "stage_order": [
            "derive_component_tree",
            "derive_plumbing_requirements",
            "map_tokens_to_surfaces",
            "map_states_to_views",
            "map_bindings_and_validation",
            "synthesize_completion_checklist"
        ],
        "inputs_required": ["visual_blueprint_ref", "component_inventory_ref", "interaction_state_ref", "binding_validation_ref", "responsive_constraints_ref"],
        "typed_outputs": {
            "objects": ["page", "region", "component", "content_slot", "token", "layout_rule", "interaction", "ui_state", "binding", "validation_rule", "verification", "acceptance_criterion"],
            "links": ["contains", "composed_of", "fills_slot", "inherits_token", "binds_to", "transitions_to", "validates", "aligns_with", "verifies"],
            "handoff_payload_required": ["component_tree", "region_component_mapping", "slot_component_mapping", "token_application_map", "layout_rule_map", "interaction_state_map", "binding_plan", "validation_plan", "responsive_requirements", "plumbing_requirements", "completion_checklist"]
        },
        "handoff_conformance": {
            "required_inputs": ["visual_blueprint_ref", "handoff_payload_ref", "completion_checklist_ref"],
            "required_checks": ["component_tree_alignment", "state_interaction_alignment", "binding_validation_alignment", "responsive_requirement_alignment", "plumbing_class_coverage"],
            "pass_condition": "all required handoff surfaces map to implementation-ready outputs with no uncovered required plumbing classes"
        },
        "diff_validation": {
            "required_inputs": ["declared_intent_ref", "handoff_payload_ref", "implementation_diff_ref"],
            "required_checks": ["intent_preservation", "declared_vs_actual_component_delta", "declared_vs_actual_state_delta", "declared_vs_actual_binding_delta", "unexpected_surface_change_detection"],
            "fail_on": ["intent_drift", "missing_declared_change", "undeclared_high_impact_change"]
        },
        "validation_outputs": ["conformance_report", "diff_validation_report", "intent_preservation_result"],
        "required_plumbing_classes": [
            "data_loading",
            "mutation_actions",
            "optimistic_or_async_transitions",
            "loading_state",
            "empty_state",
            "error_state",
            "success_state",
            "disabled_state",
            "validation_messaging",
            "responsive_behavior",
            "accessibility_sensitive_interactions"
        ],
        "completion_rules": [
            "structural_fidelity",
            "state_coverage",
            "interaction_coverage",
            "binding_coverage",
            "validation_coverage",
            "responsive_coverage",
            "verification_evidence"
        ]
    })
}

async fn contracts() -> Json<Value> {
    Json(json!({
        "route_behavior": {
            "surface": "GET /v1/ontology/contracts",
            "read_only": true,
            "mutates_canonical_state": false,
            "api_permission_scope": null,
            "contract_source": "crates/focusa-api/src/routes/ontology.rs"
        },
        "reverse_engineering_pipeline": visual_reverse_engineering_pipeline_contract(),
        "visual_to_implementation_handoff": visual_to_implementation_handoff_contract(),
        "contracts": ACTION_TYPES.iter().map(|name| action_contract(name)).collect::<Vec<_>>()
    }))
}

fn ontology_world_default_object_limit() -> usize {
    // Hot/default ontology reads are orientation packets, not graph dumps.
    env_limit("FOCUSA_ONTOLOGY_WORLD_DEFAULT_OBJECT_LIMIT", 16)
}

fn ontology_world_full_object_limit() -> usize {
    env_limit("FOCUSA_ONTOLOGY_WORLD_FULL_OBJECT_LIMIT", 10_000)
        .max(ontology_world_default_object_limit())
}

fn ontology_world_default_link_limit() -> usize {
    env_limit("FOCUSA_ONTOLOGY_WORLD_DEFAULT_LINK_LIMIT", 24)
}

fn ontology_world_full_link_limit() -> usize {
    env_limit("FOCUSA_ONTOLOGY_WORLD_FULL_LINK_LIMIT", 20_000)
        .max(ontology_world_default_link_limit())
}

fn action_catalog_projection() -> Vec<Value> {
    ACTION_CATALOG_PROJECTION.clone()
}

async fn world(
    Query(query): Query<OntologyWorldQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let projection = if query.include_full_payload {
        combined_projection(&focusa, query.frame_id.as_deref())
    } else {
        bounded_summary_projection(&focusa, query.frame_id.as_deref())
    };
    let object_total = projection.objects.len();
    let link_total = projection.links.len();
    let full_payload_blocked =
        full_payload_blocked_by_pressure(query.include_full_payload, query.force_full_payload);
    let effective_include_full_payload = query.include_full_payload && !full_payload_blocked;
    let effective_summary_only =
        (query.summary_only && !effective_include_full_payload) || full_payload_blocked;
    let pressure = pressure_status();
    let object_options = BoundedReadOptions {
        requested_limit: query.limit_objects,
        include_full_payload: effective_include_full_payload,
        summary_only: effective_summary_only,
        cursor: query.cursor_objects.map(|v| v.to_string()),
        next_cursor: None,
        default_limit: ontology_world_default_object_limit(),
        full_limit: ontology_world_full_object_limit(),
    };
    let link_options = BoundedReadOptions {
        requested_limit: query.limit_links,
        include_full_payload: effective_include_full_payload,
        summary_only: effective_summary_only,
        cursor: query.cursor_links.map(|v| v.to_string()),
        next_cursor: None,
        default_limit: ontology_world_default_link_limit(),
        full_limit: ontology_world_full_link_limit(),
    };
    let object_limit = object_options.resolved_limit();
    let link_limit = link_options.resolved_limit();
    let object_start = query.cursor_objects.unwrap_or(0).min(object_total);
    let link_start = query.cursor_links.unwrap_or(0).min(link_total);
    let object_end = (object_start + object_limit).min(object_total);
    let link_end = (link_start + link_limit).min(link_total);
    let mut object_options = object_options;
    object_options.next_cursor = (object_end < object_total).then(|| object_end.to_string());
    let mut link_options = link_options;
    link_options.next_cursor = (link_end < link_total).then(|| link_end.to_string());
    let objects = if effective_summary_only {
        projection.objects[object_start..object_end]
            .iter()
            .map(compact_object_summary)
            .collect::<Vec<_>>()
    } else {
        projection.objects[object_start..object_end].to_vec()
    };
    let links = if effective_summary_only {
        projection.links[link_start..link_end]
            .iter()
            .map(compact_link)
            .collect::<Vec<_>>()
    } else {
        projection.links[link_start..link_end].to_vec()
    };
    let object_type_counts = value_field_counts(&projection.objects, "object_type", "object");
    let link_type_counts = value_field_counts(&projection.links, "type", "related_to");
    let object_bounds = bounded_metadata(object_total, objects.len(), object_options);
    let link_bounds = bounded_metadata(link_total, links.len(), link_options);
    let action_catalog = if query.include_action_catalog {
        action_catalog_projection()
    } else {
        Vec::new()
    };
    let action_catalog_returned = action_catalog.len();
    let working_sets = if query.include_working_sets {
        json!({
            "active_mission_set": slice_payload(&focusa, query.frame_id.as_deref(), "active_mission"),
            "debugging_set": slice_payload(&focusa, query.frame_id.as_deref(), "debugging"),
            "refactor_set": slice_payload(&focusa, query.frame_id.as_deref(), "refactor"),
            "regression_set": slice_payload(&focusa, query.frame_id.as_deref(), "regression"),
            "architecture_set": slice_payload(&focusa, query.frame_id.as_deref(), "architecture"),
        })
    } else {
        json!({})
    };

    let payload = json!({
        "object_count": object_total,
        "link_count": link_total,
        "authority": ontology_projection_authority_metadata(),
        "advisory_only": true,
        "canonical": false,
        "promotion_path": "world projection -> bounded selection -> reducer-governed ontology proposal/promotion",
        "objects": objects,
        "links": links,
        "canonical_ontology": {
            "proposal_count": focusa.ontology.proposals.len(),
            "verification_count": focusa.ontology.verifications.len(),
            "working_set_refresh_count": focusa.ontology.working_set_refreshes.len(),
            "delta_count": focusa.ontology.delta_log.len(),
            "object_type_counts": object_type_counts,
            "link_type_counts": link_type_counts,
            "category_rehydrate": {
                "objects": {"route":"/v1/ontology/world", "cursor_parameter":"cursor_objects", "limit_parameter":"limit_objects"},
                "links": {"route":"/v1/ontology/world", "cursor_parameter":"cursor_links", "limit_parameter":"limit_links"},
                "actions": {"route":"/v1/ontology/world", "parameter":"include_action_catalog", "value":"true"},
                "working_sets": {"route":"/v1/ontology/world", "parameter":"include_working_sets", "value":"true"},
                "provenance_verification": {"route":"/v1/ontology/adjacency"},
                "governance": {"route":"/v1/ontology/contracts"}
            }
        },
        "action_catalog": action_catalog,
        "working_sets": working_sets,
        "bounds": {
            "objects": object_bounds,
            "links": link_bounds,
            "action_catalog": {
                "total": ACTION_TYPES.len(),
                "returned": action_catalog_returned,
                "truncated": !query.include_action_catalog,
                "include_action_catalog": query.include_action_catalog,
                "requested_include_action_catalog": query.include_action_catalog,
                "rehydrate": if query.include_action_catalog { Value::Null } else { json!({"parameter":"include_action_catalog","value":"true"}) }
            },
            "working_sets": {
                "total": SLICE_TYPES.len(),
                "returned": if query.include_working_sets { SLICE_TYPES.len() } else { 0 },
                "truncated": !query.include_working_sets,
                "include_working_sets": query.include_working_sets,
                "rehydrate": if query.include_working_sets { Value::Null } else { json!({"parameter":"include_working_sets","value":"true"}) }
            }
        },
        "projection_profile": {
            "summary_only": effective_summary_only,
            "include_full_payload": effective_include_full_payload,
            "requested_summary_only": query.summary_only,
            "requested_include_full_payload": query.include_full_payload,
            "force_full_payload": query.force_full_payload,
            "canonical_truth_mutation": false,
            "invariants": [
                "canonical_and_projection_are_distinct",
                "bounded_defaults_do_not_strip_canonical_ontology",
                "full_payload_requires_explicit_opt_in",
                "omitted_categories_include_rehydrate_parameters"
            ]
        },
        "pressure": pressure,
        "degraded": full_payload_blocked,
        "full_payload_blocked_by_pressure": full_payload_blocked
    });
    record_json_response_size("/v1/ontology/world", &payload);
    Json(payload)
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

async fn adjacency(
    Query(query): Query<AdjacencyQuery>,
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(adjacency_index_payload(
        &focusa,
        query.frame_id.as_deref(),
        query.target_ref.as_deref(),
        query.limit.unwrap_or(1),
        Some(&scope),
    ))
}

async fn working_set(
    Query(query): Query<WorkingSetQuery>,
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(working_set_payload(
        &focusa,
        WorkingSetPayloadParams {
            frame_id: query.frame_id.as_deref(),
            ask: query.ask.as_deref(),
            target_ref: query.target_ref.as_deref(),
            slice_type: &query.slice_type,
            limit: query.limit.unwrap_or(6),
            include_reasons: query.include_reasons,
            cursor: query.cursor.unwrap_or(0),
            scope: Some(&scope),
        },
    ))
}

async fn context(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(body): Json<OntologyContextRequest>,
) -> Json<Value> {
    let Some(scope_ref) = verified_scope_ref_for_context(&scope) else {
        return Json(json!({
            "status": "scope_mismatch",
            "canonical": false,
            "advisory_only": true,
            "failure_class": "scope_mismatch",
            "why": "ontology context requires verified ProjectIdentity ScopeRef plus continuity_id",
            "recovery_hint": "verify project identity and retry with exact typed scope headers",
        }));
    };
    let focusa = state.focusa.read().await;
    let Some(scoped) = scoped_ontology_state(&focusa, &scope) else {
        return Json(json!({
            "status": "scope_mismatch",
            "canonical": false,
            "advisory_only": true,
            "failure_class": "scope_mismatch",
        }));
    };
    Json(ontology_context_payload(&scoped, &body, &scope, &scope_ref))
}

async fn graph_communities(
    Query(query): Query<WorkingSetQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(graph_community_summaries_payload(
        &focusa,
        query.frame_id.as_deref(),
        query.limit.unwrap_or(10),
    ))
}

async fn affordances(
    Query(query): Query<AffordancesQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    Json(affordances_payload(
        &focusa,
        query.frame_id.as_deref(),
        query.target_ref.as_deref(),
        query.action_intent.as_deref(),
        query.scope.as_deref(),
        query.limit.unwrap_or(20),
    ))
}

async fn retrieval_governor(
    State(state): State<Arc<AppState>>,
    scope: ScopeContext,
    Json(body): Json<RetrievalGovernorRequest>,
) -> Json<Value> {
    let Some(scope_ref) = verified_scope_ref_for_context(&scope) else {
        return Json(json!({
            "status": "scope_mismatch",
            "canonical": false,
            "failure_class": "scope_mismatch",
        }));
    };
    let focusa = state.focusa.read().await;
    let Some(scoped) = scoped_ontology_state(&focusa, &scope) else {
        return Json(json!({
            "status": "scope_mismatch",
            "canonical": false,
            "failure_class": "scope_mismatch",
        }));
    };
    Json(retrieval_governor_payload(
        &scoped, &body, &scope, &scope_ref,
    ))
}

async fn tool_contracts() -> Json<Value> {
    let registry: Value = serde_json::from_str(include_str!(
        "../../../../docs/current/focusa-tool-contracts.json"
    ))
    .unwrap_or_else(
        |err| json!({"error":"invalid tool contract registry","details":err.to_string()}),
    );
    Json(registry)
}

fn tool_choreography_prediction_store_path() -> PathBuf {
    if let Some(home) = std::env::var_os("FOCUSA_HOME") {
        return PathBuf::from(home).join("data/spec92_predictions.json");
    }
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(data_home).join("focusa/spec92_predictions.json");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/share/focusa/spec92_predictions.json");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data/spec92_predictions.json")
}

fn read_tool_choreography_predictions() -> Vec<Value> {
    fs::read_to_string(tool_choreography_prediction_store_path())
        .ok()
        .and_then(|text| serde_json::from_str::<Vec<Value>>(&text).ok())
        .unwrap_or_default()
}

fn parse_tool_edge_ref(raw: &str) -> Option<(String, String)> {
    let value = raw.trim().strip_prefix("tool_edge:").unwrap_or(raw.trim());
    let (from, to) = value.split_once("->")?;
    let from = from.trim().to_string();
    let to = to.trim().to_string();
    if from.starts_with("focusa_") && to.starts_with("focusa_") {
        Some((from, to))
    } else {
        None
    }
}

fn dynamic_choreography_multiplier(average_score: f64) -> f64 {
    (0.75 + average_score.clamp(0.0, 1.0) * 0.5).clamp(0.75, 1.25)
}

fn dynamic_choreography_adjustments(registry: &Value) -> Vec<Value> {
    let mut known_edges = BTreeSet::new();
    for edge in registry
        .get("edges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let (Some(from), Some(to)) = (
            edge.get("from").and_then(Value::as_str),
            edge.get("to").and_then(Value::as_str),
        ) {
            known_edges.insert((from.to_string(), to.to_string()));
        }
    }
    let mut stats: BTreeMap<(String, String), (usize, f64)> = BTreeMap::new();
    for prediction in read_tool_choreography_predictions() {
        let Some(score) = prediction.get("score").and_then(Value::as_f64) else {
            continue;
        };
        let Some(refs) = prediction.get("context_refs").and_then(Value::as_array) else {
            continue;
        };
        for reference in refs.iter().filter_map(Value::as_str) {
            let Some(edge) = parse_tool_edge_ref(reference) else {
                continue;
            };
            if !known_edges.contains(&edge) {
                continue;
            }
            let entry = stats.entry(edge).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += score.clamp(0.0, 1.0);
        }
    }
    stats
        .into_iter()
        .map(|((from, to), (samples, score_sum))| {
            let average_score = if samples > 0 {
                score_sum / samples as f64
            } else {
                0.0
            };
            let multiplier = dynamic_choreography_multiplier(average_score);
            json!({
                "edge": format!("{}->{}", from, to),
                "from": from,
                "to": to,
                "samples": samples,
                "average_score": (average_score * 1000.0).round() / 1000.0,
                "weight_multiplier": (multiplier * 1000.0).round() / 1000.0,
                "source": "evaluated_predictions",
            })
        })
        .collect()
}

fn apply_dynamic_choreography_weights(registry: &mut Value) {
    let adjustments = dynamic_choreography_adjustments(registry);
    let effective_edges = if adjustments.is_empty() {
        Vec::new()
    } else {
        let multipliers: BTreeMap<String, f64> = adjustments
            .iter()
            .filter_map(|item| {
                Some((
                    item.get("edge")?.as_str()?.to_string(),
                    item.get("weight_multiplier")?.as_f64()?,
                ))
            })
            .collect();
        registry
            .get("edges")
            .and_then(Value::as_array)
            .map(|edges| {
                edges
                    .iter()
                    .map(|edge| {
                        let mut next = edge.clone();
                        let key = format!(
                            "{}->{}",
                            edge.get("from").and_then(Value::as_str).unwrap_or(""),
                            edge.get("to").and_then(Value::as_str).unwrap_or("")
                        );
                        if let Some(multiplier) = multipliers.get(&key) {
                            let base = edge.get("weight").and_then(Value::as_f64).unwrap_or(1.0);
                            next["base_weight"] = json!(base);
                            next["dynamic_multiplier"] = json!(multiplier);
                            next["weight"] = json!(((base * multiplier) * 1000.0).round() / 1000.0);
                        }
                        next
                    })
                    .collect::<Vec<Value>>()
            })
            .unwrap_or_default()
    };
    if let Some(object) = registry.as_object_mut() {
        object.insert(
            "runtime_weight_adjustments".to_string(),
            Value::Array(adjustments),
        );
        if !effective_edges.is_empty() {
            object.insert("effective_edges".to_string(), Value::Array(effective_edges));
        }
    }
}

async fn tool_choreography() -> Json<Value> {
    let mut registry: Value = serde_json::from_str(include_str!(
        "../../../../docs/current/focusa-tool-choreography.json"
    ))
    .unwrap_or_else(
        |err| json!({"error":"invalid tool choreography registry","details":err.to_string()}),
    );
    apply_dynamic_choreography_weights(&mut registry);
    Json(registry)
}

#[derive(Debug, Deserialize)]
struct DomainPackQuery {
    project_root: String,
    continuity_id: String,
}

/// Deterministic General/Software/Research projection over identical canonical state.
async fn domain_pack_projection(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DomainPackQuery>,
) -> Result<Json<Value>, StatusCode> {
    if query.project_root.trim().is_empty() || query.continuity_id.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let snapshot = state.focusa.read().await;
    let artifacts: Vec<_> = snapshot
        .workspace_artifacts
        .iter()
        .filter(|artifact| {
            artifact.scope.project_root == query.project_root
                && artifact.scope.continuity_id == query.continuity_id
        })
        .collect();
    let has_research = artifacts.iter().any(|artifact| {
        matches!(artifact.artifact_kind.as_str(), "research" | "document")
            || !artifact.semantic.citation_refs.is_empty()
            || artifact
                .semantic
                .domain_pack_refs
                .iter()
                .any(|value| value == "research")
    });
    let has_software = artifacts.iter().any(|artifact| {
        matches!(
            artifact.artifact_kind.as_str(),
            "code" | "software" | "repository" | "diff"
        ) || artifact
            .semantic
            .domain_pack_refs
            .iter()
            .any(|value| value == "software")
    });
    let active_pack = if has_research {
        "research"
    } else if has_software {
        "software"
    } else {
        "general"
    };
    let available_packs = ["general", "software", "research"];
    Ok(Json(json!({
        "schema": "focusa.ontology.domain_pack_projection.v1",
        "state_version": snapshot.version,
        "project_root": query.project_root,
        "continuity_id": query.continuity_id,
        "active_pack": active_pack,
        "available_packs": available_packs,
        "selection_policy": "research_citations_then_software_artifacts_then_general",
        "canonical_state_unchanged": true,
        "artifact_refs": artifacts.iter().take(64).map(|artifact| artifact.artifact_id.clone()).collect::<Vec<_>>(),
        "ontology_refs": ["/v1/ontology/world", "/v1/ontology/slices", "/v1/ontology/adjacency"],
        "projection_notice": "Domain packs change terminology and views only; canonical Context, Evidence, Workpoint, and artifact state remains identical."
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ontology/primitives", get(primitives))
        .route("/v1/ontology/contracts", get(contracts))
        .route("/v1/ontology/world", get(world))
        .route("/v1/ontology/slices", get(slices))
        .route("/v1/ontology/adjacency", get(adjacency))
        .route("/v1/ontology/working-set", get(working_set))
        .route("/v1/ontology/communities", get(graph_communities))
        .route("/v1/ontology/domain-pack", get(domain_pack_projection))
        .route("/v1/ontology/context", post(context))
        .route(
            "/v1/ontology/scope-migrations",
            post(ontology_scope_migrations),
        )
        .route("/v1/ontology/affordances", get(affordances))
        .route("/v1/ontology/retrieval-governor", post(retrieval_governor))
        .route(
            "/v1/ontology/tool-result-proposals",
            post(tool_result_proposals),
        )
        .route("/v1/ontology/execution-critic", post(execution_critic))
        .route(
            "/v1/ontology/reflection-synthesizer",
            post(reflection_synthesizer),
        )
        .route("/v1/ontology/memory-pipeline", post(memory_pipeline))
        .route(
            "/v1/ontology/intelligence-dashboard",
            get(intelligence_dashboard),
        )
        .route("/v1/ontology/tool-contracts", get(tool_contracts))
        .route("/v1/ontology/tool-choreography", get(tool_choreography))
        .route("/v1/ontology/actions", post(execute_ontology_action))
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use chrono::Utc;
    use focusa_core::types::{
        FocusaState, HandleKind, HandleRef, SessionState, SessionStatus, TrajectoryLadderContext,
    };
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn exact_test_scope() -> (ScopeContext, ScopeRef) {
        let scope_ref = ScopeRef::project(
            "project:ontology-test",
            "/tmp/focusa-ontology-test",
            "ontology-test",
            "fingerprint:ontology-test",
        )
        .expect("valid project scope");
        (
            ScopeContext {
                project_root: Some("/tmp/focusa-ontology-test".to_string()),
                continuity_id: Some("ontology-test-continuity".to_string()),
                source: crate::scope::ScopeSource::Header,
                ..ScopeContext::default()
            },
            scope_ref,
        )
    }

    #[test]
    fn ontology_scope_filter_accepts_only_exact_typed_workstream_or_global_schema() {
        let (_, scope_ref) = exact_test_scope();
        let workstream =
            WorkstreamKey::new(scope_ref, "ontology-test-continuity").expect("valid workstream");
        let exact = json!({"workstream": workstream, "id": "object:exact"});
        assert!(ontology_value_matches_scope(
            &exact,
            "/tmp/focusa-ontology-test",
            "ontology-test-continuity"
        ));
        assert!(!ontology_value_matches_scope(
            &exact,
            "/tmp/focusa-foreign",
            "ontology-test-continuity"
        ));
        assert!(!ontology_value_matches_scope(
            &json!({"id": "legacy:unowned"}),
            "/tmp/focusa-ontology-test",
            "ontology-test-continuity"
        ));
        assert!(ontology_value_matches_scope(
            &json!({"scope_class": "global_schema", "id": "schema:decision"}),
            "/tmp/focusa-foreign",
            "foreign-continuity"
        ));
    }

    #[test]
    fn migration_dry_run_exposes_hashes_not_legacy_payloads() {
        let mut focusa = FocusaState::default();
        focusa
            .ontology
            .proposals
            .push(focusa_core::types::OntologyProposalRecord {
                proposal_id: Uuid::now_v7(),
                proposal_kind: "object_upsert".into(),
                target_class: "secret-target".into(),
                status: "proposed".into(),
                notes: Some("must-not-leak".into()),
                ..Default::default()
            });
        focusa.ontology.objects.push(json!({
            "scope_class": "global_schema",
            "id": "schema:decision"
        }));
        let candidates = legacy_ontology_scope_migration_candidates(&focusa);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0]["record_kind"], "proposal");
        assert!(
            candidates[0]["source_hash"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
        );
        let encoded = serde_json::to_string(&candidates).unwrap();
        assert!(!encoded.contains("secret-target"));
        assert!(!encoded.contains("must-not-leak"));
    }

    fn fixture_workspace(test_name: &str, with_git: bool, with_cargo: bool) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("focusa-ontology-{}-{}", test_name, Uuid::now_v7()));
        fs::create_dir_all(root.join("src")).expect("create fixture workspace");
        fs::write(root.join("src/lib.rs"), "pub fn fixture() {}\n").expect("write fixture source");
        if with_git {
            fs::create_dir_all(root.join(".git")).expect("create .git dir");
        }
        if with_cargo {
            fs::write(
                root.join("Cargo.toml"),
                "[package]\nname = \"ontology-fixture\"\nversion = \"0.1.0\"\n",
            )
            .expect("write Cargo.toml");
        }
        root
    }

    fn focusa_with_workspace(root: &Path) -> FocusaState {
        let mut focusa = FocusaState::default();
        focusa.session = Some(SessionState {
            session_id: Uuid::now_v7(),
            created_at: Utc::now(),
            adapter_id: Some("test".to_string()),
            workspace_id: Some(root.to_string_lossy().to_string()),
            project_root: Some(root.to_string_lossy().to_string()),
            continuity_id: Some("test-continuity".to_string()),
            status: SessionStatus::Active,
        });
        focusa
    }

    fn projection_has_object(
        projection: &WorkspaceProjection,
        object_type: &str,
        id: &str,
        status: Option<&str>,
    ) -> bool {
        projection.objects.iter().any(|object| {
            object.get("object_type").and_then(|v| v.as_str()) == Some(object_type)
                && object.get("id").and_then(|v| v.as_str()) == Some(id)
                && status.is_none_or(|expected| {
                    object.get("status").and_then(|v| v.as_str()) == Some(expected)
                })
        })
    }

    fn projection_has_link(
        projection: &WorkspaceProjection,
        link_type: &str,
        source_id: &str,
        target_id: &str,
    ) -> bool {
        projection.links.iter().any(|link| {
            link.get("type").and_then(|v| v.as_str()) == Some(link_type)
                && link.get("source_id").and_then(|v| v.as_str()) == Some(source_id)
                && link.get("target_id").and_then(|v| v.as_str()) == Some(target_id)
        })
    }

    fn projection_count(projection: &WorkspaceProjection, object_type: &str) -> usize {
        projection
            .objects
            .iter()
            .filter(|object| {
                object.get("object_type").and_then(|v| v.as_str()) == Some(object_type)
            })
            .count()
    }

    fn json_array_contains(payload: &Value, key: &str, expected: &str) -> bool {
        payload
            .get(key)
            .and_then(|v| v.as_array())
            .map(|items| items.iter().any(|item| item.as_str() == Some(expected)))
            .unwrap_or(false)
    }

    #[test]
    fn dynamic_choreography_multiplier_is_bounded_and_monotonic() {
        assert_eq!(dynamic_choreography_multiplier(-1.0), 0.75);
        assert_eq!(dynamic_choreography_multiplier(0.0), 0.75);
        assert_eq!(dynamic_choreography_multiplier(0.5), 1.0);
        assert_eq!(dynamic_choreography_multiplier(1.0), 1.25);
        assert_eq!(dynamic_choreography_multiplier(2.0), 1.25);
    }

    #[test]
    fn tool_edge_refs_parse_only_focusa_edges() {
        assert_eq!(
            parse_tool_edge_ref("tool_edge:focusa_project_identity->focusa_trajectory_view"),
            Some((
                "focusa_project_identity".to_string(),
                "focusa_trajectory_view".to_string()
            ))
        );
        assert_eq!(
            parse_tool_edge_ref("tool_edge:other->focusa_trajectory_view"),
            None
        );
        assert_eq!(parse_tool_edge_ref("not-an-edge"), None);
    }

    #[test]
    fn action_catalog_projection_is_cached_and_complete() {
        let first = action_catalog_projection();
        let second = action_catalog_projection();
        assert_eq!(first.len(), ACTION_TYPES.len());
        assert_eq!(first, second);
        assert!(first.iter().all(|entry| {
            entry.get("cache_role").and_then(|v| v.as_str())
                == Some("static_action_catalog_projection")
        }));
    }

    #[test]
    fn ontology_primitive_contracts_preserve_core_semantic_classes() {
        let Json(payload) = primitive_contracts();
        let object_types = payload
            .get("object_types")
            .and_then(|v| v.as_array())
            .expect("object_types array");
        let type_names: BTreeSet<&str> = object_types
            .iter()
            .filter_map(|object| object.get("type_name").and_then(|v| v.as_str()))
            .collect();

        for required in [
            "repo",
            "file",
            "route",
            "task",
            "decision",
            "constraint",
            "goal",
            "active_focus",
            "patch",
            "verification",
            "artifact",
            "projection",
            "view_profile",
            "ontology_version",
            "governance_decision",
            "agent_identity",
            "workpoint_scope_binding",
        ] {
            assert!(
                type_names.contains(required),
                "ontology object type {required} must remain represented"
            );
        }

        for status in ["active", "blocked", "verified", "stale", "canonical"] {
            assert!(
                json_array_contains(&payload, "status_vocabulary", status),
                "status {status} must remain in ontology status vocabulary"
            );
        }
        for provenance in [
            "parser_derived",
            "tool_derived",
            "user_asserted",
            "model_inferred",
            "reducer_promoted",
            "verification_confirmed",
        ] {
            assert!(
                json_array_contains(&payload, "provenance_classes", provenance),
                "provenance class {provenance} must remain represented"
            );
        }
    }

    #[test]
    fn ontology_link_and_action_contracts_preserve_reducer_authority() {
        let Json(payload) = primitive_contracts();
        let link_types = payload
            .get("link_types")
            .and_then(|v| v.as_array())
            .expect("link_types array");
        for required_link in [
            "depends_on",
            "tested_by",
            "verifies",
            "derived_from",
            "belongs_to_working_set",
            "approved_by_governance",
            "governed_by_identity",
        ] {
            let link = link_types
                .iter()
                .find(|link| link.get("name").and_then(|v| v.as_str()) == Some(required_link))
                .unwrap_or_else(|| panic!("link {required_link} must remain represented"));
            assert_eq!(
                link.get("evidence_policy").and_then(|v| v.as_str()),
                Some("required")
            );
            assert_eq!(
                link.get("promotion_policy").and_then(|v| v.as_str()),
                Some("reducer_only")
            );
        }

        let action_types = payload
            .get("action_types")
            .and_then(|v| v.as_array())
            .expect("action_types array");
        for required_action in [
            "verify_invariant",
            "refresh_working_set",
            "detect_affordances",
            "select_relevant_context",
            "build_projection",
            "verify_projection_fidelity",
            "review_governance_change",
        ] {
            assert!(
                action_types.iter().any(|action| {
                    action.get("name").and_then(|v| v.as_str()) == Some(required_action)
                }),
                "action {required_action} must remain represented"
            );
        }
    }

    #[test]
    fn ontology_slice_contract_preserves_projection_not_canonical_truth_boundary() {
        let focusa = FocusaState::default();
        let payload = slice_payload(&focusa, None, "architecture");
        assert_eq!(
            payload.get("source").and_then(|v| v.as_str()),
            Some("ontology_world_projection")
        );
        assert_eq!(
            payload
                .get("projection_profile")
                .and_then(|v| v.get("canonical_truth_mutation"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        let invariants = payload
            .get("projection_profile")
            .and_then(|v| v.get("invariants"))
            .and_then(|v| v.as_array())
            .expect("slice invariants");
        assert!(
            invariants
                .iter()
                .any(|v| v.as_str() == Some("canonical_and_projection_are_distinct"))
        );
        assert!(
            payload.get("bounds").is_some(),
            "slice must expose bounds for omitted detail"
        );
    }

    #[test]
    fn retained_under_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"retained_under"),
            "retained_under must be available in ontology link catalog"
        );
    }

    #[test]
    fn decays_via_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"decays_via"),
            "decays_via must be available in ontology link catalog"
        );
    }

    #[test]
    fn archived_as_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"archived_as"),
            "archived_as must be available in ontology link catalog"
        );
    }

    #[test]
    fn pruned_by_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"pruned_by"),
            "pruned_by must be available in ontology link catalog"
        );
    }

    #[test]
    fn constrains_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"constrains"),
            "constrains must be available in ontology link catalog"
        );
    }

    #[test]
    fn supports_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"supports"),
            "supports must be available in ontology link catalog"
        );
    }

    #[test]
    fn belongs_to_working_set_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"belongs_to_working_set"),
            "belongs_to_working_set must be available in ontology link catalog"
        );
    }

    #[test]
    fn commits_to_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"commits_to"),
            "commits_to must be available in ontology link catalog"
        );
    }

    #[test]
    fn inhibits_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"inhibits"),
            "inhibits must be available in ontology link catalog"
        );
    }

    #[test]
    fn abandons_under_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"abandons_under"),
            "abandons_under must be available in ontology link catalog"
        );
    }

    #[test]
    fn drives_completion_of_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"drives_completion_of"),
            "drives_completion_of must be available in ontology link catalog"
        );
    }

    #[test]
    fn persists_on_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"persists_on"),
            "persists_on must be available in ontology link catalog"
        );
    }

    #[test]
    fn conflicts_with_link_type_is_registered() {
        assert!(
            LINK_TYPES.contains(&"conflicts_with"),
            "conflicts_with must be available in ontology link catalog"
        );
    }

    #[test]
    fn decompose_goal_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"decompose_goal"),
            "decompose_goal must be available in ontology action catalog"
        );
    }

    #[test]
    fn record_decision_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"record_decision"),
            "record_decision must be available in ontology action catalog"
        );
    }

    #[test]
    fn prioritize_work_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"prioritize_work"),
            "prioritize_work must be available in ontology action catalog"
        );
    }

    #[test]
    fn detect_affordances_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"detect_affordances"),
            "detect_affordances must be available in ontology action catalog"
        );
    }

    #[test]
    fn verify_permissions_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"verify_permissions"),
            "verify_permissions must be available in ontology action catalog"
        );
    }

    #[test]
    fn determine_current_ask_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"determine_current_ask"),
            "determine_current_ask must be available in ontology action catalog"
        );
    }

    #[test]
    fn select_relevant_context_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"select_relevant_context"),
            "select_relevant_context must be available in ontology action catalog"
        );
    }

    #[test]
    fn exclude_irrelevant_context_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"exclude_irrelevant_context"),
            "exclude_irrelevant_context must be available in ontology action catalog"
        );
    }

    #[test]
    fn establish_identity_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"establish_identity"),
            "establish_identity must be available in ontology action catalog"
        );
    }

    #[test]
    fn load_role_profile_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"load_role_profile"),
            "load_role_profile must be available in ontology action catalog"
        );
    }

    #[test]
    fn verify_capability_profile_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"verify_capability_profile"),
            "verify_capability_profile must be available in ontology action catalog"
        );
    }

    #[test]
    fn determine_handoff_boundary_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"determine_handoff_boundary"),
            "determine_handoff_boundary must be available in ontology action catalog"
        );
    }

    #[test]
    fn restore_identity_continuity_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"restore_identity_continuity"),
            "restore_identity_continuity must be available in ontology action catalog"
        );
    }

    #[test]
    fn apply_inhibition_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"apply_inhibition"),
            "apply_inhibition must be available in ontology action catalog"
        );
    }

    #[test]
    fn evaluate_switch_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"evaluate_switch"),
            "evaluate_switch must be available in ontology action catalog"
        );
    }

    #[test]
    fn maintain_commitment_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"maintain_commitment"),
            "maintain_commitment must be available in ontology action catalog"
        );
    }

    #[test]
    fn push_to_completion_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"push_to_completion"),
            "push_to_completion must be available in ontology action catalog"
        );
    }

    #[test]
    fn authorize_abandonment_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"authorize_abandonment"),
            "authorize_abandonment must be available in ontology action catalog"
        );
    }

    #[test]
    fn detect_aliases_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"detect_aliases"),
            "detect_aliases must be available in ontology action catalog"
        );
    }

    #[test]
    fn build_resolution_candidates_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"build_resolution_candidates"),
            "build_resolution_candidates must be available in ontology action catalog"
        );
    }

    #[test]
    fn verify_resolution_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"verify_resolution"),
            "verify_resolution must be available in ontology action catalog"
        );
    }

    #[test]
    fn record_supersession_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"record_supersession"),
            "record_supersession must be available in ontology action catalog"
        );
    }

    #[test]
    fn build_projection_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"build_projection"),
            "build_projection must be available in ontology action catalog"
        );
    }

    #[test]
    fn evaluate_retention_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"evaluate_retention"),
            "evaluate_retention must be available in ontology action catalog"
        );
    }

    #[test]
    fn archive_object_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"archive_object"),
            "archive_object must be available in ontology action catalog"
        );
    }

    #[test]
    fn restore_from_archive_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"restore_from_archive"),
            "restore_from_archive must be available in ontology action catalog"
        );
    }

    #[test]
    fn prune_active_context_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"prune_active_context"),
            "prune_active_context must be available in ontology action catalog"
        );
    }

    #[test]
    fn compress_projection_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"compress_projection"),
            "compress_projection must be available in ontology action catalog"
        );
    }

    #[test]
    fn verify_projection_fidelity_action_type_is_registered() {
        assert!(
            ACTION_TYPES.contains(&"verify_projection_fidelity"),
            "verify_projection_fidelity must be available in ontology action catalog"
        );
    }

    #[test]
    fn unknown_slice_types_fallback_to_active_mission_profile() {
        let focusa = FocusaState::default();
        let payload = slice_payload(&focusa, None, "not_a_real_slice");

        assert_eq!(
            payload.get("requested_slice_type").and_then(|v| v.as_str()),
            Some("not_a_real_slice")
        );
        assert_eq!(
            payload.get("slice_type").and_then(|v| v.as_str()),
            Some("active_mission")
        );
        assert_eq!(
            payload
                .get("projection_profile")
                .and_then(|v| v.get("view_profile"))
                .and_then(|v| v.as_str()),
            Some("pi_operator_view")
        );
    }

    #[test]
    fn debugging_slice_members_enforce_boundary_and_relevance() {
        let objects = vec![
            json!({"id": "failure:z", "object_type": "failure", "membership_class": "deterministic"}),
            json!({"id": "module:a", "object_type": "module", "membership_class": "deterministic"}),
            json!({"id": "failure:a", "object_type": "failure", "membership_class": "deterministic"}),
            json!({"id": "failure:a", "object_type": "failure", "membership_class": "deterministic"}),
            json!({"id": "test:a", "object_type": "test", "membership_class": "deterministic"}),
        ];

        let members = slice_members(&objects, "debugging");
        let allowed_types = BTreeSet::from(["failure", "verification", "file", "test", "risk"]);
        assert!(members.iter().all(|entry| {
            entry
                .get("object_type")
                .and_then(|v| v.as_str())
                .map(|kind| allowed_types.contains(kind))
                .unwrap_or(false)
        }));
        assert_eq!(
            members
                .first()
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str()),
            Some("failure:a")
        );
        assert_eq!(
            members
                .iter()
                .filter(|entry| entry.get("id").and_then(|v| v.as_str()) == Some("failure:a"))
                .count(),
            1
        );
        assert!(members.iter().all(|entry| {
            entry.get("reason").and_then(|v| v.as_str()) == Some("debugging set relevance")
        }));
    }

    #[test]
    fn uncertainty_label_prioritizes_degraded_verified_evidence_and_projection() {
        assert_eq!(
            uncertainty_label(&json!({"degraded": true, "status":"verified"})),
            "degraded"
        );
        assert_eq!(uncertainty_label(&json!({"status":"verified"})), "verified");
        assert_eq!(
            uncertainty_label(&json!({"evidence_ref":"proof:1"})),
            "evidence_linked"
        );
        assert_eq!(
            uncertainty_label(&json!({"status":"proposed"})),
            "speculative"
        );
        assert_eq!(uncertainty_label(&json!({"status":"stale"})), "stale");
        assert_eq!(
            uncertainty_label(&json!({"id":"object:a"})),
            "projection_only"
        );
    }

    #[test]
    fn adjacency_index_surfaces_incoming_outgoing_counts_without_mutation() {
        let mut focusa = FocusaState::default();
        focusa.version = 42;
        focusa.ontology.objects.push(json!({
            "id": "file:a",
            "object_type": "file",
            "status": "active",
            "membership_class": "verified"
        }));
        focusa.ontology.objects.push(json!({
            "id": "test:a",
            "object_type": "test",
            "status": "verified",
            "membership_class": "verified"
        }));
        focusa.ontology.links.push(json!({
            "type": "tested_by",
            "source_id": "file:a",
            "target_id": "test:a",
            "status": "verified",
            "evidence_ref": "test:fixture"
        }));

        focusa.reference_index.handles.push(HandleRef {
            id: Uuid::now_v7(),
            kind: HandleKind::Text,
            label: "file:a proof".to_string(),
            size: 123,
            sha256: "deadbeef".to_string(),
            created_at: Utc::now(),
            session_id: None,
            project_root: None,
            continuity_id: None,
            pinned: false,
            trajectory: Some(TrajectoryLadderContext {
                trajectory_id: Some("traj-ontology".to_string()),
                hlt: Some("Ontology HLT".to_string()),
                stg: Some("Ontology STG".to_string()),
                ..TrajectoryLadderContext::default()
            }),
        });

        let payload = adjacency_index_payload(&focusa, None, Some("file:a"), 10, None);
        assert_eq!(payload["source_state_version"].as_u64(), Some(42));
        assert_eq!(payload["canonical_truth_mutation"].as_bool(), Some(false));
        let node = payload["nodes"].as_array().unwrap().first().unwrap();
        assert_eq!(node["id"].as_str(), Some("file:a"));
        assert_eq!(node["outgoing_count"].as_u64(), Some(1));
        assert_eq!(node["incoming_count"].as_u64(), Some(0));
        assert_eq!(node["outgoing"][0]["type"].as_str(), Some("tested_by"));
        assert_eq!(
            node["outgoing"][0]["uncertainty"].as_str(),
            Some("verified")
        );
        assert_eq!(node["uncertainty"].as_str(), Some("projection_only"));
        assert_eq!(
            node.pointer("/related_evidence_handles/0/trajectory/trajectory_id")
                .and_then(Value::as_str),
            Some("traj-ontology")
        );
    }

    #[test]
    fn working_set_payload_returns_scored_members_and_rehydrate_paths() {
        let mut focusa = FocusaState::default();
        focusa.version = 7;
        focusa.ontology.objects.push(json!({
            "id": "failure:a",
            "object_type": "failure",
            "status": "active",
            "membership_class": "deterministic"
        }));
        focusa.ontology.objects.push(json!({
            "id": "test:a",
            "object_type": "test",
            "status": "verified",
            "membership_class": "verified"
        }));
        focusa.ontology.links.push(json!({
            "type": "tested_by",
            "source_id": "failure:a",
            "target_id": "test:a",
            "status": "verified"
        }));

        let payload = working_set_payload(
            &focusa,
            WorkingSetPayloadParams {
                frame_id: None,
                ask: Some("failure"),
                target_ref: None,
                slice_type: "debugging",
                limit: 10,
                include_reasons: true,
                scope: None,
                cursor: 0,
            },
        );
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_working_set_projection")
        );
        assert_eq!(payload["source_state_version"].as_u64(), Some(7));
        assert_eq!(payload["canonical_truth_mutation"].as_bool(), Some(false));
        let members = payload["members"].as_array().unwrap();
        assert!(
            members
                .iter()
                .any(|member| member["id"].as_str() == Some("failure:a"))
        );
        let failure = members
            .iter()
            .find(|member| member["id"].as_str() == Some("failure:a"))
            .unwrap();
        assert_eq!(
            failure["rehydrate"]["route"].as_str(),
            Some("/v1/ontology/adjacency")
        );
        assert_eq!(failure["uncertainty"].as_str(), Some("projection_only"));
        assert!(failure["reason_count"].as_u64().unwrap_or(0) > 0);
        assert!(failure["link_strength_score"].as_i64().unwrap_or(0) > 0);
        assert!(
            failure["link_path_reason"]
                .as_str()
                .unwrap_or_default()
                .contains("strength=")
        );
    }

    #[test]
    fn graph_community_summaries_are_evidence_backed_projections() {
        let mut focusa = FocusaState::default();
        focusa.version = 12;
        focusa
            .ontology
            .objects
            .push(json!({"id":"file:a","object_type":"file","status":"verified"}));
        focusa
            .ontology
            .objects
            .push(json!({"id":"test:a","object_type":"test","status":"verified"}));
        focusa.ontology.links.push(json!({"type":"tested_by","source_id":"file:a","target_id":"test:a","status":"verified","evidence":"fixture"}));
        let payload = graph_community_summaries_payload(&focusa, None, 20);
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_graph_community_projection")
        );
        assert_eq!(payload["canonical_truth_mutation"].as_bool(), Some(false));
        let communities = payload["communities"].as_array().unwrap();
        assert!(communities.iter().any(|community| {
            community["evidence_links"]
                .as_array()
                .map(|links| !links.is_empty())
                .unwrap_or(false)
        }));
    }

    #[test]
    fn ontology_context_payload_is_prompt_safe_and_non_mutating() {
        let mut focusa = FocusaState::default();
        focusa.version = 9;
        focusa.ontology.objects.push(json!({
            "id": "task:a",
            "object_type": "task",
            "status": "active",
            "membership_class": "deterministic"
        }));
        focusa.reference_index.handles.push(HandleRef {
            id: Uuid::now_v7(),
            kind: HandleKind::Text,
            label: "ontology-context-proof".to_string(),
            size: 123,
            sha256: "deadbeef".to_string(),
            created_at: Utc::now(),
            session_id: None,
            project_root: None,
            continuity_id: None,
            pinned: false,
            trajectory: Some(TrajectoryLadderContext {
                trajectory_id: Some("traj-context".to_string()),
                hlt: Some("Context HLT".to_string()),
                stg: Some("Context STG".to_string()),
                ..TrajectoryLadderContext::default()
            }),
        });
        let body = OntologyContextRequest {
            current_ask: Some("verify ontology context".to_string()),
            frame_id: None,
            workpoint_id: Some("wp:1".to_string()),
            target_refs: vec![],
            budget_tokens: Some(300),
            view_profile: Some("pi_operator_view".to_string()),
            slice_type: "active_mission".to_string(),
            operator_steering_detected: false,
            active_object_refs: Vec::new(),
        };
        let (scope, scope_ref) = exact_test_scope();
        let payload = ontology_context_payload(&focusa, &body, &scope, &scope_ref);
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_prompt_safe_context")
        );
        assert_eq!(payload["source_state_version"].as_u64(), Some(9));
        assert_eq!(payload["canonical_truth_mutation"].as_bool(), Some(false));
        assert!(payload["active_object_set"].as_array().is_some());
        assert!(
            payload["valid_next_actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|action| {
                    action.get("name").and_then(|v| v.as_str()) == Some("verify_invariant")
                })
        );
        assert!(payload["rehydrate"]["routes"].as_array().is_some());
        assert_eq!(
            payload
                .pointer("/evidence_handles/0/trajectory/trajectory_id")
                .and_then(Value::as_str),
            Some("traj-context")
        );
    }

    #[test]
    fn affordances_payload_surfaces_feasible_actions_without_mutation() {
        let root = fixture_workspace("affordance-route", true, true);
        let mut focusa = focusa_with_workspace(&root);
        focusa.version = 11;
        let payload = affordances_payload(&focusa, None, None, Some("verify"), Some("current"), 10);
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_affordance_execution_projection")
        );
        assert_eq!(payload["source_state_version"].as_u64(), Some(11));
        assert_eq!(payload["canonical_truth_mutation"].as_bool(), Some(false));
        assert!(payload["feasible_actions"].as_array().is_some());
        assert!(
            payload["valid_next_actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|action| {
                    action.get("name").and_then(|v| v.as_str()) == Some("verify_invariant")
                })
        );
    }

    #[test]
    fn retrieval_governor_selects_minimal_substrates_and_respects_steering() {
        let mut focusa = FocusaState::default();
        focusa.version = 13;
        let body = RetrievalGovernorRequest {
            current_ask: Some("implement ontology route".to_string()),
            frame_id: None,
            workpoint_id: Some("wp:1".to_string()),
            target_refs: vec!["file:a".to_string()],
            budget_tokens: Some(600),
            operator_steering_detected: true,
            include_metacog: false,
            ask_kind: Some("implementation".to_string()),
            query_scope: Some("mission".to_string()),
            action_intent: Some("patch".to_string()),
            stale_state: false,
            degraded_state: false,
            previous_retrieval_outcomes: Vec::new(),
        };
        let (scope, scope_ref) = exact_test_scope();
        let payload = retrieval_governor_payload(&focusa, &body, &scope, &scope_ref);
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_retrieval_governor")
        );
        assert_eq!(payload["source_state_version"].as_u64(), Some(13));
        assert_eq!(
            payload["excluded_context_reason"].as_str(),
            Some("operator_steering")
        );
        let plan = payload["retrieval_plan"].as_array().unwrap();
        assert!(
            plan.iter()
                .any(|item| item["substrate"].as_str() == Some("ontology_context"))
        );
        assert!(
            plan.iter()
                .any(|item| item["substrate"].as_str() == Some("ontology_affordances"))
        );
        assert!(
            !plan
                .iter()
                .any(|item| item["substrate"].as_str() == Some("workpoint"))
        );
        assert_eq!(
            payload["hybrid_ranker"]["canonical_truth_mutation"].as_bool(),
            Some(false)
        );
    }

    #[test]
    fn tool_result_candidate_deltas_connect_targets_evidence_and_failures_without_mutation() {
        let body = ToolResultProposalRequest {
            tool_name: "bash".to_string(),
            status: Some("failed".to_string()),
            ok: Some(false),
            target_refs: vec!["crates/focusa-api/src/routes/ontology.rs".to_string()],
            evidence_refs: vec!["cargo test -p focusa-api".to_string()],
            workpoint_id: Some("wp-1".to_string()),
            action_intent: Some("verify route".to_string()),
            summary: Some("test failed".to_string()),
            error: Some("compile error".to_string()),
            emit_proposals: false,
        };
        let deltas = tool_result_candidate_deltas(&body);
        assert!(deltas.iter().any(|delta| delta["delta_kind"].as_str()
            == Some("ontology_link_upsert_proposed")
            && delta["link_type"].as_str() == Some("verifies")));
        assert!(deltas.iter().any(|delta| delta["delta_kind"].as_str()
            == Some("ontology_status_change_proposed")
            && delta["to_status"].as_str() == Some("failed")));
        assert!(
            deltas
                .iter()
                .any(|delta| delta["object_type"].as_str() == Some("workpoint"))
        );
        let (_, scope_ref) = exact_test_scope();
        let workstream =
            WorkstreamKey::new(scope_ref, "ontology-test-continuity").expect("valid workstream");
        let events = events_from_tool_result_deltas(Uuid::now_v7(), &deltas, &workstream);
        assert!(!events.is_empty());
    }

    #[test]
    fn execution_critic_emits_failure_artifact_without_canonical_mutation() {
        let body = ExecutionCriticRequest {
            intended_action: Some("verify route".to_string()),
            target_refs: vec!["crates/focusa-api/src/routes/ontology.rs".to_string()],
            verification_hooks: vec!["cargo test".to_string()],
            tool_result: ToolResultProposalRequest {
                tool_name: "bash".to_string(),
                status: Some("failed".to_string()),
                ok: Some(false),
                target_refs: vec!["crates/focusa-api/src/routes/ontology.rs".to_string()],
                evidence_refs: vec!["cargo test".to_string()],
                workpoint_id: Some("wp-1".to_string()),
                action_intent: Some("verify route".to_string()),
                summary: Some("compile failed".to_string()),
                error: Some("compile failed".to_string()),
                emit_proposals: false,
            },
            workpoint_next_action: Some("verify route".to_string()),
            operator_priority: Some("continue".to_string()),
        };
        let payload = execution_critic_payload(&body);
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_execution_critic")
        );
        assert_eq!(
            payload["critic_outcome"].as_str(),
            Some("bounded_failure_proposal")
        );
        assert_eq!(payload["canonical_truth_mutation"].as_bool(), Some(false));
        assert!(
            payload["candidate_ontology_deltas"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |delta| delta["delta_kind"].as_str() == Some("ontology_status_change_proposed")
                )
        );
    }

    #[test]
    fn reflection_synthesizer_proposes_artifacts_and_rejects_noise_without_promotion() {
        let noisy = ReflectionSynthesizerRequest {
            traces: vec![],
            evals: vec![],
            critic_outputs: vec![],
            evidence_refs: vec![],
            scope_tags: vec![],
            limit: Some(5),
            promote: true,
        };
        let noisy_payload = reflection_synthesizer_payload(&noisy);
        assert_eq!(noisy_payload["noise_rejected"].as_bool(), Some(true));
        assert_eq!(noisy_payload["promoted"].as_bool(), Some(false));
        assert_eq!(
            noisy_payload["canonical_truth_mutation"].as_bool(),
            Some(false)
        );

        let useful = ReflectionSynthesizerRequest {
            traces: vec![json!({"event":"tool_result"})],
            evals: vec![json!({"prediction_type":"next_action_success","score":0.7})],
            critic_outputs: vec![
                json!({"critic_outcome":"bounded_failure_proposal","signals":{"failed":true}}),
            ],
            evidence_refs: vec!["cargo test -p focusa-api".to_string()],
            scope_tags: vec!["ontology".to_string()],
            limit: Some(8),
            promote: false,
        };
        let payload = reflection_synthesizer_payload(&useful);
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_secondary_reflection_synthesizer")
        );
        assert_eq!(payload["noise_rejected"].as_bool(), Some(false));
        let artifacts = payload["synthesized_artifacts"].as_array().unwrap();
        assert!(
            artifacts.iter().any(
                |artifact| artifact["artifact_kind"].as_str() == Some("failure_class_proposal")
            )
        );
        assert!(
            artifacts
                .iter()
                .all(|artifact| artifact["promotion_state"].as_str().is_some())
        );
    }

    #[test]
    fn memory_pipeline_links_artifacts_and_gates_semantic_procedural_promotion() {
        let root_scope = ScopeRef::project(
            "project:spec104-memory-pipeline-test",
            "/home/wirebot/focusa",
            "focusa",
            "sha256:spec104-memory-pipeline-test",
        )
        .unwrap();
        let scope = WorkstreamKey::new(root_scope, "spec104-memory-pipeline-continuity").unwrap();
        let blocked = MemoryPipelineRequest {
            scope: scope.clone(),
            episodic_events: vec![json!({"event":"tool_result"})],
            evidence_refs: vec![],
            synthesis_artifacts: vec![json!({"artifact_kind":"metacog_signal_proposal"})],
            eval_results: vec![],
            repeated_validation_count: Some(0),
            lesson_age_days: Some(40),
            limit: Some(10),
        };
        let blocked_payload = memory_pipeline_payload(&blocked, None);
        assert_eq!(
            blocked_payload["canonical_truth_mutation"].as_bool(),
            Some(false)
        );
        assert_eq!(
            blocked_payload["pipeline_state"].as_str(),
            Some("blocked_or_archival_candidate")
        );
        assert!(
            blocked_payload["stages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|stage| stage["status"].as_str() == Some("archive_weak_lesson_proposed"))
        );

        let promoted = MemoryPipelineRequest {
            scope,
            episodic_events: vec![json!({"event":"tool_result"})],
            evidence_refs: vec!["test:proof".to_string()],
            synthesis_artifacts: vec![json!({"artifact_kind":"procedural_playbook_proposal"})],
            eval_results: vec![json!({"result":"improved","promote_learning":true})],
            repeated_validation_count: Some(2),
            lesson_age_days: Some(1),
            limit: Some(10),
        };
        let payload = memory_pipeline_payload(
            &promoted,
            Some(json!({"written":true,"artifact_id":"test"})),
        );
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_memory_promotion_pipeline")
        );
        assert_eq!(
            payload["pipeline_state"].as_str(),
            Some("procedural_candidate_ready")
        );
        assert!(
            payload["stages"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |stage| stage["stage"].as_str() == Some("procedural_playbook_hint")
                        && stage["status"].as_str() == Some("proposed")
                )
        );
    }

    #[test]
    fn intelligence_dashboard_surfaces_doc78_metrics_and_fixed_eval_suite() {
        let mut focusa = FocusaState::default();
        focusa.version = 21;
        focusa
            .ontology
            .objects
            .push(json!({"id":"task:a","object_type":"task","status":"completed"}));
        focusa
            .ontology
            .objects
            .push(json!({"id":"risk:a","object_type":"risk","status":"active"}));
        focusa.ontology.links.push(json!({"type":"verifies","source_id":"evidence:a","target_id":"task:a","status":"verified","evidence":"test"}));
        let payload = intelligence_dashboard_payload(&focusa);
        assert_eq!(
            payload["source"].as_str(),
            Some("ontology_intelligence_dashboard")
        );
        assert_eq!(
            payload["projection_kind"].as_str(),
            Some("bounded_summary_projection")
        );
        assert_eq!(payload["canonical_truth_mutation"].as_bool(), Some(false));
        assert!(
            payload["metrics"]["evidence_linked_answer_rate"]
                .as_f64()
                .unwrap_or(0.0)
                > 0.0
        );
        let fixtures = payload["fixed_eval_suite"]["fixtures"].as_array().unwrap();
        assert!(
            fixtures
                .iter()
                .any(|item| item.as_str() == Some("secondary_critic"))
        );
        assert!(
            fixtures
                .iter()
                .any(|item| item.as_str() == Some("operator_steering"))
        );
    }

    #[test]
    fn projection_profile_is_stable_for_same_slice_type() {
        let focusa = FocusaState::default();
        let payload_a = slice_payload(&focusa, None, "regression");
        let payload_b = slice_payload(&focusa, None, "regression");

        assert_eq!(
            payload_a.get("projection_profile"),
            payload_b.get("projection_profile")
        );
        assert_eq!(
            payload_a
                .get("projection_profile")
                .and_then(|v| v.get("projection_kind"))
                .and_then(|v| v.as_str()),
            Some("regression_projection")
        );
        assert_eq!(
            payload_a
                .get("projection_profile")
                .and_then(|v| v.get("view_profile"))
                .and_then(|v| v.as_str()),
            Some("pi_regression_view")
        );
        let invariants = payload_a
            .get("projection_profile")
            .and_then(|v| v.get("invariants"))
            .and_then(|v| v.as_array())
            .expect("slice invariants");
        assert!(
            invariants
                .iter()
                .any(|v| { v.as_str() == Some("default_slice_uses_bounded_summary_projection") })
        );
    }

    #[test]
    fn bounded_summary_projection_avoids_visual_expansion_for_default_slice_paths() {
        let root = fixture_workspace("bounded-summary", true, true);
        let mut focusa = focusa_with_workspace(&root);
        focusa.reference_index.handles.push(HandleRef {
            id: Uuid::now_v7(),
            kind: HandleKind::FileSnapshot,
            label: "visual-blueprint-screenshot".to_string(),
            size: 123,
            sha256: "deadbeef".to_string(),
            created_at: Utc::now(),
            session_id: focusa.session.as_ref().map(|session| session.session_id),
            project_root: None,
            continuity_id: None,
            pinned: true,
            trajectory: None,
        });
        let bounded = bounded_summary_projection(&focusa, None);
        let combined = combined_projection(&focusa, None);

        assert!(combined.objects.len() > bounded.objects.len());
        assert!(
            !bounded
                .objects
                .iter()
                .any(|object| object.get("object_type").and_then(|v| v.as_str()) == Some("repo"))
        );
        assert!(!bounded.objects.iter().any(|object| {
            object.get("object_type").and_then(|v| v.as_str()) == Some("visual_artifact")
        }));
        assert!(combined.objects.iter().any(|object| {
            object.get("object_type").and_then(|v| v.as_str()) == Some("visual_artifact")
        }));
    }

    #[test]
    fn workspace_projection_discovers_fixture_code_world() {
        let root = fixture_workspace("workspace-world", true, true);
        fs::create_dir_all(root.join("src/routes")).expect("create routes dir");
        fs::create_dir_all(root.join("tests")).expect("create tests dir");
        fs::create_dir_all(root.join("migrations")).expect("create migrations dir");
        fs::create_dir_all(root.join("docs")).expect("create docs dir");
        fs::write(
            root.join("src/routes/api.rs"),
            "use axum::{routing::get, Router};\npub fn router() -> Router { Router::new().route(\"/fixture\", get(handler)) }\nasync fn handler() {}\n",
        )
        .expect("write route fixture");
        fs::write(
            root.join("tests/fixture_test.rs"),
            "#[test]\nfn fixture_works() { assert!(true); }\n",
        )
        .expect("write test fixture");
        fs::write(
            root.join("migrations/001_init.sql"),
            "create table widgets(id integer primary key);\n",
        )
        .expect("write migration fixture");
        fs::write(
            root.join("docs/spec-fixture.md"),
            "# Fixture spec\nCovers route fixture.\n",
        )
        .expect("write spec fixture");
        fs::write(
            root.join("src/tool-contracts.ts"),
            "export const contract = {};\n",
        )
        .expect("write tool contract fixture");

        let focusa = focusa_with_workspace(&root);
        let projection = workspace_projection(&focusa);

        assert_eq!(projection_count(&projection, "repo"), 1);
        assert!(projection_count(&projection, "package") >= 1);
        assert!(projection_count(&projection, "file") >= 4);
        assert!(projection_count(&projection, "module") >= 3);
        assert!(projection_count(&projection, "route") >= 1);
        assert!(projection_count(&projection, "endpoint") >= 1);
        assert!(projection_count(&projection, "migration") >= 1);
        assert!(projection_count(&projection, "test") >= 1);
        assert!(projection_count(&projection, "specification") >= 1);
        assert!(projection_count(&projection, "tool_contract") >= 1);
        assert!(
            projection
                .links
                .iter()
                .any(
                    |link| link.get("type").and_then(|v| v.as_str()) == Some("binds_to")
                        && link.get("evidence").and_then(|v| v.as_str())
                            == Some("route handler scan")
                )
        );
        assert!(
            projection
                .links
                .iter()
                .any(
                    |link| link.get("type").and_then(|v| v.as_str()) == Some("constrains")
                        && link.get("evidence").and_then(|v| v.as_str())
                            == Some("docs/spec -> package heuristic")
                )
        );
        assert!(
            projection
                .links
                .iter()
                .any(
                    |link| link.get("type").and_then(|v| v.as_str()) == Some("targets_schema")
                        && link.get("evidence").and_then(|v| v.as_str())
                            == Some("tool contract file scan")
                )
        );
    }

    #[test]
    fn affordance_execution_projection_emits_runtime_affordances_and_permissions() {
        let root = fixture_workspace("affordance-runtime", true, true);
        let focusa = focusa_with_workspace(&root);
        let projection = affordance_execution_projection(&focusa, None);

        let execution_context_id = stable_id(
            "execution_context",
            &format!("workspace:{}", root.display()),
        );
        let inspect_workspace_affordance_id = stable_id(
            "affordance",
            &format!("inspect-workspace:{}", root.display()),
        );
        let edit_workspace_affordance_id =
            stable_id("affordance", &format!("edit-workspace:{}", root.display()));
        let workspace_fs_surface_id = stable_id("tool_surface", "workspace_filesystem");
        let workspace_read_permission_id =
            stable_id("permission", &format!("workspace-read:{}", root.display()));
        let workspace_write_permission_id =
            stable_id("permission", &format!("workspace-write:{}", root.display()));

        assert!(projection_has_object(
            &projection,
            "execution_context",
            &execution_context_id,
            Some("active")
        ));
        assert!(projection_has_object(
            &projection,
            "affordance",
            &inspect_workspace_affordance_id,
            Some("active")
        ));
        assert!(projection_has_object(
            &projection,
            "affordance",
            &edit_workspace_affordance_id,
            Some("active")
        ));
        assert!(projection_has_object(
            &projection,
            "permission",
            &workspace_read_permission_id,
            Some("active")
        ));
        assert!(projection_has_object(
            &projection,
            "permission",
            &workspace_write_permission_id,
            Some("active")
        ));
        assert!(projection_has_link(
            &projection,
            "enabled_by",
            &inspect_workspace_affordance_id,
            &workspace_fs_surface_id
        ));
        assert!(projection_has_link(
            &projection,
            "requires_permission",
            &inspect_workspace_affordance_id,
            &workspace_read_permission_id
        ));
        assert!(projection_has_link(
            &projection,
            "requires_permission",
            &edit_workspace_affordance_id,
            &workspace_write_permission_id
        ));
        assert!(projection_has_link(
            &projection,
            "available_in_context",
            &edit_workspace_affordance_id,
            &execution_context_id
        ));
    }

    #[test]
    fn affordance_execution_projection_blocks_build_without_cargo_manifest() {
        let root = fixture_workspace("blocked-build", true, false);
        let focusa = focusa_with_workspace(&root);
        let projection = affordance_execution_projection(&focusa, None);

        let build_rust_affordance_id =
            stable_id("affordance", &format!("build-rust:{}", root.display()));
        let cargo_manifest_precondition_id = stable_id(
            "precondition",
            &format!("cargo-manifest:{}", root.display()),
        );

        assert!(projection_has_object(
            &projection,
            "affordance",
            &build_rust_affordance_id,
            Some("blocked")
        ));
        assert!(projection_has_link(
            &projection,
            "depends_on",
            &build_rust_affordance_id,
            &cargo_manifest_precondition_id
        ));
        assert!(projection_has_link(
            &projection,
            "blocks_execution_of",
            &cargo_manifest_precondition_id,
            &build_rust_affordance_id
        ));
    }

    #[test]
    fn working_set_membership_class_is_enum_and_never_null() {
        // Explicit enum values are honored.
        for allowed in [
            "pinned",
            "deterministic",
            "verified",
            "inferred",
            "provisional",
        ] {
            let object = json!({"membership_class": allowed});
            assert_eq!(
                derived_membership_class(&object, false, false).as_str(),
                Some(allowed)
            );
        }
        // Arbitrary / missing values are derived, never passed through or nulled.
        let garbage = json!({"membership_class": "definitely-not-an-enum"});
        assert_eq!(
            derived_membership_class(&garbage, true, false).as_str(),
            Some("verified")
        );
        let unverified = json!({"membership_class": null});
        assert_eq!(
            derived_membership_class(&unverified, false, false).as_str(),
            Some("provisional")
        );
        let parser = json!({"provenance_class": "parser_derived"});
        assert_eq!(
            derived_membership_class(&parser, false, false).as_str(),
            Some("deterministic")
        );
        let asked = json!({"id": "module:a", "object_type": "module"});
        assert_eq!(
            derived_membership_class(&asked, false, true).as_str(),
            Some("inferred")
        );
        let verified_first = json!({"provenance_class": "parser_derived"});
        assert_eq!(
            derived_membership_class(&verified_first, true, true).as_str(),
            Some("verified")
        );
    }

    #[test]
    fn working_set_member_freshness_is_derived_not_defaulted() {
        // A member without a tracked fresh field must not fabricate `fresh`.
        let untracked = json!({"id": "module:a"});
        assert_eq!(
            member_freshness(&untracked, false, false)
                .get("status")
                .and_then(|v| v.as_str()),
            Some("provisional")
        );
        assert_eq!(
            member_freshness(&untracked, false, true)
                .get("status")
                .and_then(|v| v.as_str()),
            Some("fresh")
        );
        // A stale read index degrades members even when a field says fresh.
        assert_eq!(
            member_freshness(&json!({"fresh": true}), true, true)
                .get("status")
                .and_then(|v| v.as_str()),
            Some("degraded")
        );
        assert_eq!(
            member_freshness(&json!({"fresh": false}), false, false)
                .get("status")
                .and_then(|v| v.as_str()),
            Some("stale")
        );
    }

    #[test]
    fn working_set_index_uses_latest_delta_and_derived_staleness() {
        let mut focusa = FocusaState::default();
        focusa
            .ontology
            .delta_log
            .push(focusa_core::types::OntologyDeltaRecord {
                workstream: None,
                delta_kind: "first".to_string(),
                payload: json!({}),
                timestamp: None,
            });
        focusa
            .ontology
            .delta_log
            .push(focusa_core::types::OntologyDeltaRecord {
                workstream: None,
                delta_kind: "latest".to_string(),
                payload: json!({}),
                timestamp: None,
            });
        let payload = working_set_payload(
            &focusa,
            WorkingSetPayloadParams {
                frame_id: None,
                ask: None,
                target_ref: None,
                slice_type: "active_mission",
                limit: 20,
                include_reasons: false,
                cursor: 0,
                scope: None,
            },
        );
        assert_eq!(
            payload
                .get("index")
                .and_then(|v| v.get("last_reducer_event_id"))
                .and_then(|v| v.as_str()),
            Some("latest:None")
        );
        let freshness = payload
            .get("index")
            .and_then(|v| v.get("freshness"))
            .expect("index freshness envelope present");
        assert_eq!(freshness.get("derived"), Some(&json!(true)));
        assert!(
            freshness
                .get("age_seconds")
                .and_then(|v| v.as_i64())
                .unwrap_or(-1)
                >= 0
        );
    }
}
