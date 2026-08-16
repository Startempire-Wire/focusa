//! Spec 109 AX-001 — Authoritative Agent Capabilities Endpoint.
//!
//! `GET /v1/agent/capabilities` returns a compact machine-readable index of
//! every Focusa operation, with metadata for schema, side effects, permissions,
//! and documentation refs. Agents use this to discover what Focusa can do
//! without reading docs.
//!
//! `GET /v1/agent/adapter-capabilities` publishes the separate Spec130 measured
//! native-adapter capability registry.

use crate::routes::permissions::{PermissionContext, permission_context};
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::tool_result::{FailureClass, TOOL_RESULT_SCHEMA, ToolResultV1, ToolStatus};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Arc, LazyLock};

#[derive(Debug, Serialize)]
struct CapabilitiesIndex {
    #[serde(rename = "schema")]
    schema: &'static str,
    api_version: &'static str,
    generated_at: String,
    operation_count: usize,
    families: Vec<&'static str>,
    operations: Vec<OperationEntry>,
}

#[derive(Debug, Serialize)]
struct OperationOwnership {
    subsystem: &'static str,
    core_action_ref: &'static str,
}

#[derive(Debug, Serialize)]
struct OperationContracts {
    input_schema_ref: &'static str,
    output_schema_ref: &'static str,
    error_schema_ref: &'static str,
}

#[derive(Debug, Serialize)]
struct OperationScope {
    required_keys: Vec<&'static str>,
    project_scoped: bool,
    workstream_scoped: bool,
    attachment_scoped: bool,
}

#[derive(Debug, Serialize)]
struct OperationControl {
    capability_refs: Vec<&'static str>,
    permission_scopes: Vec<&'static str>,
    mode: &'static str,
    confirmation: &'static str,
    idempotency_required: bool,
    optimistic_concurrency_required: bool,
    receipt_required: bool,
    reversible: bool,
}

#[derive(Debug, Serialize)]
struct OperationUi {
    allowed_in_generated_ui: bool,
    default_label: &'static str,
    plain_language_description: &'static str,
    input_presentation_ref: &'static str,
    result_presentation_ref: &'static str,
    advanced_only: bool,
    sensitivity: &'static str,
}

#[derive(Debug, Serialize)]
struct OperationEntry {
    #[serde(rename = "schema")]
    descriptor_schema: &'static str,
    operation_id: &'static str,
    label: &'static str,
    family: &'static str,
    method: &'static str,
    path: &'static str,
    canonical: bool,
    alias_of: Option<&'static str>,
    operation_version: &'static str,
    schema_version: &'static str,
    side_effect_profile: &'static str,
    materialization_mode: &'static str,
    supports_side_effect_policy: Vec<&'static str>,
    requires_idempotency_key: bool,
    requires_if_match_version: bool,
    requires_preview_token: bool,
    permissions_required: Vec<&'static str>,
    confirmation_required: bool,
    budget_profile: &'static str,
    response_detail_supported: Vec<&'static str>,
    request_schema_ref: &'static str,
    response_schema_ref: &'static str,
    error_taxonomy_ref: &'static str,
    examples_ref: &'static str,
    docs_ref: &'static str,
    deprecation: Option<DeprecationEntry>,
    ownership: OperationOwnership,
    contracts: OperationContracts,
    scope: OperationScope,
    control: OperationControl,
    ui: OperationUi,
}

#[derive(Debug, Serialize)]
struct DeprecationEntry {
    deprecated: bool,
    deprecation_message: &'static str,
    deprecation_removed_in: &'static str,
    deprecation_replacement: &'static str,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum AdapterTier {
    TierA,
    TierB,
    TierC,
    TierD,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AdapterCapabilityManifest {
    adapter: String,
    manifest_version: u32,
    measured_at: String,
    measured_against: String,
    tier: AdapterTier,
    supports_compaction_hook: bool,
    supports_bounded_custom_entry: bool,
    supports_session_size_preflight: bool,
    supports_automatic_native_rollover: bool,
    supports_user_command_rollover: bool,
    supports_rpc_rollover: bool,
    supports_streaming_import: bool,
    supports_external_rehydrate: bool,
    supports_preload_receipt: bool,
    evidence_refs: Vec<String>,
    limitations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AdapterCapabilityRegistry {
    schema: String,
    registry_version: String,
    adapters: Vec<AdapterCapabilityManifest>,
}

static ADAPTER_CAPABILITY_REGISTRY: LazyLock<AdapterCapabilityRegistry> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../../adapters/spec130-capability-manifests.json"
    ))
    .expect("Spec130 adapter capability manifests must match the typed registry schema")
});

#[allow(clippy::too_many_arguments)]
fn op(
    id: &'static str,
    label: &'static str,
    family: &'static str,
    method: &'static str,
    path: &'static str,
    canonical: bool,
    alias_of: Option<&'static str>,
    side_effect: &'static str,
    materialization: &'static str,
    policies: Vec<&'static str>,
    req_idempotency: bool,
    req_if_match: bool,
    req_preview: bool,
    permissions: Vec<&'static str>,
    confirm: bool,
    budget: &'static str,
    detail: Vec<&'static str>,
    req_schema: &'static str,
    res_schema: &'static str,
    docs: &'static str,
    deprecation: Option<DeprecationEntry>,
) -> OperationEntry {
    let project_scoped = !matches!(family, "health" | "device" | "license" | "events")
        && id != "focusa.compatibility_lock.read";
    let workstream_scoped = matches!(
        family,
        "trajectory"
            | "workpoint"
            | "metacognition"
            | "evidence"
            | "prediction"
            | "context_cognition"
            | "context"
            | "turn"
            | "memory"
            | "work_loop"
            | "workspace_artifact"
            | "project_role_profile"
            | "interview_strategy"
            | "project_interview"
            | "spec_workbench"
            | "provider_execution"
            | "task_plan"
            | "work_rail"
            | "mission_canvas"
    ) || matches!(
        id,
        "focusa.ui_action_bindings.read"
            | "focusa.ui_capability_snapshot.read"
            | "focusa.protocol.handshake"
    );
    let attachment_scoped = id.contains("attachment")
        || matches!(
            family,
            "context"
                | "workspace_artifact"
                | "project_role_profile"
                | "interview_strategy"
                | "project_interview"
                | "spec_workbench"
                | "provider_execution"
                | "task_plan"
                | "work_rail"
                | "mission_canvas"
        );
    let mut required_keys = Vec::new();
    if project_scoped {
        required_keys.push("project_root");
    }
    if family == "work_rail" {
        required_keys.push("working_subpath_id");
    }
    if workstream_scoped {
        required_keys.push("continuity_id");
    }
    if attachment_scoped {
        required_keys.push("attachment_id");
    }
    let mode = if method == "GET" {
        "read"
    } else if req_preview {
        "preview"
    } else {
        "commit"
    };
    let confirmation = if confirm { "consequential" } else { "none" };
    let sensitivity = if confirm {
        "consequential"
    } else if method == "GET" {
        "routine"
    } else {
        "scoped_mutation"
    };

    OperationEntry {
        descriptor_schema: "focusa.operation_descriptor.v1",
        operation_id: id,
        label,
        family,
        method,
        path,
        canonical,
        alias_of,
        operation_version: "1.0.0",
        schema_version: req_schema,
        side_effect_profile: side_effect,
        materialization_mode: materialization,
        supports_side_effect_policy: policies,
        requires_idempotency_key: req_idempotency,
        requires_if_match_version: req_if_match,
        requires_preview_token: req_preview,
        permissions_required: permissions.clone(),
        confirmation_required: confirm,
        budget_profile: budget,
        response_detail_supported: detail,
        request_schema_ref: req_schema,
        response_schema_ref: res_schema,
        error_taxonomy_ref: "/v1/agent/error-taxonomy",
        examples_ref: "/v1/agent/examples/operations",
        docs_ref: docs,
        deprecation,
        ownership: OperationOwnership {
            subsystem: family,
            core_action_ref: id,
        },
        contracts: OperationContracts {
            input_schema_ref: req_schema,
            output_schema_ref: res_schema,
            error_schema_ref: "focusa.tool_result.v1",
        },
        scope: OperationScope {
            required_keys,
            project_scoped,
            workstream_scoped,
            attachment_scoped,
        },
        control: OperationControl {
            capability_refs: vec![family],
            permission_scopes: permissions.clone(),
            mode,
            confirmation,
            idempotency_required: req_idempotency,
            optimistic_concurrency_required: req_if_match,
            receipt_required: method != "GET",
            reversible: id.contains("restore") || id.contains("rollback"),
        },
        ui: OperationUi {
            allowed_in_generated_ui: canonical && alias_of.is_none(),
            default_label: label,
            plain_language_description: label,
            input_presentation_ref: "focusa.generated_form.v1",
            result_presentation_ref: "focusa.generated_result.v1",
            advanced_only: matches!(family, "bloatgaurd" | "dxux" | "traverse"),
            sensitivity,
        },
    }
}

