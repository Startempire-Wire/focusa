#!/usr/bin/env python3
"""Generate weak-model-safe execution packets for every Spec 135 Desktop task."""
from __future__ import annotations

import argparse
import hashlib
import json
import shlex
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
GRAPH_PATH = ROOT / "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-executable-callgraph.yaml"
PARENT_PATH = ROOT / "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-desktop-callgraph.yaml"
REGISTRY_PATH = ROOT / "docs/contracts/spec135/mission-canvas-v1/operation-registry.json"
OUTPUT_PATH = ROOT / "docs/contracts/spec135-svelte-task-execution-index.v1.json"
PACKET_DIR = ROOT / "docs/contracts/spec135-svelte-task-packets"
CARDINAL_REF = "CARDINAL-135-SVELTE-001"
PACKET_REF_PREFIX = "docs/contracts/spec135-svelte-task-packets/"
PATH_PREFIXES = ("apps/", "crates/", "docs/", "packages/", "scripts/", "tests/", "/tmp/")
SOURCE_REF_MIGRATIONS = {
    "/tmp/focusa-spec158-transition/docs__spec158__01-identity-ownership-and-reducer.md": "docs/spec158/01-identity-ownership-and-reducer.md",
    "/tmp/focusa-spec158-transition/docs__spec158__02-persistence-migration-and-quarantine.md": "docs/spec158/02-persistence-migration-and-quarantine.md",
    "/tmp/focusa-spec158-transition/docs__spec158__03-client-runtime-and-desktop-contracts.md": "docs/spec158/03-client-runtime-and-desktop-contracts.md",
    "/tmp/focusa-spec158-transition/docs__transitions__FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md": "docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md",
}


def canonical_source_ref(value: str) -> str:
    return SOURCE_REF_MIGRATIONS.get(value, value)


def load_yaml(path: Path) -> dict[str, Any]:
    return yaml.safe_load(path.read_text())


def exact_paths(values: list[str]) -> tuple[list[str], list[str]]:
    exact, unresolved = [], []
    for value in values:
        (exact if value.startswith(PATH_PREFIXES) else unresolved).append(value)
    return exact, unresolved


def shell_checks(value: Any) -> list[str]:
    values = value if isinstance(value, list) else [value]
    commands = []
    for item in values:
        item = str(item)
        if item.startswith(("python3 ", "npm ", "node ", "cargo ", "bash ", "pnpm ")):
            commands.append(item)
    return commands


