#!/usr/bin/env python3
"""Generate exhaustive Spec 152 entitlement surface inventory; unknowns are explicit."""

import argparse
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs/contracts/spec152-entitlement-coverage.v1.json"
ROUTES = ROOT / "docs/contracts/spec141/generated-capability-v2/route-classification.json"
OPERATIONS = ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"
CAPABILITIES = ROOT / "docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json"
POLICY = ROOT / "docs/contracts/spec152f-entitlement-policy.v1.yaml"
RECONCILIATION_OUTPUT = ROOT / "docs/contracts/spec152f-surface-reconciliation.v1.json"
RECONCILIATION_DIR = ROOT / "docs/contracts/spec152f-surface-reconciliation"

FAMILY_FEATURE = {
    "agent": "focusa.core.workpoint",
    "agent_runtime": "focusa.core.workpoint",
    "ascc": "focusa.core.workpoint",
    "attachments": "focusa.core.evidence",
    "awareness": "focusa.core.evidence",
    "bloatgaurd": "focusa.core.evidence",
    "browser": "focusa.core.workpoint",
    "call_stack": "focusa.core.evidence",
    "commands": "focusa.core.workpoint",
    "compaction": "focusa.core.evidence",
    "connect": "focusa.team.multi_operator",
    "constitution": "focusa.core.mission",
    "context": "focusa.core.evidence",
    "context_cognition": "focusa.core.evidence",
    "contribute": "focusa.core.mission",
    "daemon_routing": "focusa.core.workpoint",
    "device": "focusa.team.multi_operator",
    "diagnostics": "focusa.core.evidence",
    "diagnostics_hygiene": "focusa.core.evidence",
    "dxux": "focusa.core.evidence",
    "ecs": "focusa.core.evidence",
    "events": "focusa.core.workpoint",
    "evidence": "focusa.core.evidence",
    "focus": "focusa.core.workpoint",
    "focus_gate": "focusa.core.workpoint",
    "focus_state": "focusa.core.workpoint",
    "focusa": "focusa.core.workpoint",
    "gate": "focusa.core.workpoint",
    "harnesses": "focusa.core.workpoint",
    "installations": "focusa.core.workpoint",
    "instances": "focusa.remote.stream",
    "adapters": "focusa.core.workpoint",
    "background_jobs": "focusa.core.workpoint",
    "callgraph": "focusa.core.workpoint",
    "callgraphs": "focusa.core.workpoint",
    "callgraph_runs": "focusa.core.workpoint",
    "completion_claims": "focusa.core.workpoint",
    "credentials": "focusa.core.workpoint",
    "direction": "focusa.core.workpoint",
    "remote_workspaces": "focusa.core.workpoint",
    "response": "focusa.core.workpoint",
    "task": "focusa.core.workpoint",
    "temporal": "focusa.core.workpoint",
    "worksets": "focusa.core.workpoint",
    "workset": "focusa.core.workpoint",
    "workstreams": "focusa.core.workpoint",
    "interview_strategy": "focusa.core.mission",
    "intuition": "focusa.core.evidence",
    "lineage": "focusa.core.evidence",
    "mcp": "focusa.core.workpoint",
    "memory": "focusa.core.evidence",
    "metacognition": "focusa.core.evidence",
    "mission_canvas": "focusa.core.mission",
    "ontology": "focusa.core.evidence",
    "prediction": "focusa.core.evidence",
    "predictions": "focusa.core.evidence",
    "preload": "focusa.core.evidence",
    "project": "focusa.core.mission",
    "project_genesis": "focusa.core.mission",
    "project_identity": "focusa.core.mission",
    "project_interview": "focusa.core.mission",
    "project_role_profile": "focusa.core.mission",
    "prompt": "focusa.core.workpoint",
    "proposals": "focusa.core.mission",
    "provider_execution": "focusa.agent.parallelism",
    "providers": "focusa.core.workpoint",
    "proxy": "focusa.core.workpoint",
    "reflect": "focusa.core.mission",
    "release": "focusa.release.proof",
    "resource": "focusa.core.evidence",
    "semantic_integrity": "focusa.core.evidence",
    "session": "focusa.core.workpoint",
    "session_transfer": "focusa.core.workpoint",
    "silent_sessions": "focusa.agent.silent_sessions",
    "spec_workbench": "focusa.core.mission",
    "state": "focusa.core.workpoint",
    "subagent": "focusa.core.workpoint",
    "sync": "focusa.remote.stream",
    "task_plan": "focusa.core.mission",
    "telemetry": "focusa.core.evidence",
    "threads": "focusa.core.workpoint",
    "tokens": "focusa.core.workpoint",
    "trajectory": "focusa.core.mission",
    "traversal": "focusa.core.evidence",
    "traverse": "focusa.core.evidence",
    "tree_lineage": "focusa.core.evidence",
    "trust": "focusa.core.workpoint",
    "turn": "focusa.core.workpoint",
    "update": "focusa.update.apply",
    "visual_workflow": "focusa.core.evidence",
    "work_items": "focusa.core.mission",
    "work_loop": "focusa.core.workpoint",
    "work_rail": "focusa.core.workpoint",
    "workpoint": "focusa.core.workpoint",
    "workspace_artifact": "focusa.core.evidence",
}
RECOVERY_FAMILIES = {"health", "license"}
RECOVERY_PATHS = {"/health", "/v1/health", "/v1/license/status"}
CANDIDATE_FAMILY_FEATURE = {
    "team_remote": "focusa.team.multi_operator",
    "automation": "focusa.agent.silent_sessions",
    "premium_updates": "focusa.update.apply",
    "release_proof": "focusa.release.proof",
}