fn build_operations() -> Vec<OperationEntry> {
    vec![
        // ── health ──────────────────────────────────────────────────────────
        op(
            "focusa.health.check",
            "Health Check",
            "health",
            "GET",
            "/v1/health",
            true,
            None,
            "read_health",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["health:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.health.request.v1",
            "focusa.health.response.v1",
            "docs/focusa-api/routes/health.md",
            None,
        ),
        // ── project identity ────────────────────────────────────────────────
        op(
            "focusa.project.identity",
            "Project Identity",
            "project",
            "GET",
            "/v1/project/identity",
            true,
            None,
            "read_identity",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["project:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.project_identity.request.v1",
            "focusa.project_identity.response.v1",
            "docs/focusa-api/routes/project.md",
            None,
        ),
        op(
            "focusa.project.verify",
            "Project Verify",
            "project",
            "GET",
            "/v1/project/verify",
            true,
            None,
            "read_verify",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["project:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.project_verify.request.v1",
            "focusa.project_verify.response.v1",
            "docs/focusa-api/routes/project.md",
            None,
        ),
        // ── trajectory ───────────────────────────────────────────────────────
        op(
            "focusa.trajectory.view",
            "Trajectory View",
            "trajectory",
            "GET",
            "/v1/trajectory/view",
            true,
            None,
            "read_trajectory",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["trajectory:read"],
            false,
            "light_read",
            vec!["compact", "standard", "debug"],
            "focusa.trajectory_view.request.v1",
            "focusa.trajectory_view.response.v1",
            "docs/focusa-api/routes/trajectory.md",
            None,
        ),
        op(
            "focusa.trajectory.define_goal",
            "Trajectory Define Goal",
            "trajectory",
            "POST",
            "/v1/trajectory/define-goal",
            true,
            None,
            "write_trajectory_goal",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["trajectory:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.trajectory_define_goal.request.v1",
            "focusa.trajectory_define_goal.response.v1",
            "docs/focusa-api/routes/trajectory.md",
            None,
        ),
        op(
            "focusa.trajectory.assess",
            "Trajectory Assess",
            "trajectory",
            "POST",
            "/v1/trajectory/assess",
            true,
            None,
            "read_trajectory_assess",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["trajectory:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.trajectory_assess.request.v1",
            "focusa.trajectory_assess.response.v1",
            "docs/focusa-api/routes/trajectory.md",
            None,
        ),
        op(
            "focusa.trajectory.propose_workpoint",
            "Trajectory Propose Workpoint",
            "trajectory",
            "POST",
            "/v1/trajectory/propose-workpoint",
            true,
            None,
            "read_trajectory_propose",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["trajectory:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.trajectory_propose_workpoint.request.v1",
            "focusa.trajectory_propose_workpoint.response.v1",
            "docs/focusa-api/routes/trajectory.md",
            None,
        ),
        op(
            "focusa.trajectory.checkpoint",
            "Trajectory Checkpoint",
            "trajectory",
            "POST",
            "/v1/trajectory/checkpoint",
            true,
            None,
            "write_trajectory_checkpoint",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["trajectory:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.trajectory_checkpoint.request.v1",
            "focusa.trajectory_checkpoint.response.v1",
            "docs/focusa-api/routes/trajectory.md",
            None,
        ),
        op(
            "focusa.trajectory.resume",
            "Trajectory Resume",
            "trajectory",
            "POST",
            "/v1/trajectory/resume",
            true,
            None,
            "read_trajectory_resume",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["trajectory:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.trajectory_resume.request.v1",
            "focusa.trajectory_resume.response.v1",
            "docs/focusa-api/routes/trajectory.md",
            None,
        ),
        // ── workpoint ────────────────────────────────────────────────────────
        op(
            "focusa.workpoint.checkpoint",
            "Workpoint Checkpoint",
            "workpoint",
            "POST",
            "/v1/workpoint/checkpoint",
            true,
            None,
            "write_workpoint_checkpoint",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["workpoint:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.workpoint_checkpoint.request.v1",
            "focusa.workpoint_checkpoint.response.v1",
            "docs/focusa-api/routes/workpoint.md",
            None,
        ),
        op(
            "focusa.workpoint.resume",
            "Workpoint Resume",
            "workpoint",
            "GET",
            "/v1/workpoint/resume",
            true,
            None,
            "read_workpoint_resume",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["workpoint:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.workpoint_resume.request.v1",
            "focusa.workpoint_resume.response.v1",
            "docs/focusa-api/routes/workpoint.md",
            None,
        ),
        op(
            "focusa.workpoint.link_evidence",
            "Workpoint Link Evidence",
            "workpoint",
            "POST",
            "/v1/workpoint/link-evidence",
            true,
            None,
            "write_workpoint_evidence",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["workpoint:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.workpoint_link_evidence.request.v1",
            "focusa.workpoint_link_evidence.response.v1",
            "docs/focusa-api/routes/workpoint.md",
            None,
        ),
        // ── metacognition ────────────────────────────────────────────────────
        op(
            "focusa.metacog.capture",
            "Metacog Capture",
            "metacognition",
            "POST",
            "/v1/metacognition/capture",
            true,
            None,
            "write_metacog_signal",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["metacog:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.metacog_capture.request.v1",
            "focusa.metacog_capture.response.v1",
            "docs/focusa-api/routes/metacognition.md",
            None,
        ),
        op(
            "focusa.metacog.retrieve",
            "Metacog Retrieve",
            "metacognition",
            "POST",
            "/v1/metacognition/retrieve",
            true,
            None,
            "read_metacog",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["metacog:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.metacog_retrieve.request.v1",
            "focusa.metacog_retrieve.response.v1",
            "docs/focusa-api/routes/metacognition.md",
            None,
        ),
        op(
            "focusa.metacog.reflect",
            "Metacog Reflect",
            "metacognition",
            "POST",
            "/v1/metacognition/reflect",
            true,
            None,
            "read_metacog_reflect",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["metacog:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.metacog_reflect.request.v1",
            "focusa.metacog_reflect.response.v1",
            "docs/focusa-api/routes/metacognition.md",
            None,
        ),
        op(
            "focusa.metacog.doctor",
            "Metacog Doctor",
            "metacognition",
            "POST",
            "/v1/metacognition/doctor",
            true,
            None,
            "read_metacog_doctor",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["metacog:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.metacog_doctor.request.v1",
            "focusa.metacog_doctor.response.v1",
            "docs/focusa-api/routes/metacognition.md",
            None,
        ),
        // ── evidence ─────────────────────────────────────────────────────────
        op(
            "focusa.evidence.capture",
            "Evidence Capture",
            "evidence",
            "POST",
            "/v1/evidence/capture",
            true,
            None,
            "write_evidence",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["evidence:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.evidence_capture.request.v1",
            "focusa.evidence_capture.response.v1",
            "docs/focusa-api/routes/evidence.md",
            None,
        ),
        // ── prediction ───────────────────────────────────────────────────────
        op(
            "focusa.prediction.record",
            "Prediction Record",
            "prediction",
            "POST",
            "/v1/predictions/record",
            true,
            None,
            "write_prediction",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["prediction:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.prediction_record.request.v1",
            "focusa.prediction_record.response.v1",
            "docs/focusa-api/routes/predictions.md",
            None,
        ),
        op(
            "focusa.prediction.evaluate",
            "Prediction Evaluate",
            "prediction",
            "POST",
            "/v1/predictions/evaluate",
            true,
            None,
            "write_prediction_eval",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["prediction:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.prediction_evaluate.request.v1",
            "focusa.prediction_evaluate.response.v1",
            "docs/focusa-api/routes/predictions.md",
            None,
        ),
        op(
            "focusa.prediction.recent",
            "Prediction Recent",
            "prediction",
            "GET",
            "/v1/predictions/recent",
            true,
            None,
            "read_prediction",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["prediction:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.prediction_recent.request.v1",
            "focusa.prediction_recent.response.v1",
            "docs/focusa-api/routes/predictions.md",
            None,
        ),
        // ── context cognition ────────────────────────────────────────────────
        op(
            "focusa.context_cognition.packet",
            "Context Cognition Packet",
            "context_cognition",
            "GET",
            "/v1/context-cognition/packet",
            true,
            None,
            "read_context",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["context:read"],
            false,
            "heavy_read",
            vec!["compact", "standard", "debug"],
            "focusa.context_cognition_packet.request.v1",
            "focusa.context_cognition_packet.response.v1",
            "docs/focusa-api/routes/context_cognition.md",
            None,
        ),
        op(
            "focusa.context_cognition.curate",
            "Context Cognition Curate",
            "context_cognition",
            "POST",
            "/v1/context-cognition/curate",
            true,
            None,
            "read_context_curate",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["context:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.context_cognition_curate.request.v1",
            "focusa.context_cognition_curate.response.v1",
            "docs/focusa-api/routes/context_cognition.md",
            None,
        ),
        // ── bounded rich Workspace Artifact bridge ───────────────────────────
        op(
            "focusa.workspace.artifact.list",
            "List Linked Workspace Artifacts",
            "workspace_artifact",
            "GET",
            "/v1/workspace/artifacts",
            true,
            None,
            "read_artifact_links",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["artifact:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.workspace_artifact_list.request.v1",
            "focusa.workspace_artifact_list.v1",
            "docs/135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md",
            None,
        ),
        op(
            "focusa.workspace.artifact.intake",
            "Link Workspace Artifact",
            "workspace_artifact",
            "POST",
            "/v1/workspace/artifacts/intake",
            true,
            None,
            "link_artifact_descriptor",
            "canonical_event",
            vec!["commit"],
            true,
            true,
            false,
            vec!["artifact:write"],
            false,
            "standard_write",
            vec!["compact", "standard", "debug"],
            "focusa.workspace_artifact_intake.request.v1",
            "focusa.workspace_artifact_intake_result.v1",
            "docs/135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md",
            None,
        ),
        // ── Context-grounded project Role Profile ─────────────────────────────
        op(
            "focusa.role_profile.list",
            "List Project Role Profile Revisions",
            "project_role_profile",
            "GET",
            "/v1/roles/profiles",
            true,
            None,
            "read_role_profiles",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["role:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.project_agent_role_profile_list.request.v1",
            "focusa.project_agent_role_profile_list.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.role_profile.draft",
            "Draft Grounded Project Role Profile",
            "project_role_profile",
            "POST",
            "/v1/roles/profiles/draft",
            true,
            None,
            "draft_role_profile_revision",
            "canonical_event",
            vec!["commit"],
            true,
            true,
            false,
            vec!["role:write"],
            false,
            "standard_write",
            vec!["compact", "standard", "debug"],
            "focusa.project_agent_role_profile_draft.request.v1",
            "focusa.project_agent_role_profile_mutation_result.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.role_profile.review",
            "Approve, Reject, or Defer Project Role Profile",
            "project_role_profile",
            "POST",
            "/v1/roles/profiles/review",
            true,
            None,
            "review_role_profile_revision",
            "canonical_event",
            vec!["commit"],
            true,
            true,
            false,
            vec!["role:approve"],
            true,
            "standard_write",
            vec!["compact", "standard", "debug"],
            "focusa.project_agent_role_profile_review.request.v1",
            "focusa.project_agent_role_profile_mutation_result.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.mission_canvas.surface_binding.list",
            "List Exact Work Surface Bindings",
            "mission_canvas",
            "GET",
            "/v1/mission-canvas/surface-bindings",
            true,
            None,
            "read_mission_canvas_surface_bindings",
            "canonical_projection_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["mission_canvas:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.mission_canvas_surface_binding_list.request.v1",
            "focusa.mission_canvas_surface_binding_list.v1",
            "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
            None,
        ),
        op(
            "focusa.mission_canvas.surface_binding.mutate",
            "Bind or Unbind an Exact Work Surface Target",
            "mission_canvas",
            "POST",
            "/v1/mission-canvas/surface-bindings/mutate",
            true,
            None,
            "append_mission_canvas_surface_binding_revision",
            "canonical_projection_event",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["mission_canvas:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.mission_canvas_surface_binding_mutation.request.v1",
            "focusa.mission_canvas_surface_binding_mutation_result.v1",
            "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
            None,
        ),
        op(
            "focusa.mission_canvas.surface.list",
            "List Mission Canvas Work Surface Revisions",
            "mission_canvas",
            "GET",
            "/v1/mission-canvas/surfaces",
            true,
            None,
            "read_mission_canvas_surfaces",
            "canonical_projection_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["mission_canvas:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.mission_canvas_surface_list.request.v1",
            "focusa.mission_canvas_surface_list.v1",
            "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
            None,
        ),
        op(
            "focusa.mission_canvas.surface.mutate",
            "Create, Arrange, Suspend, Resume, or Close Work Surface",
            "mission_canvas",
            "POST",
            "/v1/mission-canvas/surfaces/mutate",
            true,
            None,
            "append_mission_canvas_surface_revision",
            "canonical_projection_event",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["mission_canvas:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.mission_canvas_surface_mutation.request.v1",
            "focusa.mission_canvas_surface_mutation_result.v1",
            "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
            None,
        ),
        op(
            "focusa.mission_canvas.state.get",
            "Rehydrate Exact Mission Canvas Client State",
            "mission_canvas",
            "GET",
            "/v1/mission-canvas/state",
            true,
            None,
            "read_mission_canvas_state",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["mission_canvas:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.mission_canvas_state_get.request.v1",
            "focusa.mission_canvas_state.v1",
            "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
            None,
        ),
        op(
            "focusa.mission_canvas.state.mutate",
            "Persist Exact Mission Canvas Client State",
            "mission_canvas",
            "POST",
            "/v1/mission-canvas/state/mutate",
            true,
            None,
            "append_mission_canvas_state_revision",
            "canonical_projection_event",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["mission_canvas:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.mission_canvas_state_mutation.request.v1",
            "focusa.mission_canvas_state_mutation_result.v1",
            "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
            None,
        ),
        op(
            "focusa.work_rail.list",
            "List Scoped Work Rail Revisions",
            "work_rail",
            "GET",
            "/v1/work-rail",
            true,
            None,
            "read_work_rail",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["work_rail:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.work_rail_list.request.v1",
            "focusa.work_rail_list.v1",
            "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md",
            None,
        ),
        op(
            "focusa.work_rail.mutate",
            "Bind, Activate, Verify, or Close Work Rail Row",
            "work_rail",
            "POST",
            "/v1/work-rail/mutate",
            true,
            None,
            "append_work_rail_revision",
            "canonical_and_provider_event",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            true,
            vec!["work_rail:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.work_rail_mutation.request.v1",
            "focusa.work_rail_mutation_result.v1",
            "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md",
            None,
        ),
        op(
            "focusa.task_plan.materialize.beads",
            "Materialize Approved Task Plan into Canonical Beads",
            "task_plan",
            "POST",
            "/v1/task-plans/materialize/beads",
            true,
            None,
            "materialize_task_plan_beads",
            "external_governed_mutation",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            true,
            vec!["task:materialize"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.task_plan_beads_materialization.request.v1",
            "focusa.task_plan_beads_materialization_result.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.task_plan.list",
            "List Provider-Neutral Task Plan Revisions",
            "task_plan",
            "GET",
            "/v1/task-plans",
            true,
            None,
            "read_task_plan",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["task:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.provider_neutral_task_plan_list.request.v1",
            "focusa.provider_neutral_task_plan_list.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.task_plan.mutate",
            "Draft, Preview, Edit, and Approve Task Plan",
            "task_plan",
            "POST",
            "/v1/task-plans/mutate",
            true,
            None,
            "append_task_plan_revision",
            "canonical_event",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            true,
            vec!["task:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.provider_neutral_task_plan_mutation.request.v1",
            "focusa.provider_neutral_task_plan_mutation_result.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.provider.contract.list",
            "List Provider Governance Contracts",
            "provider_execution",
            "GET",
            "/v1/providers/contracts",
            true,
            None,
            "read_provider_contracts",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["provider:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.provider_contract_list.request.v1",
            "focusa.provider_contract_list.v1",
            "docs/135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md",
            None,
        ),
        op(
            "focusa.provider.conformance.evaluate",
            "Evaluate Provider Governance Conformance",
            "provider_execution",
            "POST",
            "/v1/providers/conformance",
            true,
            None,
            "evaluate_provider_conformance",
            "governed_validation",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["provider:execute"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.provider_conformance.request.v1",
            "focusa.provider_conformance_response.v1",
            "docs/135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md",
            None,
        ),
        op(
            "focusa.spec_workbench.session.list",
            "List Spec Workbench Session Revisions",
            "spec_workbench",
            "GET",
            "/v1/spec-workbench/sessions",
            true,
            None,
            "read_spec_workbench",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["spec:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.spec_workbench_session_list.request.v1",
            "focusa.spec_workbench_session_list.v1",
            "docs/120-adversarial-spec-workbench-and-operator-approval-gates.md",
            None,
        ),
        op(
            "focusa.spec_workbench.session.mutate",
            "Mutate Canonical Spec Workbench",
            "spec_workbench",
            "POST",
            "/v1/spec-workbench/session/mutate",
            true,
            None,
            "append_spec_workbench_revision",
            "canonical_event",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["spec:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.spec_workbench_mutation.request.v1",
            "focusa.spec_workbench_mutation_result.v1",
            "docs/120-adversarial-spec-workbench-and-operator-approval-gates.md",
            None,
        ),
        op(
            "focusa.interview.session.list",
            "List Durable Interview Session Revisions",
            "project_interview",
            "GET",
            "/v1/interviews/sessions",
            true,
            None,
            "read_interview_session",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["interview:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.project_interview_session_list.request.v1",
            "focusa.project_interview_session_list.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.interview.closure_package.get",
            "Get Governed Interview Closure Package",
            "project_interview",
            "GET",
            "/v1/interviews/closure-package",
            true,
            None,
            "project_interview_closure_package",
            "canonical_projection_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["interview:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.project_interview_session_list.request.v1",
            "focusa.interview_closure_package.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.interview.session.mutate",
            "Mutate Durable Interview Session",
            "project_interview",
            "POST",
            "/v1/interviews/sessions/mutate",
            true,
            None,
            "append_interview_session_revision",
            "canonical_event",
            vec!["dry_run", "preview", "commit"],
            true,
            true,
            false,
            vec!["interview:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard", "debug"],
            "focusa.project_interview_session_mutation.request.v1",
            "focusa.project_interview_session_mutation_result.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.interview.strategy.grill_with_docs.next_question",
            "Propose Next Retrieval-Grounded Grill Question",
            "interview_strategy",
            "POST",
            "/v1/interview/strategy/grill-with-docs/next-question",
            true,
            None,
            "propose_one_interview_question",
            "advisory_projection",
            vec!["preview"],
            false,
            false,
            false,
            vec!["interview:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.grill_interview_context.v1",
            "focusa.grill_interview_strategy_response.v1",
            "docs/135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md",
            None,
        ),
        // ── canonical Context corpus ─────────────────────────────────────────
        op(
            "focusa.context.source.list",
            "List Canonical Context Sources",
            "context",
            "GET",
            "/v1/context/sources",
            true,
            None,
            "read_context",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["context:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.context_source_list.request.v1",
            "focusa.context_source_list.v1",
            "docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md",
            None,
        ),
        op(
            "focusa.context.source.commit",
            "Commit Context Source",
            "context",
            "POST",
            "/v1/context/sources/commit",
            true,
            None,
            "write_context",
            "canonical_event",
            vec!["commit"],
            true,
            true,
            false,
            vec!["context:write"],
            false,
            "standard_write",
            vec!["compact", "standard"],
            "focusa.context_source_commit.request.v1",
            "focusa.context_source_commit_result.v1",
            "docs/135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md",
            None,
        ),
        op(
            "focusa.context.source.ingest",
            "Ingest Context Source",
            "context",
            "POST",
            "/v1/context/sources/ingest",
            true,
            None,
            "write_context",
            "canonical_event",
            vec!["commit"],
            true,
            true,
            false,
            vec!["context:write"],
            false,
            "heavy_write",
            vec!["compact", "standard", "debug"],
            "focusa.context_source_ingest.request.v1",
            "focusa.context_source_ingest_result.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.context.retrieve",
            "Retrieve Cited Context",
            "context",
            "POST",
            "/v1/context/retrieve",
            true,
            None,
            "read_context",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["context:read"],
            false,
            "heavy_read",
            vec!["compact", "standard", "debug"],
            "focusa.context_retrieve.request.v1",
            "focusa.context_retrieve_response.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.context.graph.read",
            "Read Context Claim Graph",
            "context",
            "GET",
            "/v1/context/graph",
            true,
            None,
            "read_context",
            "canonical_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["context:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.context_graph_read.request.v1",
            "focusa.context_graph.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.context.graph.mutate",
            "Review Context Claims and Contradictions",
            "context",
            "POST",
            "/v1/context/graph/mutate",
            true,
            None,
            "write_context",
            "canonical_event",
            vec!["commit"],
            true,
            true,
            false,
            vec!["context:write"],
            false,
            "standard_write",
            vec!["compact", "standard", "debug"],
            "focusa.context_graph_mutation.request.v1",
            "focusa.context_graph_mutation_result.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        op(
            "focusa.context.adapter.docling.health",
            "Read Docling Context Adapter Health",
            "context",
            "GET",
            "/v1/context/adapters/docling/health",
            true,
            None,
            "read_context",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["context:read"],
            false,
            "light_read",
            vec!["compact", "standard", "debug"],
            "focusa.context_adapter_health.request.v1",
            "focusa.context_adapter_health.v1",
            "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md",
            None,
        ),
        // ── tool doctor ──────────────────────────────────────────────────────
        op(
            "focusa.tool_doctor",
            "Tool Doctor",
            "diagnostics",
            "GET",
            "/v1/tool-doctor",
            true,
            None,
            "read_diagnostics",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["diagnostics:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.tool_doctor.request.v1",
            "focusa.tool_doctor.response.v1",
            "docs/focusa-api/routes/tool_doctor.md",
            None,
        ),
        // ── awareness ────────────────────────────────────────────────────────
        op(
            "focusa.awareness.packet",
            "Awareness Packet",
            "awareness",
            "POST",
            "/v1/awareness/packet",
            true,
            None,
            "read_awareness",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["awareness:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.awareness_packet.request.v1",
            "focusa.awareness_packet.response.v1",
            "docs/focusa-api/routes/awareness.md",
            None,
        ),
        // ── resource mode ────────────────────────────────────────────────────
        op(
            "focusa.resource_mode",
            "Resource Mode",
            "resource",
            "GET",
            "/v1/resource/mode",
            true,
            None,
            "read_resource",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["resource:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.resource_mode.request.v1",
            "focusa.resource_mode.response.v1",
            "docs/focusa-api/routes/resource.md",
            None,
        ),
        // ── traverse ─────────────────────────────────────────────────────────
        op(
            "focusa.traverse",
            "Traverse",
            "traverse",
            "POST",
            "/v1/traverse",
            true,
            None,
            "read_traverse",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["traverse:read"],
            false,
            "heavy_read",
            vec!["compact", "standard", "debug"],
            "focusa.traverse.request.v1",
            "focusa.traverse.response.v1",
            "docs/focusa-api/routes/traverse.md",
            None,
        ),
        // ── state ────────────────────────────────────────────────────────────
        op(
            "focusa.state.current",
            "State Current",
            "state",
            "GET",
            "/v1/state/current",
            true,
            None,
            "read_state",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["state:read"],
            false,
            "standard_read",
            vec!["compact", "standard", "debug"],
            "focusa.state_current.request.v1",
            "focusa.state_current.response.v1",
            "docs/focusa-api/routes/state.md",
            None,
        ),
        // ── lineage ──────────────────────────────────────────────────────────
        op(
            "focusa.lineage.head",
            "Lineage Head",
            "lineage",
            "GET",
            "/v1/lineage/head",
            true,
            None,
            "read_lineage",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["lineage:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.lineage_head.request.v1",
            "focusa.lineage_head.response.v1",
            "docs/focusa-api/routes/lineage.md",
            None,
        ),
        op(
            "focusa.lineage.tree",
            "Lineage Tree",
            "lineage",
            "GET",
            "/v1/lineage/tree",
            true,
            None,
            "read_lineage_tree",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["lineage:read"],
            false,
            "heavy_read",
            vec!["compact", "standard", "debug"],
            "focusa.lineage_tree.request.v1",
            "focusa.lineage_tree.response.v1",
            "docs/focusa-api/routes/lineage.md",
            None,
        ),
        // ── bloatgaurd ───────────────────────────────────────────────────────
        op(
            "focusa.bloatgaurd.report",
            "Bloatgaurd Report",
            "bloatgaurd",
            "GET",
            "/v1/bloatgaurd/report",
            true,
            None,
            "read_bloatgaurd",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["bloatgaurd:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.bloatgaurd_report.request.v1",
            "focusa.bloatgaurd_report.response.v1",
            "docs/focusa-api/routes/bloatgaurd.md",
            None,
        ),
        // ── device pairing ───────────────────────────────────────────────────
        op(
            "focusa.device_pair.start",
            "Device Pair Start",
            "device",
            "POST",
            "/v1/device/pair/start",
            true,
            None,
            "write_pairing",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            false,
            false,
            false,
            vec!["device:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.device_pair_start.request.v1",
            "focusa.device_pair_start.response.v1",
            "docs/focusa-api/routes/device.md",
            None,
        ),
        op(
            "focusa.device_pair.status",
            "Device Pair Status",
            "device",
            "GET",
            "/v1/device/pair/status",
            true,
            None,
            "read_pairing",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["device:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.device_pair_status.request.v1",
            "focusa.device_pair_status.response.v1",
            "docs/focusa-api/routes/device.md",
            None,
        ),
        // ── work loop ────────────────────────────────────────────────────────
        op(
            "focusa.work_loop.status",
            "Work Loop Status",
            "work_loop",
            "GET",
            "/v1/work-loop/status",
            true,
            None,
            "read_work_loop",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["work_loop:read"],
            false,
            "light_read",
            vec!["compact", "standard"],
            "focusa.work_loop_status.request.v1",
            "focusa.work_loop_status.response.v1",
            "docs/focusa-api/routes/work_loop.md",
            None,
        ),
        op(
            "focusa.work_loop.control",
            "Work Loop Control",
            "work_loop",
            "POST",
            "/v1/work-loop/control",
            true,
            None,
            "write_work_loop",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["work_loop:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.work_loop_control.request.v1",
            "focusa.work_loop_control.response.v1",
            "docs/focusa-api/routes/work_loop.md",
            None,
        ),
        // ── license ──────────────────────────────────────────────────────────
        op(
            "focusa.license.validate",
            "License Validate",
            "license",
            "POST",
            "/v1/license/validate",
            true,
            None,
            "write_license",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["license:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.license_validate.request.v1",
            "focusa.license_validate.response.v1",
            "docs/focusa-api/routes/license.md",
            None,
        ),
        // ── DXUX ─────────────────────────────────────────────────────────────
        op(
            "focusa.dxux.report",
            "DXUX Report",
            "dxux",
            "GET",
            "/v1/dxux/report",
            true,
            None,
            "read_dxux",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["dxux:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.dxux_report.request.v1",
            "focusa.dxux_report.response.v1",
            "docs/focusa-api/routes/dxux.md",
            None,
        ),
        // ── call stack ───────────────────────────────────────────────────────
        op(
            "focusa.call_stack.design",
            "Call Stack Design",
            "call_stack",
            "POST",
            "/v1/call-stack/design",
            true,
            None,
            "write_call_stack",
            "daemon_dispatch",
            vec!["dry_run", "preview", "commit"],
            true,
            false,
            false,
            vec!["call_stack:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.call_stack_design.request.v1",
            "focusa.call_stack_design.response.v1",
            "docs/focusa-api/routes/call_stack.md",
            None,
        ),
        op(
            "focusa.call_stack.verify",
            "Call Stack Verify",
            "call_stack",
            "POST",
            "/v1/call-stack/verify",
            true,
            None,
            "read_call_stack_verify",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["call_stack:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.call_stack_verify.request.v1",
            "focusa.call_stack_verify.response.v1",
            "docs/focusa-api/routes/call_stack.md",
            None,
        ),
        // ── turn lifecycle ──────────────────────────────────────────────────
        op(
            "focusa.turn.start",
            "Start Turn",
            "turn",
            "POST",
            "/v1/turn/start",
            true,
            None,
            "write_turn",
            "daemon_dispatch",
            vec!["commit"],
            false,
            false,
            false,
            vec!["turn:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.turn_start.request.v1",
            "focusa.turn_start.response.v1",
            "docs/G1-detail-04-proxy-adapter.md",
            None,
        ),
        op(
            "focusa.turn.append",
            "Append Turn Chunk",
            "turn",
            "POST",
            "/v1/turn/append",
            true,
            None,
            "write_turn_chunk",
            "daemon_dispatch",
            vec!["commit"],
            false,
            false,
            false,
            vec!["turn:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.turn_append.request.v1",
            "focusa.turn_append.response.v1",
            "docs/G1-detail-04-proxy-adapter.md",
            None,
        ),
        op(
            "focusa.turn.complete",
            "Complete Turn",
            "turn",
            "POST",
            "/v1/turn/complete",
            true,
            None,
            "write_turn_completion",
            "daemon_dispatch",
            vec!["commit"],
            false,
            false,
            false,
            vec!["turn:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.turn_complete.request.v1",
            "focusa.turn_complete.response.v1",
            "docs/G1-detail-04-proxy-adapter.md",
            None,
        ),
        // ── memory ──────────────────────────────────────────────────────────
        op(
            "focusa.memory.semantic.read",
            "Read Semantic Memory",
            "memory",
            "GET",
            "/v1/memory/semantic",
            true,
            None,
            "read_memory",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["memory:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.memory_semantic_read.request.v1",
            "focusa.memory_semantic_read.response.v1",
            "docs/G1-detail-04-proxy-adapter.md",
            None,
        ),
        op(
            "focusa.memory.semantic.upsert",
            "Upsert Semantic Memory",
            "memory",
            "POST",
            "/v1/memory/semantic/upsert",
            true,
            None,
            "write_memory",
            "daemon_dispatch",
            vec!["commit"],
            false,
            false,
            false,
            vec!["memory:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.memory_semantic_upsert.request.v1",
            "focusa.memory_semantic_upsert.response.v1",
            "docs/G1-detail-04-proxy-adapter.md",
            None,
        ),
        op(
            "focusa.memory.procedural.read",
            "Read Procedural Memory",
            "memory",
            "GET",
            "/v1/memory/procedural",
            true,
            None,
            "read_memory",
            "advisory_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["memory:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.memory_procedural_read.request.v1",
            "focusa.memory_procedural_read.response.v1",
            "docs/G1-detail-04-proxy-adapter.md",
            None,
        ),
        op(
            "focusa.memory.procedural.reinforce",
            "Reinforce Procedural Rule",
            "memory",
            "POST",
            "/v1/memory/procedural/reinforce",
            true,
            None,
            "write_memory",
            "daemon_dispatch",
            vec!["commit"],
            false,
            false,
            false,
            vec!["memory:write"],
            false,
            "standard_mutation",
            vec!["compact", "standard"],
            "focusa.memory_procedural_reinforce.request.v1",
            "focusa.memory_procedural_reinforce.response.v1",
            "docs/G1-detail-04-proxy-adapter.md",
            None,
        ),
        op(
            "focusa.operation_registry.read",
            "Read Operation Registry",
            "agent",
            "GET",
            "/v1/agent/operations",
            true,
            None,
            "read_state",
            "registry_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec![],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.operation_registry.request.v1",
            "focusa.operation_registry.response.v1",
            "docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md",
            None,
        ),
        op(
            "focusa.ui_action_bindings.read",
            "Read Generated UI Action Bindings",
            "agent",
            "GET",
            "/v1/agent/ui-action-bindings",
            true,
            None,
            "read_state",
            "registry_projection",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["project:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.ui_action_bindings.request.v1",
            "focusa.ui_action_bindings.response.v1",
            "docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md",
            None,
        ),
        op(
            "focusa.ui_capability_snapshot.read",
            "Read Generated UI Capability Snapshot",
            "agent",
            "GET",
            "/v1/agent/ui-capabilities",
            true,
            None,
            "read_state",
            "capability_projection",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["project:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.ui_capability_snapshot.request.v1",
            "focusa.ui_capability_snapshot.response.v1",
            "docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md",
            None,
        ),
        op(
            "focusa.events.stream",
            "Replay and Stream Durable Events",
            "events",
            "GET",
            "/v1/events/stream",
            true,
            None,
            "read_state",
            "durable_replay_live_tail",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["events:read"],
            false,
            "streaming_read",
            vec!["stream"],
            "focusa.events_stream.request.v1",
            "focusa.stream_event.v1",
            "docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md",
            None,
        ),
        op(
            "focusa.compatibility_lock.read",
            "Read Protocol Compatibility Lock",
            "agent",
            "GET",
            "/v1/agent/compatibility-lock",
            true,
            None,
            "read_state",
            "compatibility_read",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec![],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.compatibility_lock.request.v1",
            "focusa.compatibility_lock.v1",
            "docs/135e-cross-spec-amendments-migration-and-closure-matrix.md",
            None,
        ),
        op(
            "focusa.protocol.handshake",
            "Negotiate Focusa Protocol Compatibility",
            "agent",
            "POST",
            "/v1/agent/handshake",
            true,
            None,
            "read_state",
            "fail_closed_negotiation",
            vec!["preview", "commit"],
            false,
            false,
            false,
            vec!["project:read"],
            false,
            "standard_read",
            vec!["compact", "standard"],
            "focusa.protocol_handshake.request.v1",
            "focusa.protocol_handshake.response.v1",
            "docs/135e-cross-spec-amendments-migration-and-closure-matrix.md",
            None,
        ),
        op(
            "focusa.agent_execution.start",
            "Start or Resume Governed Pi RPC Execution",
            "work_loop",
            "POST",
            "/v1/work-loop/driver/start",
            true,
            None,
            "write_process",
            "pi_rpc",
            vec!["commit"],
            true,
            false,
            false,
            vec!["work-loop:write"],
            false,
            "process_control",
            vec!["standard"],
            "focusa.agent_execution_start.request.v1",
            "focusa.agent_execution_adapter_result.v1",
            "docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md",
            None,
        ),
        op(
            "focusa.agent_execution.prompt",
            "Prompt Governed Pi RPC Execution",
            "work_loop",
            "POST",
            "/v1/work-loop/driver/prompt",
            true,
            None,
            "write_process",
            "pi_rpc",
            vec!["commit"],
            true,
            false,
            false,
            vec!["work-loop:write"],
            false,
            "process_control",
            vec!["standard"],
            "focusa.agent_execution_prompt.request.v1",
            "focusa.agent_execution_adapter_result.v1",
            "docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md",
            None,
        ),
        op(
            "focusa.agent_execution.abort",
            "Abort Current Pi RPC Turn",
            "work_loop",
            "POST",
            "/v1/work-loop/driver/abort",
            true,
            None,
            "write_process",
            "pi_rpc",
            vec!["commit"],
            true,
            false,
            false,
            vec!["work-loop:write"],
            false,
            "process_control",
            vec!["compact"],
            "focusa.agent_execution_abort.request.v1",
            "focusa.agent_execution_adapter_result.v1",
            "docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md",
            None,
        ),
        op(
            "focusa.agent_execution.stop",
            "Stop Governed Pi RPC Execution",
            "work_loop",
            "POST",
            "/v1/work-loop/driver/stop",
            true,
            None,
            "write_process",
            "pi_rpc",
            vec!["commit"],
            true,
            false,
            false,
            vec!["work-loop:write"],
            false,
            "process_control",
            vec!["compact"],
            "focusa.agent_execution_stop.request.v1",
            "focusa.agent_execution_adapter_result.v1",
            "docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md",
            None,
        ),
    ]
}

fn build_families() -> Vec<&'static str> {
    vec![
        "health",
        "agent",
        "events",
        "project",
        "trajectory",
        "workpoint",
        "metacognition",
        "evidence",
        "prediction",
        "context_cognition",
        "context",
        "diagnostics",
        "awareness",
        "resource",
        "traverse",
        "state",
        "lineage",
        "bloatgaurd",
        "device",
        "work_loop",
        "license",
        "dxux",
        "call_stack",
        "turn",
        "memory",
    ]
}

async fn adapter_capabilities_handler(
    State(_state): State<Arc<AppState>>,
) -> Json<AdapterCapabilityRegistry> {
    Json(ADAPTER_CAPABILITY_REGISTRY.clone())
}

static SPEC141_CAPABILITY_REGISTRY: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../../docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json"
    ))
    .expect("generated Spec141 capability registry must be valid JSON")
});

static SPEC141_AGENT_CARD: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../../docs/contracts/spec141/generated-capability-v2/agent-card.json"
    ))
    .expect("generated Spec141 Agent Card must be valid JSON")
});

#[derive(Debug, Default, Deserialize)]
struct AgentToolQuery {
    query: Option<String>,
    family: Option<String>,
    cursor: Option<usize>,
    limit: Option<usize>,
    include_schemas: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct AgentToolGraphQuery {
    anchor: Option<String>,
    depth: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
struct AgentToolChangesQuery {
    since_digest: Option<String>,
}

fn spec141_descriptors() -> &'static [Value] {
    SPEC141_CAPABILITY_REGISTRY
        .get("descriptors")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn descriptor_name(descriptor: &Value) -> &str {
    descriptor
        .pointer("/tool_names/pi")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

fn metadata_only(mut descriptor: Value) -> Value {
    if let Some(object) = descriptor.as_object_mut() {
        object.remove("input_schema");
        object.remove("output_schema");
        object.remove("error_schema");
        object.insert(
            "schema_loading".to_string(),
            Value::String("deferred".to_string()),
        );
    }
    descriptor
}

async fn agent_card_handler(State(_state): State<Arc<AppState>>) -> Json<Value> {
    Json(SPEC141_AGENT_CARD.clone())
}

async fn agent_tools_handler(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<AgentToolQuery>,
) -> Json<Value> {
    let terms: Vec<String> = query
        .query
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    let mut matches: Vec<(usize, Value)> = spec141_descriptors()
        .iter()
        .filter(|descriptor| {
            query.family.as_deref().is_none_or(|family| {
                descriptor.get("family").and_then(Value::as_str) == Some(family)
            })
        })
        .filter_map(|descriptor| {
            let searchable = [
                descriptor_name(descriptor),
                descriptor
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                descriptor
                    .get("summary")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                descriptor
                    .get("family")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            ]
            .join(" ")
            .to_lowercase();
            let score = terms
                .iter()
                .filter(|term| searchable.contains(term.as_str()))
                .count();
            (terms.is_empty() || score > 0).then(|| (score, descriptor.clone()))
        })
        .collect();
    matches.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| descriptor_name(&a.1).cmp(descriptor_name(&b.1)))
    });
    let total = matches.len();
    let cursor = query.cursor.unwrap_or(0).min(total);
    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let end = (cursor + limit).min(total);
    let include_schemas = query.include_schemas.unwrap_or(false);
    let tools: Vec<Value> = matches[cursor..end]
        .iter()
        .map(|(_, descriptor)| {
            if include_schemas {
                descriptor.clone()
            } else {
                metadata_only(descriptor.clone())
            }
        })
        .collect();
    Json(json!({
        "schema": "focusa.agent_tool_search.v2",
        "registry_digest": SPEC141_CAPABILITY_REGISTRY.get("registry_digest"),
        "query": query.query,
        "family": query.family,
        "total": total,
        "cursor": cursor,
        "next_cursor": (end < total).then_some(end),
        "schema_loading": if include_schemas { "cold_loaded" } else { "deferred" },
        "tools": tools,
    }))
}

async fn agent_tool_describe_handler(
    State(_state): State<Arc<AppState>>,
    Path(tool_name): Path<String>,
) -> (StatusCode, Json<Value>) {
    match spec141_descriptors()
        .iter()
        .find(|descriptor| descriptor_name(descriptor) == tool_name)
    {
        Some(descriptor) => (StatusCode::OK, Json(descriptor.clone())),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "schema": "focusa.agent_tool_error.v1",
                "status": "not_found",
                "tool_name": tool_name,
                "recovery": ["GET /v1/agent/tools?query=<terms>"]
            })),
        ),
    }
}

async fn agent_tool_bundles_handler(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<AgentToolQuery>,
) -> Json<Value> {
    let family = query.family.unwrap_or_default();
    let include_schemas = query.include_schemas.unwrap_or(false);
    let limit = query.limit.unwrap_or(25).clamp(1, 50);
    let tools: Vec<Value> = spec141_descriptors()
        .iter()
        .filter(|descriptor| {
            descriptor.get("family").and_then(Value::as_str) == Some(family.as_str())
        })
        .take(limit)
        .map(|descriptor| {
            if include_schemas {
                descriptor.clone()
            } else {
                metadata_only(descriptor.clone())
            }
        })
        .collect();
    Json(json!({
        "schema": "focusa.agent_tool_bundle.v1",
        "registry_digest": SPEC141_CAPABILITY_REGISTRY.get("registry_digest"),
        "family": family,
        "count": tools.len(),
        "schema_loading": if include_schemas { "cold_loaded" } else { "deferred" },
        "tools": tools,
    }))
}

async fn agent_tool_changes_handler(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<AgentToolChangesQuery>,
) -> Json<Value> {
    let digest = SPEC141_CAPABILITY_REGISTRY
        .get("registry_digest")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Json(json!({
        "schema": "focusa.agent_tool_changes.v1",
        "registry_digest": digest,
        "since_digest": query.since_digest,
        "list_changed": query.since_digest.as_deref() != Some(digest),
        "capability_count": spec141_descriptors().len(),
        "recovery": if query.since_digest.as_deref() == Some(digest) { Value::Null } else { json!("refresh tools/list or GET /v1/agent/tools") },
    }))
}

async fn agent_tool_graph_handler(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<AgentToolGraphQuery>,
) -> Json<Value> {
    let anchor = query.anchor.unwrap_or_else(|| "workpoint".to_string());
    let depth = query.depth.unwrap_or(2).clamp(1, 4);
    let limit = query.limit.unwrap_or(40).clamp(1, 100);
    let mut nodes: std::collections::BTreeSet<String> = spec141_descriptors()
        .iter()
        .filter(|descriptor| {
            descriptor_name(descriptor) == anchor
                || descriptor.get("family").and_then(Value::as_str) == Some(anchor.as_str())
        })
        .map(|descriptor| descriptor_name(descriptor).to_string())
        .collect();
    let mut frontier: Vec<String> = nodes.iter().cloned().collect();
    let mut edges = Vec::new();
    for _ in 0..depth {
        let mut next = Vec::new();
        for from in &frontier {
            let Some(descriptor) = spec141_descriptors()
                .iter()
                .find(|descriptor| descriptor_name(descriptor) == from)
            else {
                continue;
            };
            for target in descriptor
                .get("likely_next_capabilities")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                edges.push(json!({"from": from, "to": target, "relation": "likely_next"}));
                if nodes.len() < limit && nodes.insert(target.to_string()) {
                    next.push(target.to_string());
                }
            }
        }
        frontier = next;
        if frontier.is_empty() || nodes.len() >= limit {
            break;
        }
    }
    Json(json!({
        "schema": "focusa.agent_tool_graph.v1",
        "registry_digest": SPEC141_CAPABILITY_REGISTRY.get("registry_digest"),
        "anchor": anchor,
        "depth": depth,
        "nodes": nodes,
        "edges": edges,
    }))
}

pub async fn capabilities_index_handler(State(_state): State<Arc<AppState>>) -> Json<Value> {
    let operations = build_operations();
    let families = build_families();
    let index = CapabilitiesIndex {
        schema: "focusa.agent_capabilities.index.v1",
        api_version: "v1",
        generated_at: Utc::now().to_rfc3339(),
        operation_count: operations.len(),
        families,
        operations,
    };
    Json(serde_json::to_value(&index).unwrap_or(json!({
        "status": "failed", "failure_class": "operation_failed", "retry_posture": "safe_retry", "safe_recovery": "retry the operation or inspect the recovery graph",
        "failure_class": "serialization_error",
        "message": "Failed to serialize capabilities index"
    })))
}

pub(crate) fn registered_operation_ids() -> std::collections::BTreeSet<String> {
    build_operations()
        .into_iter()
        .map(|operation| operation.operation_id.to_string())
        .collect()
}

pub async fn operation_registry_handler(State(_state): State<Arc<AppState>>) -> Json<Value> {
    let operations = build_operations();
    Json(json!({
        "schema": "focusa.operation_registry.v1",
        "registry_version": "1.0.0",
        "generated_at": Utc::now().to_rfc3339(),
        "operation_count": operations.len(),
        "operations": operations,
    }))
}

#[derive(Debug, Default, Deserialize)]
struct UiProjectionQuery {
    project_root: Option<String>,
    continuity_id: Option<String>,
    attachment_id: Option<String>,
    agent_id: Option<String>,
}

fn projection_scope_error(scope: &UiProjectionQuery) -> Option<ToolResultV1> {
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
    if project_root.is_empty() || !std::path::Path::new(project_root).is_absolute() {
        return Some(
            ToolResultV1::failure(
                ToolStatus::ValidationRejected,
                FailureClass::ScopeMismatch,
                "project_root must be a non-empty absolute path",
            )
            .with_recovery(
                "Resolve project identity and retry with its exact project_root",
                "Do not infer project scope from a folder name",
                ["focusa_project_identity", "focusa_project_verify"],
            ),
        );
    }
    if continuity_id.is_empty() {
        return Some(
            ToolResultV1::failure(
                ToolStatus::ValidationRejected,
                FailureClass::ScopeMismatch,
                "continuity_id is required for workstream-exact projection",
            )
            .with_recovery(
                "Resume the canonical Workpoint and use its continuity_id",
                "Do not merge permission state across workstreams",
                ["focusa_workpoint_resume", "focusa_project_identity"],
            ),
        );
    }
    None
}

fn projection_scope_rejection(result: ToolResultV1) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(
            serde_json::to_value(result)
                .unwrap_or_else(|_| json!({"schema": TOOL_RESULT_SCHEMA, "ok": false})),
        ),
    )
}

fn ui_action_bindings_document(scope: &UiProjectionQuery) -> Value {
    let bindings: Vec<Value> = build_operations()
        .into_iter()
        .filter(|operation| operation.ui.allowed_in_generated_ui)
        .map(|operation| {
            json!({
                "schema": "focusa.ui_action_binding.v1",
                "action_id": operation.operation_id,
                "operation_descriptor_ref": format!("/v1/agent/operations#{}", operation.operation_id),
                "canonical_revision": operation.operation_version,
                "scope": {
                    "project_root": scope.project_root.as_deref(),
                    "continuity_id": scope.continuity_id.as_deref(),
                    "attachment_id": scope.attachment_id.as_deref(),
                    "required_keys": operation.scope.required_keys,
                },
                "capability_refs": operation.control.capability_refs,
                "permission_scopes": operation.control.permission_scopes,
                "contracts": operation.contracts,
                "control": {
                    "mode": operation.control.mode,
                    "confirmation": operation.control.confirmation,
                    "idempotency_required": operation.control.idempotency_required,
                    "optimistic_concurrency_required": operation.control.optimistic_concurrency_required,
                    "receipt_required": operation.control.receipt_required,
                    "reversible": operation.control.reversible,
                },
                "presentation": operation.ui,
                "result_envelope_ref": "focusa.tool_result.v1",
                "recovery_envelope_ref": "focusa.tool_result.v1",
            })
        })
        .collect();
    json!({
        "schema": "focusa.ui_action_binding_index.v1",
        "registry_version": "1.0.0",
        "project_root": scope.project_root.as_deref(),
        "continuity_id": scope.continuity_id.as_deref(),
        "attachment_id": scope.attachment_id.as_deref(),
        "binding_count": bindings.len(),
        "bindings": bindings,
    })
}

async fn ui_action_bindings_handler(
    Query(scope): Query<UiProjectionQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Some(error) = projection_scope_error(&scope) {
        return Err(projection_scope_rejection(error));
    }
    Ok(Json(ui_action_bindings_document(&scope)))
}

fn ui_capability_snapshot_document(
    scope: &UiProjectionQuery,
    permissions: &PermissionContext,
) -> Value {
    let operations = build_operations();
    let capability_ids: std::collections::BTreeSet<_> = operations
        .iter()
        .flat_map(|operation| operation.control.capability_refs.iter().copied())
        .collect();
    let permission_scopes: std::collections::BTreeSet<_> = operations
        .iter()
        .flat_map(|operation| operation.control.permission_scopes.iter().copied())
        .collect();
    let granted_scopes: Vec<_> = permission_scopes
        .iter()
        .copied()
        .filter(|scope| permissions.allows(scope))
        .collect();
    let missing_scopes: Vec<_> = permission_scopes
        .iter()
        .copied()
        .filter(|scope| !permissions.allows(scope))
        .collect();
    let capabilities: Vec<Value> = capability_ids
        .into_iter()
        .map(|capability_id| {
            let required: std::collections::BTreeSet<_> = operations
                .iter()
                .filter(|operation| operation.control.capability_refs.contains(&capability_id))
                .flat_map(|operation| operation.control.permission_scopes.iter().copied())
                .collect();
            let missing: Vec<_> = required
                .into_iter()
                .filter(|required_scope| !permissions.allows(required_scope))
                .collect();
            let available = missing.is_empty();
            json!({
                "capability_id": capability_id,
                "status": if available { "available" } else { "approval_required" },
                "reason": if available {
                    "Required permission scopes are granted"
                } else {
                    "One or more required permission scopes are missing"
                },
                "missing_permission_scopes": missing,
                "recovery_action_ref": if available { Value::Null } else { json!("focusa_tool_doctor") },
            })
        })
        .collect();
    json!({
        "schema": "focusa.ui_capability_snapshot.v1",
        "project_root": scope.project_root.as_deref(),
        "continuity_id": scope.continuity_id.as_deref(),
        "attachment_id": scope.attachment_id.as_deref(),
        "agent_id": scope.agent_id.as_deref(),
        "scope_validated": true,
        "capabilities": capabilities,
        "permissions": {
            "granted_scopes": granted_scopes,
            "missing_scopes": missing_scopes,
            "effective_token_scopes": permissions.list(),
        },
        "providers": [],
        "connectors": [],
        "client_capabilities": ["openapi-3.0.3", "json-schema-2020-12", "a2ui-web_core-0.9.1", "a2ui-lit-0.9.1", "focusa-svelte-elements-0.9.120-dev", "a2ui-action-bindings", "protocol-handshake-v1"],
        "source_state_revision": "operation-registry-1.0.0",
    })
}

async fn ui_capability_snapshot_handler(
    State(state): State<Arc<AppState>>,
    Query(scope): Query<UiProjectionQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Some(error) = projection_scope_error(&scope) {
        return Err(projection_scope_rejection(error));
    }
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    Ok(Json(ui_capability_snapshot_document(&scope, &permissions)))
}

const REQUIRED_PROTOCOL_VERSIONS: [(&str, &str); 8] = [
    ("focusa_api", "1.0.0"),
    ("operation_registry", "1.0.0"),
    ("tool_result", "1.0.0"),
    ("event_stream", "1.0.0"),
    ("openapi", "3.0.3"),
    ("json_schema", "2020-12"),
    ("a2ui_protocol", "0.9.1"),
    ("a2ui_catalog", "0.9.1"),
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtocolHandshakeRequest {
    client_id: String,
    #[serde(default)]
    client_versions: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    requested_capabilities: Vec<String>,
}

fn compatibility_lock_document() -> Value {
    let minimum_reader_versions: std::collections::BTreeMap<_, _> =
        REQUIRED_PROTOCOL_VERSIONS.into_iter().collect();
    json!({
        "schema": "focusa.compatibility_lock.v1",
        "focusa_runtime": env!("CARGO_PKG_VERSION"),
        "focusa_api": "1.0.0",
        "operation_registry": "1.0.0",
        "tool_result": "1.0.0",
        "event_stream": "1.0.0",
        "a2ui_protocol": "0.9.1",
        "a2ui_catalog": "0.9.1",
        "ag_ui_adapter": "0.0.0",
        "pi_runtime": env!("CARGO_PKG_VERSION"),
        "uiai_engine": "external",
        "uiai_focusa_client": "0.0.0",
        "docling": "external",
        "embedding_profile": "project-configured",
        "domain_pack_versions": [],
        "minimum_reader_versions": minimum_reader_versions,
        "minimum_writer_versions": {
            "focusa_api": "1.0.0",
            "operation_registry": "1.0.0",
            "tool_result": "1.0.0",
            "event_stream": "1.0.0",
            "a2ui_protocol": "0.9.1",
            "a2ui_catalog": "0.9.1"
        }
    })
}

fn handshake_mismatches(request: &ProtocolHandshakeRequest) -> Vec<Value> {
    REQUIRED_PROTOCOL_VERSIONS
        .iter()
        .filter_map(|(component, required)| {
            let actual = request.client_versions.get(*component).map(String::as_str);
            (actual != Some(*required)).then(|| {
                json!({
                    "component": component,
                    "required": required,
                    "actual": actual,
                    "upgrade_action": format!("upgrade {component} support to {required}"),
                })
            })
        })
        .collect()
}

async fn compatibility_lock_handler() -> Json<Value> {
    Json(compatibility_lock_document())
}

async fn protocol_handshake_handler(
    State(state): State<Arc<AppState>>,
    Query(scope): Query<UiProjectionQuery>,
    headers: HeaderMap,
    Json(request): Json<ProtocolHandshakeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Some(error) = projection_scope_error(&scope) {
        return Err(projection_scope_rejection(error));
    }
    let permissions = permission_context(&headers, state.config.auth_token.is_some());
    if !permissions.allows("project:read") {
        let mut result = ToolResultV1::failure(
            ToolStatus::Blocked,
            FailureClass::PermissionDenied,
            "Protocol handshake requires project:read",
        )
        .with_recovery(
            "Request project:read for this exact project and workstream",
            "Do not reuse permission state from another scope",
            ["focusa_tool_doctor", "focusa_project_verify"],
        );
        result.raw = Some(json!({
            "project_root": scope.project_root,
            "continuity_id": scope.continuity_id,
            "required_permission": "project:read",
        }));
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::to_value(result).unwrap_or_default()),
        ));
    }

    let mismatches = handshake_mismatches(&request);
    if !mismatches.is_empty() {
        let mut result = ToolResultV1::failure(
            ToolStatus::Blocked,
            FailureClass::StaleRuntimeRegistry,
            "Protocol handshake blocked by incompatible or missing component versions",
        )
        .with_recovery(
            "Upgrade every listed component, then repeat the startup handshake",
            "Silent compatibility guessing and partial startup are forbidden",
            ["focusa_tool_doctor", "focusa_project_identity"],
        );
        result.raw = Some(json!({
            "client_id": request.client_id,
            "compatible": false,
            "mismatches": mismatches,
            "safe_state_retained": true,
            "compatibility_lock": compatibility_lock_document(),
        }));
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::to_value(result).unwrap_or_default()),
        ));
    }

    let capability_snapshot = ui_capability_snapshot_document(&scope, &permissions);
    Ok(Json(json!({
        "schema": "focusa.protocol_handshake.response.v1",
        "status": "accepted",
        "compatible": true,
        "client_id": request.client_id,
        "project_root": scope.project_root,
        "continuity_id": scope.continuity_id,
        "requested_capabilities": request.requested_capabilities,
        "server_versions": REQUIRED_PROTOCOL_VERSIONS.into_iter().collect::<std::collections::BTreeMap<_, _>>(),
        "capability_snapshot": capability_snapshot,
        "compatibility_lock": compatibility_lock_document(),
        "safe_state_retained": true,
    })))
}

const JSON_SCHEMA_DIALECT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";
const OPENAPI_VERSION: &str = "3.0.3";

fn schema_component_name(schema_id: &str) -> String {
    schema_id.replace(['.', '/'], "_")
}

fn registered_schema_ids() -> std::collections::BTreeSet<&'static str> {
    let mut schema_ids: std::collections::BTreeSet<_> = build_operations()
        .iter()
        .flat_map(|op| [op.request_schema_ref, op.response_schema_ref])
        .collect();
    schema_ids.insert(TOOL_RESULT_SCHEMA);
    schema_ids.insert("focusa.workspace_event.v1");
    schema_ids
}

fn context_source_record_schema() -> Value {
    json!({
        "type": "object",
        "required": ["source_id", "project_root", "continuity_id", "attachment_id", "source_kind", "title", "content", "content_hash", "idempotency_key", "revision", "committed_at", "evidence", "receipt"],
        "properties": {
            "source_id": {"type": "string"}, "project_root": {"type": "string"},
            "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"},
            "source_kind": {"enum": ["markdown", "text", "code", "pdf"]},
            "title": {"type": "string", "maxLength": 240},
            "content": {"type": "string", "maxLength": 2097152},
            "content_hash": {"type": "string", "pattern": "^[a-f0-9]{64}$"},
            "idempotency_key": {"type": "string", "maxLength": 160},
            "revision": {"type": "integer", "minimum": 1},
            "committed_at": {"type": "string", "format": "date-time"},
            "evidence": {
                "type": "object",
                "required": ["evidence_ref", "target_ref", "result", "content_hash", "captured_at"],
                "properties": {
                    "evidence_ref": {"type": "string"}, "target_ref": {"type": "string"},
                    "result": {"type": "string"}, "content_hash": {"type": "string"},
                    "captured_at": {"type": "string", "format": "date-time"}
                }
            },
            "receipt": {
                "type": "object",
                "required": ["receipt_ref", "operation_id", "idempotency_key", "before_state_version", "after_state_version", "reversible", "committed_at"],
                "properties": {
                    "receipt_ref": {"type": "string"},
                    "operation_id": {"enum": ["focusa.context.source.commit", "focusa.context.source.ingest"]},
                    "idempotency_key": {"type": "string"},
                    "before_state_version": {"type": "integer", "minimum": 0},
                    "after_state_version": {"type": "integer", "minimum": 1},
                    "reversible": {"type": "boolean"},
                    "committed_at": {"type": "string", "format": "date-time"}
                }
            },
            "source_locator": {"type": "string"},
            "source_revision": {"type": "string"},
            "mime_type": {"type": "string"},
            "adapter_id": {"type": "string"},
            "ingestion_status": {"type": "string"},
            "extraction_diagnostics": {"type": "array", "items": {"type": "string"}},
            "health": {
                "type": "object",
                "required": ["status", "adapter_id", "message"],
                "properties": {
                    "status": {"type": "string"},
                    "adapter_id": {"type": "string"},
                    "message": {"type": "string"},
                    "recovery_action": {"type": "string"},
                    "last_successful_sync": {"type": "string", "format": "date-time"}
                }
            }
        }
    })
}

fn workspace_artifact_schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["artifact_id", "artifact_kind", "mime_type", "title", "summary", "content", "source", "scope", "origin", "trust", "semantic", "diagnostics_refs", "evidence_refs", "retention", "render", "idempotency_key", "revision", "linked_at", "updated_at"],
        "properties": {
            "artifact_id": {"type": "string"},
            "artifact_kind": {"enum": ["image", "markdown", "dataset", "diff", "browser_snapshot", "diagnostics", "chart", "document", "media", "fpv_session"]},
            "mime_type": {"type": "string"}, "title": {"type": "string"}, "summary": {"type": "string", "maxLength": 2000},
            "content": {"type": "object", "additionalProperties": false, "required": ["handle_ref", "sha256", "size_bytes"], "properties": {
                "handle_ref": {"type": "string"}, "artifact_url": {"type": "string"}, "artifact_path": {"type": "string"},
                "inline_preview": {"type": "string", "maxLength": 2000}, "sha256": {"type": "string", "pattern": "^[A-Fa-f0-9]{64}$"}, "size_bytes": {"type": "integer", "minimum": 0}
            }},
            "source": {"type": "object", "additionalProperties": false, "required": ["system", "source_ref", "captured_at"], "properties": {
                "system": {"enum": ["uiai", "focusa", "local_file", "connector", "provider", "operator"]}, "source_ref": {"type": "string"},
                "source_url": {"type": "string"}, "captured_at": {"type": "string", "format": "date-time"}
            }},
            "scope": {"type": "object", "additionalProperties": false, "required": ["project_root", "continuity_id"], "properties": {
                "project_root": {"type": "string"}, "continuity_id": {"type": "string"}, "project_identity_ref": {"type": "string"},
                "workpoint_id": {"type": "string"}, "work_item_ref": {"type": "string"}
            }},
            "origin": {"type": "object", "additionalProperties": false, "required": ["instance_id", "attachment_id"], "properties": {
                "instance_id": {"type": "string"}, "attachment_id": {"type": "string"}, "focusa_session_id": {"type": "string"}, "work_surface_id": {"type": "string"},
                "harness_session_ref": {"type": "string"}, "silent_session_id": {"type": "string"}, "silent_run_id": {"type": "string"},
                "uiai_session_id": {"type": "string"}, "browser_context_id": {"type": "string"}, "browser_target_id": {"type": "string"}
            }},
            "trust": {"type": "object", "additionalProperties": false, "required": ["evidence_status", "redaction_status", "freshness_status", "provenance_status"], "properties": {
                "evidence_status": {"enum": ["proposal_only", "capture_pending", "captured", "linked", "verified", "stale", "blocked", "scope_mismatch"]},
                "redaction_status": {"type": "string"}, "freshness_status": {"type": "string"}, "provenance_status": {"type": "string"}
            }},
            "semantic": {"type": "object", "additionalProperties": false,
                "required": ["domain_pack_refs", "candidate_object_refs", "candidate_link_refs", "candidate_claim_refs", "verification_policy_refs", "semantic_delta_refs", "citation_refs"],
                "properties": {
                    "domain_pack_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                    "candidate_object_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                    "candidate_link_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                    "candidate_claim_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                    "verification_policy_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                    "semantic_delta_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                    "citation_refs": {"type": "array", "maxItems": 64, "items": {"type": "string"}}
                }
            },
            "diagnostics_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
            "evidence_refs": {"type": "array", "minItems": 1, "maxItems": 32, "items": {"type": "string"}},
            "retention": {"type": "object", "additionalProperties": false, "required": ["policy", "cleanup_action"], "properties": {
                "policy": {"type": "string"}, "expires_at": {"type": "string", "format": "date-time"}, "cleanup_action": {"type": "string"}
            }},
            "render": {"type": "object", "additionalProperties": false, "required": ["preferred_renderer", "fallback_renderer"], "properties": {
                "preferred_renderer": {"type": "string"}, "fallback_renderer": {"type": "string"},
                "width": {"type": "integer", "minimum": 1, "maximum": 16384}, "height": {"type": "integer", "minimum": 1, "maximum": 16384}
            }},
            "idempotency_key": {"type": "string"}, "revision": {"type": "integer", "minimum": 1},
            "linked_at": {"type": "string", "format": "date-time"}, "updated_at": {"type": "string", "format": "date-time"}
        }
    })
}

fn provider_neutral_task_schema() -> Value {
    let strings = || json!({"type":"array","items":{"type":"string"}});
    json!({"type":"object","required":["provider_neutral_id","title","description","order_index","linked_spec_sections","requirement_refs","acceptance_criteria","evidence_requirements","semantic_object_refs","allowed_action_type_ids","verification_policy_ref","allowed_scope","dependencies","blockers","task_class","closure_kind","closure_policy_ref"],"properties":{"provider_neutral_id":{"type":"string"},"title":{"type":"string"},"description":{"type":"string"},"order_index":{"type":"integer","minimum":0},"linked_spec_sections":strings(),"requirement_refs":strings(),"acceptance_criteria":strings(),"evidence_requirements":strings(),"semantic_object_refs":strings(),"allowed_action_type_ids":strings(),"verification_policy_ref":{"type":"string"},"allowed_scope":strings(),"dependencies":strings(),"blockers":strings(),"task_class":{"type":"string"},"closure_kind":{"type":"string"},"closure_policy_ref":{"type":"string"},"preferred_provider":{"type":"string"},"provider_ref":{"type":"string"}}})
}
fn provider_neutral_task_plan_schema() -> Value {
    json!({"type":"object","required":["task_plan_id","project_root","continuity_id","attachment_id","workbench_session_id","final_spec_id","state_revision","status","tasks","receipt_refs","materialized","idempotency_key","created_at","updated_at"],"properties":{"task_plan_id":{"type":"string"},"project_root":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"workbench_session_id":{"type":"string"},"final_spec_id":{"type":"string"},"state_revision":{"type":"integer","minimum":1},"status":{"enum":["draft","pending_operator","approved"]},"tasks":{"type":"array","items":provider_neutral_task_schema()},"preview_token":{"type":"string"},"previewed_revision":{"type":"integer"},"approved_revision":{"type":"integer"},"approved_by":{"type":"string"},"receipt_refs":{"type":"array","items":{"type":"string"}},"materialized":{"const":false},"idempotency_key":{"type":"string"},"created_at":{"type":"string","format":"date-time"},"updated_at":{"type":"string","format":"date-time"}}})
}
fn provider_task_plan_mutation_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id","idempotency_key","expected_state_version","expected_plan_revision","action"],"properties":{"project_root":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"attachment_id":{"type":"string","minLength":1},"idempotency_key":{"type":"string","minLength":1},"expected_state_version":{"type":"integer","minimum":0},"expected_plan_revision":{"type":"integer","minimum":0},"action":{"enum":["open","upsert_task","remove_task","preview","approve"]},"task_plan_id":{"type":"string"},"workbench_session_id":{"type":"string"},"task":provider_neutral_task_schema(),"task_id":{"type":"string"},"preview_token":{"type":"string"},"approved_by":{"type":"string"}}})
}

fn mission_canvas_surface_schema() -> Value {
    json!({"type":"object","required":["work_surface_id","state_revision","project_root","continuity_id","attachment_id","instance_id","mission_ref","title","surface_kind","status","pane_id","tab_index","pinned","unread","canonical_state_refs","idempotency_key","created_at","updated_at"],"properties":{"work_surface_id":{"type":"string"},"state_revision":{"type":"integer","minimum":1},"project_root":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"instance_id":{"type":"string"},"session_id":{"type":"string"},"workpoint_id":{"type":"string"},"mission_ref":{"type":"string"},"title":{"type":"string"},"surface_kind":{"type":"string"},"status":{"enum":["active","suspended","view_closed"]},"pane_id":{"type":"string"},"tab_index":{"type":"integer","minimum":0},"pinned":{"type":"boolean"},"unread":{"type":"boolean"},"canonical_state_refs":{"type":"array","items":{"type":"string"}},"idempotency_key":{"type":"string"},"created_at":{"type":"string","format":"date-time"},"updated_at":{"type":"string","format":"date-time"}}})
}

fn mission_canvas_state_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["canvas_id","state_revision","project_root","continuity_id","client_instance_id","user_id","device_id","open_work_surface_ids","group_order","aggregate_project_roots","aggregate_continuity_ids","aggregate_surface_kinds","aggregate_surface_states","selected_context_refs","session_projection_revision","idempotency_key","created_at","updated_at"],"properties":{"canvas_id":{"type":"string"},"state_revision":{"type":"integer","minimum":1},"project_root":{"type":"string"},"continuity_id":{"type":"string"},"client_instance_id":{"type":"string"},"user_id":{"type":"string"},"device_id":{"type":"string"},"open_work_surface_ids":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"focused_work_surface_id":{"type":"string"},"secondary_focused_surface_id":{"type":"string"},"split_layout_ref":{"type":"string"},"group_order":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_project_roots":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_continuity_ids":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_surface_kinds":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_surface_states":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"selected_context_refs":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"unread_event_cursor":{"type":"integer","minimum":0},"session_projection_revision":{"type":"integer","minimum":0},"idempotency_key":{"type":"string"},"created_at":{"type":"string","format":"date-time"},"updated_at":{"type":"string","format":"date-time"}}})
}
fn mission_canvas_surface_mutation_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id","idempotency_key","expected_state_version","expected_surface_revision","action"],"properties":{"project_root":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"attachment_id":{"type":"string","minLength":1},"idempotency_key":{"type":"string","minLength":1},"expected_state_version":{"type":"integer","minimum":0},"expected_surface_revision":{"type":"integer","minimum":0},"action":{"enum":["create","arrange","suspend","resume","close_view"]},"work_surface_id":{"type":"string"},"instance_id":{"type":"string"},"session_id":{"type":"string"},"workpoint_id":{"type":"string"},"mission_ref":{"type":"string"},"title":{"type":"string"},"surface_kind":{"type":"string"},"pane_id":{"type":"string"},"tab_index":{"type":"integer","minimum":0},"pinned":{"type":"boolean"},"unread":{"type":"boolean"},"canonical_state_refs":{"type":"array","items":{"type":"string"}}}})
}

fn work_rail_schema() -> Value {
    let strings = || json!({"type":"array","items":{"type":"string"}});
    json!({"type":"object","required":["work_rail_id","state_revision","provider","provider_item_id","title","provider_status","focusa_status","workpoint_id","project_root","working_subpath_id","continuity_id","attachment_id","dependencies","blockers","evidence_refs","artifact_refs","idempotency_key","created_at","updated_at"],"properties":{"work_rail_id":{"type":"string"},"state_revision":{"type":"integer","minimum":1},"provider":{"const":"work_item.bd"},"provider_item_id":{"type":"string"},"title":{"type":"string"},"provider_status":{"type":"string"},"focusa_status":{"enum":["ready","active","verifying","proof_missing","reconciling","verified_complete","provider_closed_focusa_unverified","cancelled"]},"workpoint_id":{"type":"string","format":"uuid"},"project_root":{"type":"string"},"working_subpath_id":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"dependencies":strings(),"blockers":strings(),"evidence_refs":strings(),"artifact_refs":strings(),"receipt_ref":{"type":"string"},"closure_claim_ref":{"type":"string"},"idempotency_key":{"type":"string"},"created_at":{"type":"string","format":"date-time"},"updated_at":{"type":"string","format":"date-time"}}})
}
fn work_rail_mutation_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["project_root","working_subpath_id","continuity_id","attachment_id","idempotency_key","expected_state_version","expected_rail_revision","action","workpoint_id","provider_item_id"],"properties":{"project_root":{"type":"string","minLength":1},"working_subpath_id":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"attachment_id":{"type":"string","minLength":1},"idempotency_key":{"type":"string","minLength":1},"expected_state_version":{"type":"integer","minimum":0},"expected_rail_revision":{"type":"integer","minimum":0},"action":{"enum":["bind","activate","verify_close","cancel"]},"work_rail_id":{"type":"string"},"workpoint_id":{"type":"string","format":"uuid"},"provider_item_id":{"type":"string","minLength":1},"title":{"type":"string"},"evidence_refs":{"type":"array","items":{"type":"string"}},"artifact_refs":{"type":"array","items":{"type":"string"}},"closure_claim_ref":{"type":"string"},"cancellation_reason":{"type":"string"}}})
}

fn task_materialization_schema() -> Value {
    json!({"type":"object","required":["materialization_id","task_plan_id","task_plan_revision","project_root","continuity_id","attachment_id","provider","worktree_prefix","target_ledger_ref","tasks","permission_grant_ref","idempotency_key","evidence_ref","receipt_ref","created_at"],"properties":{"materialization_id":{"type":"string"},"task_plan_id":{"type":"string"},"task_plan_revision":{"type":"integer","minimum":1},"project_root":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"provider":{"const":"work_item.bd"},"worktree_prefix":{"type":"string"},"target_ledger_ref":{"type":"string"},"tasks":{"type":"array","items":{"type":"object","required":["provider_neutral_id","provider_id","provider_dependency_ids","external_ref"],"properties":{"provider_neutral_id":{"type":"string"},"provider_id":{"type":"string"},"provider_dependency_ids":{"type":"array","items":{"type":"string"}},"external_ref":{"type":"string"}}}},"permission_grant_ref":{"type":"string"},"idempotency_key":{"type":"string"},"evidence_ref":{"type":"string"},"receipt_ref":{"type":"string"},"created_at":{"type":"string","format":"date-time"}}})
}

fn provider_contract_schema() -> Value {
    json!({"type":"object","required":["provider_id","provider_class","implementation_owner","execution_owner","operation_prefixes","exact_scope_required","permission_required","idempotency_required","receipt_required","operation_registry_required","canonical_state_owner","direct_canonical_mutation_allowed"],"properties":{"provider_id":{"type":"string"},"provider_class":{"enum":["focusa_operation","work_item","model","browser","agent_transport"]},"implementation_owner":{"type":"string"},"execution_owner":{"type":"string"},"operation_prefixes":{"type":"array","items":{"type":"string"}},"exact_scope_required":{"const":true},"permission_required":{"const":true},"idempotency_required":{"const":true},"receipt_required":{"const":true},"operation_registry_required":{"const":true},"canonical_state_owner":{"const":"focusa_core_reducer"},"direct_canonical_mutation_allowed":{"const":false}}})
}

fn provider_conformance_request_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["provider_id","operation_id","scope","permission_grant_ref","idempotency_key","receipt_required","payload_ref"],"properties":{"provider_id":{"type":"string","minLength":1},"operation_id":{"type":"string","minLength":1},"scope":{"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id"],"properties":{"project_root":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"attachment_id":{"type":"string","minLength":1}}},"permission_grant_ref":{"type":"string","minLength":1},"idempotency_key":{"type":"string","minLength":1},"receipt_required":{"const":true},"payload_ref":{"type":"string","minLength":1}}})
}

fn spec_workbench_session_schema() -> Value {
    let strings = || json!({"type": "array", "items": {"type": "string"}});
    let grounding = json!({"type":"object","required":["context_refs","evidence_refs","codebase_refs","research_refs","docs_only"],"properties":{"context_refs":strings(),"evidence_refs":strings(),"codebase_refs":strings(),"research_refs":strings(),"docs_only":{"type":"boolean"}}});
    let section = json!({"type":"object","required":["section_id","title","section_kind","status","order_index","revision","content","grounding","objection_ids","amendment_ids","created_at","updated_at"],"properties":{"section_id":{"type":"string"},"title":{"type":"string"},"section_kind":{"type":"string"},"status":{"enum":["draft","grounded","challenged","pending_approval","approved","rejected","amended"]},"order_index":{"type":"integer"},"revision":{"type":"integer","minimum":1},"content":{"type":"string"},"grounding":grounding,"objection_ids":strings(),"approved_revision":{"type":"integer"},"operator_gate_id":{"type":"string"},"amendment_ids":strings(),"created_at":{"type":"string","format":"date-time"},"updated_at":{"type":"string","format":"date-time"}}});
    json!({"type":"object","required":["workbench_session_id","project_root","continuity_id","attachment_id","current_ask","state_revision","status","canonical","advisory_agents","operator_required","sections","rounds","objections","gates","amendments","receipt_refs","idempotency_key","created_at","updated_at"],"properties":{"workbench_session_id":{"type":"string"},"project_root":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"current_ask":{"type":"string"},"state_revision":{"type":"integer","minimum":1},"status":{"enum":["active","closed","final_approved"]},"canonical":{"const":true},"advisory_agents":{"const":true},"operator_required":{"const":true},"current_section_id":{"type":"string"},"sections":{"type":"array","items":section},"rounds":{"type":"array","items":{"type":"object"}},"objections":{"type":"array","items":{"type":"object"}},"gates":{"type":"array","items":{"type":"object"}},"amendments":{"type":"array","items":{"type":"object"}},"receipt_refs":strings(),"final_spec_id":{"type":"string"},"idempotency_key":{"type":"string"},"created_at":{"type":"string","format":"date-time"},"updated_at":{"type":"string","format":"date-time"},"closed_at":{"type":"string","format":"date-time"}}})
}

fn spec_workbench_mutation_request_schema() -> Value {
    json!({"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id","idempotency_key","expected_state_version","expected_session_revision","action"],"properties":{"project_root":{"type":"string","minLength":1,"maxLength":4096},"continuity_id":{"type":"string","minLength":1,"maxLength":256},"attachment_id":{"type":"string","minLength":1,"maxLength":256},"idempotency_key":{"type":"string","minLength":1,"maxLength":256},"expected_state_version":{"type":"integer","minimum":0},"expected_session_revision":{"type":"integer","minimum":0},"action":{"enum":["open","upsert_section","add_round","add_objection","resolve_objection","approve_section","reject_section","amend_section","close","reopen","final_approve"]},"workbench_session_id":{"type":"string"},"current_ask":{"type":"string"},"section":{"type":"object","additionalProperties":true},"round":{"type":"object","additionalProperties":true},"objection":{"type":"object","additionalProperties":true},"objection_id":{"type":"string"},"resolution":{"type":"string"},"decision":{"type":"object","additionalProperties":true},"amendment":{"type":"object","additionalProperties":true}}})
}

fn project_interview_session_schema() -> Value {
    let strings = || json!({"type": "array", "items": {"type": "string"}});
    let branch = json!({
        "type": "object", "additionalProperties": false,
        "required": ["decision_branch_id", "tranche", "label", "status", "question_ids", "updated_at"],
        "properties": {
            "decision_branch_id": {"type": "string"}, "parent_branch_id": {"type": "string"}, "tranche": {"type": "string"}, "label": {"type": "string"},
            "status": {"enum": ["active", "deferred", "resolved"]}, "question_ids": strings(), "deferred_reason": {"type": "string"}, "updated_at": {"type": "string", "format": "date-time"}
        }
    });
    let question = json!({
        "type": "object", "additionalProperties": false,
        "required": ["question_id", "session_id", "decision_branch_id", "question", "reason_for_asking", "triggering_gap", "recommendation", "recommendation_basis_refs", "environment_facts_checked", "contradiction_refs", "linked_context_refs", "linked_spec_sections", "decision_required", "priority", "answer_type", "sensitivity", "readiness_effect", "stop_condition", "status", "created_at"],
        "properties": {
            "question_id": {"type": "string"}, "session_id": {"type": "string"}, "decision_branch_id": {"type": "string"}, "parent_question_id": {"type": "string"}, "question": {"type": "string"},
            "reason_for_asking": {"type": "string"}, "triggering_gap": {"type": "string"}, "recommendation": {"type": "string"}, "recommendation_basis_refs": strings(), "environment_facts_checked": strings(),
            "contradiction_refs": strings(), "linked_context_refs": strings(), "linked_spec_sections": strings(), "decision_required": {"type": "boolean"}, "priority": {"type": "string"}, "answer_type": {"type": "string"},
            "sensitivity": {"type": "string"}, "readiness_effect": {"type": "string"}, "stop_condition": {"type": "string"}, "status": {"enum": ["queued", "asked", "answered", "deferred", "skipped", "superseded"]},
            "created_at": {"type": "string", "format": "date-time"}, "answered_at": {"type": "string", "format": "date-time"}
        }
    });
    let answer = json!({
        "type": "object", "additionalProperties": false,
        "required": ["answer_id", "question_id", "answer", "attachment_refs", "operator_id", "status", "notes", "created_at"],
        "properties": {
            "answer_id": {"type": "string"}, "question_id": {"type": "string"}, "answer": {}, "attachment_refs": strings(), "operator_id": {"type": "string"},
            "status": {"enum": ["active", "amended", "superseded", "withdrawn"]}, "confidence": {"type": "number", "minimum": 0, "maximum": 1}, "notes": {"type": "string"},
            "created_at": {"type": "string", "format": "date-time"}, "supersedes": {"type": "string"}
        }
    });
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["interview_session_id", "project_root", "continuity_id", "attachment_id", "strategy_id", "strategy_version", "approved_role_profile_ref", "state_revision", "status", "branches", "questions", "answers", "idempotency_key", "created_at", "updated_at"],
        "properties": {
            "interview_session_id": {"type": "string"}, "project_root": {"type": "string"}, "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"},
            "strategy_id": {"const": "focusa.interview.strategy.grill-with-docs.v1"}, "strategy_version": {"const": 1}, "approved_role_profile_ref": {"type": "string"}, "state_revision": {"type": "integer", "minimum": 1},
            "status": {"enum": ["active", "paused", "closed", "ready_for_spec"]}, "active_branch_id": {"type": "string"}, "current_question_id": {"type": "string"},
            "branches": {"type": "array", "items": branch}, "questions": {"type": "array", "items": question}, "answers": {"type": "array", "items": answer},
            "idempotency_key": {"type": "string"}, "created_at": {"type": "string", "format": "date-time"}, "updated_at": {"type": "string", "format": "date-time"}, "closed_at": {"type": "string", "format": "date-time"}
        }
    })
}

fn project_interview_mutation_request_schema() -> Value {
    let strings = || json!({"type": "array", "items": {"type": "string"}});
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["project_root", "continuity_id", "attachment_id", "idempotency_key", "expected_state_version", "expected_session_revision", "action"],
        "properties": {
            "project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
            "idempotency_key": {"type": "string", "minLength": 1, "maxLength": 256}, "expected_state_version": {"type": "integer", "minimum": 0}, "expected_session_revision": {"type": "integer", "minimum": 0},
            "action": {"enum": ["open", "upsert_branch", "queue_question", "record_answer", "pause", "close", "reopen", "defer_branch", "resolve_branch"]}, "interview_session_id": {"type": "string"}, "approved_role_profile_ref": {"type": "string"},
            "decision_branch_id": {"type": "string"}, "deferred_reason": {"type": "string"},
            "branch": {"type": "object", "additionalProperties": false, "required": ["decision_branch_id", "tranche", "label"], "properties": {"decision_branch_id": {"type": "string"}, "parent_branch_id": {"type": "string"}, "tranche": {"type": "string"}, "label": {"type": "string"}, "deferred_reason": {"type": "string"}}},
            "question": {"type": "object", "additionalProperties": false, "required": ["decision_branch_id", "question", "reason_for_asking", "triggering_gap", "recommendation", "recommendation_basis_refs", "environment_facts_checked", "contradiction_refs", "linked_context_refs", "linked_spec_sections", "decision_required", "priority", "answer_type", "readiness_effect", "stop_condition"], "properties": {"question_id": {"type": "string"}, "decision_branch_id": {"type": "string"}, "parent_question_id": {"type": "string"}, "question": {"type": "string"}, "reason_for_asking": {"type": "string"}, "triggering_gap": {"type": "string"}, "recommendation": {"type": "string"}, "recommendation_basis_refs": strings(), "environment_facts_checked": strings(), "contradiction_refs": strings(), "linked_context_refs": strings(), "linked_spec_sections": strings(), "decision_required": {"type": "boolean"}, "priority": {"type": "string"}, "answer_type": {"type": "string"}, "sensitivity": {"type": "string"}, "readiness_effect": {"type": "string"}, "stop_condition": {"type": "string"}}},
            "answer": {"type": "object", "additionalProperties": false, "required": ["question_id", "answer", "operator_id"], "properties": {"answer_id": {"type": "string"}, "question_id": {"type": "string"}, "answer": {}, "attachment_refs": strings(), "operator_id": {"type": "string"}, "confidence": {"type": "number", "minimum": 0, "maximum": 1}, "notes": {"type": "string"}, "supersedes": {"type": "string"}}}
        }
    })
}

fn interview_gap_schema() -> Value {
    let strings = || json!({"type": "array", "maxItems": 64, "items": {"type": "string"}});
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["gap_id", "tranche", "decision_branch_id", "question", "reason_for_asking", "triggering_gap", "recommendation", "recommendation_basis_refs", "environment_facts_checked", "contradiction_refs", "linked_context_refs", "linked_spec_sections", "domain_term_candidates", "architecture_decision_candidates", "decision_required", "priority", "answer_type", "readiness_effect", "stop_condition", "downstream_dependency_count", "resolved"],
        "properties": {
            "gap_id": {"type": "string"}, "tranche": {"enum": ["discovery", "boundary", "failure", "evidence", "architecture", "spec_readiness"]}, "decision_branch_id": {"type": "string"}, "parent_question_id": {"type": "string"},
            "question": {"type": "string"}, "reason_for_asking": {"type": "string"}, "triggering_gap": {"type": "string"}, "recommendation": {"type": "string"},
            "recommendation_basis_refs": strings(), "environment_facts_checked": strings(), "contradiction_refs": strings(), "linked_context_refs": strings(), "linked_spec_sections": strings(), "domain_term_candidates": strings(), "architecture_decision_candidates": strings(),
            "decision_required": {"type": "boolean"}, "priority": {"enum": ["blocker", "high", "normal", "optional"]}, "answer_type": {"type": "string"}, "readiness_effect": {"type": "string"}, "stop_condition": {"type": "string"}, "downstream_dependency_count": {"type": "integer", "minimum": 0}, "resolved": {"type": "boolean"}
        }
    })
}

fn interview_proposal_schema() -> Value {
    let mut schema = interview_gap_schema();
    let properties = schema["properties"]
        .as_object_mut()
        .expect("gap properties");
    properties.remove("gap_id");
    properties.remove("downstream_dependency_count");
    properties.remove("resolved");
    properties.insert(
        "schema".into(),
        json!({"const": "focusa.interview_next_question_proposal.v1"}),
    );
    properties.insert(
        "strategy_id".into(),
        json!({"const": "focusa.interview.strategy.grill-with-docs.v1"}),
    );
    properties.insert("strategy_version".into(), json!({"const": 1}));
    properties.insert("session_id".into(), json!({"type": "string"}));
    properties.insert("branch_progress".into(), json!({"type": "string"}));
    properties.insert(
        "operator_answer_is_authoritative".into(),
        json!({"const": true}),
    );
    schema["required"] = json!([
        "schema",
        "strategy_id",
        "strategy_version",
        "session_id",
        "tranche",
        "decision_branch_id",
        "question",
        "reason_for_asking",
        "triggering_gap",
        "recommendation",
        "recommendation_basis_refs",
        "environment_facts_checked",
        "contradiction_refs",
        "linked_context_refs",
        "linked_spec_sections",
        "domain_term_candidates",
        "architecture_decision_candidates",
        "decision_required",
        "priority",
        "answer_type",
        "readiness_effect",
        "stop_condition",
        "branch_progress",
        "operator_answer_is_authoritative"
    ]);
    schema
}

fn project_role_profile_schema() -> Value {
    let string_array = || json!({"type": "array", "maxItems": 64, "items": {"type": "string"}});
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["role_profile_id", "project_root", "continuity_id", "attachment_id", "revision", "original_seed", "title", "purpose", "expertise", "primary_responsibilities", "secondary_responsibilities", "expected_deliverables", "quality_standards", "decision_principles", "evidence_expectations", "evidence_behavior", "communication_posture", "stakeholder_posture", "non_responsibilities", "forbidden_assumptions", "escalation_triggers", "handoff_boundaries", "tool_preferences", "reviewer_lenses", "grounding", "assumptions", "unresolved_questions", "redlines", "grants_permissions", "permission_profile_refs", "status", "idempotency_key", "created_at", "updated_at"],
        "properties": {
            "role_profile_id": {"type": "string"}, "project_root": {"type": "string"}, "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"},
            "revision": {"type": "integer", "minimum": 1}, "original_seed": {"type": "string"}, "title": {"type": "string"}, "purpose": {"type": "string"},
            "expertise": string_array(), "primary_responsibilities": string_array(), "secondary_responsibilities": string_array(), "expected_deliverables": string_array(),
            "quality_standards": string_array(), "decision_principles": string_array(), "evidence_expectations": string_array(),
            "evidence_behavior": {"type": "string"}, "communication_posture": {"type": "string"}, "stakeholder_posture": {"type": "string"},
            "non_responsibilities": string_array(), "forbidden_assumptions": string_array(), "escalation_triggers": string_array(), "handoff_boundaries": string_array(), "tool_preferences": string_array(), "reviewer_lenses": string_array(),
            "grounding": {
                "type": "object", "additionalProperties": false,
                "required": ["context_artifact_refs", "context_claim_refs", "interview_answer_refs", "operator_seed_ref"],
                "properties": {"context_artifact_refs": string_array(), "context_claim_refs": string_array(), "interview_answer_refs": string_array(), "operator_seed_ref": {"type": "string"}}
            },
            "assumptions": {
                "type": "array", "maxItems": 64, "items": {
                    "type": "object", "additionalProperties": false, "required": ["assumption_id", "statement", "source_refs", "status"],
                    "properties": {"assumption_id": {"type": "string"}, "statement": {"type": "string"}, "source_refs": string_array(), "status": {"enum": ["unverified", "grounded", "rejected"]}}
                }
            },
            "unresolved_questions": string_array(),
            "redlines": {
                "type": "array", "maxItems": 64, "items": {
                    "type": "object", "additionalProperties": false, "required": ["field", "before", "after", "rationale"],
                    "properties": {"field": {"type": "string"}, "before": {"type": "string"}, "after": {"type": "string"}, "rationale": {"type": "string"}}
                }
            },
            "grants_permissions": {"const": false}, "permission_profile_refs": string_array(),
            "status": {"enum": ["draft", "pending_operator", "approved", "superseded"]},
            "review": {
                "type": "object", "additionalProperties": false, "required": ["decision", "reviewed_by", "reviewed_at", "rationale"],
                "properties": {"decision": {"enum": ["approve", "reject", "defer"]}, "reviewed_by": {"type": "string"}, "reviewed_at": {"type": "string", "format": "date-time"}, "rationale": {"type": "string"}}
            },
            "idempotency_key": {"type": "string"}, "created_at": {"type": "string", "format": "date-time"}, "updated_at": {"type": "string", "format": "date-time"}
        }
    })
}

fn context_claim_schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["claim_id", "project_root", "continuity_id", "attachment_id", "claim", "source_citation_refs", "confidence", "status", "contradiction_refs", "idempotency_key", "revision", "committed_at"],
        "properties": {
            "claim_id": {"type": "string"}, "project_root": {"type": "string"},
            "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"},
            "claim": {"type": "string", "maxLength": 4096},
            "source_citation_refs": {"type": "array", "minItems": 1, "maxItems": 32, "items": {"type": "string"}},
            "confidence": {"type": "number", "minimum": 0, "maximum": 1},
            "status": {"enum": ["candidate", "accepted", "contradicted", "rejected", "superseded"]},
            "contradiction_refs": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
            "reviewed_by": {"type": "string"}, "reviewed_at": {"type": "string", "format": "date-time"},
            "supersedes_claim_id": {"type": "string"}, "idempotency_key": {"type": "string"},
            "revision": {"type": "integer", "minimum": 1}, "committed_at": {"type": "string", "format": "date-time"}
        }
    })
}

fn context_contradiction_schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["contradiction_id", "project_root", "continuity_id", "attachment_id", "left_claim_id", "right_claim_id", "status", "idempotency_key", "revision", "committed_at"],
        "properties": {
            "contradiction_id": {"type": "string"}, "project_root": {"type": "string"},
            "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"},
            "left_claim_id": {"type": "string"}, "right_claim_id": {"type": "string"},
            "status": {"enum": ["open", "resolved"]}, "selected_claim_id": {"type": "string"},
            "resolution": {"type": "string"}, "resolved_by": {"type": "string"},
            "resolved_at": {"type": "string", "format": "date-time"}, "idempotency_key": {"type": "string"},
            "revision": {"type": "integer", "minimum": 1}, "committed_at": {"type": "string", "format": "date-time"}
        }
    })
}

