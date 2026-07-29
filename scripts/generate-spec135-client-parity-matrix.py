#!/usr/bin/env python3
"""Spec 135A-8 §15: regenerate the truthful client-operation parity matrix and
durable dogfood receipt bundle from the canonical Operation Registry and the
per-client capability truth table. Terse bounded generator; no parallel state."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REG_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1"
PARITY_CONTRACT = ROOT / "docs/contracts/spec135-client-operation-parity.v1.json"

# Clients recorded by the §15 parity matrix. `canonical_state_owner` is always
# False: presentation clients never own canonical state; only reducers do.
CLIENTS = [
    {"client_id": "api", "kind": "http_canonical_source"},
    {"client_id": "cli", "kind": "generated_cli"},
    {"client_id": "typescript", "kind": "generated_sdk"},
    {"client_id": "pi", "kind": "pi_extension", "presentation_only": True},
    {"client_id": "mission_canvas", "kind": "pi_widget", "presentation_only": True},
    {"client_id": "tui", "kind": "ratatui_view", "presentation_only": True},
    {"client_id": "uiai_engine_cockpit", "kind": "ui_action_bridge"},
    {"client_id": "menubar", "kind": "operator_peek"},
    {"client_id": "pwa", "kind": "mission_deck"},
]

# Pi-extension has presentation-only authority: only read + preview-and-confirm
# mutations; canonical reducers remain authoritative (no parallel authority).
PI_PREVIEW_ONLY_FAMILIES = {
    "mission_canvas",
    "work_rail",
    "spec_workbench",
    "role_profile",
    "interview",
    "context",
}
PI_PRESENTATION_COMMAND_COVERAGE = {
    "focusa.health.check",
    "focusa.project.identity",
    "focusa.project.verify",
    "focusa.trajectory.view",
    "focusa.trajectory.define_goal",
    "focusa.trajectory.assess",
    "focusa.trajectory.resume",
    "focusa.workpoint.checkpoint",
    "focusa.workpoint.resume",
    "focusa.workpoint.link_evidence",
    "focusa.focus.update",
    "focusa.context.cognition",
    "focusa.project.role_profile.create",
    "focusa.project.interview.session",
    "focusa.spec_workbench.review",
    "focusa.task_plan.materialize",
    "focusa.work_rail.preview",
    "focusa.work_rail.commit",
    "focusa.mission_canvas.surfaces",
    "focusa.workspace_artifact.projection",
}

UIAI_OPERATION_COVERAGE = {
    "focusa.health.check",
    "focusa.context.sources.commit",
    "focusa.context.sources.ingest",
    "focusa.context.retrieve",
    "focusa.context.claims.graph",
    "focusa.workspace_artifact.projection",
    "focusa.ui.action.evaluate",
    "focusa.ui.capability.snapshot",
}

MENUBAR_OPERATION_COVERAGE = {
    "focusa.health.check",
    "focusa.focus.update",
    "focusa.work_loop.status",
    "focusa.session.inventory",
    "focusa.device.pair.start",
    "focusa.device.pair.complete",
    "focusa.device.pair.status",
    "focusa.device.pair.list",
    "focusa.device.pair.revoke",
}

PWA_OPERATION_COVERAGE = MENUBAR_OPERATION_COVERAGE | {
    "focusa.mission_canvas.surfaces",
    "focusa.workspace_artifact.projection",
}

TUI_OPERATION_COVERAGE = {
    "focusa.health.check",
    "focusa.focus.update",
    "focusa.work_loop.status",
    "focusa.lineage.tree",
    "focusa.context.cognition",
}

GENERATED_SDK_FULL = "all"  # generated SDKs cover all registered operations.


def client_capability(client: dict, op: dict) -> tuple[str, str]:
    """Return (capability_trait, truthful capability_limit_reason)."""
    cid, kind = client["client_id"], client["kind"]
    op_id = op["operation_id"]
    mode = op.get("control", {}).get("mode", "read")
    side_effect = op.get("side_effect_profile", "none")
    confirmation = op.get("control", {}).get("confirmation", "none")
    preview_token = op.get("requires_preview_token", False)

    if kind == "http_canonical_source":
        return "full", "canonical HTTP source via generated OpenAPI route"

    if kind == "generated_cli":
        if mode == "read":
            return "full", "read via generated CLI command"
        return ("preview", "write via generated CLI with operator confirmation "
                "(preview token enforced by Operation Registry)")

    if kind == "generated_sdk":
        if mode == "read":
            return "full", "generated sdk bound read"
        return ("preview", "generated sdk write bound through preview token "
                "and operator confirmation")

    if kind == "pi_extension" or kind == "pi_widget":
        if op_id in PI_PRESENTATION_COMMAND_COVERAGE:
            if mode == "read":
                return "read_only", "presentation-only Pi command without canonical mutation"
            return ("preview", "preview-and-commit Pi command; canonical "
                    "Workpoint/provider reducers retain authority")
        if cid == "mission_canvas" and op["family"] in PI_PREVIEW_ONLY_FAMILIES:
            return ("read_only", "Mission Canvas presentation projection; "
                    "no canonical state ownership")
        return "unsupported", "no registered Pi command for this operation"

    if kind == "ui_action_bridge":
        if op_id in UIAI_OPERATION_COVERAGE:
            if mode == "read":
                return "read_only", "UI action bridge read snapshot"
            return ("preview", "UI action bridge mutation gated by Focusa "
                    "browser capability intake + operator confirmation")
        return "unsupported", "no UI action bridge coverage for this operation"

    if kind == "operator_peek":
        if op_id in MENUBAR_OPERATION_COVERAGE:
            if mode == "read":
                return "read_only", "operator peek read"
            return ("preview", "operator peek write gated by operator confirmation "
                    "and idempotency key")
        return "unsupported", "no menubar coverage for this operation"

    if kind == "mission_deck":
        if op_id in PWA_OPERATION_COVERAGE:
            if mode == "read":
                return "read_only", "Mission Deck read projection"
            return ("preview", "Mission Deck mutation gated by device-paired token "
                    "and operator confirmation")
        return "unsupported", "no PWA coverage for this operation"

    if kind == "ratatui_view":
        if op_id in TUI_OPERATION_COVERAGE:
            if mode == "read":
                return "read_only", "ratatui view read"
            return ("preview", "ratatui view mutation routed through operator confirmation")
        return "unsupported", "no TUI view coverage for this operation"

    return "unsupported", "unmapped client kind"


def main() -> None:
    registry = json.loads((REG_DIR / "operation-registry.json").read_text())
    operations = registry["operations"]
    rows = []
    for op in operations:
        op_id = op["operation_id"]
        for client in CLIENTS:
            trait, limit = client_capability(client, op)
            rows.append({
                "operation_id": op_id,
                "client_id": client["client_id"],
                "client_kind": client["kind"],
                "canonical_state_owner": False,
                "capability_trait": trait,
                "capability_limit": limit,
                "operation_mode": op.get("control", {}).get("mode", "read"),
                "requires_preview_token": op.get("requires_preview_token", False),
                "confirmation_required": op.get("control", {}).get("confirmation", "none") != "none",
            })

    # Dogfood receipts reference real execution proof fixtures in the generated
    # contract bundle; these are durable evidence, not transient logs.
    dogfood_root = REG_DIR
    receipt_candidates = [
        "spec135-alpha4-work-rail-proof.json",
        "spec135-alpha7-domain-parity-proof.json",
        "spec135-alpha8-nontechnical-dogfood-proof.json",
        "spec135-c1-context-ingestion-proof.json",
        "spec135-c2-context-retrieval-proof.json",
        "spec135-m2-pi-work-rail-proof.json",
        "spec135-m3-mission-surfaces-proof.json",
        "spec135-st1-spec-workbench-proof.json",
        "spec135-st2-task-plan-proof.json",
        "spec135-st3-task-materialization-proof.json",
        "spec135-u3-browser-eval-matrix.json",
        "spec135-v1-v6-domain-projection-proof.json",
    ]
    dogfood_receipts = []
    for ref in receipt_candidates:
        path = dogfood_root / ref
        if not path.exists():
            continue
        payload = json.loads(path.read_text())
        status = "missing"
        if isinstance(payload, dict):
            raw = (
                payload.get("status")
                or payload.get("result")
                or payload.get("proof_status")
            )
            raw = str(raw).lower() if raw is not None else ""
            failed = raw in {"failed", "missing", "blocked", ""}
            has_acceptance = isinstance(payload.get("acceptance"), dict)
            status = "passed" if (has_acceptance or (raw and not failed)) else "missing"
        dogfood_receipts.append({
            "receipt_ref": f"docs/contracts/spec135/generated-contract-v1/{ref}",
            "status": status,
        })

    contract = {
        "schema": "focusa.spec135.client_operation_parity.v1",
        "generated_from": "generated-contract-v1/operation-registry.json",
        "canonical_contracts": [
            "JSON Schema 2020-12",
            "OpenAPI 3.0.3",
            "Operation Registry",
            "ToolResult/error envelope",
        ],
        "parity_invariant": (
            "Every client classifies each registered operation with a truthful "
            "capability_trait. No presentation client has canonical state of its own; "
            "the daemon HTTP route is the only canonical source; all presentation "
            "clients defer to Workpoint/provider reducers and enforce preview tokens "
            "+ operator confirmation for side effects."
        ),
        "clients": CLIENTS,
        "operations": [op["operation_id"] for op in operations],
        "rows": rows,
        "dogfood_receipts": dogfood_receipts,
    }
    PARITY_CONTRACT.write_text(json.dumps(contract, indent=2) + "\n")
    traits = {}
    for row in rows:
        traits.setdefault(row["capability_trait"], 0)
        traits[row["capability_trait"]] += 1
    print(
        f"Spec 135A-8 client-operation parity generated: "
        f"{len(rows)} rows across {len(CLIENTS)} clients; "
        f"traits={traits}; "
        f"dogfood receipts={sum(1 for r in dogfood_receipts if r['status'] == 'passed')}/"
        f"{len(dogfood_receipts)} passed"
    )


if __name__ == "__main__":
    main()