def task_packet(
    task: dict[str, Any],
    task_class: str,
    defaults: dict[str, Any],
    parents: dict[str, dict[str, Any]],
    operations: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    is_operation = task_class == "operation"
    parent = parents.get(task.get("parent", ""), {})
    descriptor = operations.get(task.get("operation", ""), {})
    reads = list(task.get("read", defaults.get("read", [])))
    target_values = list(task.get("targets", []))
    if is_operation:
        target_values = [
            "crates/focusa-api/src/routes/mission_canvas.rs",
            "crates/focusa-core/src/mission_canvas/",
            "docs/contracts/spec135/mission-canvas-v1/operation-registry.json",
            "docs/contracts/spec135/mission-canvas-v1/openapi-3.0.3.json",
            "docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated.ts",
            "docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated.ts",
            "apps/desktop/src/lib/mission-canvas/http-transport.ts",
        ]
    exact_targets, unresolved_targets = exact_paths([str(v) for v in target_values])
    checks = shell_checks(task.get("check", defaults.get("check", [])))
    if is_operation:
        checks = [
            "python3 tests/spec135_mission_canvas_generated_parity_test.py",
            f"python3 tests/spec135_mission_canvas_operation_contract_test.py --operation {task['operation']}",
            f"node apps/desktop/tests/operation-consumer-runtime.mjs --operation {task['operation']} --client {task['client']}",
        ]
    for command in checks:
        for token in shlex.split(command):
            if token.startswith(PATH_PREFIXES) and token not in exact_targets:
                exact_targets.append(token)
    dependencies = list(task.get("depends_on", []))
    status = task.get("status", defaults.get("status", "blocked_until_dependencies_complete"))
    source_pi = [path for path in reads if path.startswith("apps/pi-extension/")]
    operation_id = task.get("operation")
    client_method = task.get("client")
    calls = list(task.get("calls", defaults.get("required_call_stack", [])))
    if client_method:
        calls = [client_method, operation_id, *defaults.get("required_call_stack", [])]
    input_text = task.get("input") or (
        f"Generated request schema {descriptor.get('request_schema_ref')} with exact scope {descriptor.get('scope_required', [])}."
    )
    output_text = task.get("output") or (
        f"Validated {descriptor.get('response_schema_ref')} returned through generated method {client_method}."
    )
    done = task.get("done", defaults.get("completion_rule"))
    parent_acceptance = parent.get("acceptance", [])
    unresolved = list(unresolved_targets)
    if not checks:
        unresolved.append("exact executable validation command")
    if status == "blocked_external":
        mode = "stop_external_dependency"
    elif unresolved:
        mode = "stop_specification_gap"
    elif status == "complete":
        mode = "verify_existing_before_any_edit"
    else:
        mode = "execute_only_after_dependencies_complete"
    source_refs = []
    for raw_value in [*parent.get("read_before_edit", []), *reads]:
        value = canonical_source_ref(raw_value)
        if value not in source_refs:
            source_refs.append(value)
    ordered_steps = [
        {"step": 1, "action": "dependency_gate", "instruction": f"Verify every dependency is complete with evidence: {dependencies or ['none']}. Stop on blocked, partial, or missing evidence."},
        {"step": 2, "action": "read_authority", "instruction": f"Read these sources completely before editing: {source_refs}. Follow {CARDINAL_REF}; Pi-overlay files are behavior sources, not the destination host."},
        {"step": 3, "action": "inspect_destination", "instruction": f"Inspect only these destination paths and named symbols before changing code: {exact_targets}; symbols/calls: {calls}. Do not invent a substitute path or API."},
        {"step": 4, "action": "implement", "instruction": f"Transform the typed input into the required output: INPUT={input_text!s}; OUTPUT={output_text!s}. Preserve every source behavior and use generated DTOs/operation metadata."},
        {"step": 5, "action": "negative_cases", "instruction": "Prove foreign scope, missing authority, stale revision/cursor, unavailable capability, and ineligible/empty contribution behavior whenever applicable; fail closed rather than infer."},
        {"step": 6, "action": "validate", "instruction": f"Run exactly these commands: {checks}. Do not substitute screenshots for functional checks."},
        {"step": 7, "action": "evidence", "instruction": f"Write the bounded result and command outputs to docs/contracts/evidence/spec135-svelte-tasks/{task['id']}.json and link the changed files and exact acceptance statements."},
    ]
    return {
        "schema": "focusa.spec135.svelte_task_execution_packet.v1",
        "task_id": task["id"],
        "task_class": task_class,
        "title": task.get("title") or f"Implement operation {operation_id}",
        "cardinal_translation_ref": CARDINAL_REF,
        "current_status": status,
        "execution_mode": mode,
        "depends_on": dependencies,
        "source_requirement": {
            "parent_callgraph_node": task.get("parent"),
            "source_refs": source_refs,
            "pi_overlay_behavior_sources": source_pi,
            "behavior_to_preserve": task.get("input") or done,
            "parent_acceptance": parent_acceptance,
        },
        "destination": {
            "host": "Focusa Desktop Mission Canvas Svelte GUI tab",
            "exact_target_paths": exact_targets,
            "named_symbols_or_calls": calls,
            "generated_operation": descriptor or None,
            "agent_tui_boundary": "Agent TUI remains the separate authentic PTY-backed Pi terminal; it is never the Mission Canvas renderer.",
        },
        "typed_input": input_text,
        "required_output": output_text,
        "ordered_steps": ordered_steps,
        "authority_boundaries": [
            "Core owns eligibility, layout resolution, identity, authority, persistence, operations, and recomposition.",
            "Svelte renders canonical ResolvedWorkspaceProjection output and approved generated renderers; it does not infer contributions or layout.",
            "ScopeRef/ProjectRootKey -> WorkstreamId -> ContinuityId -> AttachmentKey -> SessionId/InstanceId -> runtime object -> WorkSurfaceId is the authority chain.",
            "UIAI Engine exclusively owns browser execution and visual proof.",
            "Generated OpenAPI, operation registry, TypeScript client, and validators own transport DTOs and route metadata.",
        ],
        "prohibited_shortcuts": [
            "fixed dashboard or route-local screen inventory",
            "client-local eligibility or layout resolver",
            "invented ID, label, operation, renderer, activity, profile, panel, or workflow",
            "project_root plus continuity_id treated as complete authority",
            "ordinary child-process pipes substituted for PTY",
            "competing Svelte A2UI/schema renderer",
            "fixture, screenshot, or terminal output treated as production completion",
            "editing an unresolved symbolic target or bypassing an incomplete dependency",
        ],
        "validation_commands": checks,
        "acceptance": [done, *parent_acceptance],
        "evidence_artifact": f"docs/contracts/evidence/spec135-svelte-tasks/{task['id']}.json",
        "stop_conditions": {
            "unresolved_specification_items": unresolved,
            "stop_if": [
                "any dependency lacks completion evidence",
                "exact Workstream/Attachment authority is unavailable for an authority-bearing action",
                "a named generated operation, renderer, semantic binding, or target path cannot be found",
                "the requested behavior would require inventing product semantics",
                "the check command is absent or cannot exercise the required output",
            ],
            "recovery": "Do not guess. Mark the packet blocked with the missing exact item and continue only when its owning dependency or graph amendment resolves it.",
        },
        "reopen_if": [
            "any cited Spec 135 behavior is absent from the Svelte destination",
            "the implementation renders only a static/fixture path",
            "authority, stale-state, empty-state, or negative-case checks fail",
            "the generated contract or operation registry changes",
            "visual composition contradicts the adaptive handoff grammar after functional checks pass",
        ],
    }


def build() -> dict[str, Any]:
    graph = load_yaml(GRAPH_PATH)
    parent = load_yaml(PARENT_PATH)
    registry = json.loads(REGISTRY_PATH.read_text())
    parents = {node["id"]: node for node in parent["nodes"]}
    operations = {op["operation_id"]: op for op in registry["operations"]}
    defaults = graph["operation_task_defaults"]
    tasks: list[tuple[str, dict[str, Any]]] = []
    tasks += [("atomic", task) for task in graph["atomic_tasks"]]
    tasks += [("operation", task) for task in graph["operation_tasks"]]
    tasks += [("integration", task) for task in graph["integration_tasks"]]
    packets = {task["id"]: task_packet(task, kind, defaults if kind == "operation" else {}, parents, operations) for kind, task in tasks}
    unresolved = {task_id: packet["stop_conditions"]["unresolved_specification_items"] for task_id, packet in packets.items() if packet["stop_conditions"]["unresolved_specification_items"]}
    document: dict[str, Any] = {
        "schema": "focusa.spec135.svelte_task_execution_packets.v1",
        "cardinal_translation_ref": CARDINAL_REF,
        "source_graph": str(GRAPH_PATH.relative_to(ROOT)),
        "translation_matrix": "docs/transitions/FOCUSA-TRANSITION-001-spec135-svelte-translation-matrix.md",
        "task_count": len(packets),
        "execution_protocol": [
            "Select exactly one packet by task ID from the executable graph order.",
            "Honor execution_mode and stop_conditions before editing.",
            "Read all source_refs, then edit only exact_target_paths.",
            "Execute ordered_steps without skipping dependency, authority, negative-case, validation, or evidence gates.",
            "Never use the broad completion DAG as direct edit instructions; it is requirement and dependency provenance.",
        ],
        "unresolved_packet_count": len(unresolved),
        "unresolved_packets": unresolved,
        "packets": packets,
    }
    digest_source = json.dumps(document, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    document["digest_sha256"] = hashlib.sha256(digest_source.encode()).hexdigest()
    return document


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    document = build()
    packets = document.pop("packets")
    document["packet_refs"] = {task_id: f"{PACKET_REF_PREFIX}{task_id}.json" for task_id in packets}
    rendered_index = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    rendered_packets = {task_id: json.dumps(packet, indent=2, ensure_ascii=False) + "\n" for task_id, packet in packets.items()}
    if args.check:
        assert OUTPUT_PATH.exists(), f"missing generated packet index: {OUTPUT_PATH}"
        assert OUTPUT_PATH.read_text() == rendered_index, f"stale generated packet index: {OUTPUT_PATH}"
        for task_id, rendered in rendered_packets.items():
            packet_path = PACKET_DIR / f"{task_id}.json"
            assert packet_path.exists(), f"missing generated packet: {packet_path}"
            assert packet_path.read_text() == rendered, f"stale generated packet: {packet_path}"
        assert {path.stem for path in PACKET_DIR.glob("*.json")} == set(packets), "unexpected or missing task packet files"
        print("Spec 135 Svelte execution packets: PASS")
        return
    PACKET_DIR.mkdir(parents=True, exist_ok=True)
    for stale in PACKET_DIR.glob("*.json"):
        if stale.stem not in packets:
            stale.unlink()
    OUTPUT_PATH.write_text(rendered_index)
    for task_id, rendered in rendered_packets.items():
        (PACKET_DIR / f"{task_id}.json").write_text(rendered)
    print(f"Generated {OUTPUT_PATH.relative_to(ROOT)} and {len(packets)} bounded task packets")


if __name__ == "__main__":
    main()