fn context_decision_schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["decision_id", "project_root", "continuity_id", "attachment_id", "decision_kind", "target_ref", "outcome", "rationale", "decided_by", "decided_at", "evidence_refs", "receipt_ref"],
        "properties": {
            "decision_id": {"type": "string"}, "project_root": {"type": "string"},
            "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"},
            "decision_kind": {"enum": ["claim_review", "contradiction_resolution"]},
            "target_ref": {"type": "string"}, "outcome": {"type": "string"},
            "rationale": {"type": "string"}, "decided_by": {"type": "string"},
            "decided_at": {"type": "string", "format": "date-time"},
            "evidence_refs": {"type": "array", "items": {"type": "string"}}, "receipt_ref": {"type": "string"}
        }
    })
}

fn reactive_context_projection_schema() -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["project_root", "continuity_id", "attachment_id", "accepted_claim_refs", "candidate_claim_refs", "blocked_claim_refs", "unresolved_contradiction_refs", "revision"],
        "properties": {
            "project_root": {"type": "string"}, "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"},
            "accepted_claim_refs": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
            "candidate_claim_refs": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
            "blocked_claim_refs": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
            "unresolved_contradiction_refs": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
            "revision": {"type": "integer", "minimum": 0}, "updated_at": {"type": "string", "format": "date-time"}
        }
    })
}