def mutation_class(methods, side_effect=None):
    if side_effect:
        if side_effect in {"read", "read_only", "none"}: return "read"
        return "mutation"
    normalized = {str(method).upper() for method in methods}
    if normalized and normalized <= {"GET", "HEAD", "OPTIONS"}: return "read"
    if normalized & {"POST", "PUT", "PATCH", "DELETE"}: return "mutation"
    return "unknown"


def entry(surface, symbol, mutation, product="focusa", feature=None, limit=None,
          gate=None, pre_side_effect_test=None, recovery=False, source=None):
    return {
        "surface": surface,
        "symbol_or_route": symbol,
        "mutation_class": mutation,
        "product": product,
        "feature": feature,
        "limit_bucket": limit,
        "gate_location": gate,
        "pre_side_effect_test": pre_side_effect_test,
        "recovery_allowance": recovery,
        "source": source,
    }


def discover_cli():
    text = (ROOT / "crates/focusa-cli/src/main.rs").read_text()
    start = text.index("enum Commands")
    brace = text.index("{", start)
    depth, end = 0, None
    for index in range(brace, len(text)):
        if text[index] == "{": depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                end = index
                break
    body = text[brace + 1:end]
    return sorted(set(re.findall(r"^    ([A-Z][A-Za-z0-9_]*)\s*(?:\(|\{|,)", body, re.MULTILINE)))


def discover_ui_actions(base):
    actions = []
    for path in sorted(base.rglob("*.svelte")):
        for line_number, line in enumerate(path.read_text().splitlines(), 1):
            if "onclick=" in line or "on:click=" in line:
                actions.append(f"{path.relative_to(ROOT)}:{line_number}")
    return actions


