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
API_REFERENCE = ROOT / "docs/current/API_REFERENCE_CURRENT.md"


def without_inline_test_modules(body: str) -> str:
    """Exclude explicit cfg(test) modules without truncating later production code.

    Mask Rust strings/chars/comments only for delimiter discovery. Route literals
    outside test modules remain intact for this inventory's existing parser.
    This is not a general Rust cfg evaluator or proof of HTTP registration.
    """
    tokens = re.compile(
        r'(?:br|r)(?P<hashes>\#{0,255})".*?"(?P=hashes)'
        r'|b?"(?:\\.|[^"\\])*"'
        r"|b?'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'"
        r'|//[^\n]*|/\*', re.S,
    )
    masked = list(body)
    delimiters = re.compile(r'/\*|\*/')
    cursor = 0
    while match := tokens.search(body, cursor):
        end = match.end()
        if match.group() == "/*":
            depth = 1
            while depth:
                delimiter = delimiters.search(body, end)
                if delimiter is None:
                    raise ValueError("unterminated Rust block comment")
                depth += 1 if delimiter.group() == "/*" else -1
                end = delimiter.end()
        masked[match.start():end] = ["\n" if c == "\n" else " " for c in body[match.start():end]]
        cursor = end
    lexical = "".join(masked)
    module = re.compile(
        r'#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*'
        r'(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?'
        r'mod\s+(?:r#)?\w+\s*\{'
    )
    result = list(body)
    cursor = 0
    while match := module.search(lexical, cursor):
        depth, end = 1, match.end()
        while depth and end < len(lexical):
            depth += (lexical[end] == "{") - (lexical[end] == "}")
            end += 1
        if depth:
            raise ValueError("unterminated cfg(test) module")
        result[match.start():end] = ["\n" if c == "\n" else " " for c in body[match.start():end]]
        cursor = end
    return "".join(result)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    source_files = sorted((ROOT / "crates/focusa-api/src").rglob("*.rs"))
    paths: dict[str, set[str]] = {}
    methods: dict[str, set[str]] = {}
    for source in source_files:
        body = without_inline_test_modules(source.read_text(errors="strict"))
        relative_source = str(source.relative_to(ROOT))
        string_constants = dict(
            re.findall(
                r'^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*&str\s*=\s*"([^"]+)"\s*;',
                body,
                re.M,
            )
        )
        constant_route_names = re.findall(
            r'^\s*\.route\(\s*([A-Z][A-Z0-9_]*)\s*,', body, re.M
        )
        unresolved = sorted(set(constant_route_names) - string_constants.keys())
        if unresolved:
            raise SystemExit(
                f"{relative_source}: unresolved route path constants: {', '.join(unresolved)}"
            )

        for path in re.findall(r'\.route\(\s*"([^"]+)"', body, re.S):
            paths.setdefault(path, set()).add(relative_source)
        for constant_name in constant_route_names:
            paths.setdefault(string_constants[constant_name], set()).add(relative_source)

        for path, method in re.findall(
            r'\.route\(\s*"([^"]+)"\s*,\s*(?:axum::routing::)?(get|post|patch|delete|put|head|options)\(',
            body,
            re.S,
        ):
            methods.setdefault(path, set()).add(method.upper())
        for constant_name, method in re.findall(
            r'^\s*\.route\(\s*([A-Z][A-Z0-9_]*)\s*,\s*(?:axum::routing::)?(get|post|patch|delete|put|head|options)\(',
            body,
            re.M | re.S,
        ):
            methods.setdefault(string_constants[constant_name], set()).add(method.upper())

    registry = json.loads(
        (
            ROOT
            / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"
        ).read_text()
    )
    agent_paths = {item["path"] for item in registry["operations"]}
    for operation in registry["operations"]:
        method = operation.get("method")
        if method:
            methods.setdefault(operation["path"], set()).add(method.upper())
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
                "methods": sorted(methods.get(path, set())),
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
    api_lines = [
        "# Current API Route Inventory",
        "",
        "Generated from current Axum route declarations plus the Spec135/Spec141 operation registry. Explicit inline cfg(test) modules are excluded. This source inventory does not prove HTTP mounting, permissions, or installed availability. It is release-gated; do not edit route rows manually.",
        "",
        f"- Classified paths: `{len(classifications)}`",
        f"- Agent eligible: `{counts.get('agent_eligible', 0)}`",
        f"- Operator only: `{counts.get('operator_only', 0)}`",
        f"- Public health/pairing: `{counts.get('public_health', 0) + counts.get('public_pairing', 0)}`",
        f"- Internal: `{counts.get('internal', 0)}`",
        "",
        "## Release-current architecture",
        "",
        "Exact authority is `project_root + continuity_id`; worktrees are typed working subpaths. Agent discovery is progressive through the Agent Card, tool search/describe/graph/bundle, and strict schemas. Silent Sessions are daemon-native. Mission Canvas and Work Rail bind scoped Work Surfaces, connectors, domain projections, UIAI context, and adaptive generated UI to canonical operations.",
        "",
        "Machine authority: [`route-classification.json`](../contracts/spec141/generated-capability-v2/route-classification.json), [`rest-agent-operations.json`](../contracts/spec141/generated-capability-v2/rest-agent-operations.json), and [`pi-tools.json`](../contracts/spec141/generated-capability-v2/pi-tools.json). Human per-tool references: [`docs/focusa-tools/tools/`](../focusa-tools/tools/).",
        "",
        "## Registered routes",
        "",
    ]
    for item in classifications:
        method_labels = item["methods"] or ["ROUTE"]
        routes = ", ".join(f"`{method} {item['path']}`" for method in method_labels)
        source_refs = ", ".join(f"`{source}`" for source in item["sources"])
        api_lines.extend(
            [
                f"### `{item['path']}`",
                "",
                f"- Methods: {routes}",
                f"- Classification: `{item['classification']}`",
                f"- Rationale: {item['rationale']}",
                f"- Sources: {source_refs}",
                f"- Agent operations: {', '.join(f'`{ref}`' for ref in item['operation_refs']) or 'none'}",
                "",
            ]
        )
    api_body = "\n".join(api_lines).rstrip() + "\n"
    if args.check:
        drift = []
        if not OUTPUT.exists() or OUTPUT.read_text() != body:
            drift.append(str(OUTPUT.relative_to(ROOT)))
        if not API_REFERENCE.exists() or API_REFERENCE.read_text() != api_body:
            drift.append(str(API_REFERENCE.relative_to(ROOT)))
        if drift:
            print(f"Spec141 route/API reference drift: {', '.join(drift)}", flush=True)
            return 1
    else:
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(body)
        API_REFERENCE.parent.mkdir(parents=True, exist_ok=True)
        API_REFERENCE.write_text(api_body)
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
