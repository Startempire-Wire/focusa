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
    "evidence": "focusa.core.evidence",
    "workpoint": "focusa.core.workpoint",
    "trajectory": "focusa.core.mission",
    "project": "focusa.core.mission",
    "project_genesis": "focusa.core.mission",
    "silent_sessions": "focusa.agent.silent_sessions",
    "release": "focusa.release.proof",
    "update": "focusa.update.apply",
}
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
        covered = mutation == "read" or feature is not None
        record = entry(
            "operation", item["operation_id"], mutation, feature=feature,
            gate="focusa-api entitlement middleware" if feature else None,
            pre_side_effect_test="required" if mutation == "mutation" else "not_applicable",
            recovery=False, source=item.get("docs_ref"),
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
        name = descriptor.get("tool_name") or descriptor.get("name") or descriptor.get("operation_id")
        operation_ref = descriptor.get("operation_ref") or descriptor.get("operation_id")
        operation = operation_by_id.get(operation_ref, {})
        mutation = mutation_class([], operation.get("side_effect_profile") or descriptor.get("side_effect"))
        feature = FAMILY_FEATURE.get(operation.get("family")) if mutation == "mutation" else None
        record = entry("pi_tool", str(name), mutation, feature=feature,
                       gate="Pi tool preflight" if feature else None,
                       pre_side_effect_test="required" if mutation == "mutation" else "not_applicable",
                       source="docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json")
        (coverage if mutation == "read" or feature else unmatched).append(record)

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