def build(include_test_files=False):
    routes = json.loads(ROUTES.read_text())["routes"]
    operations = json.loads(OPERATIONS.read_text())["operations"]
    capabilities = json.loads(CAPABILITIES.read_text())["descriptors"]
    coverage, unmatched, scanner_exclusions = [], [], []

    operation_by_id = {item["operation_id"]: item for item in operations}
    for item in operations:
        mutation = mutation_class([], item.get("side_effect_profile"))
        family = item.get("family")
        feature = FAMILY_FEATURE.get(family) if mutation == "mutation" else None
        recovery = family in RECOVERY_FAMILIES
        covered = mutation == "read" or feature is not None or recovery
        record = entry(
            "operation", item["operation_id"], mutation, feature=feature,
            gate="focusa-api entitlement middleware" if feature else None,
            pre_side_effect_test="required" if mutation == "mutation" and not recovery else "not_applicable",
            recovery=recovery, source=item.get("docs_ref"),
        )
        (coverage if covered else unmatched).append(record)

    for route in routes:
        refs = route.get("operation_refs", [])
        linked = [operation_by_id[ref] for ref in refs if ref in operation_by_id]
        methods = route.get("methods", [])
        mutation = mutation_class(methods)
        features = {FAMILY_FEATURE.get(item.get("family")) for item in linked}
        features.discard(None)
        feature = next(iter(features)) if len(features) == 1 else None
        recovery = route["path"] in RECOVERY_PATHS
        covered = mutation == "read" or recovery or (mutation == "mutation" and feature is not None)
        record = entry(
            "rest", route["path"], mutation, feature=feature,
            gate="route entitlement middleware" if feature else None,
            pre_side_effect_test="required" if mutation == "mutation" else "not_applicable",
            recovery=recovery, source=route.get("sources"),
        )
        (coverage if covered else unmatched).append(record)

    for descriptor in capabilities:
        name = descriptor.get("tool_names", {}).get("pi") or descriptor.get("capability_id")
        effects = set(descriptor.get("side_effects", []))
        mutation = "read" if effects and effects <= {"read", "read_only", "none"} else "mutation"
        family = descriptor.get("family")
        feature = FAMILY_FEATURE.get(family) if mutation == "mutation" else None
        recovery = family in RECOVERY_FAMILIES
        record = entry("pi_tool", str(name), mutation, feature=feature,
                       gate="Pi tool preflight" if feature else None,
                       pre_side_effect_test="required" if mutation == "mutation" and not recovery else "not_applicable",
                       recovery=recovery,
                       source="docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json")
        (coverage if mutation == "read" or feature or recovery else unmatched).append(record)

    for command in discover_cli():
        unmatched.append(entry("cli", command, "unknown", source="crates/focusa-cli/src/main.rs"))
    for surface, base in [("menubar", ROOT / "apps/menubar/src"), ("tui", ROOT / "crates/focusa-tui/src")]:
        for action in discover_ui_actions(base):
            unmatched.append(entry(surface, action, "unknown", source=action.split(":")[0]))
    for kind in ["worker", "scheduler", "export", "update", "release"]:
        for path in sorted(ROOT.rglob(f"*{kind}*.rs")):
            if "target" in path.parts:
                continue
            relative = str(path.relative_to(ROOT))
            if _is_test_path(relative):
                scanner_exclusions.append({
                    "surface": kind,
                    "path": relative,
                    "rule": "tests_directory" if "tests" in Path(relative).parts else "recognized_test_module",
                })
                if not include_test_files:
                    continue
            unmatched.append(entry(kind, relative, "unknown", source=relative))

    # Resolve unmatched surfaces through the reconciliation contract.
    # Unknown side effects remain explicit failures (metadata_repair_required).
    # Everything else resolves to covered: base, premium, recovery, or inheritance.
    # When include_test_files is True, preserve unmatched as-is for the frozen
    # reconciliation baseline; runtime coverage (include_test_files=False) resolves
    # every surface to zero unmatched.
    if not include_test_files:
        resolved = []
        still_unmatched = []
        for item in unmatched:
            resolution, family, _owner_task, _rationale = _resolution(item)
            if resolution == "scanner_exclusion_test_only":
                continue
            if resolution == "metadata_repair_required":
                still_unmatched.append(item)
                continue
            if resolution == "inherit_canonical_operation":
                resolved.append(entry(
                    item["surface"], item["symbol_or_route"], "mutation",
                    feature="focusa.core.workpoint",
                    gate="inherit_canonical_operation",
                    pre_side_effect_test="required",
                    source=item["source"],
                ))
            elif resolution == "base_entitlement_candidate":
                resolved.append(entry(
                    item["surface"], item["symbol_or_route"], item["mutation_class"],
                    feature="focusa.core.workpoint",
                    gate="route entitlement middleware",
                    pre_side_effect_test="required",
                    source=item["source"],
                ))
            elif resolution == "premium_family_candidate":
                feature = CANDIDATE_FAMILY_FEATURE.get(family, "focusa.core.workpoint")
                resolved.append(entry(
                    item["surface"], item["symbol_or_route"], item["mutation_class"],
                    feature=feature,
                    gate="route entitlement middleware",
                    pre_side_effect_test="required",
                    source=item["source"],
                ))
            elif resolution == "recovery_or_read_allowance":
                resolved.append(entry(
                    item["surface"], item["symbol_or_route"], item["mutation_class"],
                    recovery=True,
                    pre_side_effect_test="not_applicable",
                    source=item["source"],
                ))
        coverage.extend(resolved)
        unmatched = still_unmatched

    key = lambda item: (item["surface"], item["symbol_or_route"], json.dumps(item, sort_keys=True))
    coverage.sort(key=key); unmatched.sort(key=key)
    payload = {
        "schema": "focusa.entitlement_coverage.v1",
        "source_digests": {
            str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in [ROUTES, OPERATIONS, CAPABILITIES]
        },
        "coverage": coverage,
        "unmatched_surfaces": unmatched,
        "counts": {"covered": len(coverage), "unmatched": len(unmatched), "total": len(coverage) + len(unmatched)},
    }
    if not include_test_files:
        scanner_exclusions.sort(key=lambda item: (item["surface"], item["path"]))
        payload["scanner_exclusions"] = {
            "schema": "focusa.spec152f.surface_scanner_exclusions.v1",
            "count": len(scanner_exclusions),
            "entries": scanner_exclusions,
        }
    return payload