fn context_graph_properties() -> Value {
    json!({
        "canonical": {"const": true}, "state_version": {"type": "integer", "minimum": 0},
        "claims": {"type": "array", "items": context_claim_schema()},
        "contradictions": {"type": "array", "items": context_contradiction_schema()},
        "decisions": {"type": "array", "items": context_decision_schema()},
        "projection": reactive_context_projection_schema()
    })
}

fn context_graph_response_schema(schema_id: &str, mutation: bool) -> Value {
    let mut properties = context_graph_properties()
        .as_object()
        .cloned()
        .expect("Context graph properties are an object");
    properties.insert(
        "schema".to_string(),
        json!({"const": if mutation { "focusa.context_graph_mutation_result.v1" } else { "focusa.context_graph.v1" }}),
    );
    let mut required = vec![
        "schema",
        "canonical",
        "state_version",
        "claims",
        "contradictions",
        "decisions",
        "projection",
    ];
    if mutation {
        properties.insert("replayed".to_string(), json!({"type": "boolean"}));
        properties.insert("evidence_ref".to_string(), json!({"type": "string"}));
        properties.insert("receipt_ref".to_string(), json!({"type": "string"}));
        properties.insert("tool_result".to_string(), json!({"type": "object"}));
        required.extend(["replayed", "evidence_ref", "receipt_ref", "tool_result"]);
    }
    json!({
        "$schema": JSON_SCHEMA_DIALECT_2020_12,
        "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
        "type": "object", "additionalProperties": false,
        "required": required, "properties": properties,
        "x-focusa-schema-id": schema_id,
        "x-focusa-generated-from": if mutation { "ContextGraphResponse" } else { "ContextGraphReadResponse" }
    })
}

