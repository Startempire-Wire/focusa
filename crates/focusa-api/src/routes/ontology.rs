//! Ontology inspection routes.
//!
//! Read-only projection of the typed software/work/mission/execution world.
//! This keeps ontology additive in implementation while making the bounded
//! working world inspectable at runtime.

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use focusa_core::types::{FocusaState, FrameRecord, HandleKind};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
    "milestone",
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
];
const ACTION_TYPES: &[&str] = &[
    "refactor_module",
    "modify_schema",
    "add_route",
    "add_test",
    "verify_invariant",
    "promote_decision",
    "mark_blocked",
    "resolve_risk",
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
];
const SLICE_TYPES: &[&str] = &[
    "active_mission",
    "debugging",
    "refactor",
    "regression",
    "architecture",
];
const MAX_DISCOVERED_PATHS: usize = 512;
const MAX_DISCOVERY_SCAN_PATHS: usize = 4096;
const MAX_DISCOVERED_SYMBOLS: usize = 24;
const MAX_DISCOVERED_ENDPOINTS: usize = 16;
const WORKSPACE_FALLBACK_ROOT: &str = "/home/wirebot/focusa";

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

fn normalize_slice_type(slice_type: &str) -> &str {
    if SLICE_TYPES.contains(&slice_type) {
        slice_type
    } else {
        "active_mission"
    }
}