def _sha256(data):
    if isinstance(data, str): data = data.encode()
    return hashlib.sha256(data).hexdigest()


def _is_test_path(value):
    path = "/" + value.replace("\\", "/")
    name = Path(value).name
    return "/tests/" in path or name.endswith("_test.rs")


def _rest_segment(path):
    parts = path.strip("/").split("/")
    return parts[1] if len(parts) > 1 and parts[0] == "v1" else parts[0]


def _resolution(row):
    surface, symbol = row["surface"], row["symbol_or_route"]
    if surface == "rest":
        if row["mutation_class"] == "unknown":
            return "metadata_repair_required", None, "focusa-vbcqu.20.14.23", "HTTP method and side effects must be source-backed before policy."
        segment = _rest_segment(symbol)
        if symbol == "/v1/update/rollback":
            return "recovery_or_read_allowance", "account_recovery", "focusa-vbcqu.20.14.24", "Rollback remains reachable subject to security and artifact trust."
        premium = {
            "connect": "team_remote",
            "device": "team_remote",
            "instances": "team_remote",
            "silent-sessions": "automation",
            "sync": "team_remote",
            "update": "premium_updates",
        }.get(segment)
        if premium:
            return "premium_family_candidate", premium, "focusa-vbcqu.20.14.24", "Route requires operation-level premium or exception resolution."
        return "base_entitlement_candidate", "base_focusa", "focusa-vbcqu.20.14.24", "Known mutation defaults to base until canonical operation metadata proves an exception."
    if surface == "cli":
        return "inherit_canonical_operation", None, "focusa-vbcqu.20.14.25", "Top-level command names are presenters, not SKUs."
    if surface == "menubar":
        return "inherit_canonical_operation", None, "focusa-vbcqu.20.14.26", "Click handlers inherit invoked operations; navigation and display are not paywalls."
    if _is_test_path(symbol):
        return "scanner_exclusion_test_only", None, "focusa-vbcqu.20.14.27", "Test-only source is not a runtime entitlement surface."
    if surface == "export":
        return "recovery_or_read_allowance", "customer_data_export", "focusa-vbcqu.20.14.28", "Basic customer-data export remains available; premium packaging is operation-bound."
    if surface == "release":
        return "premium_family_candidate", "release_proof", "focusa-vbcqu.20.14.28", "Filename is not policy; callable entrypoints must distinguish base read from premium proof."
    return "inherit_canonical_operation", None, "focusa-vbcqu.20.14.28", "Runtime helper inherits its initiating operation and dispatch-time authority."


def _render_shard(payload):
    lines = ["{", f'  "schema": {json.dumps(payload["schema"])},', f'  "surface_group": {json.dumps(payload["surface_group"])},', f'  "row_count": {payload["row_count"]},', '  "rows": [']
    for index, row in enumerate(payload["rows"]):
        lines.append("    " + json.dumps(row, sort_keys=True, separators=(",", ":")) + ("," if index + 1 < len(payload["rows"]) else ""))
    lines.extend(["  ]", "}"])
    return "\n".join(lines) + "\n"


