#!/usr/bin/env python3
"""Generate Desktop-only projection fixtures from the canonical Spec 135 schema fixture."""
from __future__ import annotations

import contextlib
import copy
import io
import json
import runpy
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "tests/spec135_resolved_projection_contract_test.py"
OUT = ROOT / "apps/desktop/tests/fixtures/mission-canvas"

with contextlib.redirect_stdout(io.StringIO()):
    namespace = runpy.run_path(str(SOURCE))

base = namespace["projection"]
validate = namespace["validator"]("ResolvedWorkspaceProjection").validate


def contribution(identifier: str, kind: str = "focused_work_surface") -> dict:
    item = copy.deepcopy(namespace["valid_resolved"]())
    item["contribution_id"] = identifier
    item["kind"] = kind
    item["semantic_binding_id"] = f"semantic:{identifier.removeprefix('contribution:')}"
    item["renderer_binding_id"] = f"renderer:{identifier.removeprefix('contribution:')}@v1"
    item["accessibility"]["label"] = identifier.removeprefix("contribution:").replace("-", " ").title()
    return item


def projection(name: str, contributions: list[dict], layout: dict, omissions: list[dict] | None = None) -> dict:
    value = copy.deepcopy(base)
    value["candidate_contribution_ids"] = [item["contribution_id"] for item in contributions]
    value["eligible_contributions"] = contributions
    value["omission_diagnostics"] = omissions or []
    value["layout_tree"] = layout
    value["operation_bindings"] = []
    value["projection_digest"] = "sha256:" + "0" * 64
    value["evidence_refs"] = [f"fixture:{name}"]
    validate(value)
    return value


def single(node_id: str, identifier: str) -> dict:
    return {"node_id": node_id, "kind": "single", "contribution_id": identifier}


primary = contribution("contribution:pi-session")
secondary = contribution("contribution:focusa-inspector", "inspector")
work_rail = contribution("contribution:work-rail", "work_rail")
work_rail["data_ref"] = {"kind": "work_rail", "ref": "work-rail:project", "revision": 4, "freshness": "current"}
work_rail["operation_ids"] = []
work_rail["accessibility"]["label"] = "Focusa Work Rail"
work_rail["accessibility"]["description"] = "Canonical project work for the focused Work Surface"

queue = contribution("contribution:steering-queue", "steering_queue")
queue["data_ref"] = {"kind": "steering_queue", "ref": "queue:steering", "revision": 1, "freshness": "current"}
queue["operation_ids"] = []
queue["accessibility"]["label"] = "Steering Queue"
queue["accessibility"]["description"] = "Pending steering requests for the focused Work Surface"

follow_up = contribution("contribution:follow-up-queue", "follow_up_queue")
follow_up["data_ref"] = {"kind": "follow_up_queue", "ref": "queue:follow-up", "revision": 1, "freshness": "current"}
follow_up["operation_ids"] = []
follow_up["accessibility"]["label"] = "Follow-up Queue"
follow_up["accessibility"]["description"] = "Accepted follow-up work routed to an exact recipient"

prompt = contribution("contribution:prompt-editor", "prompt_editor")
prompt["data_ref"] = {"kind": "canvas_draft", "ref": "draft:prompt-preview", "revision": 1, "freshness": "current"}
prompt["operation_ids"] = ["focusa.agent_execution.prompt"]
prompt["accessibility"]["label"] = "Prompt Editor"
prompt["accessibility"]["description"] = "Governed draft and prompt routing for the focused Work Surface"

populated = copy.deepcopy(base)
validate(populated)

empty_diagnostic = copy.deepcopy(namespace["valid_diagnostic"]())
empty_diagnostic["contribution_id"] = "contribution:empty-work-rail"
empty_optionals = projection(
    "empty-optionals",
    [primary],
    single("layout:primary", primary["contribution_id"]),
    [empty_diagnostic],
)
empty_optionals["candidate_contribution_ids"].append(empty_diagnostic["contribution_id"])
validate(empty_optionals)