fn infer_slice_type_from_operator_context<'a>(focusa: &'a FocusaState, requested: &'a str) -> &'a str {
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
        "mark_blocked" => &["task", "goal", "risk", "failure"],
        "resolve_risk" => &["risk", "verification", "task"],
        "complete_task" => &["task", "goal", "milestone"],
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
        "derive_plumbing_requirements" => &["interaction", "ui_state", "binding", "validation_rule"],
        "map_tokens_to_surfaces" => &["token", "layout_rule", "component", "region"],
        "map_states_to_views" => &["ui_state", "interaction", "component", "page"],
        "map_bindings_and_validation" => &["binding", "validation_rule", "component", "ui_state"],
        "synthesize_completion_checklist" => &["verification", "acceptance_criterion", "task", "artifact"],
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
            "contract_role": "declarative_projection_only",
            "route_surfaces": ["GET /v1/ontology/contracts", "GET /v1/ontology/world"]
        },
        "trace_metadata": {
            "trace_surface": "projection_snapshot",
            "emits_reducer_event_on_read": false,
            "source_inputs": ["focus_state", "workspace_scan"]
        },
        "eval_metadata": {
            "validation_mode": "route-contract-regression",
            "backing_tests": ["tests/ontology_world_contract_test.sh", "tests/tool_contract_test.sh"]
        },
        "projection_metadata": {
            "projection_kind": "read_only_runtime_projection",
            "mutates_canonical_state": false,
            "snapshot_consistency": "best_effort"
        },
        "governance_metadata": {
            "api_permission_scope": null,
            "writes_allowed": false,
            "authority_note": "ontology routes project reducer/workspace state only"
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
        if out.len() >= MAX_DISCOVERY_SCAN_PATHS {
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
                if matches!(
                    name,
                    "target" | "node_modules" | "dist" | "build" | ".beads"
                ) {
                    continue;
                }
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
                if out.len() >= MAX_DISCOVERY_SCAN_PATHS {
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
                } else if trimmed.starts_with("import ") {
                    if let Some(start) = trimmed.find('"').or_else(|| trimmed.find('\'')) {
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
                && !matches!(candidate, "if" | "for" | "while" | "match" | "loop" | "return")
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
    let session_workspace = focusa
        .session
        .as_ref()
        .and_then(|s| s.workspace_id.clone())
        .filter(|path| {
            let candidate = PathBuf::from(path);
            candidate.exists() && candidate.is_dir()
        });

    let root = session_workspace
        .map(PathBuf::from)
        .or_else(|| {
            let fallback = PathBuf::from(WORKSPACE_FALLBACK_ROOT);
            if fallback.exists() && fallback.is_dir() {
                Some(fallback)
            } else {
                None
            }
        })
        .unwrap_or_default();

    if root.as_os_str().is_empty() {
        return WorkspaceProjection::default();
    }

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

        if file_type == "manifest" {
            if let Some(content) = read_text(&path) {
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
                for (method, endpoint_path) in parse_endpoints(&content) {
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
            .unwrap_or_else(|| WORKSPACE_FALLBACK_ROOT.to_string());
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
    if let Some(frame_ref) = frame {
        if let Some(ref id) = page_id {
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
                let variant_id = stable_id("variant", &format!("{}:{}", component_id, variant_kind));
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

            if label.contains("grid") || label.contains("layout") || label.contains("container") || label.contains("alignment") {
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

    let permission_profile_id = stable_id("permission_profile", &format!(
        "{}:{}:{}",
        focusa.work_loop.policy.allow_destructive_actions,
        focusa.work_loop.policy.require_operator_for_governance,
        focusa.work_loop.policy.require_operator_for_scope_change
    ));
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
        || focusa.work_loop.pause_flags.destructive_confirmation_required
        || focusa.work_loop.policy.require_operator_for_scope_change
        || focusa.work_loop.policy.require_operator_for_governance
    {
        let boundary_id = stable_id("handoff_boundary", &format!(
            "{}:{}:{}",
            focusa.work_loop.pause_flags.operator_override_active,
            focusa.work_loop.pause_flags.governance_decision_pending,
            focusa.work_loop.pause_flags.destructive_confirmation_required
        ));
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
        || focusa.work_loop.pause_flags.destructive_confirmation_required
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

        if let (Some(source_id), Some(target_id)) = (proposal.source_id.as_ref(), proposal.target_id.as_ref()) {
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
        if proposal.proposal_kind.to_ascii_lowercase().contains("supersed")
            || proposal.target_class.to_ascii_lowercase().contains("supersed")
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
                let incoming_preferred = matches!(incoming_status, "canonical" | "verified" | "active")
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

fn combined_projection(focusa: &FocusaState, frame_id: Option<&str>) -> WorkspaceProjection {
    let frame = selected_frame(focusa, frame_id);
    let mission = mission_projection(focusa, frame);
    let workspace = workspace_projection(focusa);
    let canonical = canonical_ontology_projection(focusa);
    let visual = visual_projection(focusa, frame);
    let identity = identity_projection(focusa);
    let governance = governance_projection(focusa);
    let reference_resolution = reference_resolution_projection(focusa);
    let projection_view = projection_view_semantics_projection(focusa);

    let objects = dedupe_objects(
        [
            mission.objects,
            workspace.objects,
            canonical.objects,
            visual.objects,
            identity.objects,
            governance.objects,
            reference_resolution.objects,
            projection_view.objects,
        ]
        .concat(),
    );
    let links = dedupe_links(
        [
            mission.links,
            workspace.links,
            canonical.links,
            visual.links,
            identity.links,
            governance.links,
            reference_resolution.links,
            projection_view.links,
        ]
        .concat(),
    );

    WorkspaceProjection { objects, links }
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
    let projection = combined_projection(focusa, frame_id);
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
                "membership_is_capped_and_deduplicated"
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

async fn world(
    Query(query): Query<OntologyWorldQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let projection = combined_projection(&focusa, query.frame_id.as_deref());
    let action_catalog: Vec<Value> = ACTION_TYPES
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
                "catalog_role": "contract_projection_reference"
            })
        })
        .collect();

    Json(json!({
        "object_count": projection.objects.len(),
        "link_count": projection.links.len(),
        "objects": projection.objects,
        "links": projection.links,
        "canonical_ontology": {
            "proposal_count": focusa.ontology.proposals.len(),
            "verification_count": focusa.ontology.verifications.len(),
            "working_set_refresh_count": focusa.ontology.working_set_refreshes.len(),
            "delta_count": focusa.ontology.delta_log.len()
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::types::FocusaState;
    use serde_json::json;

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
    }
}
