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

FAMILY_FEATURE = {
    "agent": "focusa.core.workpoint",
    "awareness": "focusa.core.evidence",
    "bloatgaurd": "focusa.core.evidence",
    "call_stack": "focusa.core.evidence",
    "context": "focusa.core.evidence",
    "context_cognition": "focusa.core.evidence",
    "device": "focusa.team.multi_operator",
    "diagnostics": "focusa.core.evidence",
    "dxux": "focusa.core.evidence",
    "events": "focusa.core.workpoint",
    "evidence": "focusa.core.evidence",
    "interview_strategy": "focusa.core.mission",
    "lineage": "focusa.core.evidence",
    "memory": "focusa.core.evidence",
    "metacognition": "focusa.core.evidence",
    "mission_canvas": "focusa.core.mission",
    "prediction": "focusa.core.evidence",
    "project": "focusa.core.mission",
    "project_genesis": "focusa.core.mission",
    "project_interview": "focusa.core.mission",
    "project_role_profile": "focusa.core.mission",
    "provider_execution": "focusa.agent.parallelism",
    "release": "focusa.release.proof",
    "resource": "focusa.core.evidence",
    "silent_sessions": "focusa.agent.silent_sessions",
    "spec_workbench": "focusa.core.mission",
    "state": "focusa.core.workpoint",
    "task_plan": "focusa.core.mission",
    "trajectory": "focusa.core.mission",
    "traverse": "focusa.core.evidence",
    "turn": "focusa.core.workpoint",
    "update": "focusa.update.apply",
    "work_loop": "focusa.core.workpoint",
    "work_rail": "focusa.core.workpoint",
    "workpoint": "focusa.core.workpoint",
    "workspace_artifact": "focusa.core.evidence",
}
RECOVERY_FAMILIES = {"health", "license"}
RECOVERY_PATHS = {"/health", "/v1/health", "/v1/license/status"}


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


def build():
    routes = json.loads(ROUTES.read_text())["routes"]
    operations = json.loads(OPERATIONS.read_text())["operations"]
    capabilities = json.loads(CAPABILITIES.read_text())["descriptors"]
    coverage, unmatched = [], []

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
            if "target" not in path.parts:
                unmatched.append(entry(kind, str(path.relative_to(ROOT)), "unknown", source=str(path.relative_to(ROOT))))

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
    return payload


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--check", action="store_true")
    args = parser.parse_args(); payload = build()
    rendered = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != rendered: raise SystemExit("entitlement coverage is stale")
    else: OUTPUT.write_text(rendered)
    print(json.dumps(payload["counts"], sort_keys=True))


if __name__ == "__main__": main()
