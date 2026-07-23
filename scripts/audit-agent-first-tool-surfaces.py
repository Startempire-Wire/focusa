#!/usr/bin/env python3
"""Audit Focusa tool surfaces for agent-first machine readability.

Default mode reports gaps without failing so it can bootstrap remediation.
Use --strict to fail while release-gating findings remain.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def text(path: str) -> str:
    return (ROOT / path).read_text()


def finding(code: str, severity: str, surface: str, message: str, evidence: dict, remediation: str) -> dict:
    return {
        "code": code,
        "severity": severity,
        "surface": surface,
        "message": message,
        "evidence": evidence,
        "remediation": remediation,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", dest="json_path")
    parser.add_argument("--markdown", dest="markdown_path")
    parser.add_argument("--strict", action="store_true")
    args = parser.parse_args()

    contracts_path = ROOT / "docs/current/focusa-tool-contracts.json"
    contracts_doc = json.loads(contracts_path.read_text())
    contracts = contracts_doc["contracts"]
    contract_names = {item["name"] for item in contracts}
    tool_docs = list((ROOT / "docs/focusa-tools/tools").glob("*.md"))
    tools_src = text("apps/pi-extension/src/tools.ts")
    contract_src = text("apps/pi-extension/src/tool-contracts.ts")
    mcp_src = text("crates/focusa-api/src/routes/mcp.rs")
    cli_help_src = text("crates/focusa-cli/src/commands/help.rs")
    cli_main_src = text("crates/focusa-cli/src/main.rs")
    rust_api_src = "\n".join(p.read_text(errors="replace") for p in (ROOT / "crates/focusa-api/src").rglob("*.rs"))

    operation_registry = json.loads(
        text("docs/contracts/spec135/generated-contract-v1/operation-registry.json")
    )
    openapi = json.loads(text("docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"))
    operations = operation_registry["operations"]

    route_paths = set(re.findall(r'\.route\(\s*"([^"]+)"', rust_api_src, re.S))
    operation_paths = {item["path"] for item in operations}
    missing_operation_docs = sorted(
        {item["docs_ref"] for item in operations if not (ROOT / item["docs_ref"]).exists()}
    )

    schema_refs = {
        value
        for item in operations
        for value in item.get("contracts", {}).values()
        if isinstance(value, str) and value.startswith("focusa.")
    }
    openapi_schema_names = set(openapi.get("components", {}).get("schemas", {}))
    normalized_schema_refs = {value.replace(".", "_") for value in schema_refs}

    generic_when = sum(
        "when its specific Focusa state or workflow surface is the narrowest tool" in p.read_text()
        for p in tool_docs
    )
    docs_with_examples = sum("## Example usage" in p.read_text() for p in tool_docs)
    docs_with_input = sum(
        bool(re.search(r"Input schema|Parameters|Required arguments", p.read_text(), re.I))
        for p in tool_docs
    )
    docs_with_dependency = sum(
        bool(re.search(r"^## (Dependency|Prerequisite|Sequence|Workflow)", p.read_text(), re.M | re.I))
        for p in tool_docs
    )

    validator = subprocess.run(
        ["node", "scripts/validate-focusa-tool-contracts.mjs"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )

    inventory_match = re.search(r"fn inventory_lines\(\).*?vec!\[(.*?)\n\s*\]", cli_help_src, re.S)
    cli_inventory_count = len(re.findall(r'^\s*"focusa ', inventory_match.group(1), re.M)) if inventory_match else 0
    commands_match = re.search(r"enum Commands \{(.*?)\n\}", cli_main_src, re.S)
    cli_top_commands = (
        len(re.findall(r"^\s{4}(?:#\[[^\n]+\]\s*)*([A-Z]\w*)", commands_match.group(1), re.M))
        if commands_match
        else 0
    )

    mcp_names = set(re.findall(r'"name"\s*:\s*"(focusa\.[^"]+)"', mcp_src))
    typebox_properties = len(re.findall(r"Type\.(?:String|Boolean|Number|Integer|Array|Object|Union|Optional)\(", tools_src))
    parameter_descriptions = len(re.findall(r"description\s*:", tools_src))
    strict_objects = len(re.findall(r"additionalProperties\s*:\s*false", tools_src))
    output_schemas = len(re.findall(r"outputSchema|output_schema|resultSchema|result_schema", tools_src + contract_src))
    next_tool_overrides = 0
    if "const TOOL_NEXT_TOOLS:" in contract_src:
        block = contract_src.split("const TOOL_NEXT_TOOLS:", 1)[1].split("\n};", 1)[0]
        next_tool_overrides = len(re.findall(r"^\s*focusa_[a-z0-9_]+:", block, re.M))

    projected_contract_fields = set(contracts[0]) if contracts else set()
    required_agent_fields = {
        "input_schema",
        "output_schema",
        "error_schema",
        "examples",
        "anti_examples",
        "dependencies",
        "skill_refs",
        "operation_version",
        "deprecation",
        "cost_hint",
        "latency_hint",
    }
    absent_contract_fields = sorted(required_agent_fields - projected_contract_fields)

    stale_count_docs = []
    for path in [
        "docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md",
        "docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md",
        "docs/current/PI_EXTENSION_FINAL_TOOLSET_AUDIT.md",
        "docs/current/TOOL_RELIABILITY_AUDIT.md",
    ]:
        body = text(path)
        stated = [int(v) for v in re.findall(r"\b(\d{2,3})\s+(?:registered\s+)?(?:Focusa\s+)?tools?\b", body, re.I)]
        if stated and any(value != len(contracts) for value in stated):
            stale_count_docs.append({"path": path, "stated_counts": sorted(set(stated))})

    metrics = {
        "pi_registered_tools": len(contract_names),
        "tool_contracts": len(contracts),
        "per_tool_docs": len(tool_docs),
        "tool_families": dict(sorted(Counter(item["family"] for item in contracts).items())),
        "contract_json_validator_passed": validator.returncode == 0,
        "generic_tool_docs": generic_when,
        "docs_with_examples": docs_with_examples,
        "docs_with_explicit_input_schema": docs_with_input,
        "docs_with_dependency_section": docs_with_dependency,
        "pi_typebox_nodes": typebox_properties,
        "pi_schema_description_tokens": parameter_descriptions,
        "pi_strict_object_schemas": strict_objects,
        "pi_output_schemas": output_schemas,
        "tools_with_explicit_next_tool_graph": next_tool_overrides,
        "mcp_exposed_tools": len(mcp_names),
        "mcp_tool_names": sorted(mcp_names),
        "cli_top_level_commands": cli_top_commands,
        "cli_machine_help_inventory_entries": cli_inventory_count,
        "api_route_paths": len(route_paths),
        "agent_operation_registry_entries": len(operations),
        "agent_operation_openapi_paths": len(openapi.get("paths", {})),
        "operation_schema_refs": len(schema_refs),
        "materialized_openapi_schema_refs": len(normalized_schema_refs & openapi_schema_names),
        "missing_operation_docs_refs": len(missing_operation_docs),
        "tool_contracts_without_api_route": sum(not item["api_routes"] for item in contracts),
        "tool_contracts_without_cli_command": sum(not item["cli_commands"] for item in contracts),
    }

    findings = []
    if validator.returncode:
        findings.append(finding(
            "AF-TOOL-001", "critical", "tool_contract_registry",
            "Generated JSON contract registry has drifted from the TypeScript authority.",
            {"validator_stderr": validator.stderr.strip(), "tools": len(contracts)},
            "Generate all projections from one canonical descriptor and fail CI on drift.",
        ))
    if absent_contract_fields:
        findings.append(finding(
            "AF-TOOL-002", "critical", "tool_contract_registry",
            "Public machine-readable tool contracts omit invocation and composition fields.",
            {"absent_fields": absent_contract_fields},
            "Publish strict input/output/error schemas, examples, dependencies, skill refs, versions, deprecation, and budget hints per tool.",
        ))
    if output_schemas == 0:
        findings.append(finding(
            "AF-TOOL-003", "critical", "pi_tools",
            "Pi tool registrations/contracts expose no per-tool output schemas.",
            {"tools": len(contracts), "output_schemas": output_schemas},
            "Add structured output schemas and validate tool_result_v1 details for every tool.",
        ))
    if strict_objects < len(contracts):
        findings.append(finding(
            "AF-TOOL-004", "high", "pi_tools",
            "Most Pi input object schemas do not explicitly reject unknown properties.",
            {"tools": len(contracts), "strict_objects": strict_objects},
            "Generate strict schemas with additionalProperties=false and conditional requirement tests.",
        ))
    if len(mcp_names) < len(operations):
        findings.append(finding(
            "AF-TOOL-005", "critical", "mcp",
            "MCP exposes only a health probe rather than the curated agent operation catalog.",
            {"mcp_tools": sorted(mcp_names), "agent_operations": len(operations)},
            "Generate paginated MCP tools/list and tools/call from the canonical operation registry, including outputSchema, structuredContent, annotations, tasks, and listChanged.",
        ))
    if len(operations) < len(route_paths):
        findings.append(finding(
            "AF-TOOL-006", "high", "rest_openapi",
            "The agent operation/OpenAPI registry covers only a subset of API routes and does not classify every remaining route as agent-eligible or internal.",
            {"api_routes": len(route_paths), "agent_operations": len(operations)},
            "Classify every route; fully contract agent-eligible routes and explicitly mark internal/operator-only routes.",
        ))
    if missing_operation_docs:
        findings.append(finding(
            "AF-TOOL-007", "critical", "operation_docs",
            "Every operation family docs_ref currently points to a missing document.",
            {"missing_count": len(missing_operation_docs), "refs": missing_operation_docs},
            "Materialize and validate every operation docs_ref from the canonical descriptor.",
        ))
    if generic_when or docs_with_input < len(tool_docs) or docs_with_dependency < len(tool_docs):
        findings.append(finding(
            "AF-TOOL-008", "critical", "tool_docs",
            "Per-tool documentation is structurally present but frequently generic and incomplete for deep agent operation.",
            {
                "docs": len(tool_docs),
                "generic": generic_when,
                "with_examples": docs_with_examples,
                "with_input_schema": docs_with_input,
                "with_dependencies": docs_with_dependency,
            },
            "Generate specific parameter tables, positive/negative examples, failure recovery, prerequisites, dependency chains, and workflow position for every tool.",
        ))
    if cli_inventory_count < cli_top_commands:
        findings.append(finding(
            "AF-TOOL-009", "high", "cli",
            "Machine-readable CLI help is a curated subset rather than an exhaustive generated command schema.",
            {"top_level_commands": cli_top_commands, "help_inventory": cli_inventory_count},
            "Generate JSON command schemas, flags, defaults, examples, effects, and migration metadata from Clap authority.",
        ))
    if stale_count_docs:
        findings.append(finding(
            "AF-TOOL-010", "high", "internal_docs",
            "Canonical/current audit documents contain stale tool totals.",
            {"current_tools": len(contracts), "documents": stale_count_docs},
            "Replace hand-maintained totals with generated values and freshness checks.",
        ))
    if generic_when:
        findings.append(finding(
            "AF-TOOL-011", "high", "progressive_discovery",
            "Focusa lacks a dedicated search/describe/graph surface for cold-loading tool schemas and uses family-generic affordances for many tools.",
            {"generic_affordances": generic_when, "total_tools": len(contracts)},
            "Add tool search, describe, dependency graph, namespaced bundles, digest/listChanged, and token-budgeted deferred schema loading.",
        ))
    findings.append(finding(
        "AF-TOOL-012", "critical", "cross_harness_interop",
        "No single versioned Agent Card/capability manifest projects equivalent Pi, MCP, OpenAI, CLI, REST, skill, and browser affordances.",
        {"pi_tools": len(contracts), "agent_operations": len(operations), "mcp_tools": len(mcp_names)},
        "Generate a signed/versioned Focusa Agent Capability Manifest with protocol bindings, auth, skills, examples, compatibility, and conformance refs.",
    ))
    findings.append(finding(
        "AF-TOOL-013", "high", "browser_interop",
        "Browser interoperability is diagnostics-intake centric; Focusa lacks a machine-readable WebMCP/UIAI capability bridge and browser workflow dependency graph.",
        {"current_focusa_browser_tool": "focusa_browser_diagnostics_intake"},
        "Add UIAI/WebMCP capability discovery, session-isolated browser operation descriptors, evidence contracts, and browser-to-Workpoint workflow graphs.",
    ))
    findings.append(finding(
        "AF-TOOL-014", "high", "agent_evaluation",
        "Current tests validate contracts and surfaces but do not score weak-to-strong agents on tool selection, parameter completion, recovery, and cross-tool workflows.",
        {"existing_static_audits": 7},
        "Add dumb-agent conformance fixtures, golden workflow tasks, invalid-call repair tests, token-cost budgets, and cross-harness behavioral parity evaluation.",
    ))

    severity_counts = dict(sorted(Counter(item["severity"] for item in findings).items()))
    report = {
        "schema": "focusa.agent_first_tool_audit.v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "project_root": str(ROOT),
        "status": "gaps_found" if findings else "pass",
        "release_gate": "fail" if any(item["severity"] in {"critical", "high"} for item in findings) else "pass",
        "metrics": metrics,
        "severity_counts": severity_counts,
        "findings": findings,
        "external_benchmark_refs": [
            "https://www.anthropic.com/engineering/advanced-tool-use",
            "https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools",
            "https://platform.openai.com/docs/guides/function-calling",
            "https://modelcontextprotocol.io/specification/2025-11-25/server/tools",
            "https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks",
            "https://modelcontextprotocol.io/specification/2025-11-25/client/elicitation",
            "https://agentskills.io/specification",
            "https://a2a-protocol.org/latest/specification/",
            "https://webmachinelearning.github.io/webmcp/",
            "https://github.com/open-telemetry/semantic-conventions-genai",
            "https://llmstxt.org/",
        ],
    }

    rendered = json.dumps(report, indent=2) + "\n"
    if args.json_path:
        Path(args.json_path).write_text(rendered)
    else:
        print(rendered, end="")

    if args.markdown_path:
        lines = [
            "# Spec141 Focusa Agent-First Tool Surface Audit",
            "",
            f"Generated: `{report['generated_at']}`",
            "",
            f"**Status:** `{report['status']}`",
            f"**Release gate:** `{report['release_gate']}`",
            "",
            "## Metrics",
            "",
        ]
        lines.extend(f"- **{key}:** `{value}`" for key, value in metrics.items())
        lines.extend(["", "## Findings", ""])
        for item in findings:
            lines.extend([
                f"### {item['code']} — {item['severity'].upper()} — {item['surface']}",
                "",
                item["message"],
                "",
                f"**Remediation:** {item['remediation']}",
                "",
                "```json",
                json.dumps(item["evidence"], indent=2),
                "```",
                "",
            ])
        lines.extend(["## External benchmark sources", ""])
        lines.extend(f"- {ref}" for ref in report["external_benchmark_refs"])
        Path(args.markdown_path).write_text("\n".join(lines) + "\n")

    if args.strict and report["release_gate"] == "fail":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
