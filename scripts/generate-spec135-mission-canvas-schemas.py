#!/usr/bin/env python3
"""Generate portable Spec 135 Mission Canvas composition schemas."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "schemas/spec135/mission-canvas"
BASE = "https://focusa.dev/schemas/spec135/mission-canvas"
DIALECT = "https://json-schema.org/draft/2020-12/schema"


def string_array(unique: bool = True) -> dict[str, Any]:
    result: dict[str, Any] = {"type": "array", "items": {"type": "string"}}
    if unique:
        result["uniqueItems"] = True
    return result


def definitions() -> dict[str, Any]:
    return {
        # These identity definitions mirror the canonical Spec 158 Rust owners.
        # Continuity and runtime/presentation identifiers are subordinate to a
        # Workstream and are never accepted as standalone authority.
        "ProjectRootKey": {
            "type": "object",
            "additionalProperties": False,
            "required": ["scope_kind", "scope_id", "root_path", "canonical_name", "fingerprint"],
            "properties": {
                "scope_kind": {"const": "project"},
                "scope_id": {"type": "string", "minLength": 1},
                "root_path": {"type": "string", "minLength": 1},
                "canonical_name": {"type": "string", "minLength": 1},
                "fingerprint": {"type": "string", "minLength": 1},
            },
        },
        "HostScopeKey": {
            "type": "object",
            "additionalProperties": False,
            "required": ["scope_kind", "scope_id", "root_path", "canonical_name", "fingerprint"],
            "properties": {
                "scope_kind": {"const": "host"},
                "scope_id": {"type": "string", "minLength": 1},
                "root_path": {"type": "string", "minLength": 1},
                "canonical_name": {"type": "string", "minLength": 1},
                "fingerprint": {"type": "string", "minLength": 1},
            },
        },
        "ScopeRef": {
            "oneOf": [
                {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["scope_kind", "scope_key"],
                    "properties": {
                        "scope_kind": {"const": "project"},
                        "scope_key": {"$ref": "#/$defs/ProjectRootKey"},
                    },
                },
                {
                    "type": "object",
                    "additionalProperties": False,
                    "required": ["scope_kind", "scope_key"],
                    "properties": {
                        "scope_kind": {"const": "host"},
                        "scope_key": {"$ref": "#/$defs/HostScopeKey"},
                    },
                },
            ],
        },
        "WorkstreamId": {"type": "string", "minLength": 1},
        "WorkstreamKey": {
            "type": "object",
            "additionalProperties": False,
            "required": ["scope", "workstream_id"],
            "properties": {
                "scope": {"$ref": "#/$defs/ScopeRef"},
                "workstream_id": {"$ref": "#/$defs/WorkstreamId"},
            },
        },
        "ContinuityId": {"type": "string", "minLength": 1},
        "InstanceId": {"type": "string", "minLength": 1},
        "SessionId": {"type": "string", "minLength": 1},
        "AttachmentId": {"type": "string", "minLength": 1},
        "WorkspaceBindingId": {"type": "string", "minLength": 1},
        "AttachmentKey": {
            "type": "object",
            "additionalProperties": False,
            "required": ["workstream", "instance_id", "session_id", "attachment_id", "workspace_binding_id"],
            "properties": {
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "continuity_id": {"anyOf": [{"$ref": "#/$defs/ContinuityId"}, {"type": "null"}]},
                "instance_id": {"$ref": "#/$defs/InstanceId"},
                "session_id": {"$ref": "#/$defs/SessionId"},
                "attachment_id": {"$ref": "#/$defs/AttachmentId"},
                "workspace_binding_id": {"$ref": "#/$defs/WorkspaceBindingId"},
            },
        },
        "RuntimeObjectRef": {
            "type": "object",
            "additionalProperties": False,
            "required": ["runtime_kind", "runtime_id"],
            "properties": {
                "runtime_kind": {"type": "string", "minLength": 1},
                "runtime_id": {"type": "string", "minLength": 1},
            },
        },
        "WorkSurfaceId": {"type": "string", "minLength": 1},
        "WorkSurfaceIdentity": {
            "type": "object",
            "additionalProperties": False,
            "required": ["work_surface_id", "workstream"],
            "properties": {
                "work_surface_id": {"$ref": "#/$defs/WorkSurfaceId"},
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "continuity_id": {"anyOf": [{"$ref": "#/$defs/ContinuityId"}, {"type": "null"}]},
                "attachment": {"anyOf": [{"$ref": "#/$defs/AttachmentKey"}, {"type": "null"}]},
                "runtime_object": {"anyOf": [{"$ref": "#/$defs/RuntimeObjectRef"}, {"type": "null"}]},
            },
        },
        "WorkstreamAuthorityContext": {
            "type": "object",
            "additionalProperties": False,
            "required": ["workstream"],
            "properties": {
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "continuity_id": {"anyOf": [{"$ref": "#/$defs/ContinuityId"}, {"type": "null"}]},
                "attachment": {"anyOf": [{"$ref": "#/$defs/AttachmentKey"}, {"type": "null"}]},
                "workspace_binding_id": {"anyOf": [{"$ref": "#/$defs/WorkspaceBindingId"}, {"type": "null"}]},
                "runtime_object": {"anyOf": [{"$ref": "#/$defs/RuntimeObjectRef"}, {"type": "null"}]},
                "work_surface_id": {"anyOf": [{"$ref": "#/$defs/WorkSurfaceId"}, {"type": "null"}]},
            },
        },
        "ActorRef": {
            "type": "object",
            "additionalProperties": False,
            "required": ["actor_type", "actor_id"],
            "properties": {
                "actor_type": {"enum": ["operator", "agent", "pi", "desktop", "web", "service"]},
                "actor_id": {"type": "string", "minLength": 1},
            },
        },
        "AuthorityEnvelope": {
            "type": "object",
            "additionalProperties": False,
            "required": ["status", "why"],
            "properties": {
                "status": {"enum": ["canonical", "blocked", "degraded", "missing", "stale"]},
                "why": {"type": "string", "minLength": 1},
            },
        },
        "AuthorityContext": {
            "type": "object",
            "additionalProperties": False,
            "required": ["authority_ref", "envelope"],
            "properties": {
                "authority_ref": {"type": "string", "minLength": 1},
                "envelope": {"$ref": "#/$defs/AuthorityEnvelope"},
            },
        },
        "WorkstreamOperationRequest": {
            "type": "object",
            "additionalProperties": False,
            "required": ["schema", "workstream", "actor", "authority", "command_id", "input"],
            "properties": {
                "schema": {"const": "focusa.workstream_operation_request.v1"},
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "continuity_id": {"anyOf": [{"$ref": "#/$defs/ContinuityId"}, {"type": "null"}]},
                "attachment": {"anyOf": [{"$ref": "#/$defs/AttachmentKey"}, {"type": "null"}]},
                "workspace_binding_id": {"anyOf": [{"$ref": "#/$defs/WorkspaceBindingId"}, {"type": "null"}]},
                "runtime_object": {"anyOf": [{"$ref": "#/$defs/RuntimeObjectRef"}, {"type": "null"}]},
                "work_surface_id": {"anyOf": [{"$ref": "#/$defs/WorkSurfaceId"}, {"type": "null"}]},
                "actor": {"$ref": "#/$defs/ActorRef"},
                "authority": {"$ref": "#/$defs/AuthorityContext"},
                "command_id": {"type": "string", "minLength": 1},
                "input": {},
                "idempotency_key": {"type": ["string", "null"]},
                "expected_revision": {"type": ["integer", "null"], "minimum": 0},
                "expected_fencing_token": {"type": ["integer", "null"], "minimum": 0},
            },
        },
        "RecipientResolution": {
            "type": "object",
            "additionalProperties": False,
            "required": ["schema", "workstream", "recipient_ref", "routable"],
            "properties": {
                "schema": {"const": "focusa.mission_canvas.recipient_resolution.v1"},
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "continuity_id": {"anyOf": [{"$ref": "#/$defs/ContinuityId"}, {"type": "null"}]},
                "attachment": {"anyOf": [{"$ref": "#/$defs/AttachmentKey"}, {"type": "null"}]},
                "workspace_binding_id": {"anyOf": [{"$ref": "#/$defs/WorkspaceBindingId"}, {"type": "null"}]},
                "runtime_object": {"anyOf": [{"$ref": "#/$defs/RuntimeObjectRef"}, {"type": "null"}]},
                "work_surface_id": {"anyOf": [{"$ref": "#/$defs/WorkSurfaceId"}, {"type": "null"}]},
                "recipient_ref": {"type": "string", "minLength": 1},
                "routable": {"type": "boolean"},
            },
        },
        "DomainPackInstallReceipt": {
            "type": "object",
            "additionalProperties": False,
            "required": ["schema", "workstream", "installed", "pack_id", "receipt_ref"],
            "properties": {
                "schema": {"const": "focusa.mission_canvas.domain_pack_install_receipt.v1"},
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "installed": {"type": "boolean"},
                "pack_id": {"type": "string", "minLength": 1},
                "receipt_ref": {"type": "string", "minLength": 1},
            },
        },
        "PiSessionEventReceipt": {
            "type": "object",
            "additionalProperties": False,
            "required": ["schema", "workstream", "event_id", "accepted", "receipt_ref"],
            "properties": {
                "schema": {"const": "focusa.mission_canvas.pi_session_event_receipt.v1"},
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "event_id": {"type": "string", "minLength": 1},
                "accepted": {"type": "boolean"},
                "receipt_ref": {"type": "string", "minLength": 1},
            },
        },
        "LegacyExactScopeCompatibilityInput": {
            "type": "object",
            "additionalProperties": False,
            "required": ["project_root", "continuity_id", "session_id", "attachment_id"],
            "properties": {
                "project_root": {"type": "string", "minLength": 1},
                "continuity_id": {"type": "string", "minLength": 1},
                "instance_id": {"type": ["string", "null"]},
                "session_id": {"type": "string", "minLength": 1},
                "attachment_id": {"type": "string", "minLength": 1},
                "working_subpath_id": {"type": ["string", "null"]},
            },
            "description": "Compatibility input only. It must be resolved to an exact WorkstreamKey with provenance before canonical use and never grants authority by itself.",
            "x-focusa-compatibility-only": True,
        },
        "ContributionId": {
            "type": "string",
            "pattern": r"^contribution:[a-z0-9][a-z0-9._:-]{0,159}$",
            "description": "Stable opaque semantic contribution identity; never inferred from a visual slot.",
        },
        "SemanticBindingId": {
            "type": "string",
            "pattern": r"^semantic:[a-z0-9][a-z0-9._:-]{0,159}$",
        },
        "RendererBindingId": {
            "type": "string",
            "pattern": r"^renderer:[a-z0-9][a-z0-9._:@/-]{0,239}$",
        },
        "ContributionKind": {
            "type": "string",
            "enum": [
                "work_surface_strip",
                "focused_work_surface",
                "inspector",
                "inspector_section",
                "work_rail",
                "steering_queue",
                "follow_up_queue",
                "prompt_editor",
                "scope_bar",
                "activity_navigation",
                "toolbar_control",
                "contextual_action",
                "transient_notification",
                "generated_surface",
            ],
        },
        "RegionKind": {
            "type": "string",
            "enum": [
                "primary",
                "secondary",
                "inspector",
                "rail",
                "queue",
                "composer",
                "navigation",
                "overlay",
            ],
        },
        "OmissionReason": {
            "type": "string",
            "enum": [
                "no_relevant_content",
                "not_applicable",
                "capability_not_present",
                "not_authorized",
                "merged",
                "compacted",
                "suspended",
                "viewport_omitted",
            ],
        },
        "ViewportDescriptor": {
            "type": "object",
            "additionalProperties": False,
            "required": ["class", "css_width", "css_height", "platform", "device_pixel_ratio"],
            "properties": {
                "class": {"enum": ["minimum", "compact", "standard", "productive", "wide", "reference_capture"]},
                "css_width": {"type": "integer", "minimum": 1024},
                "css_height": {"type": "integer", "minimum": 720},
                "platform": {"enum": ["macOS", "Windows", "Linux"]},
                "device_pixel_ratio": {"type": "number", "minimum": 1, "maximum": 4},
                "zoom_percent": {"enum": [100, 125, 150, 200]},
                "high_contrast": {"type": "boolean"},
                "reduced_motion": {"type": "boolean"},
                "reduced_transparency": {"type": "boolean"},
                "text_scale_percent": {"type": "integer", "minimum": 100, "maximum": 200},
            },
        },
        "GeometryPreference": {
            "type": "object",
            "additionalProperties": False,
            "required": ["preferred_regions", "minimum_span", "maximum_span", "merge_policy", "tab_policy"],
            "properties": {
                "preferred_regions": {
                    "type": "array",
                    "minItems": 1,
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/RegionKind"},
                },
                "preferred_adjacency": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "minimum_span": {"type": "integer", "minimum": 1, "maximum": 12},
                "maximum_span": {"type": "integer", "minimum": 1, "maximum": 12},
                "preferred_order": {"type": "integer", "minimum": 0},
                "merge_policy": {"enum": ["never", "compatible", "preferred"]},
                "tab_policy": {"enum": ["never", "compatible", "preferred"]},
                "inspector_side": {"enum": ["start", "end", "profile_default", "none"]},
            },
            "allOf": [
                {
                    "if": {"required": ["minimum_span", "maximum_span"]},
                    "then": {
                        "description": "minimum_span must be less than or equal to maximum_span; enforced by portable semantic validator."
                    },
                }
            ],
        },
        "CanonicalRef": {
            "type": "object",
            "additionalProperties": False,
            "required": ["kind", "ref", "revision"],
            "properties": {
                "kind": {"type": "string", "minLength": 1},
                "ref": {"type": "string", "minLength": 1},
                "revision": {"type": ["integer", "string"], "minLength": 1},
                "freshness": {"enum": ["current", "stale", "unknown", "not_applicable"]},
            },
        },
        "ProjectionDigest": {
            "type": "string",
            "pattern": r"^sha256:[a-f0-9]{64}$",
            "description": "Digest of normalized canonical projection inputs and output.",
        },
        "OperationBinding": {
            "type": "object",
            "additionalProperties": False,
            "required": ["operation_id", "target_contribution_id", "enabled", "authority_ref"],
            "properties": {
                "operation_id": {"type": "string", "minLength": 1},
                "target_contribution_id": {"$ref": "#/$defs/ContributionId"},
                "enabled": {"type": "boolean"},
                "authority_ref": {"type": "string", "minLength": 1},
                "confirmation": {"enum": ["none", "preview", "explicit"]},
                "disabled_reason_ref": {"type": ["string", "null"]},
            },
        },
        "SingleLayoutNode": {
            "type": "object",
            "additionalProperties": False,
            "required": ["node_id", "kind", "contribution_id"],
            "properties": {
                "node_id": {"type": "string", "minLength": 1},
                "kind": {"const": "single"},
                "contribution_id": {"$ref": "#/$defs/ContributionId"},
            },
        },
        "SplitLayoutNode": {
            "type": "object",
            "additionalProperties": False,
            "required": ["node_id", "kind", "orientation", "ratio", "children"],
            "properties": {
                "node_id": {"type": "string", "minLength": 1},
                "kind": {"const": "split"},
                "orientation": {"enum": ["horizontal", "vertical"]},
                "ratio": {"type": "number", "minimum": 0.1, "maximum": 0.9},
                "children": {
                    "type": "array",
                    "minItems": 2,
                    "maxItems": 2,
                    "items": {"$ref": "#/$defs/LayoutNode"},
                },
            },
        },
        "StackLayoutNode": {
            "type": "object",
            "additionalProperties": False,
            "required": ["node_id", "kind", "children"],
            "properties": {
                "node_id": {"type": "string", "minLength": 1},
                "kind": {"const": "stack"},
                "children": {
                    "type": "array",
                    "minItems": 1,
                    "items": {"$ref": "#/$defs/LayoutNode"},
                },
                "gap_token": {"type": "string", "minLength": 1},
            },
        },
        "GridLayoutNode": {
            "type": "object",
            "additionalProperties": False,
            "required": ["node_id", "kind", "columns", "children"],
            "properties": {
                "node_id": {"type": "string", "minLength": 1},
                "kind": {"const": "grid"},
                "columns": {"type": "integer", "minimum": 1, "maximum": 12},
                "children": {
                    "type": "array",
                    "minItems": 1,
                    "items": {"$ref": "#/$defs/LayoutNode"},
                },
                "gap_token": {"type": "string", "minLength": 1},
            },
        },
        "TabLayoutNode": {
            "type": "object",
            "additionalProperties": False,
            "required": ["node_id", "kind", "contribution_ids", "active_contribution_id"],
            "properties": {
                "node_id": {"type": "string", "minLength": 1},
                "kind": {"const": "tabs"},
                "contribution_ids": {
                    "type": "array",
                    "minItems": 1,
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "active_contribution_id": {"$ref": "#/$defs/ContributionId"},
            },
        },
        "InspectorLayoutNode": {
            "type": "object",
            "additionalProperties": False,
            "required": ["node_id", "kind", "side", "primary", "inspector_contribution_ids"],
            "properties": {
                "node_id": {"type": "string", "minLength": 1},
                "kind": {"const": "inspector"},
                "side": {"enum": ["start", "end"]},
                "primary": {"$ref": "#/$defs/LayoutNode"},
                "inspector_contribution_ids": {
                    "type": "array",
                    "minItems": 1,
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "span": {"type": "integer", "minimum": 1, "maximum": 6},
            },
        },
        "LayoutNode": {
            "oneOf": [
                {"$ref": "#/$defs/SingleLayoutNode"},
                {"$ref": "#/$defs/SplitLayoutNode"},
                {"$ref": "#/$defs/StackLayoutNode"},
                {"$ref": "#/$defs/GridLayoutNode"},
                {"$ref": "#/$defs/TabLayoutNode"},
                {"$ref": "#/$defs/InspectorLayoutNode"},
            ]
        },
        "WorkspaceProfile": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "profile_id",
                "revision",
                "display_name",
                "candidate_contribution_ids",
                "density",
                "terminology_registry_ref",
                "renderer_registry_ref",
                "viability_rule_revision",
            ],
            "properties": {
                "profile_id": {"type": "string", "pattern": r"^[a-z][a-z0-9._-]{0,79}$"},
                "revision": {"type": "integer", "minimum": 0},
                "display_name": {"type": "string", "minLength": 1},
                "candidate_contribution_ids": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "density": {"enum": ["comfortable", "standard", "compact", "dense"]},
                "terminology_registry_ref": {"type": "string", "minLength": 1},
                "renderer_registry_ref": {"type": "string", "minLength": 1},
                "domain_semantic_binding_registry_ref": {"type": ["string", "null"]},
                "viability_rule_revision": {"type": "string", "minLength": 1},
                "installed": {"type": "boolean"},
            },
        },
        "ActivityMode": {
            "type": "object",
            "additionalProperties": False,
            "required": ["activity_mode_id", "revision", "display_name", "candidate_contribution_ids", "viability_rule_revision"],
            "properties": {
                "activity_mode_id": {"type": "string", "pattern": r"^[a-z][a-z0-9._-]{0,79}$"},
                "revision": {"type": "integer", "minimum": 0},
                "display_name": {"type": "string", "minLength": 1},
                "candidate_contribution_ids": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "terminology_overrides_ref": {"type": ["string", "null"]},
                "viability_rule_revision": {"type": "string", "minLength": 1},
            },
        },
        "RegistryEntry": {
            "type": "object",
            "additionalProperties": False,
            "required": ["registry_kind", "entry_id", "revision", "schema_ref", "payload_ref", "enabled"],
            "properties": {
                "registry_kind": {
                    "enum": [
                        "WorkspaceProfileRegistry",
                        "ActivityModeRegistry",
                        "PanelRegistry",
                        "HomeCanvasRegistry",
                        "WorkSurfaceRendererRegistry",
                        "ArtifactRendererRegistry",
                        "TerminologyRegistry",
                        "DomainSemanticBindingRegistry",
                    ]
                },
                "entry_id": {"type": "string", "minLength": 1},
                "revision": {"type": "integer", "minimum": 0},
                "schema_ref": {"type": "string", "minLength": 1},
                "payload_ref": {"type": "string", "minLength": 1},
                "required_capabilities": string_array(),
                "required_permissions": string_array(),
                "enabled": {"type": "boolean"},
                "supersedes_entry_id": {"type": ["string", "null"]},
            },
        },
        "ResolvedWorkspaceProjection": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "schema",
                "scope",
                "workspace_profile_id",
                "workspace_profile_revision",
                "activity_mode_id",
                "activity_mode_revision",
                "focused_work_surface_id",
                "canonical_read_model_revision",
                "candidate_contribution_ids",
                "eligible_contributions",
                "omission_diagnostics",
                "layout_tree",
                "operation_bindings",
                "focused_semantic_target",
                "projection_revision",
                "layout_revision",
                "durable_event_cursor",
                "projection_digest",
            ],
            "properties": {
                "schema": {"const": "focusa.resolved_workspace_projection.v1"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "workspace_profile_id": {"type": "string", "minLength": 1},
                "workspace_profile_revision": {"type": "integer", "minimum": 0},
                "activity_mode_id": {"type": "string", "minLength": 1},
                "activity_mode_revision": {"type": "integer", "minimum": 0},
                "focused_work_surface_id": {"type": ["string", "null"]},
                "canonical_read_model_revision": {"type": "integer", "minimum": 0},
                "candidate_contribution_ids": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "eligible_contributions": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/ResolvedContribution"},
                },
                "omission_diagnostics": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/OmissionDiagnostic"},
                },
                "layout_tree": {"$ref": "#/$defs/LayoutNode"},
                "operation_bindings": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/OperationBinding"},
                },
                "focused_semantic_target": {"type": "string", "minLength": 1},
                "projection_revision": {"type": "integer", "minimum": 0},
                "layout_revision": {"type": "integer", "minimum": 0},
                "durable_event_cursor": {"type": "string", "minLength": 1},
                "projection_digest": {"$ref": "#/$defs/ProjectionDigest"},
                "resolved_at": {"type": "string", "format": "date-time"},
                "evidence_refs": string_array(),
                "receipt_refs": string_array(),
            },
        },
        "ContributionPlacementPreference": {
            "type": "object",
            "additionalProperties": False,
            "required": ["contribution_id", "preferred_regions", "preferred_order", "minimum_span", "maximum_span"],
            "properties": {
                "contribution_id": {"$ref": "#/$defs/ContributionId"},
                "preferred_regions": {
                    "type": "array",
                    "minItems": 1,
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/RegionKind"},
                },
                "preferred_order": {"type": "integer", "minimum": 0},
                "minimum_span": {"type": "integer", "minimum": 1, "maximum": 12},
                "maximum_span": {"type": "integer", "minimum": 1, "maximum": 12},
                "preferred_adjacency": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "last_compatible_layout_node_id": {"type": ["string", "null"]},
            },
        },
        "ProfileLayoutMemory": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "memory_id",
                "scope",
                "profile_id",
                "activity_mode_id",
                "viewport_class",
                "placements",
                "absent_contribution_ids",
                "memory_revision",
                "idempotency_key",
                "updated_at",
            ],
            "properties": {
                "memory_id": {"type": "string", "pattern": r"^layout-memory:[a-z0-9._:-]+$"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "profile_id": {"type": "string", "minLength": 1},
                "activity_mode_id": {"type": "string", "minLength": 1},
                "viewport_class": {"enum": ["minimum", "compact", "standard", "productive", "wide", "reference_capture"]},
                "placements": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/ContributionPlacementPreference"},
                },
                "absent_contribution_ids": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "focused_semantic_target": {"type": ["string", "null"]},
                "memory_revision": {"type": "integer", "minimum": 0},
                "idempotency_key": {"type": "string", "minLength": 1},
                "updated_at": {"type": "string", "format": "date-time"},
            },
        },
        "CanvasDraftState": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "draft_id",
                "scope",
                "owner",
                "content",
                "content_sha256",
                "recipient_ref",
                "attachment_id",
                "draft_revision",
                "sync_state",
                "idempotency_key",
                "updated_at",
            ],
            "properties": {
                "draft_id": {"type": "string", "pattern": r"^draft:[a-z0-9._:-]+$"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "owner": {"enum": ["pi_editor", "canvas_prompt_editor"]},
                "content": {"type": "string"},
                "content_sha256": {"type": "string", "pattern": r"^[a-f0-9]{64}$"},
                "recipient_ref": {"type": "string", "minLength": 1},
                "attachment_id": {"type": "string", "minLength": 1},
                "selection_start": {"type": "integer", "minimum": 0},
                "selection_end": {"type": "integer", "minimum": 0},
                "draft_revision": {"type": "integer", "minimum": 0},
                "sync_state": {"enum": ["synchronized", "pi_newer", "canvas_newer", "conflict", "offline"]},
                "conflict_ref": {"type": ["string", "null"]},
                "idempotency_key": {"type": "string", "minLength": 1},
                "updated_at": {"type": "string", "format": "date-time"},
            },
        },
        "HostRendererResolution": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "interaction_mode",
                "selected_renderer",
                "platform",
                "availability",
                "resolution_reason",
                "resolver_revision",
            ],
            "properties": {
                "interaction_mode": {"enum": ["canvas-guided", "terminal-guided", "headless"]},
                "selected_renderer": {
                    "enum": [
                        "focusa_pi_rich_window",
                        "uiai_engine_cockpit",
                        "mission_deck_web",
                        "pi_terminal_projection",
                        "native_tui",
                        "menubar_peek",
                        "headless_none",
                    ]
                },
                "platform": {"enum": ["macOS", "Windows", "Linux"]},
                "availability": {"enum": ["available", "fallback", "unavailable", "headless"]},
                "resolution_reason": {"type": "string", "minLength": 1},
                "asset_version": {"type": ["string", "null"]},
                "asset_digest": {"type": ["string", "null"]},
                "resolver_revision": {"type": "string", "minLength": 1},
                "diagnostic_ref": {"type": ["string", "null"]},
            },
        },
        "HostLifecycleState": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "host_instance_id",
                "scope",
                "renderer_resolution",
                "state",
                "focused",
                "durable_event_cursor",
                "lifecycle_revision",
                "updated_at",
            ],
            "properties": {
                "host_instance_id": {"type": "string", "pattern": r"^rich-host:[a-z0-9._:-]+$"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "renderer_resolution": {"$ref": "#/$defs/HostRendererResolution"},
                "state": {"enum": ["absent", "launching", "visible", "focused", "hidden", "closing", "reconnecting", "failed"]},
                "process_id": {"type": ["integer", "null"], "minimum": 1},
                "window_id": {"type": ["string", "null"]},
                "focused": {"type": "boolean"},
                "durable_event_cursor": {"type": "string", "minLength": 1},
                "pi_draft_ref": {"type": ["string", "null"]},
                "canvas_draft_ref": {"type": ["string", "null"]},
                "last_error_ref": {"type": ["string", "null"]},
                "lifecycle_revision": {"type": "integer", "minimum": 0},
                "updated_at": {"type": "string", "format": "date-time"},
            },
        },
        "UnavailableOperationDiagnostic": {
            "type": "object",
            "additionalProperties": False,
            "required": ["operation_id", "reason", "diagnostic_ref"],
            "properties": {
                "operation_id": {"type": "string", "minLength": 1},
                "reason": {"enum": ["capability_not_present", "not_authorized", "not_applicable", "suspended", "offline"]},
                "diagnostic_ref": {"type": "string", "minLength": 1},
            },
        },
        "CapabilityProjection": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "scope",
                "capabilities",
                "permissions",
                "available_operation_ids",
                "unavailable_operations",
                "capability_revision",
                "observed_at",
            ],
            "properties": {
                "scope": {"$ref": "#/$defs/ExactScope"},
                "capabilities": string_array(),
                "permissions": string_array(),
                "available_operation_ids": string_array(),
                "unavailable_operations": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/UnavailableOperationDiagnostic"},
                },
                "capability_revision": {"type": "integer", "minimum": 0},
                "observed_at": {"type": "string", "format": "date-time"},
            },
        },
        "LayoutMutationCommand": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "command_id",
                "scope",
                "action",
                "attachment_id",
                "expected_projection_revision",
                "expected_layout_revision",
                "idempotency_key",
            ],
            "properties": {
                "command_id": {"type": "string", "pattern": r"^layout-command:[a-z0-9._:-]+$"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "action": {
                    "enum": [
                        "open",
                        "focus",
                        "pin",
                        "unpin",
                        "group",
                        "ungroup",
                        "reorder",
                        "split_horizontal",
                        "split_vertical",
                        "resize_split",
                        "compare",
                        "suspend_projection",
                        "rehydrate",
                        "close_projection",
                        "set_active_tab",
                    ]
                },
                "attachment_id": {"type": "string", "minLength": 1},
                "target_work_surface_id": {"type": ["string", "null"]},
                "secondary_work_surface_id": {"type": ["string", "null"]},
                "target_contribution_id": {
                    "anyOf": [{"$ref": "#/$defs/ContributionId"}, {"type": "null"}]
                },
                "target_layout_node_id": {"type": ["string", "null"]},
                "target_index": {"type": ["integer", "null"], "minimum": 0},
                "split_ratio": {"type": ["number", "null"], "minimum": 0.1, "maximum": 0.9},
                "expected_projection_revision": {"type": "integer", "minimum": 0},
                "expected_layout_revision": {"type": "integer", "minimum": 0},
                "idempotency_key": {"type": "string", "minLength": 1},
                "requested_at": {"type": "string", "format": "date-time"},
            },
        },
        "LayoutMutationResult": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "command_id",
                "accepted",
                "projection_revision",
                "layout_revision",
                "projection_digest",
                "event_cursor",
            ],
            "properties": {
                "command_id": {"type": "string", "minLength": 1},
                "accepted": {"type": "boolean"},
                "projection_revision": {"type": "integer", "minimum": 0},
                "layout_revision": {"type": "integer", "minimum": 0},
                "projection_digest": {"$ref": "#/$defs/ProjectionDigest"},
                "event_cursor": {"type": "string", "minLength": 1},
                "error_ref": {"type": ["string", "null"]},
                "evidence_ref": {"type": ["string", "null"]},
                "receipt_ref": {"type": ["string", "null"]},
            },
        },
        "CandidateContribution": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "contribution_id",
                "kind",
                "semantic_binding_id",
                "renderer_binding_id",
                "priority",
                "applicable_profile_ids",
                "applicable_activity_mode_ids",
                "geometry",
                "canonical_content_refs",
                "required_capabilities",
                "required_permissions",
                "required_operations",
            ],
            "properties": {
                "contribution_id": {"$ref": "#/$defs/ContributionId"},
                "kind": {"$ref": "#/$defs/ContributionKind"},
                "semantic_binding_id": {"$ref": "#/$defs/SemanticBindingId"},
                "renderer_binding_id": {"$ref": "#/$defs/RendererBindingId"},
                "priority": {"type": "integer", "minimum": -100000, "maximum": 100000},
                "applicable_profile_ids": string_array(),
                "applicable_activity_mode_ids": string_array(),
                "active_work_surface_relationships": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"enum": ["focused", "open", "pinned", "contextual", "aggregate", "none"]},
                },
                "geometry": {"$ref": "#/$defs/GeometryPreference"},
                "canonical_content_refs": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/CanonicalRef"},
                },
                "required_capabilities": string_array(),
                "required_permissions": string_array(),
                "required_operations": string_array(),
                "preference_ref": {"type": ["string", "null"]},
                "configuration_ref": {"type": ["string", "null"]},
                "candidate_revision": {"type": "integer", "minimum": 0},
            },
        },
        "ContributionEligibilityContext": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "scope",
                "workspace_profile_id",
                "workspace_profile_revision",
                "activity_mode_id",
                "activity_mode_revision",
                "focused_work_surface_id",
                "canonical_read_model_revision",
                "available_operations",
                "capabilities",
                "permissions",
                "viewport",
                "project_constraint_refs",
                "user_preference_ref",
                "resolver_rule_revision",
            ],
            "properties": {
                "scope": {"$ref": "#/$defs/ExactScope"},
                "workspace_profile_id": {"type": "string", "minLength": 1},
                "workspace_profile_revision": {"type": "integer", "minimum": 0},
                "activity_mode_id": {"type": "string", "minLength": 1},
                "activity_mode_revision": {"type": "integer", "minimum": 0},
                "focused_work_surface_id": {"type": ["string", "null"]},
                "open_work_surface_ids": string_array(),
                "pinned_work_surface_ids": string_array(),
                "canonical_read_model_revision": {"type": "integer", "minimum": 0},
                "available_operations": string_array(),
                "capabilities": string_array(),
                "permissions": string_array(),
                "viewport": {"$ref": "#/$defs/ViewportDescriptor"},
                "project_constraint_refs": string_array(),
                "user_preference_ref": {"type": ["string", "null"]},
                "resolver_rule_revision": {"type": "string", "minLength": 1},
                "observed_at": {"type": "string", "format": "date-time"},
            },
        },
        "OmissionDiagnostic": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "contribution_id",
                "reason",
                "rule_revision",
                "projection_revision",
                "canonical_input_refs",
                "observed_at",
            ],
            "properties": {
                "contribution_id": {"$ref": "#/$defs/ContributionId"},
                "reason": {"$ref": "#/$defs/OmissionReason"},
                "rule_revision": {"type": "string", "minLength": 1},
                "projection_revision": {"type": "integer", "minimum": 0},
                "canonical_input_refs": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/CanonicalRef"},
                },
                "details_ref": {"type": ["string", "null"]},
                "observed_at": {"type": "string", "format": "date-time"},
            },
        },
        "EligibilityDecision": {
            "type": "object",
            "additionalProperties": False,
            "required": ["contribution_id", "outcome", "rule_revision", "projection_revision"],
            "properties": {
                "contribution_id": {"$ref": "#/$defs/ContributionId"},
                "outcome": {"enum": ["eligible", "omitted", "merged", "compacted", "suspended"]},
                "omission": {"anyOf": [{"$ref": "#/$defs/OmissionDiagnostic"}, {"type": "null"}]},
                "merged_into_contribution_id": {
                    "anyOf": [{"$ref": "#/$defs/ContributionId"}, {"type": "null"}]
                },
                "rule_revision": {"type": "string", "minLength": 1},
                "projection_revision": {"type": "integer", "minimum": 0},
                "evidence_refs": string_array(),
            },
            "allOf": [
                {
                    "if": {"properties": {"outcome": {"const": "omitted"}}},
                    "then": {
                        "required": ["omission"],
                        "properties": {"omission": {"$ref": "#/$defs/OmissionDiagnostic"}},
                    },
                },
                {
                    "if": {"properties": {"outcome": {"const": "merged"}}},
                    "then": {
                        "required": ["merged_into_contribution_id"],
                        "properties": {"merged_into_contribution_id": {"$ref": "#/$defs/ContributionId"}},
                    },
                },
            ],
        },
        "AuthorityDescriptor": {
            "type": "object",
            "additionalProperties": False,
            "required": ["canonical_owner", "mutation_owner", "scope", "read_only"],
            "properties": {
                "canonical_owner": {"type": "string", "minLength": 1},
                "mutation_owner": {"type": "string", "minLength": 1},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "read_only": {"type": "boolean"},
                "approval_required": {"type": "boolean"},
                "contention_ref": {"type": ["string", "null"]},
            },
        },
        "FreshnessDescriptor": {
            "type": "object",
            "additionalProperties": False,
            "required": ["status", "observed_at"],
            "properties": {
                "status": {"enum": ["current", "stale", "unknown", "not_applicable"]},
                "observed_at": {"type": "string", "format": "date-time"},
                "stale_reason": {"type": ["string", "null"]},
                "refresh_operation_id": {"type": ["string", "null"]},
            },
        },
        "AccessibilityDescriptor": {
            "type": "object",
            "additionalProperties": False,
            "required": ["label", "landmark_role", "focus_semantic_id"],
            "properties": {
                "label": {"type": "string", "minLength": 1},
                "description": {"type": ["string", "null"]},
                "landmark_role": {"type": "string", "minLength": 1},
                "focus_semantic_id": {"type": "string", "minLength": 1},
                "live_region": {"enum": ["off", "polite", "assertive"]},
                "keyboard_operation_ids": string_array(),
            },
        },
        "ResolvedContribution": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "contribution_id",
                "kind",
                "semantic_binding_id",
                "renderer_binding_id",
                "data_ref",
                "operation_ids",
                "authority",
                "freshness",
                "resolved_geometry",
                "accessibility",
                "contribution_revision",
            ],
            "properties": {
                "contribution_id": {"$ref": "#/$defs/ContributionId"},
                "kind": {"$ref": "#/$defs/ContributionKind"},
                "semantic_binding_id": {"$ref": "#/$defs/SemanticBindingId"},
                "renderer_binding_id": {"$ref": "#/$defs/RendererBindingId"},
                "data_ref": {"$ref": "#/$defs/CanonicalRef"},
                "operation_ids": string_array(),
                "authority": {"$ref": "#/$defs/AuthorityDescriptor"},
                "freshness": {"$ref": "#/$defs/FreshnessDescriptor"},
                "resolved_geometry": {"$ref": "#/$defs/GeometryPreference"},
                "accessibility": {"$ref": "#/$defs/AccessibilityDescriptor"},
                "contribution_revision": {"type": "integer", "minimum": 0},
                "evidence_refs": string_array(),
            },
        },
    }


def proof_definitions() -> dict[str, Any]:
    return {
        "ProjectionEventKind": {
            "type": "string",
            "enum": [
                "candidate_discovered",
                "contribution_eligible",
                "contribution_omitted",
                "contribution_merged",
                "projection_resolved",
                "layout_changed",
                "focus_changed",
                "profile_changed",
                "activity_mode_changed",
                "capability_changed",
                "host_launch_requested",
                "host_visible",
                "host_focused",
                "host_hidden",
                "host_closed",
                "host_reconnected",
                "host_failed",
                "draft_synchronized",
                "draft_conflict",
                "projection_suspended",
                "projection_rehydrated",
                "migration_started",
                "migration_completed",
                "migration_failed",
                "pi_turn_started",
                "pi_turn_completed",
                "pi_message_updated",
                "pi_tool_started",
                "pi_tool_completed",
            ],
        },
        "ProjectionLifecycleEvent": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "event_id",
                "event_kind",
                "scope",
                "projection_revision",
                "layout_revision",
                "event_cursor",
                "occurred_at",
                "payload_ref",
                "evidence_refs",
                "receipt_refs",
            ],
            "properties": {
                "event_id": {"type": "string", "pattern": r"^projection-event:[a-z0-9._:-]+$"},
                "event_kind": {"$ref": "#/$defs/ProjectionEventKind"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "contribution_id": {
                    "anyOf": [{"$ref": "#/$defs/ContributionId"}, {"type": "null"}]
                },
                "host_instance_id": {"type": ["string", "null"]},
                "projection_revision": {"type": "integer", "minimum": 0},
                "layout_revision": {"type": "integer", "minimum": 0},
                "event_cursor": {"type": "string", "minLength": 1},
                "causation_id": {"type": ["string", "null"]},
                "correlation_id": {"type": ["string", "null"]},
                "occurred_at": {"type": "string", "format": "date-time"},
                "payload_ref": {"type": "string", "minLength": 1},
                "evidence_refs": string_array(),
                "receipt_refs": string_array(),
            },
        },
        "RecompositionEvidence": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "evidence_id",
                "scope",
                "trigger",
                "input_projection_digest",
                "output_projection_digest",
                "rule_revision",
                "candidate_contribution_ids",
                "eligibility_decisions",
                "observed_at",
            ],
            "properties": {
                "evidence_id": {"type": "string", "pattern": r"^recomposition-evidence:[a-z0-9._:-]+$"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "trigger": {
                    "enum": [
                        "canonical_read_change",
                        "profile_change",
                        "activity_mode_change",
                        "focus_change",
                        "viewport_change",
                        "capability_change",
                        "preference_change",
                        "migration",
                        "explicit_resolve",
                    ]
                },
                "input_projection_digest": {
                    "anyOf": [{"$ref": "#/$defs/ProjectionDigest"}, {"type": "null"}]
                },
                "output_projection_digest": {"$ref": "#/$defs/ProjectionDigest"},
                "rule_revision": {"type": "string", "minLength": 1},
                "candidate_contribution_ids": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "eligibility_decisions": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/EligibilityDecision"},
                },
                "layout_decision_refs": string_array(),
                "diagnostic_refs": string_array(),
                "observed_at": {"type": "string", "format": "date-time"},
            },
        },
        "RecompositionReceipt": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "receipt_id",
                "scope",
                "accepted",
                "projection_revision",
                "layout_revision",
                "projection_digest",
                "event_cursor",
                "evidence_id",
                "idempotency_key",
                "issued_at",
            ],
            "properties": {
                "receipt_id": {"type": "string", "pattern": r"^recomposition-receipt:[a-z0-9._:-]+$"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "accepted": {"type": "boolean"},
                "projection_revision": {"type": "integer", "minimum": 0},
                "layout_revision": {"type": "integer", "minimum": 0},
                "projection_digest": {"$ref": "#/$defs/ProjectionDigest"},
                "event_cursor": {"type": "string", "minLength": 1},
                "evidence_id": {"type": "string", "minLength": 1},
                "idempotency_key": {"type": "string", "minLength": 1},
                "error_ref": {"type": ["string", "null"]},
                "issued_at": {"type": "string", "format": "date-time"},
            },
        },
        "ResponsiveEvaluationFixture": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "fixture_id",
                "viewport",
                "profile_id",
                "activity_mode_id",
                "candidate_contribution_ids",
                "expected_eligible_contribution_ids",
                "expected_omissions",
                "expected_layout_kinds",
                "minimum_primary_span",
            ],
            "properties": {
                "fixture_id": {"type": "string", "pattern": r"^responsive-fixture:[a-z0-9._:-]+$"},
                "viewport": {"$ref": "#/$defs/ViewportDescriptor"},
                "profile_id": {"type": "string", "minLength": 1},
                "activity_mode_id": {"type": "string", "minLength": 1},
                "candidate_contribution_ids": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "expected_eligible_contribution_ids": {
                    "type": "array",
                    "uniqueItems": True,
                    "items": {"$ref": "#/$defs/ContributionId"},
                },
                "expected_omissions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "additionalProperties": False,
                        "required": ["contribution_id", "reason"],
                        "properties": {
                            "contribution_id": {"$ref": "#/$defs/ContributionId"},
                            "reason": {"$ref": "#/$defs/OmissionReason"},
                        },
                    },
                },
                "expected_layout_kinds": {
                    "type": "array",
                    "minItems": 1,
                    "uniqueItems": True,
                    "items": {"enum": ["single", "split", "stack", "grid", "tabs", "inspector"]},
                },
                "minimum_primary_span": {"type": "integer", "minimum": 1, "maximum": 12},
                "maximum_visible_contributions": {"type": ["integer", "null"], "minimum": 1},
                "must_preserve_focus": {"type": "boolean"},
            },
        },
        "LegacyLayoutMapping": {
            "type": "object",
            "additionalProperties": False,
            "required": ["legacy_ref", "target_contribution_id", "mapping_status"],
            "properties": {
                "legacy_ref": {"type": "string", "minLength": 1},
                "target_contribution_id": {"$ref": "#/$defs/ContributionId"},
                "mapping_status": {"enum": ["mapped", "omitted", "manual_review"]},
                "diagnostic_ref": {"type": ["string", "null"]},
            },
        },
        "LegacyLayoutMigrationEnvelope": {
            "type": "object",
            "additionalProperties": False,
            "required": [
                "migration_id",
                "scope",
                "source_kind",
                "source_revision",
                "source_digest",
                "target_profile_id",
                "target_activity_mode_id",
                "mappings",
                "preserved_draft_ref",
                "status",
                "idempotency_key",
                "created_at",
            ],
            "properties": {
                "migration_id": {"type": "string", "pattern": r"^layout-migration:[a-z0-9._:-]+$"},
                "scope": {"$ref": "#/$defs/ExactScope"},
                "source_kind": {"enum": ["terminal_local", "process_local", "legacy_canvas", "imported_snapshot"]},
                "source_revision": {"type": ["integer", "string"]},
                "source_digest": {"type": "string", "pattern": r"^sha256:[a-f0-9]{64}$"},
                "target_profile_id": {"type": "string", "minLength": 1},
                "target_activity_mode_id": {"type": "string", "minLength": 1},
                "mappings": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/LegacyLayoutMapping"},
                },
                "preserved_draft_ref": {"type": ["string", "null"]},
                "target_layout_memory_ref": {"type": ["string", "null"]},
                "status": {"enum": ["pending", "validated", "applied", "rejected", "rolled_back"]},
                "warning_refs": string_array(),
                "error_ref": {"type": ["string", "null"]},
                "idempotency_key": {"type": "string", "minLength": 1},
                "created_at": {"type": "string", "format": "date-time"},
            },
        },
    }


def _nullable_ref(name: str) -> dict[str, Any]:
    return {"anyOf": [{"$ref": f"#/$defs/{name}"}, {"type": "null"}]}


def canonicalize_identity_contract(definitions_by_name: dict[str, Any]) -> dict[str, Any]:
    """Replace the pre-Spec-158 flat scope shape in every canonical DTO.

    The old ExactScope object is intentionally not emitted.  A legacy caller may
    still use LegacyExactScopeCompatibilityInput, but it has no canonical DTO
    references and cannot grant authority without an explicit Workstream mapping.
    """
    definitions_by_name.pop("ExactScope", None)
    for schema_name, schema in definitions_by_name.items():
        if schema_name == "LegacyExactScopeCompatibilityInput":
            continue
        properties = schema.get("properties")
        if not isinstance(properties, dict):
            continue
        required = schema.setdefault("required", [])
        if "scope" in properties and properties["scope"].get("$ref", "").endswith("/ExactScope"):
            properties.pop("scope")
            properties.update(
                {
                    "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                    "continuity_id": _nullable_ref("ContinuityId"),
                    "attachment": _nullable_ref("AttachmentKey"),
                    "workspace_binding_id": _nullable_ref("WorkspaceBindingId"),
                    "runtime_object": _nullable_ref("RuntimeObjectRef"),
                    "work_surface_id": _nullable_ref("WorkSurfaceId"),
                }
            )
            schema["required"] = ["workstream" if item == "scope" else item for item in required]
            required = schema["required"]
        if "attachment_id" in properties and properties["attachment_id"].get("type") == "string":
            properties.pop("attachment_id")
            properties["attachment"] = {"$ref": "#/$defs/AttachmentKey"}
            schema["required"] = ["attachment" if item == "attachment_id" else item for item in required]
        if "focused_work_surface_id" in properties:
            properties["focused_work_surface_id"] = _nullable_ref("WorkSurfaceId")
        for field in ("open_work_surface_ids", "pinned_work_surface_ids"):
            if field in properties:
                properties[field] = {"type": "array", "uniqueItems": True, "items": {"$ref": "#/$defs/WorkSurfaceId"}}
        for field in ("target_work_surface_id", "secondary_work_surface_id"):
            if field in properties:
                properties[field] = _nullable_ref("WorkSurfaceId")
    layout_result = definitions_by_name.get("LayoutMutationResult")
    if isinstance(layout_result, dict):
        properties = layout_result.setdefault("properties", {})
        properties.update(
            {
                "workstream": {"$ref": "#/$defs/WorkstreamKey"},
                "continuity_id": _nullable_ref("ContinuityId"),
                "attachment": _nullable_ref("AttachmentKey"),
                "workspace_binding_id": _nullable_ref("WorkspaceBindingId"),
                "runtime_object": _nullable_ref("RuntimeObjectRef"),
                "work_surface_id": _nullable_ref("WorkSurfaceId"),
            }
        )
        required = layout_result.setdefault("required", [])
        if "workstream" not in required:
            required.insert(0, "workstream")
    return definitions_by_name


def bundle() -> dict[str, Any]:
    definitions_by_name = canonicalize_identity_contract({**definitions(), **proof_definitions()})
    return {
        "$schema": DIALECT,
        "$id": f"{BASE}/composition-bundle.v1.schema.json",
        "title": "Focusa Spec 135 Mission Canvas composition contract bundle",
        "description": "Portable Workstream identity, contribution eligibility, omission, and resolved contribution contracts.",
        "$defs": definitions_by_name,
    }


ROOTS = {
    "contribution-id.schema.json": "ContributionId",
    "candidate-contribution.schema.json": "CandidateContribution",
    "eligibility-context.schema.json": "ContributionEligibilityContext",
    "eligibility-decision.schema.json": "EligibilityDecision",
    "omission-diagnostic.schema.json": "OmissionDiagnostic",
    "resolved-contribution.schema.json": "ResolvedContribution",
    "layout-node.schema.json": "LayoutNode",
    "resolved-workspace-projection.schema.json": "ResolvedWorkspaceProjection",
    "projection-digest.schema.json": "ProjectionDigest",
    "workspace-profile.schema.json": "WorkspaceProfile",
    "activity-mode.schema.json": "ActivityMode",
    "registry-entry.schema.json": "RegistryEntry",
    "profile-layout-memory.schema.json": "ProfileLayoutMemory",
    "canvas-draft.schema.json": "CanvasDraftState",
    "host-renderer-resolution.schema.json": "HostRendererResolution",
    "host-lifecycle.schema.json": "HostLifecycleState",
    "capability-projection.schema.json": "CapabilityProjection",
    "layout-mutation.schema.json": "LayoutMutationCommand",
    "layout-mutation-result.schema.json": "LayoutMutationResult",
    "projection-event.schema.json": "ProjectionLifecycleEvent",
    "recomposition-evidence.schema.json": "RecompositionEvidence",
    "recomposition-receipt.schema.json": "RecompositionReceipt",
    "responsive-evaluation-fixture.schema.json": "ResponsiveEvaluationFixture",
    "legacy-layout-migration.schema.json": "LegacyLayoutMigrationEnvelope",
}


def root_schema(filename: str, definition: str) -> dict[str, Any]:
    return {
        "$schema": DIALECT,
        "$id": f"{BASE}/{filename}",
        "title": f"Focusa Spec 135 {definition}",
        "$ref": f"composition-bundle.v1.schema.json#/$defs/{definition}",
    }


def render(value: dict[str, Any]) -> str:
    return json.dumps(value, indent=2, ensure_ascii=False, sort_keys=True) + "\n"


def outputs() -> dict[Path, str]:
    result = {OUT / "composition-bundle.v1.schema.json": render(bundle())}
    result.update({OUT / filename: render(root_schema(filename, definition)) for filename, definition in ROOTS.items()})
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = outputs()
    if args.check:
        for path, expected in generated.items():
            assert path.exists(), f"missing generated schema: {path}"
            assert path.read_text() == expected, f"stale generated schema: {path}"
        print(f"Spec 135 Mission Canvas contribution schemas: PASS ({len(generated)} files)")
        return
    OUT.mkdir(parents=True, exist_ok=True)
    for path, text in generated.items():
        path.write_text(text)
        print(f"Generated {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