def build_reconciliation():
    # Spec 152F.00.04 freezes the unmatched reconciliation frontier. Newly
    # discovered covered routes may grow the total inventory without changing
    # those reconciliation rows. Runtime coverage excludes test-only matches.
    coverage = build(include_test_files=True)
    rows = []
    for source_row in coverage["unmatched_surfaces"]:
        resolution, family, owner_task, rationale = _resolution(source_row)
        canonical = json.dumps(source_row, sort_keys=True, separators=(",", ":"))
        rows.append({
            "baseline_id": f'{source_row["surface"]}:{_sha256(canonical)[:16]}',
            "surface": source_row["surface"],
            "symbol_or_route": source_row["symbol_or_route"],
            "mutation_class": source_row["mutation_class"],
            "source": source_row["source"],
            "resolution": resolution,
            "candidate_family": family,
            "owner_task": owner_task,
            "rationale": rationale,
        })
    rows.sort(key=lambda row: (row["surface"], row["symbol_or_route"], row["baseline_id"]))
    groups = {
        "rest": [row for row in rows if row["surface"] == "rest"],
        "cli": [row for row in rows if row["surface"] == "cli"],
        "menubar": [row for row in rows if row["surface"] == "menubar"],
        "runtime_files": [row for row in rows if row["surface"] not in {"rest", "cli", "menubar"}],
    }
    rendered_shards, shard_refs = {}, {}
    for group, group_rows in groups.items():
        relative = f"docs/contracts/spec152f-surface-reconciliation/{group}.v1.json"
        rendered = _render_shard({"schema": "focusa.spec152f.surface_reconciliation_shard.v1", "surface_group": group, "row_count": len(group_rows), "rows": group_rows})
        rendered_shards[relative] = rendered
        shard_refs[group] = {"path": relative, "row_count": len(group_rows), "sha256": _sha256(rendered)}
    resolution_counts = {}
    for row in rows: resolution_counts[row["resolution"]] = resolution_counts.get(row["resolution"], 0) + 1
    surface_counts = {}
    for row in rows: surface_counts[row["surface"]] = surface_counts.get(row["surface"], 0) + 1
    index = {
        "schema": "focusa.spec152f.surface_reconciliation.v1",
        "authority": "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md",
        "policy": "docs/contracts/spec152f-entitlement-policy.v1.yaml",
        "baseline_coverage": "docs/contracts/spec152-entitlement-coverage.v1.json",
        "baseline_counts": coverage["counts"],
        "surface_counts": dict(sorted(surface_counts.items())),
        "resolution_counts": dict(sorted(resolution_counts.items())),
        "unknown_method_routes": sum(row["surface"] == "rest" and row["mutation_class"] == "unknown" for row in rows),
        "test_only_scanner_exclusions": sum(row["resolution"] == "scanner_exclusion_test_only" for row in rows),
        "runtime_file_entries": len(groups["runtime_files"]),
        "runtime_file_entries_after_test_exclusion": sum(row["surface"] not in {"rest", "cli", "menubar"} and row["resolution"] != "scanner_exclusion_test_only" for row in rows),
        "coverage_canonical_sha256": _sha256(json.dumps(coverage, sort_keys=True, separators=(",", ":"))),
        "policy_file_sha256": _sha256(POLICY.read_bytes()),
        "source_digests": coverage["source_digests"],
        "shards": shard_refs,
        "rules": [
            "inventory rows are not prices SKUs or independent paywalls",
            "unknown methods require source-backed metadata repair",
            "presenters inherit canonical operation policy",
            "test-only files are excluded without hiding production entrypoints",
            "recovery read export repair rollback and stable security paths remain available subject to security",
        ],
    }
    return index, rendered_shards


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--check", action="store_true")
    args = parser.parse_args(); payload = build()
    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    reconciliation, shards = build_reconciliation()
    rendered_reconciliation = json.dumps(reconciliation, indent=2, sort_keys=True) + "\n"
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != rendered: raise SystemExit("entitlement coverage is stale")
        if not RECONCILIATION_OUTPUT.exists() or RECONCILIATION_OUTPUT.read_text() != rendered_reconciliation: raise SystemExit("Spec 152F reconciliation index is stale")
        for relative, expected in shards.items():
            path = ROOT / relative
            if not path.exists() or path.read_text() != expected: raise SystemExit(f"Spec 152F reconciliation shard is stale: {relative}")
    else:
        OUTPUT.write_text(rendered)
        RECONCILIATION_DIR.mkdir(parents=True, exist_ok=True)
        RECONCILIATION_OUTPUT.write_text(rendered_reconciliation)
        for relative, content in shards.items(): (ROOT / relative).write_text(content)
    print(json.dumps({"coverage": payload["counts"], "reconciliation": reconciliation["resolution_counts"]}, sort_keys=True))


if __name__ == "__main__": main()
