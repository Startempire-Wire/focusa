#!/usr/bin/env python3
"""Generate explicit Spec141 eligibility classification for every Axum route path."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = (
    ROOT / "docs/contracts/spec141/generated-capability-v2/route-classification.json"
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    source_files = sorted((ROOT / "crates/focusa-api/src").rglob("*.rs"))
    paths: dict[str, set[str]] = {}
    for source in source_files:
        body = source.read_text(errors="replace")
        for path in re.findall(r'\.route\(\s*"([^"]+)"', body, re.S):
            paths.setdefault(path, set()).add(str(source.relative_to(ROOT)))

    registry = json.loads(
        (
            ROOT
            / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"
        ).read_text()
    )
    agent_paths = {item["path"] for item in registry["operations"]}
    spec141_agent_paths = {
        "/mcp",
        "/v1/mcp",
        "/v1/agent/card",
        "/v1/agent/tools",
        "/v1/agent/tools/{name}",
        "/v1/agent/tool-graph",
        "/v1/agent/tool-bundles",
        "/v1/agent/tool-changes",
        "/v1/agent/capabilities",
        "/v1/agent/operations",
        "/v1/agent/schemas",
        "/v1/agent/schemas/{schema_id}",
        "/v1/openapi.json",
        "/v1/browser/capabilities/intake",
        "/v1/browser/webmcp/intake",
        "/v1/browser/workflow/plan",
    }
    public_health = {
        "/health",
        "/v1/health",
        "/ready",
        "/v1/ready",
        "/version",
        "/v1/version",
    }

    classifications = []
    for path in sorted(paths):
        if path in agent_paths or path in spec141_agent_paths:
            classification = "agent_eligible"
            rationale = "Covered by the generated operation registry or Spec141 capability-discovery/MCP contract."
        elif path in public_health:
            classification = "public_health"
            rationale = "Bounded public liveness/readiness/version probe."
        elif any(
            token in path
            for token in ("/pair", "/device/", "/oauth", "/license/activate")
        ):
            classification = "public_pairing"
            rationale = "Pairing/auth/license bootstrap surface; governed by its own token and expiry checks."
        elif any(
            token in path
            for token in ("/internal", "/debug", "/metrics", "/events/raw", "/admin/")
        ):
            classification = "internal"
            rationale = "Runtime/operator diagnostic surface not projected as an agent capability."
        elif any(token in path for token in ("/deprecated", "/legacy")):
            classification = "deprecated"
            rationale = "Compatibility-only route; agents use the declared replacement."
        else:
            classification = "operator_only"
            rationale = "Not in the curated agent operation registry; requires explicit operator/application workflow authority."
        classifications.append(
            {
                "path": path,
                "classification": classification,
                "rationale": rationale,
                "sources": sorted(paths[path]),
                "operation_refs": sorted(
                    item["operation_id"]
                    for item in registry["operations"]
                    if item["path"] == path
                ),
            }
        )

    counts: dict[str, int] = {}
    for item in classifications:
        counts[item["classification"]] = counts.get(item["classification"], 0) + 1
    report = {
        "schema": "focusa.agent_route_classification.v1",
        "route_count": len(classifications),
        "classification_counts": dict(sorted(counts.items())),
        "allowed_classifications": [
            "agent_eligible",
            "operator_only",
            "internal",
            "public_health",
            "public_pairing",
            "deprecated",
        ],
        "routes": classifications,
    }
    body = json.dumps(report, indent=2) + "\n"
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text() != body:
            print("Spec141 route classification drift", flush=True)
            return 1
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(body)
    print(
        json.dumps(
            {
                "status": "passed",
                "mode": "check" if args.check else "write",
                "routes": len(classifications),
                "counts": counts,
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