fn json_schema_document(schema_id: &str) -> Value {
    if schema_id == "focusa.context_source_commit.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id", "idempotency_key", "expected_state_version", "source_kind", "title", "content"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096},
                "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "idempotency_key": {"type": "string", "minLength": 1, "maxLength": 160},
                "expected_state_version": {"type": "integer", "minimum": 0},
                "source_kind": {"enum": ["markdown", "text", "code", "pdf"]},
                "title": {"type": "string", "minLength": 1, "maxLength": 240},
                "content": {"type": "string", "minLength": 1, "maxLength": 65536}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextSourceCommitRequest"
        });
    }
    if schema_id == "focusa.context_source_commit_result.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object",
            "required": ["schema", "canonical", "replayed", "state_version", "source", "evidence_ref", "receipt_ref", "tool_result"],
            "properties": {
                "schema": {"const": "focusa.context_source_commit_result.v1"},
                "canonical": {"const": true}, "replayed": {"type": "boolean"},
                "state_version": {"type": "integer", "minimum": 1},
                "source": context_source_record_schema(),
                "evidence_ref": {"type": "string"}, "receipt_ref": {"type": "string"},
                "tool_result": {"type": "object"}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextSourceCommitResponse"
        });
    }
    if schema_id == "focusa.context_source_ingest.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id", "idempotency_key", "expected_state_version", "source_kind", "source_locator", "source_revision", "title", "mime_type"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096},
                "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "idempotency_key": {"type": "string", "minLength": 1, "maxLength": 160},
                "expected_state_version": {"type": "integer", "minimum": 0},
                "source_kind": {"enum": ["markdown", "code", "pdf"]},
                "source_locator": {"type": "string", "minLength": 1, "maxLength": 1024},
                "source_revision": {"type": "string", "minLength": 1, "maxLength": 256},
                "title": {"type": "string", "minLength": 1, "maxLength": 240},
                "mime_type": {"type": "string", "minLength": 1, "maxLength": 128},
                "content": {"type": "string", "maxLength": 2097152},
                "content_base64": {"type": "string", "maxLength": 27962028}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextSourceIngestRequest"
        });
    }
    if schema_id == "focusa.context_source_ingest_result.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object",
            "required": ["schema", "canonical", "replayed", "state_version", "source", "evidence_ref", "receipt_ref", "tool_result"],
            "properties": {
                "schema": {"const": "focusa.context_source_ingest_result.v1"},
                "canonical": {"const": true}, "replayed": {"type": "boolean"},
                "state_version": {"type": "integer", "minimum": 1},
                "source": context_source_record_schema(),
                "evidence_ref": {"type": "string"}, "receipt_ref": {"type": "string"},
                "tool_result": {"type": "object"}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextSourceIngestResponse"
        });
    }
    if schema_id == "focusa.context_retrieve.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "query"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096},
                "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "query": {"type": "string", "minLength": 1, "maxLength": 2048},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50, "default": 8},
                "mode": {"enum": ["lexical", "hybrid"], "default": "hybrid"},
                "include_contradictions": {"type": "boolean", "default": false}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextRetrieveRequest"
        });
    }
    if schema_id == "focusa.context_retrieve_response.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["schema", "canonical_sources", "result", "evidence_ref", "receipt_ref", "tool_result"],
            "properties": {
                "schema": {"const": "focusa.context_retrieve_response.v1"},
                "canonical_sources": {"const": true},
                "result": {
                    "type": "object", "additionalProperties": false,
                    "required": ["schema", "query", "mode_requested", "mode_used", "result_count", "indexed_source_count", "indexed_chunk_count", "hits", "contradictions", "capabilities"],
                    "properties": {
                        "schema": {"const": "focusa.context_retrieval_result.v1"},
                        "query": {"type": "string"},
                        "mode_requested": {"enum": ["lexical", "hybrid"]},
                        "mode_used": {"enum": ["lexical", "hybrid"]},
                        "result_count": {"type": "integer", "minimum": 0, "maximum": 50},
                        "indexed_source_count": {"type": "integer", "minimum": 0},
                        "indexed_chunk_count": {"type": "integer", "minimum": 0},
                        "hits": {
                            "type": "array", "maxItems": 50,
                            "items": {
                                "type": "object", "additionalProperties": false,
                                "required": ["chunk_id", "snippet", "score", "retrieval_modes", "citation", "contradiction_refs"],
                                "properties": {
                                    "chunk_id": {"type": "string"}, "snippet": {"type": "string", "maxLength": 1600},
                                    "score": {"type": "number", "minimum": 0, "maximum": 1},
                                    "retrieval_modes": {"type": "array", "items": {"enum": ["lexical", "vector"]}, "uniqueItems": true},
                                    "contradiction_refs": {"type": "array", "items": {"type": "string"}, "uniqueItems": true},
                                    "citation": {
                                        "type": "object", "additionalProperties": false,
                                        "required": ["citation_id", "source_id", "source_revision", "source_kind", "title", "source_locator", "content_hash", "chunk_id", "chunk_ordinal", "line_start", "line_end"],
                                        "properties": {
                                            "citation_id": {"type": "string"}, "source_id": {"type": "string"},
                                            "source_revision": {"type": "string"}, "source_kind": {"type": "string"},
                                            "title": {"type": "string"}, "source_locator": {"type": "string"},
                                            "content_hash": {"type": "string"}, "chunk_id": {"type": "string"},
                                            "chunk_ordinal": {"type": "integer", "minimum": 0},
                                            "line_start": {"type": "integer", "minimum": 1},
                                            "line_end": {"type": "integer", "minimum": 1}
                                        }
                                    }
                                }
                            }
                        },
                        "contradictions": {
                            "type": "array", "maxItems": 1225,
                            "items": {
                                "type": "object", "additionalProperties": false,
                                "required": ["contradiction_id", "status", "summary", "left_citation_id", "right_citation_id", "shared_terms"],
                                "properties": {
                                    "contradiction_id": {"type": "string"}, "status": {"const": "candidate"},
                                    "summary": {"type": "string"}, "left_citation_id": {"type": "string"},
                                    "right_citation_id": {"type": "string"},
                                    "shared_terms": {"type": "array", "maxItems": 12, "items": {"type": "string"}}
                                }
                            }
                        },
                        "capabilities": {
                            "type": "object", "additionalProperties": false,
                            "required": ["lexical", "vector_index", "embedding_provider", "degraded_to_lexical"],
                            "properties": {
                                "lexical": {"type": "string"}, "vector_index": {"type": "string"},
                                "embedding_provider": {"type": "string"}, "embedding_model": {"type": "string"},
                                "degraded_to_lexical": {"type": "boolean"}, "degradation_reason": {"type": "string", "maxLength": 320}
                            }
                        }
                    }
                },
                "evidence_ref": {"type": "string"}, "receipt_ref": {"type": "string"},
                "tool_result": {"type": "object"}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextRetrieveResponse"
        });
    }
    if schema_id == "focusa.mission_canvas_surface_list.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id"],"properties":{"project_root":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"work_surface_id":{"type":"string"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.mission_canvas_surface_list.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","surfaces"],"properties":{"schema":{"const":"focusa.mission_canvas_surface_list.v1"},"state_version":{"type":"integer"},"surfaces":{"type":"array","items":mission_canvas_surface_schema()}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.mission_canvas_surface_mutation.request.v1" {
        let mut schema = mission_canvas_surface_mutation_schema();
        let object = schema
            .as_object_mut()
            .expect("Mission Canvas surface mutation schema");
        object.insert("$schema".into(), json!(JSON_SCHEMA_DIALECT_2020_12));
        object.insert(
            "$id".into(),
            json!(format!("/v1/agent/schemas/{schema_id}")),
        );
        object.insert("title".into(), json!(schema_id));
        object.insert("x-focusa-schema-id".into(), json!(schema_id));
        return schema;
    }
    if schema_id == "focusa.mission_canvas_surface_mutation_result.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","replayed","surface","evidence_ref","receipt_ref","tool_result"],"properties":{"schema":{"const":"focusa.mission_canvas_surface_mutation_result.v1"},"state_version":{"type":"integer"},"replayed":{"type":"boolean"},"surface":mission_canvas_surface_schema(),"evidence_ref":{"type":"string"},"receipt_ref":{"type":"string"},"tool_result":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.mission_canvas_state_get.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","continuity_id","client_instance_id","user_id","device_id"],"properties":{"project_root":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"client_instance_id":{"type":"string","minLength":1},"user_id":{"type":"string","minLength":1},"device_id":{"type":"string","minLength":1}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.mission_canvas_state.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["schema","state_version","canvas","surfaces","recovery_actions"],"properties":{"schema":{"const":"focusa.mission_canvas_state.v1"},"state_version":{"type":"integer","minimum":0},"canvas":mission_canvas_state_schema(),"surfaces":{"type":"array","items":mission_canvas_surface_schema()},"recovery_actions":{"type":"array","items":{"type":"string"}}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.mission_canvas_state_mutation.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","continuity_id","client_instance_id","user_id","device_id","idempotency_key","expected_state_version","expected_canvas_revision","session_projection_revision"],"properties":{"project_root":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"client_instance_id":{"type":"string","minLength":1},"user_id":{"type":"string","minLength":1},"device_id":{"type":"string","minLength":1},"idempotency_key":{"type":"string","minLength":1},"expected_state_version":{"type":"integer","minimum":0},"expected_canvas_revision":{"type":"integer","minimum":0},"open_work_surface_ids":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"focused_work_surface_id":{"type":"string"},"secondary_focused_surface_id":{"type":"string"},"split_layout_ref":{"type":"string"},"group_order":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_project_roots":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_continuity_ids":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_surface_kinds":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"aggregate_surface_states":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"selected_context_refs":{"type":"array","maxItems":64,"uniqueItems":true,"items":{"type":"string"}},"unread_event_cursor":{"type":"integer","minimum":0},"session_projection_revision":{"type":"integer","minimum":0}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.mission_canvas_state_mutation_result.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["schema","state_version","replayed","canvas","evidence_ref","receipt_ref","tool_result"],"properties":{"schema":{"const":"focusa.mission_canvas_state_mutation_result.v1"},"state_version":{"type":"integer","minimum":0},"replayed":{"type":"boolean"},"canvas":mission_canvas_state_schema(),"evidence_ref":{"type":"string"},"receipt_ref":{"type":"string"},"tool_result":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.work_rail_list.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","working_subpath_id","continuity_id","attachment_id"],"properties":{"project_root":{"type":"string"},"working_subpath_id":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"work_rail_id":{"type":"string"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.work_rail_list.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","rows"],"properties":{"schema":{"const":"focusa.work_rail_list.v1"},"state_version":{"type":"integer"},"rows":{"type":"array","items":work_rail_schema()}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.work_rail_mutation.request.v1" {
        let mut schema = work_rail_mutation_schema();
        let object = schema.as_object_mut().expect("Work Rail mutation schema");
        object.insert("$schema".into(), json!(JSON_SCHEMA_DIALECT_2020_12));
        object.insert(
            "$id".into(),
            json!(format!("/v1/agent/schemas/{schema_id}")),
        );
        object.insert("title".into(), json!(schema_id));
        object.insert("x-focusa-schema-id".into(), json!(schema_id));
        return schema;
    }
    if schema_id == "focusa.work_rail_mutation_result.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","replayed","row","evidence_ref","receipt_ref","tool_result"],"properties":{"schema":{"const":"focusa.work_rail_mutation_result.v1"},"state_version":{"type":"integer"},"replayed":{"type":"boolean"},"row":work_rail_schema(),"evidence_ref":{"type":"string"},"receipt_ref":{"type":"string"},"tool_result":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.task_plan_beads_materialization.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id","task_plan_id","expected_state_version","expected_plan_revision","worktree_prefix","permission_grant_ref","idempotency_key"],"properties":{"project_root":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"attachment_id":{"type":"string","minLength":1},"task_plan_id":{"type":"string","minLength":1},"expected_state_version":{"type":"integer","minimum":0},"expected_plan_revision":{"type":"integer","minimum":1},"worktree_prefix":{"type":"string","pattern":"^[a-z0-9][a-z0-9-]*$"},"permission_grant_ref":{"type":"string","minLength":1},"idempotency_key":{"type":"string","minLength":1}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.task_plan_beads_materialization_result.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","replayed","materialization","evidence_ref","receipt_ref","tool_result"],"properties":{"schema":{"const":"focusa.task_plan_beads_materialization_result.v1"},"state_version":{"type":"integer"},"replayed":{"type":"boolean"},"materialization":task_materialization_schema(),"evidence_ref":{"type":"string"},"receipt_ref":{"type":"string"},"tool_result":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.provider_neutral_task_plan_list.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id"],"properties":{"project_root":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"task_plan_id":{"type":"string"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.provider_neutral_task_plan_list.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","task_plans"],"properties":{"schema":{"const":"focusa.provider_neutral_task_plan_list.v1"},"state_version":{"type":"integer"},"task_plans":{"type":"array","items":provider_neutral_task_plan_schema()}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.provider_neutral_task_plan_mutation.request.v1" {
        let mut schema = provider_task_plan_mutation_schema();
        let object = schema.as_object_mut().expect("task plan mutation schema");
        object.insert("$schema".into(), json!(JSON_SCHEMA_DIALECT_2020_12));
        object.insert(
            "$id".into(),
            json!(format!("/v1/agent/schemas/{schema_id}")),
        );
        object.insert("title".into(), json!(schema_id));
        object.insert("x-focusa-schema-id".into(), json!(schema_id));
        return schema;
    }
    if schema_id == "focusa.provider_neutral_task_plan_mutation_result.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","replayed","materialization_allowed","task_plan","evidence_ref","receipt_ref","tool_result"],"properties":{"schema":{"const":"focusa.provider_neutral_task_plan_mutation_result.v1"},"state_version":{"type":"integer"},"replayed":{"type":"boolean"},"materialization_allowed":{"type":"boolean"},"task_plan":provider_neutral_task_plan_schema(),"evidence_ref":{"type":"string"},"receipt_ref":{"type":"string"},"tool_result":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.provider_contract_list.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id"],"properties":{"project_root":{"type":"string","minLength":1},"continuity_id":{"type":"string","minLength":1},"attachment_id":{"type":"string","minLength":1}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.provider_contract_list.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","scope","contracts","parity"],"properties":{"schema":{"const":"focusa.provider_contract_list.v1"},"scope":{"type":"object"},"contracts":{"type":"array","items":provider_contract_schema(),"minItems":1},"parity":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.provider_conformance.request.v1" {
        let mut schema = provider_conformance_request_schema();
        let object = schema.as_object_mut().expect("provider conformance schema");
        object.insert("$schema".into(), json!(JSON_SCHEMA_DIALECT_2020_12));
        object.insert(
            "$id".into(),
            json!(format!("/v1/agent/schemas/{schema_id}")),
        );
        object.insert("title".into(), json!(schema_id));
        object.insert("x-focusa-schema-id".into(), json!(schema_id));
        return schema;
    }
    if schema_id == "focusa.provider_conformance_response.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","result","execution_performed","canonical_state_mutated","evidence_ref","tool_result"],"properties":{"schema":{"const":"focusa.provider_conformance_response.v1"},"result":{"type":"object","required":["schema","conformant","provider_id","operation_id","checks","violations","receipt_ref"],"properties":{"schema":{"const":"focusa.provider_conformance_result.v1"},"conformant":{"type":"boolean"},"provider_id":{"type":"string"},"operation_id":{"type":"string"},"checks":{"type":"array","items":{"type":"string"}},"violations":{"type":"array","items":{"type":"string"}},"receipt_ref":{"type":"string"}}},"execution_performed":{"const":false},"canonical_state_mutated":{"const":false},"evidence_ref":{"type":"string"},"tool_result":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.spec_workbench_session_list.request.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","additionalProperties":false,"required":["project_root","continuity_id","attachment_id"],"properties":{"project_root":{"type":"string"},"continuity_id":{"type":"string"},"attachment_id":{"type":"string"},"workbench_session_id":{"type":"string"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.spec_workbench_session_list.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","sessions"],"properties":{"schema":{"const":"focusa.spec_workbench_session_list.v1"},"state_version":{"type":"integer"},"sessions":{"type":"array","items":spec_workbench_session_schema()}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.spec_workbench_mutation.request.v1" {
        let mut schema = spec_workbench_mutation_request_schema();
        let object = schema.as_object_mut().expect("Spec mutation schema");
        object.insert("$schema".into(), json!(JSON_SCHEMA_DIALECT_2020_12));
        object.insert(
            "$id".into(),
            json!(format!("/v1/agent/schemas/{schema_id}")),
        );
        object.insert("title".into(), json!(schema_id));
        object.insert("x-focusa-schema-id".into(), json!(schema_id));
        return schema;
    }
    if schema_id == "focusa.spec_workbench_mutation_result.v1" {
        return json!({"$schema":JSON_SCHEMA_DIALECT_2020_12,"$id":format!("/v1/agent/schemas/{schema_id}"),"title":schema_id,"type":"object","required":["schema","state_version","replayed","exact_resume","session","evidence_ref","receipt_ref","tool_result"],"properties":{"schema":{"const":"focusa.spec_workbench_mutation_result.v1"},"state_version":{"type":"integer"},"replayed":{"type":"boolean"},"exact_resume":{"const":true},"session":spec_workbench_session_schema(),"evidence_ref":{"type":"string"},"receipt_ref":{"type":"string"},"tool_result":{"type":"object"}},"x-focusa-schema-id":schema_id});
    }
    if schema_id == "focusa.project_interview_session_list.request.v1" {
        return json!({"$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id, "type": "object", "additionalProperties": false, "required": ["project_root", "continuity_id", "attachment_id"], "properties": {"project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256}, "interview_session_id": {"type": "string"}}, "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "InterviewSessionQuery"});
    }
    if schema_id == "focusa.project_interview_session_list.v1" {
        return json!({"$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id, "type": "object", "additionalProperties": false, "required": ["schema", "state_version", "sessions"], "properties": {"schema": {"const": "focusa.project_interview_session_list.v1"}, "state_version": {"type": "integer", "minimum": 0}, "sessions": {"type": "array", "items": project_interview_session_schema()}}, "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "InterviewSessionListResponse"});
    }
    if schema_id == "focusa.interview_closure_package.v1" {
        return json!({"$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id, "type": "object", "additionalProperties": false, "required": ["schema", "closure_ref", "project_root", "continuity_id", "attachment_id", "interview_session_id", "source_state_revision", "glossary_candidates", "adr_candidates", "compendium", "receipt_ref"], "properties": {"schema": {"const": "focusa.interview_closure_package.v1"}, "closure_ref": {"type": "string"}, "project_root": {"type": "string"}, "continuity_id": {"type": "string"}, "attachment_id": {"type": "string"}, "interview_session_id": {"type": "string"}, "source_state_revision": {"type": "integer", "minimum": 1}, "approved_role_profile_ref": {"type": ["string", "null"]}, "glossary_candidates": {"type": "array", "maxItems": 64, "items": {"type": "object"}}, "adr_candidates": {"type": "array", "maxItems": 64, "items": {"type": "object"}}, "compendium": {"type": "array", "maxItems": 128, "items": {"type": "object"}}, "receipt_ref": {"type": "string"}}, "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "ProjectInterviewClosurePackage"});
    }
    if schema_id == "focusa.project_interview_session_mutation.request.v1" {
        let mut schema = project_interview_mutation_request_schema();
        let object = schema
            .as_object_mut()
            .expect("Interview mutation schema object");
        object.insert("$schema".into(), json!(JSON_SCHEMA_DIALECT_2020_12));
        object.insert(
            "$id".into(),
            json!(format!("/v1/agent/schemas/{schema_id}")),
        );
        object.insert("title".into(), json!(schema_id));
        object.insert("x-focusa-schema-id".into(), json!(schema_id));
        object.insert(
            "x-focusa-generated-from".into(),
            json!("InterviewSessionMutationRequest"),
        );
        return schema;
    }
    if schema_id == "focusa.project_interview_session_mutation_result.v1" {
        return json!({"$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id, "type": "object", "additionalProperties": false, "required": ["schema", "state_version", "replayed", "exact_resume", "session", "evidence_ref", "receipt_ref", "tool_result"], "properties": {"schema": {"const": "focusa.project_interview_session_mutation_result.v1"}, "state_version": {"type": "integer", "minimum": 0}, "replayed": {"type": "boolean"}, "exact_resume": {"const": true}, "session": project_interview_session_schema(), "evidence_ref": {"type": "string"}, "receipt_ref": {"type": "string"}, "tool_result": {"type": "object"}}, "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "InterviewSessionMutationResponse"});
    }
    if schema_id == "focusa.grill_interview_context.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id", "session_id", "approved_role_profile_ref", "completed_tranches", "gaps"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256}, "session_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "approved_role_profile_ref": {"type": "string", "minLength": 1}, "active_branch_id": {"type": "string"},
                "completed_tranches": {"type": "array", "uniqueItems": true, "items": {"enum": ["discovery", "boundary", "failure", "evidence", "architecture", "spec_readiness"]}},
                "gaps": {"type": "array", "maxItems": 256, "items": interview_gap_schema()}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "GrillInterviewContext"
        });
    }
    if schema_id == "focusa.grill_interview_strategy_response.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["schema", "advisory_strategy", "canonical_inputs_verified", "interview_state_authority", "result", "tool_result"],
            "properties": {
                "schema": {"const": "focusa.grill_interview_strategy_response.v1"}, "advisory_strategy": {"const": true}, "canonical_inputs_verified": {"const": true}, "interview_state_authority": {"const": "Focusa Interview Engine"},
                "result": {
                    "type": "object", "additionalProperties": false,
                    "required": ["schema", "strategy_id", "strategy_version", "retrieval_performed_before_question", "one_question_only", "all_core_tranches_accounted_for", "ready_for_spec"],
                    "properties": {"schema": {"const": "focusa.grill_interview_strategy_result.v1"}, "strategy_id": {"const": "focusa.interview.strategy.grill-with-docs.v1"}, "strategy_version": {"const": 1}, "retrieval_performed_before_question": {"const": true}, "one_question_only": {"const": true}, "all_core_tranches_accounted_for": {"const": true}, "ready_for_spec": {"type": "boolean"}, "proposal": interview_proposal_schema()}
                },
                "tool_result": {"type": "object"}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "GrillInterviewStrategyResponse"
        });
    }
    if schema_id == "focusa.project_agent_role_profile_list.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false, "required": ["project_root", "continuity_id", "attachment_id"],
            "properties": {"project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256}},
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "RoleProfileQuery"
        });
    }
    if schema_id == "focusa.project_agent_role_profile_list.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false, "required": ["schema", "responsibility_is_not_permission", "state_version", "profiles"],
            "properties": {
                "schema": {"const": "focusa.project_agent_role_profile_list.v1"}, "responsibility_is_not_permission": {"const": true}, "state_version": {"type": "integer", "minimum": 0},
                "profiles": {"type": "array", "items": project_role_profile_schema()},
                "latest": project_role_profile_schema(), "approved": project_role_profile_schema()
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "RoleProfileListResponse"
        });
    }
    if schema_id == "focusa.project_agent_role_profile_draft.request.v1" {
        let strings = || json!({"type": "array", "maxItems": 64, "items": {"type": "string", "minLength": 1}});
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id", "idempotency_key", "expected_state_version", "original_seed", "title", "purpose", "expertise", "primary_responsibilities", "secondary_responsibilities", "expected_deliverables", "quality_standards", "decision_principles", "evidence_expectations", "evidence_behavior", "communication_posture", "stakeholder_posture", "non_responsibilities", "forbidden_assumptions", "escalation_triggers", "handoff_boundaries", "tool_preferences", "reviewer_lenses", "context_artifact_refs", "context_claim_refs", "interview_answer_refs", "assumptions", "unresolved_questions", "redlines", "permission_profile_refs", "permission_assertions"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "idempotency_key": {"type": "string", "minLength": 1, "maxLength": 256}, "expected_state_version": {"type": "integer", "minimum": 0},
                "original_seed": {"type": "string", "minLength": 1, "maxLength": 2000}, "title": {"type": "string", "minLength": 1, "maxLength": 200}, "purpose": {"type": "string", "minLength": 1, "maxLength": 2000},
                "expertise": strings(), "primary_responsibilities": strings(), "secondary_responsibilities": strings(), "expected_deliverables": strings(), "quality_standards": strings(), "decision_principles": strings(), "evidence_expectations": strings(),
                "evidence_behavior": {"type": "string"}, "communication_posture": {"type": "string"}, "stakeholder_posture": {"type": "string"},
                "non_responsibilities": strings(), "forbidden_assumptions": strings(), "escalation_triggers": strings(), "handoff_boundaries": strings(), "tool_preferences": strings(), "reviewer_lenses": strings(),
                "context_artifact_refs": strings(), "context_claim_refs": strings(), "interview_answer_refs": strings(),
                "assumptions": {"type": "array", "maxItems": 64, "items": {"type": "object", "additionalProperties": false, "required": ["statement", "source_refs", "status"], "properties": {"statement": {"type": "string"}, "source_refs": strings(), "status": {"enum": ["unverified", "grounded", "rejected"]}}}},
                "unresolved_questions": strings(),
                "redlines": {"type": "array", "maxItems": 64, "items": {"type": "object", "additionalProperties": false, "required": ["field", "before", "after", "rationale"], "properties": {"field": {"type": "string"}, "before": {"type": "string"}, "after": {"type": "string"}, "rationale": {"type": "string"}}}},
                "permission_profile_refs": strings(), "permission_assertions": {"type": "array", "maxItems": 0, "items": {"type": "string"}}
            },
            "x-focusa-at-least-one-canonical-grounding-ref": ["context_artifact_refs", "context_claim_refs"],
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "RoleProfileDraftRequest"
        });
    }
    if schema_id == "focusa.project_agent_role_profile_review.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id", "role_profile_id", "profile_revision", "idempotency_key", "expected_state_version", "decision", "reviewed_by", "rationale"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "role_profile_id": {"type": "string", "minLength": 1, "maxLength": 256}, "profile_revision": {"type": "integer", "minimum": 1}, "idempotency_key": {"type": "string", "minLength": 1, "maxLength": 256}, "expected_state_version": {"type": "integer", "minimum": 0},
                "decision": {"enum": ["approve", "reject", "defer"]}, "reviewed_by": {"type": "string", "minLength": 1, "maxLength": 256}, "rationale": {"type": "string", "minLength": 1, "maxLength": 2000}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "RoleProfileReviewRequest"
        });
    }
    if schema_id == "focusa.project_agent_role_profile_mutation_result.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["schema", "canonical", "responsibility_is_not_permission", "replayed", "state_version", "profile", "evidence_ref", "receipt_ref", "tool_result"],
            "properties": {
                "schema": {"const": "focusa.project_agent_role_profile_mutation_result.v1"}, "canonical": {"const": true}, "responsibility_is_not_permission": {"const": true}, "replayed": {"type": "boolean"}, "state_version": {"type": "integer", "minimum": 0},
                "profile": project_role_profile_schema(), "evidence_ref": {"type": "string"}, "receipt_ref": {"type": "string"}, "tool_result": {"type": "object"}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "RoleProfileMutationResponse"
        });
    }
    if schema_id == "focusa.events_stream.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "properties": {
                "cursor": {"type": "string"}, "project_root": {"type": "string"}, "continuity_id": {"type": "string"},
                "attachment_id": {"type": "string"}, "session_id": {"type": "string"}, "work_surface_id": {"type": "string"}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "StreamQuery"
        });
    }
    if schema_id == "focusa.workspace_artifact_list.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false, "required": ["project_root", "continuity_id", "attachment_id"],
            "properties": {"project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256}},
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "WorkspaceArtifactQuery"
        });
    }
    if schema_id == "focusa.workspace_artifact_list.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false, "required": ["schema", "canonical_links", "external_artifact_authority", "state_version", "artifacts"],
            "properties": {"schema": {"const": "focusa.workspace_artifact_list.v1"}, "canonical_links": {"const": true}, "external_artifact_authority": {"const": true}, "state_version": {"type": "integer", "minimum": 0}, "artifacts": {"type": "array", "items": workspace_artifact_schema()}},
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "WorkspaceArtifactListResponse"
        });
    }
    if schema_id == "focusa.workspace_artifact_intake.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id", "idempotency_key", "expected_state_version", "artifact_kind", "mime_type", "title", "summary", "handle_ref", "sha256", "size_bytes", "source_system", "source_ref", "instance_id", "evidence_refs", "evidence_status", "redaction_status", "freshness_status", "provenance_status", "retention_policy", "cleanup_action", "preferred_renderer", "fallback_renderer"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096}, "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256}, "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "idempotency_key": {"type": "string", "minLength": 1, "maxLength": 256}, "expected_state_version": {"type": "integer", "minimum": 0},
                "artifact_kind": {"enum": ["image", "markdown", "dataset", "diff", "browser_snapshot", "diagnostics", "chart", "document", "media", "fpv_session"]},
                "mime_type": {"type": "string"}, "title": {"type": "string"}, "summary": {"type": "string", "maxLength": 2000},
                "handle_ref": {"type": "string"}, "artifact_url": {"type": "string"}, "artifact_path": {"type": "string"}, "inline_preview": {"type": "string", "maxLength": 2000},
                "sha256": {"type": "string", "pattern": "^[A-Fa-f0-9]{64}$"}, "size_bytes": {"type": "integer", "minimum": 0},
                "source_system": {"enum": ["uiai", "focusa", "local_file", "connector", "provider", "operator"]}, "source_ref": {"type": "string"}, "source_url": {"type": "string"}, "captured_at": {"type": "string", "format": "date-time"},
                "project_identity_ref": {"type": "string"}, "workpoint_id": {"type": "string"}, "work_item_ref": {"type": "string"},
                "instance_id": {"type": "string"}, "focusa_session_id": {"type": "string"}, "work_surface_id": {"type": "string"}, "harness_session_ref": {"type": "string"},
                "silent_session_id": {"type": "string"}, "silent_run_id": {"type": "string"}, "uiai_session_id": {"type": "string"}, "browser_context_id": {"type": "string"}, "browser_target_id": {"type": "string"},
                "diagnostics_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}}, "evidence_refs": {"type": "array", "minItems": 1, "maxItems": 32, "items": {"type": "string"}},
                "domain_pack_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}}, "candidate_object_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                "candidate_link_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}}, "candidate_claim_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                "verification_policy_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}}, "semantic_delta_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                "citation_refs": {"type": "array", "maxItems": 64, "items": {"type": "string"}},
                "evidence_status": {"enum": ["proposal_only", "capture_pending", "captured", "linked", "verified", "stale", "blocked", "scope_mismatch"]},
                "redaction_status": {"type": "string"}, "freshness_status": {"type": "string"}, "provenance_status": {"type": "string"}, "retention_policy": {"type": "string"}, "expires_at": {"type": "string", "format": "date-time"}, "cleanup_action": {"type": "string"},
                "preferred_renderer": {"type": "string"}, "fallback_renderer": {"type": "string"},
                "render_width": {"type": "integer", "minimum": 1, "maximum": 16384}, "render_height": {"type": "integer", "minimum": 1, "maximum": 16384}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "WorkspaceArtifactIntakeRequest"
        });
    }
    if schema_id == "focusa.workspace_artifact_intake_result.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12, "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false, "required": ["schema", "canonical_link", "external_artifact_authority", "replayed", "state_version", "artifact", "evidence_ref", "receipt_ref", "tool_result"],
            "properties": {"schema": {"const": "focusa.workspace_artifact_intake_result.v1"}, "canonical_link": {"const": true}, "external_artifact_authority": {"const": true}, "replayed": {"type": "boolean"}, "state_version": {"type": "integer", "minimum": 1}, "artifact": workspace_artifact_schema(), "evidence_ref": {"type": "string"}, "receipt_ref": {"type": "string"}, "tool_result": {"type": "object"}},
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "WorkspaceArtifactIntakeResponse"
        });
    }
    if schema_id == "focusa.context_graph_read.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096},
                "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "ContextGraphScope"
        });
    }
    if schema_id == "focusa.context_graph.v1" {
        return context_graph_response_schema(schema_id, false);
    }
    if schema_id == "focusa.context_graph_mutation.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id", "idempotency_key", "expected_state_version", "action"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1, "maxLength": 4096},
                "continuity_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "attachment_id": {"type": "string", "minLength": 1, "maxLength": 256},
                "idempotency_key": {"type": "string", "minLength": 1, "maxLength": 256},
                "expected_state_version": {"type": "integer", "minimum": 0},
                "action": {"enum": ["propose_claim", "review_claim", "open_contradiction", "resolve_contradiction"]},
                "claim_id": {"type": "string"}, "claim": {"type": "string", "maxLength": 4096},
                "source_citation_refs": {"type": "array", "maxItems": 32, "items": {"type": "string"}},
                "confidence": {"type": "number", "minimum": 0, "maximum": 1},
                "supersedes_claim_id": {"type": "string"},
                "review_outcome": {"enum": ["accept", "reject"]},
                "contradiction_id": {"type": "string"}, "left_claim_id": {"type": "string"},
                "right_claim_id": {"type": "string"},
                "resolution": {"enum": ["accept_left", "accept_right", "reject_both"]},
                "selected_claim_id": {"type": "string"}, "actor": {"type": "string", "maxLength": 256},
                "rationale": {"type": "string", "maxLength": 2048}
            },
            "x-focusa-schema-id": schema_id, "x-focusa-generated-from": "ContextGraphMutationRequest"
        });
    }
    if schema_id == "focusa.context_graph_mutation_result.v1" {
        return context_graph_response_schema(schema_id, true);
    }
    if schema_id == "focusa.context_adapter_health.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false, "properties": {},
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "DoclingHealthRequest"
        });
    }
    if schema_id == "focusa.context_adapter_health.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object",
            "required": ["schema", "adapter_id", "configured", "status", "message", "checked_at"],
            "properties": {
                "schema": {"const": "focusa.context_adapter_health.v1"},
                "adapter_id": {"const": "docling-serve.v1"},
                "configured": {"type": "boolean"},
                "status": {"enum": ["healthy", "degraded", "offline"]},
                "endpoint": {"type": "string"},
                "message": {"type": "string"},
                "recovery_action": {"type": "string"},
                "checked_at": {"type": "string", "format": "date-time"}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "DoclingHealthResponse"
        });
    }
    if schema_id == "focusa.context_source_list.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object", "additionalProperties": false,
            "required": ["project_root", "continuity_id", "attachment_id"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1},
                "continuity_id": {"type": "string", "minLength": 1},
                "attachment_id": {"type": "string", "minLength": 1}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextSourceListQuery"
        });
    }
    if schema_id == "focusa.context_source_list.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"), "title": schema_id,
            "type": "object",
            "required": ["schema", "canonical", "state_version", "sources"],
            "properties": {
                "schema": {"const": "focusa.context_source_list.v1"},
                "canonical": {"const": true},
                "state_version": {"type": "integer", "minimum": 0},
                "sources": {"type": "array", "items": context_source_record_schema()}
            },
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ContextSourceListResponse"
        });
    }

    if schema_id == "focusa.agent_execution_start.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Pi RPC Agent Execution Start Request v1",
            "type": "object",
            "required": ["idempotency_key"],
            "properties": {
                "cwd": {"type": "string"},
                "models": {"type": "string"},
                "resume_session": {"type": "string"},
                "session_dir": {"type": "string"},
                "session_name": {"type": "string"},
                "workpoint_id": {"type": "string"},
                "idempotency_key": {"type": "string", "minLength": 1}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "PiDriverStartRequest"
        });
    }
    if schema_id == "focusa.agent_execution_prompt.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Pi RPC Agent Execution Prompt Request v1",
            "type": "object",
            "required": ["message"],
            "properties": {
                "message": {"type": "string", "minLength": 1},
                "streaming_behavior": {"type": "string", "enum": ["steer", "followUp"]}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "PiDriverPromptRequest"
        });
    }
    if matches!(
        schema_id,
        "focusa.agent_execution_abort.request.v1" | "focusa.agent_execution_stop.request.v1"
    ) {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": schema_id,
            "type": "object",
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "Spec133 Pi RPC process control"
        });
    }
    if schema_id == "focusa.agent_execution_adapter_result.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Agent Execution Adapter Result v1",
            "type": "object",
            "required": ["schema", "status", "adapter", "session_id", "resumable", "authority", "tool_result"],
            "properties": {
                "schema": {"type": "string"},
                "status": {"type": "string", "enum": ["accepted", "stopped"]},
                "adapter": {"type": "string", "enum": ["pi-rpc"]},
                "session_id": {"type": "string"},
                "resumable": {"type": "boolean"},
                "cancelled": {"type": "boolean"},
                "idempotent_replay": {"type": "boolean"},
                "resumed_from": {},
                "workpoint_id": {},
                "cancellation": {"type": "object"},
                "authority": {"type": "string"},
                "tool_result": {"type": "object"}
            },
            "additionalProperties": true,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "Spec133 PiRpcSession"
        });
    }
    if schema_id == "focusa.protocol_handshake.request.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Protocol Handshake Request v1",
            "type": "object",
            "required": ["client_id", "client_versions"],
            "properties": {
                "client_id": {"type": "string", "minLength": 1},
                "client_versions": {"type": "object", "additionalProperties": {"type": "string"}},
                "requested_capabilities": {"type": "array", "items": {"type": "string"}}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "ProtocolHandshakeRequest"
        });
    }
    if schema_id == "focusa.protocol_handshake.response.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Protocol Handshake Response v1",
            "type": "object",
            "required": ["schema", "status", "compatible", "client_id", "project_root", "continuity_id", "server_versions", "capability_snapshot", "compatibility_lock", "safe_state_retained"],
            "properties": {
                "schema": {"type": "string", "enum": ["focusa.protocol_handshake.response.v1"]},
                "status": {"type": "string", "enum": ["accepted"]},
                "compatible": {"type": "boolean", "enum": [true]},
                "client_id": {"type": "string"},
                "project_root": {"type": "string"},
                "continuity_id": {"type": "string"},
                "requested_capabilities": {"type": "array", "items": {"type": "string"}},
                "server_versions": {"type": "object", "additionalProperties": {"type": "string"}},
                "capability_snapshot": {"type": "object"},
                "compatibility_lock": {"type": "object"},
                "safe_state_retained": {"type": "boolean"}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "protocol_handshake_handler"
        });
    }
    if schema_id == "focusa.compatibility_lock.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Compatibility Lock v1",
            "type": "object",
            "required": ["schema", "focusa_runtime", "focusa_api", "operation_registry", "tool_result", "event_stream", "a2ui_protocol", "minimum_reader_versions", "minimum_writer_versions"],
            "properties": {
                "schema": {"type": "string", "enum": ["focusa.compatibility_lock.v1"]},
                "focusa_runtime": {"type": "string"},
                "focusa_api": {"type": "string"},
                "operation_registry": {"type": "string"},
                "tool_result": {"type": "string"},
                "event_stream": {"type": "string"},
                "a2ui_protocol": {"type": "string"},
                "a2ui_catalog": {"type": "string"},
                "ag_ui_adapter": {"type": "string"},
                "pi_runtime": {"type": "string"},
                "uiai_engine": {"type": "string"},
                "uiai_focusa_client": {"type": "string"},
                "docling": {"type": "string"},
                "embedding_profile": {"type": "string"},
                "domain_pack_versions": {"type": "array", "items": {"type": "string"}},
                "minimum_reader_versions": {"type": "object", "additionalProperties": {"type": "string"}},
                "minimum_writer_versions": {"type": "object", "additionalProperties": {"type": "string"}}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "compatibility_lock_document"
        });
    }
    if schema_id == "focusa.workspace_event.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Workspace Invalidation Event v1",
            "description": "Bounded ref-only Mission Canvas projection invalidation; never semantic authority",
            "type": "object",
            "required": ["schema", "event", "project_root", "continuity_id", "instance_id", "attachment_id", "artifact_id", "artifact_kind", "source_state_revision", "payload_ref", "invalidate", "semantic_authority"],
            "properties": {
                "schema": {"const": "focusa.workspace_event.v1"},
                "event": {"enum": ["uiai_session_opened", "uiai_session_status_changed", "uiai_fpv_share_created", "browser_context_created", "browser_context_status_changed", "browser_context_closed", "browser_target_opened", "browser_target_navigated", "browser_target_moved", "browser_target_closed", "workspace_artifact_capture_pending", "workspace_artifact_linked", "workspace_artifact_verified", "workspace_artifact_stale", "workspace_artifact_redacted", "workspace_artifact_removed", "workspace_artifact_render_failed"]},
                "project_root": {"type": "string"}, "continuity_id": {"type": "string"}, "workpoint_id": {"type": "string"},
                "instance_id": {"type": "string"}, "session_id": {"type": "string"}, "attachment_id": {"type": "string"},
                "work_surface_id": {"type": "string"}, "uiai_session_id": {"type": "string"}, "browser_context_id": {"type": "string"}, "browser_target_id": {"type": "string"},
                "artifact_id": {"type": "string"}, "artifact_kind": {"type": "string"},
                "source_state_revision": {"type": "integer", "minimum": 1}, "payload_ref": {"type": "string"},
                "invalidate": {"type": "array", "minItems": 1, "maxItems": 16, "uniqueItems": true, "items": {"type": "string"}},
                "semantic_authority": {"const": false}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "WorkspaceEventRecord"
        });
    }
    if schema_id == "focusa.stream_event.v1" {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa Stream Event v1",
            "description": "Stable SQLite replay and live-tail event envelope",
            "type": "object",
            "required": ["schema", "event_id", "sequence", "cursor", "timestamp", "event_type", "schema_version", "scope", "source_state_revision", "payload_ref", "invalidate", "correlation_id", "causation_id", "payload"],
            "properties": {
                "schema": {"type": "string", "enum": ["focusa.stream_event.v1"]},
                "event_id": {"type": "string"},
                "sequence": {"type": "integer", "minimum": 1},
                "cursor": {"type": "string", "pattern": "^[1-9][0-9]*$"},
                "timestamp": {"type": "string", "format": "date-time"},
                "event_type": {"type": "string"},
                "schema_version": {"type": "string"},
                "scope": {
                    "type": "object",
                    "required": ["project_root", "continuity_id", "attachment_id", "work_surface_id"],
                    "properties": {
                        "project_root": {},
                        "continuity_id": {},
                        "attachment_id": {},
                        "work_surface_id": {}
                    },
                    "additionalProperties": false
                },
                "source_state_revision": {},
                "payload_ref": {},
                "invalidate": {"type": "array", "items": {"type": "string"}},
                "correlation_id": {},
                "causation_id": {},
                "payload": {}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "event_hash_chain.chain_index"
        });
    }
    if schema_id == TOOL_RESULT_SCHEMA {
        return json!({
            "$schema": JSON_SCHEMA_DIALECT_2020_12,
            "$id": format!("/v1/agent/schemas/{schema_id}"),
            "title": "Focusa ToolResult v1",
            "description": "Canonical Focusa success, failure, retry, and recovery envelope",
            "type": "object",
            "required": ["schema", "ok", "status", "canonical", "degraded", "summary", "retry", "side_effects", "evidence_refs", "next_tools"],
            "properties": {
                "schema": {"type": "string", "enum": [TOOL_RESULT_SCHEMA]},
                "ok": {"type": "boolean"},
                "status": {"type": "string", "enum": ["accepted", "completed", "no_op", "blocked", "validation_rejected", "degraded", "offline", "error"]},
                "failure_class": {"type": "string", "enum": ["validation_rejected", "schema_invalid", "not_found", "frame_unavailable", "daemon_unavailable", "stale_runtime_registry", "resource_exhausted", "null_response", "hot_path_timeout", "cold_path_timeout", "writer_conflict", "scope_mismatch", "scope_conflict", "approval_required", "permission_denied", "process_control_failed", "noncanonical_fallback", "read_model_lag", "unknown_ambiguous_completion"]},
                "canonical": {"type": "boolean"},
                "degraded": {"type": "boolean"},
                "summary": {"type": "string", "maxLength": 240},
                "tool": {"type": "string"},
                "family": {"type": "string"},
                "endpoint": {"type": "string"},
                "workpoint_id": {"type": "string"},
                "retry": {
                    "type": "object",
                    "required": ["safe", "posture"],
                    "properties": {
                        "safe": {"type": "boolean"},
                        "posture": {"type": "string", "enum": ["safe_retry", "retry_with_idempotency_key", "check_side_effects_first", "do_not_retry_unchanged", "operator_required"]},
                        "reason": {"type": "string"}
                    },
                    "additionalProperties": false
                },
                "recovery_hint": {"type": "string"},
                "misuse_hint": {"type": "string"},
                "side_effects": {"type": "array", "items": {"type": "string"}},
                "evidence_refs": {"type": "array", "items": {"type": "string"}},
                "next_tools": {"type": "array", "items": {"type": "string"}},
                "reflex_suggestions": {"type": "array", "items": {"type": "string"}},
                "ontology_candidate_delta_refs": {"type": "array", "items": {"type": "string"}},
                "error": {"type": "object", "additionalProperties": true},
                "raw": {}
            },
            "additionalProperties": false,
            "x-focusa-schema-id": schema_id,
            "x-focusa-generated-from": "focusa_core::tool_result::ToolResultV1"
        });
    }
    json!({
        "$schema": JSON_SCHEMA_DIALECT_2020_12,
        "$id": format!("/v1/agent/schemas/{schema_id}"),
        "title": schema_id,
        "description": format!("Generated contract for Focusa schema {schema_id}"),
        "type": "object",
        "properties": {},
        "additionalProperties": true,
        "x-focusa-schema-id": schema_id,
        "x-focusa-generated-from": "focusa.agent_operation_registry.v1",
    })
}

