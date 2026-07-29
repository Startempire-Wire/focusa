#!/usr/bin/env python3
"""Spec 135C-2: generate the durable UIAI research bridge contract proving
that browser research intake produces cited durable artifacts without merging
browser/session origins."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
CONTRACT_PATH = ROOT / "docs/contracts/spec135-research-bridge-origin-isolation.v1.json"
SCHEMA_PATH = SCHEMA_DIR / "focusa.research_diagnostics_packet.v1.json"

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.research_diagnostics_packet.v1.json",
    "title": "Focusa Research Diagnostics Packet v1",
    "description": "Durable packet produced by the UIAI research bridge. Browser/session origins are never merged; each cited artifact retains its exact session_origin, browser_context_ref, and attachment_id.",
    "type": "object",
    "required": [
        "schema", "packet_id", "goal", "project_root", "continuity_id",
        "attachment_id", "session_origin", "browser_context_ref",
        "artifacts", "evidence_refs", "cleanup_posture", "origin_merge_prohibited",
    ],
    "properties": {
        "schema": {"const": "focusa.research_diagnostics_packet.v1"},
        "packet_id": {"type": "string", "minLength": 1},
        "goal": {"type": "string", "minLength": 1},
        "mode": {"type": "string", "enum": ["research", "diagnose", "proof"]},
        "project_root": {"type": "string", "minLength": 1},
        "continuity_id": {"type": "string", "minLength": 1},
        "attachment_id": {"type": "string", "minLength": 1},
        "session_origin": {"type": "string", "minLength": 1},
        "browser_context_ref": {"type": "string", "minLength": 1},
        "source_origins": {
            "type": "array",
            "minItems": 1,
            "items": {
                "type": "object",
                "required": ["source_id", "url", "session_origin", "browser_context_ref"],
                "properties": {
                    "source_id": {"type": "string", "minLength": 1},
                    "url": {"type": "string", "minLength": 1},
                    "session_origin": {"type": "string", "minLength": 1},
                    "browser_context_ref": {"type": "string", "minLength": 1},
                    "markdown_ref": {"type": "string"},
                    "citation_ref": {"type": "string"},
                    "authoritative": {"type": "boolean"},
                },
            },
        },
        "artifacts": {
            "type": "array",
            "items": {
                "type": "object",
                "required": ["artifact_id", "artifact_kind", "session_origin", "browser_context_ref"],
                "properties": {
                    "artifact_id": {"type": "string", "minLength": 1},
                    "artifact_kind": {"type": "string", "minLength": 1},
                    "title": {"type": "string"},
                    "session_origin": {"type": "string", "minLength": 1},
                    "browser_context_ref": {"type": "string", "minLength": 1},
                    "evidence_ref": {"type": "string"},
                    "cleanup_recommended": {"type": "boolean"},
                },
            },
        },
        "evidence_refs": {"type": "array", "items": {"type": "string"}},
        "cleanup_posture": {"type": "string", "enum": ["keep", "close_session", "already_closed"]},
        "origin_merge_prohibited": {"const": True},
        "recommended_next_action": {"type": "string"},
        "cleanup_session_id": {"type": "string"},
    },
    "additionalProperties": False,
}

CONTRACT = {
    "schema": "focusa.spec135.research_bridge_origin_isolation.v1",
    "spec_ref": "docs/135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md",
    "acceptance_criteria": "Research workflow produces cited durable artifacts without merging browser/session origins.",
    "authority_split": {
        "uiai_engine": "Browser/search/session/context/target/media/diagnostics execution and stable artifacts",
        "focusa": "ProjectIdentity, Workstream/Attachment identity, Workpoint, Evidence, Context Authority, artifact linkage",
        "mission_canvas": "Tool selection, artifact viewing, explicit steering, bounded session inventory",
    },
    "origin_isolation_invariants": [
        "Each cited artifact retains its exact session_origin and browser_context_ref",
        "Two Work Surfaces must not share a browser context without an explicit action and visible badge",
        "Separate targets inside one context do not constitute container isolation",
        "Isolation may not be inferred from separate tabs alone",
        "Research packet fields must remain bounded and secret-safe",
    ],
    "packet_capture_types": [
        "search", "source_markdown", "browser_read", "snapshot",
        "screenshot", "diagnostics", "error", "share_fpv",
    ],
    "cleanup_postures": ["keep", "close_session", "already_closed"],
    "schema_ref": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.research_diagnostics_packet.v1.json",
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    SCHEMA_PATH.write_text(json.dumps(SCHEMA, indent=2) + "\n")
    CONTRACT_PATH.write_text(json.dumps(CONTRACT, indent=2) + "\n")
    print("Spec 135C-2 research bridge origin-isolation contract generated")
    print(f"  schema: docs/contracts/spec135/generated-contract-v1/json-schema/focusa.research_diagnostics_packet.v1.json")
    print(f"  contract: docs/contracts/spec135-research-bridge-origin-isolation.v1.json")


if __name__ == "__main__":
    main()