one_queue = projection(
    "one-queue",
    [primary, queue, prompt],
    {
        "node_id": "layout:one-queue",
        "kind": "stack",
        "gap_token": "cluster",
        "children": [
            single("layout:primary", primary["contribution_id"]),
            single("layout:queue", queue["contribution_id"]),
            single("layout:prompt", prompt["contribution_id"]),
        ],
    },
)
one_queue["operation_bindings"] = [{
    "operation_id": "focusa.agent_execution.prompt",
    "target_contribution_id": prompt["contribution_id"],
    "enabled": True,
    "disabled_reason_ref": None,
    "confirmation": "none",
    "authority_ref": "authority:fixture:prompt-preview",
}]
validate(one_queue)

two_queue = projection(
    "two-queue",
    [primary, secondary, work_rail, queue, follow_up, prompt],
    {
        "node_id": "layout:two-queue",
        "kind": "stack",
        "gap_token": "cluster",
        "children": [
            {
                "node_id": "layout:work-region",
                "kind": "split",
                "orientation": "horizontal",
                "ratio": 0.68,
                "children": [
                    single("layout:primary", primary["contribution_id"]),
                    single("layout:inspector", secondary["contribution_id"]),
                ],
            },
            single("layout:work-rail", work_rail["contribution_id"]),
            {
                "node_id": "layout:queue-region",
                "kind": "split",
                "orientation": "horizontal",
                "ratio": 0.5,
                "children": [
                    single("layout:steering", queue["contribution_id"]),
                    single("layout:follow-up", follow_up["contribution_id"]),
                ],
            },
            single("layout:prompt", prompt["contribution_id"]),
        ],
    },
)
two_queue["operation_bindings"] = copy.deepcopy(one_queue["operation_bindings"])
validate(two_queue)

zero_queue = projection(
    "zero-queue",
    [primary, prompt],
    {
        "node_id": "layout:zero-queue",
        "kind": "stack",
        "gap_token": "cluster",
        "children": [
            single("layout:primary", primary["contribution_id"]),
            single("layout:prompt", prompt["contribution_id"]),
        ],
    },
)
zero_queue["operation_bindings"] = copy.deepcopy(one_queue["operation_bindings"])
validate(zero_queue)

variants = {
    "single": projection("layout-single", [primary], single("layout:single", primary["contribution_id"])),
    "split": projection(
        "layout-split",
        [primary, secondary],
        {
            "node_id": "layout:split",
            "kind": "split",
            "orientation": "horizontal",
            "ratio": 0.68,
            "children": [
                single("layout:split-primary", primary["contribution_id"]),
                single("layout:split-secondary", secondary["contribution_id"]),
            ],
        },
    ),
    "stack": projection(
        "layout-stack",
        [primary, secondary],
        {
            "node_id": "layout:stack",
            "kind": "stack",
            "gap_token": "cluster",
            "children": [
                single("layout:stack-primary", primary["contribution_id"]),
                single("layout:stack-secondary", secondary["contribution_id"]),
            ],
        },
    ),
    "grid": projection(
        "layout-grid",
        [primary, secondary],
        {
            "node_id": "layout:grid",
            "kind": "grid",
            "columns": 2,
            "gap_token": "cluster",
            "children": [
                single("layout:grid-primary", primary["contribution_id"]),
                single("layout:grid-secondary", secondary["contribution_id"]),
            ],
        },
    ),
    "tabs": projection(
        "layout-tabs",
        [primary, secondary],
        {
            "node_id": "layout:tabs",
            "kind": "tabs",
            "active_contribution_id": primary["contribution_id"],
            "contribution_ids": [primary["contribution_id"], secondary["contribution_id"]],
        },
    ),
    "inspector": projection(
        "layout-inspector",
        [primary, secondary],
        {
            "node_id": "layout:inspector",
            "kind": "inspector",
            "primary": single("layout:inspector-primary", primary["contribution_id"]),
            "inspector_contribution_ids": [secondary["contribution_id"]],
            "side": "end",
            "span": 4,
        },
    ),
}

OUT.mkdir(parents=True, exist_ok=True)
outputs = {
    "populated-projection.json": populated,
    "empty-optionals-projection.json": empty_optionals,
    "one-queue-projection.json": one_queue,
    "two-queue-projection.json": two_queue,
    "zero-queue-projection.json": zero_queue,
    "layout-variants.json": variants,
}
for filename, value in outputs.items():
    (OUT / filename).write_text(json.dumps(value, indent=2) + "\n")

print(f"Mission Canvas Desktop fixtures: PASS ({len(outputs)} files, {len(variants)} layout variants)")