fn openapi_schema_component(schema_id: &str) -> Value {
    let mut schema = json_schema_document(schema_id);
    if let Some(object) = schema.as_object_mut() {
        object.remove("$schema");
        object.remove("$id");
        object.insert(
            "x-focusa-json-schema-dialect".to_string(),
            json!(JSON_SCHEMA_DIALECT_2020_12),
        );
    }
    schema
}

pub async fn list_schemas(State(_state): State<Arc<AppState>>) -> Json<Value> {
    let schema_ids: Vec<&str> = registered_schema_ids().into_iter().collect();
    Json(json!({
        "schema": "focusa.agent_schema_index.v1",
        "api_version": "v1",
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "schema_count": schema_ids.len(),
        "schemas": schema_ids,
        "list_detail": "GET /v1/agent/schemas/{schema_id}",
    }))
}

pub async fn get_schema(
    State(_state): State<Arc<AppState>>,
    Path(schema_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let normalized = schema_id.trim().to_string();
    if normalized.is_empty() || normalized.len() > 256 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "validation_rejected",
                "failure_class": "invalid_schema_id",
                "message": "Schema ID must be 1-256 characters",
                "requested": schema_id,
            })),
        ));
    }
    if !registered_schema_ids().contains(normalized.as_str()) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "status": "not_found",
                "failure_class": "unknown_schema_id",
                "message": format!("Schema ID is not registered: {normalized}"),
                "requested": normalized,
                "recovery_hint": "List registered contracts at GET /v1/agent/schemas",
            })),
        ));
    }
    Ok(Json(json_schema_document(&normalized)))
}

