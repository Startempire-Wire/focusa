//! Spec 109 AX-001 — Authoritative Agent Capabilities Endpoint.
//!
//! `GET /v1/agent/capabilities` returns a compact machine-readable index of
//! every Focusa operation, with metadata for schema, side effects, permissions,
//! and documentation refs. Agents use this to discover what Focusa can do
//! without reading docs.
//!
//! `GET /v1/agent/adapter-capabilities` publishes the separate Spec130 measured
//! native-adapter capability registry.

use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Json, Router, routing::get};
use chrono::Utc;
use focusa_core::tool_result::TOOL_RESULT_SCHEMA;
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
    let project_scoped = !matches!(family, "health" | "device" | "license");
    let workstream_scoped = matches!(
        family,
        "trajectory"
            | "workpoint"
            | "metacognition"
            | "evidence"
            | "prediction"
            | "context_cognition"
            | "turn"
            | "memory"
    );
    let attachment_scoped = id.contains("attachment");
    let mut required_keys = Vec::new();
    if project_scoped {
        required_keys.push("project_root");
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
    ]
}

fn build_families() -> Vec<&'static str> {
    vec![
        "health",
        "agent",
        "project",
        "trajectory",
        "workpoint",
        "metacognition",
        "evidence",
        "prediction",
        "context_cognition",
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
        "status": "error",
        "failure_class": "serialization_error",
        "message": "Failed to serialize capabilities index"
    })))
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

async fn ui_action_bindings_handler(Query(scope): Query<UiProjectionQuery>) -> Json<Value> {
    Json(ui_action_bindings_document(&scope))
}

fn ui_capability_snapshot_document(scope: &UiProjectionQuery) -> Value {
    let operations = build_operations();
    let capability_ids: std::collections::BTreeSet<_> = operations
        .iter()
        .flat_map(|operation| operation.control.capability_refs.iter().copied())
        .collect();
    let permission_scopes: std::collections::BTreeSet<_> = operations
        .iter()
        .flat_map(|operation| operation.control.permission_scopes.iter().copied())
        .collect();
    let capabilities: Vec<Value> = capability_ids
        .into_iter()
        .map(|capability_id| {
            json!({
                "capability_id": capability_id,
                "status": "available",
                "reason": "Published by the canonical Focusa Operation Registry",
                "recovery_action_ref": Value::Null,
            })
        })
        .collect();
    json!({
        "schema": "focusa.ui_capability_snapshot.v1",
        "project_root": scope.project_root.as_deref(),
        "continuity_id": scope.continuity_id.as_deref(),
        "attachment_id": scope.attachment_id.as_deref(),
        "agent_id": scope.agent_id.as_deref(),
        "capabilities": capabilities,
        "permissions": {
            "granted_scopes": [],
            "missing_scopes": permission_scopes,
        },
        "providers": [],
        "connectors": [],
        "client_capabilities": ["openapi-3.0.3", "json-schema-2020-12", "a2ui-action-bindings"],
        "source_state_revision": "operation-registry-1.0.0",
    })
}

async fn ui_capability_snapshot_handler(Query(scope): Query<UiProjectionQuery>) -> Json<Value> {
    Json(ui_capability_snapshot_document(&scope))
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
    schema_ids
}

fn json_schema_document(schema_id: &str) -> Value {
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
            let parameters: Vec<Value> = op
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
            let response_schema = json!({
                "description": format!("{} response", op.label),
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": format!("#/components/schemas/{}", op.response_schema_ref.replace('.', "_")),
                        }
                    }
                }
            });
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
        .route("/v1/agent/capabilities", get(capabilities_index_handler))
        .route("/v1/agent/tools", get(capabilities_index_handler))
        .route("/v1/agent/operations", get(operation_registry_handler))
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

        let snapshot = ui_capability_snapshot_document(&scope);
        assert_eq!(snapshot["schema"], "focusa.ui_capability_snapshot.v1");
        assert_eq!(snapshot["project_root"], "/tmp/project");
        assert!(
            snapshot["capabilities"]
                .as_array()
                .is_some_and(|items| !items.is_empty())
        );
    }

    fn ids_contains(id: &str) -> bool {
        build_operations()
            .iter()
            .any(|operation| operation.operation_id == id)
    }
}