fn openapi_document() -> Value {
    let ops = build_operations();
    let mut paths = serde_json::Map::new();
    let mut schemas = serde_json::Map::new();
    for schema_id in registered_schema_ids() {
        schemas.insert(
            schema_component_name(schema_id),
            openapi_schema_component(schema_id),
        );
    }
    for op in &ops {
        let path_key = op.path.to_string();
        let method_key = op.method.to_lowercase();
        let entry = paths.entry(path_key).or_insert_with(|| json!({}));
        if let Some(obj) = entry.as_object_mut() {
            let mut parameters: Vec<Value> = op
                .scope
                .required_keys
                .iter()
                .map(|key| {
                    json!({
                        "name": key,
                        "in": "query",
                        "required": true,
                        "schema": {"type": "string"},
                        "description": format!("Required Focusa scope key: {key}"),
                    })
                })
                .collect();
            if op.operation_id == "focusa.events.stream" {
                parameters.extend([
                    json!({
                        "name": "cursor",
                        "in": "query",
                        "required": false,
                        "schema": {"type": "string"},
                        "description": "Stable 1-based durable event sequence cursor",
                    }),
                    json!({
                        "name": "Last-Event-ID",
                        "in": "header",
                        "required": false,
                        "schema": {"type": "string"},
                        "description": "Prior SSE sequence cursor or durable event UUID",
                    }),
                    json!({"name": "project_root", "in": "query", "required": false, "schema": {"type": "string"}, "description": "Optional exact project filter"}),
                    json!({"name": "continuity_id", "in": "query", "required": false, "schema": {"type": "string"}, "description": "Optional exact workstream filter"}),
                    json!({"name": "attachment_id", "in": "query", "required": false, "schema": {"type": "string"}, "description": "Optional exact Attachment filter"}),
                    json!({"name": "session_id", "in": "query", "required": false, "schema": {"type": "string"}, "description": "Optional exact producing-session filter"}),
                    json!({"name": "work_surface_id", "in": "query", "required": false, "schema": {"type": "string"}, "description": "Optional exact Work Surface filter"}),
                ]);
            }
            let response_schema = if op.operation_id == "focusa.events.stream" {
                json!({
                    "description": "Durable replay followed by gap-free live event tail",
                    "content": {
                        "text/event-stream": {
                            "schema": {"type": "string"},
                            "x-focusa-event-schema": op.response_schema_ref,
                        }
                    }
                })
            } else {
                json!({
                    "description": format!("{} response", op.label),
                    "content": {
                        "application/json": {
                            "schema": {
                                "$ref": format!("#/components/schemas/{}", op.response_schema_ref.replace('.', "_")),
                            }
                        }
                    }
                })
            };
            let mut operation = json!({
                "operationId": op.operation_id,
                "summary": op.label,
                "description": format!("{} — family={} budget={} materialization={}", op.label, op.family, op.budget_profile, op.materialization_mode),
                "tags": [op.family],
                "parameters": parameters,
                "responses": {
                    "200": response_schema,
                    "default": {
                        "description": "Standard error envelope",
                        "content": {
                            "application/json": {
                                "schema": {"$ref": "#/components/schemas/focusa_tool_result_v1"}
                            }
                        }
                    }
                },
                "x-focusa-subsystem": op.ownership.subsystem,
                "x-focusa-core-action": op.ownership.core_action_ref,
                "x-focusa-scope-keys": op.scope.required_keys,
                "x-focusa-capabilities": op.control.capability_refs,
                "x-focusa-permissions": op.control.permission_scopes,
                "x-focusa-mode": op.control.mode,
                "x-focusa-confirmation": op.control.confirmation,
                "x-focusa-idempotency": op.control.idempotency_required,
                "x-focusa-concurrency": op.control.optimistic_concurrency_required,
                "x-focusa-receipt": op.control.receipt_required,
                "x-focusa-reversible": op.control.reversible,
                "x-focusa-generated-ui": op.ui.allowed_in_generated_ui,
                "x-focusa-plain-label": op.ui.default_label,
                "x-focusa-advanced-only": op.ui.advanced_only,
                "x-focusa-sensitive": op.ui.sensitivity,
                "x-focusa-result-envelope": TOOL_RESULT_SCHEMA,
                "x-focusa": {
                    "family": op.family,
                    "canonical": op.canonical,
                    "budget_profile": op.budget_profile,
                    "side_effect_profile": op.side_effect_profile,
                    "materialization_mode": op.materialization_mode,
                    "supports_side_effect_policy": op.supports_side_effect_policy,
                    "requires_preview_token": op.requires_preview_token,
                    "deprecation": op.deprecation,
                },
            });
            if op.method != "GET" {
                operation["requestBody"] = json!({
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": {
                                "$ref": format!("#/components/schemas/{}", schema_component_name(op.request_schema_ref)),
                            }
                        }
                    }
                });
            }
            obj.insert(method_key, operation);
        }
    }
    json!({
        "openapi": OPENAPI_VERSION,
        "x-focusa-json-schema-dialect": JSON_SCHEMA_DIALECT_2020_12,
        "info": {
            "title": "Focusa Agent Runtime API",
            "version": "v1",
            "description": "Agent-first REST API for Focusa — mission state, Workpoints, trajectory, evidence, metacognition, predictions, and governance.",
        },
        "servers": [{"url": "/", "description": "Focusa daemon"}],
        "paths": paths,
        "components": {
            "schemas": Value::Object(schemas),
        },
    })
}

pub async fn openapi_export(State(_state): State<Arc<AppState>>) -> Json<Value> {
    Json(openapi_document())
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/v1/agent/adapter-capabilities",
            get(adapter_capabilities_handler),
        )
        .route("/v1/agent/card", get(agent_card_handler))
        .route("/v1/agent/capabilities", get(capabilities_index_handler))
        .route("/v1/agent/tools", get(agent_tools_handler))
        .route("/v1/agent/tools/{name}", get(agent_tool_describe_handler))
        .route("/v1/agent/tool-graph", get(agent_tool_graph_handler))
        .route("/v1/agent/tool-bundles", get(agent_tool_bundles_handler))
        .route("/v1/agent/tool-changes", get(agent_tool_changes_handler))
        .route("/v1/agent/operations", get(operation_registry_handler))
        .route(
            "/v1/agent/compatibility-lock",
            get(compatibility_lock_handler),
        )
        .route("/v1/agent/handshake", post(protocol_handshake_handler))
        .route(
            "/v1/agent/ui-action-bindings",
            get(ui_action_bindings_handler),
        )
        .route(
            "/v1/agent/ui-capabilities",
            get(ui_capability_snapshot_handler),
        )
        .route("/v1/agent/schemas", get(list_schemas))
        .route("/v1/agent/schemas/{schema_id}", get(get_schema))
        .route("/v1/openapi.json", get(openapi_export))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_capability_registry_is_typed_and_truthful() {
        let registry = &*ADAPTER_CAPABILITY_REGISTRY;
        assert_eq!(registry.schema, "focusa.adapter_capability_registry.v1");
        assert!(!registry.registry_version.is_empty());

        let adapters: std::collections::BTreeSet<_> = registry
            .adapters
            .iter()
            .map(|manifest| manifest.adapter.as_str())
            .collect();
        assert_eq!(adapters, ["claude", "codex", "opencode", "pi"].into());

        for manifest in &registry.adapters {
            assert!(manifest.manifest_version > 0);
            assert!(!manifest.measured_at.is_empty());
            assert!(!manifest.measured_against.is_empty());
            assert!(!manifest.evidence_refs.is_empty());
            assert!(!manifest.limitations.is_empty());
            match manifest.tier {
                AdapterTier::TierA => assert!(manifest.supports_automatic_native_rollover),
                AdapterTier::TierB => {
                    assert!(!manifest.supports_automatic_native_rollover);
                    assert!(
                        manifest.supports_user_command_rollover || manifest.supports_rpc_rollover
                    );
                }
                AdapterTier::TierC => {
                    assert!(!manifest.supports_automatic_native_rollover);
                    assert!(!manifest.supports_user_command_rollover);
                    assert!(!manifest.supports_rpc_rollover);
                    assert!(manifest.supports_preload_receipt);
                }
                AdapterTier::TierD => {
                    assert!(!manifest.supports_automatic_native_rollover);
                    assert!(!manifest.supports_user_command_rollover);
                    assert!(!manifest.supports_rpc_rollover);
                }
            }
        }

        let pi = registry
            .adapters
            .iter()
            .find(|manifest| manifest.adapter == "pi")
            .expect("Pi manifest");
        assert_eq!(pi.tier, AdapterTier::TierB);
        assert!(pi.supports_compaction_hook);
        assert!(pi.supports_session_size_preflight);
        assert!(pi.supports_streaming_import);
        assert!(!pi.supports_automatic_native_rollover);
    }

    #[test]
    fn operation_catalog_includes_turn_and_memory_routes() {
        let operations = build_operations();
        let paths: std::collections::BTreeSet<_> =
            operations.iter().map(|operation| operation.path).collect();
        for path in [
            "/v1/turn/start",
            "/v1/turn/append",
            "/v1/turn/complete",
            "/v1/memory/semantic",
            "/v1/memory/semantic/upsert",
            "/v1/memory/procedural",
            "/v1/memory/procedural/reinforce",
        ] {
            assert!(paths.contains(path), "missing operation path {path}");
        }
    }

    #[test]
    fn operation_catalog_exposes_turn_schema_ids() {
        let schemas: std::collections::BTreeSet<_> = build_operations()
            .iter()
            .flat_map(|operation| [operation.request_schema_ref, operation.response_schema_ref])
            .collect();
        for schema in [
            "focusa.turn_start.request.v1",
            "focusa.turn_start.response.v1",
            "focusa.turn_append.request.v1",
            "focusa.turn_append.response.v1",
            "focusa.turn_complete.request.v1",
            "focusa.turn_complete.response.v1",
        ] {
            assert!(schemas.contains(schema), "missing schema {schema}");
        }
    }

    #[test]
    fn turn_and_memory_families_are_advertised() {
        let families = build_families();
        assert!(families.contains(&"turn"));
        assert!(families.contains(&"memory"));
    }

    #[test]
    fn registered_schema_documents_use_json_schema_2020_12() {
        for schema_id in registered_schema_ids() {
            let schema = json_schema_document(schema_id);
            assert_eq!(schema["$schema"], JSON_SCHEMA_DIALECT_2020_12);
            assert_eq!(schema["type"], "object");
            assert_eq!(schema["x-focusa-schema-id"], schema_id);
        }
    }

    #[test]
    fn openapi_export_is_3_0_3_and_resolves_every_operation_schema() {
        let document = openapi_document();
        assert_eq!(document["openapi"], OPENAPI_VERSION);
        assert_eq!(
            document["x-focusa-json-schema-dialect"],
            JSON_SCHEMA_DIALECT_2020_12
        );
        let schemas = document["components"]["schemas"]
            .as_object()
            .expect("OpenAPI component schemas");
        assert_eq!(schemas.len(), registered_schema_ids().len());
        assert!(schemas.contains_key("focusa_tool_result_v1"));

        for operation in build_operations() {
            assert!(schemas.contains_key(&schema_component_name(operation.request_schema_ref)));
            assert!(schemas.contains_key(&schema_component_name(operation.response_schema_ref)));
            let rendered = &document["paths"][operation.path][operation.method.to_lowercase()];
            assert_eq!(rendered["operationId"], operation.operation_id);
            if operation.method == "GET" {
                assert!(rendered.get("requestBody").is_none());
            } else {
                assert!(rendered.get("requestBody").is_some());
            }
            for extension in [
                "x-focusa-subsystem",
                "x-focusa-core-action",
                "x-focusa-scope-keys",
                "x-focusa-capabilities",
                "x-focusa-permissions",
                "x-focusa-mode",
                "x-focusa-confirmation",
                "x-focusa-idempotency",
                "x-focusa-concurrency",
                "x-focusa-receipt",
                "x-focusa-reversible",
                "x-focusa-generated-ui",
                "x-focusa-plain-label",
                "x-focusa-advanced-only",
                "x-focusa-sensitive",
                "x-focusa-result-envelope",
            ] {
                assert!(rendered.get(extension).is_some(), "missing {extension}");
            }
        }
    }

    #[test]
    fn operation_registry_descriptors_are_complete_and_unique() {
        let operations = build_operations();
        let mut ids = std::collections::BTreeSet::new();
        let mut routes = std::collections::BTreeSet::new();
        for operation in &operations {
            assert!(ids.insert(operation.operation_id));
            assert!(routes.insert((operation.method, operation.path)));
            assert_eq!(
                operation.descriptor_schema,
                "focusa.operation_descriptor.v1"
            );
            assert_eq!(operation.ownership.core_action_ref, operation.operation_id);
            assert_eq!(
                operation.contracts.input_schema_ref,
                operation.request_schema_ref
            );
            assert_eq!(
                operation.contracts.output_schema_ref,
                operation.response_schema_ref
            );
            assert_eq!(
                operation.contracts.error_schema_ref,
                "focusa.tool_result.v1"
            );
            assert!(!operation.control.capability_refs.is_empty());
            assert!(!operation.ui.default_label.is_empty());
            if operation.method != "GET" {
                assert!(operation.control.receipt_required);
            }
        }
        assert!(ids.contains("focusa.operation_registry.read"));
        assert!(ids.contains("focusa.ui_action_bindings.read"));
        assert!(ids.contains("focusa.ui_capability_snapshot.read"));
    }

    #[test]
    fn generated_ui_bindings_and_capability_projection_share_registry_authority() {
        let scope = UiProjectionQuery {
            project_root: Some("/tmp/project".into()),
            continuity_id: Some("continuity-1".into()),
            attachment_id: None,
            agent_id: Some("agent-1".into()),
        };
        let bindings = ui_action_bindings_document(&scope);
        let expected = build_operations()
            .iter()
            .filter(|operation| operation.ui.allowed_in_generated_ui)
            .count();
        assert_eq!(bindings["schema"], "focusa.ui_action_binding_index.v1");
        assert_eq!(bindings["binding_count"], expected);
        assert_eq!(
            bindings["bindings"].as_array().map(Vec::len),
            Some(expected)
        );
        for binding in bindings["bindings"].as_array().expect("bindings") {
            assert_eq!(binding["schema"], "focusa.ui_action_binding.v1");
            assert_eq!(binding["result_envelope_ref"], "focusa.tool_result.v1");
            assert!(ids_contains(
                binding["action_id"].as_str().unwrap_or_default()
            ));
        }

        let permissions = permission_context(&HeaderMap::new(), false);
        let snapshot = ui_capability_snapshot_document(&scope, &permissions);
        assert_eq!(snapshot["schema"], "focusa.ui_capability_snapshot.v1");
        assert_eq!(snapshot["project_root"], "/tmp/project");
        assert!(
            snapshot["capabilities"]
                .as_array()
                .is_some_and(|items| !items.is_empty())
        );

        let restricted = permission_context(&HeaderMap::new(), true);
        let restricted_snapshot = ui_capability_snapshot_document(&scope, &restricted);
        assert!(
            restricted_snapshot["permissions"]["missing_scopes"]
                .as_array()
                .is_some_and(|scopes| scopes.iter().any(|scope| scope == "project:read"))
        );
        assert!(
            restricted_snapshot["capabilities"]
                .as_array()
                .is_some_and(|items| items.iter().any(|item| {
                    item["capability_id"] == "project" && item["status"] == "approval_required"
                }))
        );
    }

    #[test]
    fn protocol_handshake_is_fail_closed_and_compatibility_lock_is_complete() {
        let mut request = ProtocolHandshakeRequest {
            client_id: "test-client".to_string(),
            client_versions: REQUIRED_PROTOCOL_VERSIONS
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            requested_capabilities: vec!["project".to_string()],
        };
        assert!(handshake_mismatches(&request).is_empty());
        request.client_versions.remove("event_stream");
        let mismatches = handshake_mismatches(&request);
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0]["component"], "event_stream");
        assert_eq!(mismatches[0]["required"], "1.0.0");

        let lock = compatibility_lock_document();
        assert_eq!(lock["schema"], "focusa.compatibility_lock.v1");
        for component in [
            "focusa_runtime",
            "focusa_api",
            "operation_registry",
            "tool_result",
            "event_stream",
            "a2ui_protocol",
            "minimum_reader_versions",
            "minimum_writer_versions",
        ] {
            assert!(lock.get(component).is_some(), "missing {component}");
        }
        assert!(projection_scope_error(&UiProjectionQuery::default()).is_some());
    }

    fn ids_contains(id: &str) -> bool {
        build_operations()
            .iter()
            .any(|operation| operation.operation_id == id)
    }
}
