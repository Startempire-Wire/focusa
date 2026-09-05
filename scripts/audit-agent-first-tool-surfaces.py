#!/usr/bin/env python3
"""Audit Focusa tool surfaces for agent-first machine readability.

Default mode reports gaps without failing so it can bootstrap remediation.
Use --strict to fail while release-gating findings remain.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ROUTE_SPEC = importlib.util.spec_from_file_location(
    "route_classification", ROOT / "scripts/generate-agent-route-classification.py"
)
ROUTE_CLASSIFIER = importlib.util.module_from_spec(ROUTE_SPEC)
ROUTE_SPEC.loader.exec_module(ROUTE_CLASSIFIER)


def text(path: str) -> str:
    return (ROOT / path).read_text()


def finding(
    code: str,
    severity: str,
    surface: str,
    message: str,
    evidence: dict,
    remediation: str,
) -> dict:
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
    capability_dir = ROOT / "docs/contracts/spec141/generated-capability-v2"
    capability_path = capability_dir / "agent-capability-descriptors.json"
    capability_registry = (
        json.loads(capability_path.read_text())
        if capability_path.exists()
        else {"descriptors": []}
    )
    capability_descriptors = capability_registry.get("descriptors", [])
    agent_card_path = capability_dir / "agent-card.json"
    agent_card = (
        json.loads(agent_card_path.read_text()) if agent_card_path.exists() else None
    )
    mcp_projection_path = capability_dir / "mcp-tools.json"
    mcp_projection = (
        json.loads(mcp_projection_path.read_text())
        if mcp_projection_path.exists()
        else {"tools": []}
    )
    cli_projection_path = capability_dir / "cli-commands.json"
    cli_projection = (
        json.loads(cli_projection_path.read_text())
        if cli_projection_path.exists()
        else {"commands": []}
    )
    skill_coverage_path = ROOT / "docs/evidence/141-focusa-skill-runbook-coverage.json"
    skill_coverage = (
        json.loads(skill_coverage_path.read_text())
        if skill_coverage_path.exists()
        else {}
    )
    public_alignment_path = (
        ROOT / "docs/evidence/141-focusa-latest-spec-public-doc-alignment.json"
    )
    public_alignment = (
        json.loads(public_alignment_path.read_text())
        if public_alignment_path.exists()
        else {}
    )
    conformance_path = ROOT / "docs/evidence/141-focusa-agent-conformance-result.json"
    conformance = (
        json.loads(conformance_path.read_text()) if conformance_path.exists() else {}
    )
    tool_docs = list((ROOT / "docs/focusa-tools/tools").glob("*.md"))
    tools_src = text("apps/pi-extension/src/tools.ts")
    contract_src = text("apps/pi-extension/src/tool-contracts.ts")
    mcp_src = text("crates/focusa-api/src/routes/mcp.rs")
    cli_help_src = text("crates/focusa-cli/src/commands/help.rs")
    cli_main_src = text("crates/focusa-cli/src/main.rs")
    rust_api_sources = sorted((ROOT / "crates/focusa-api/src").rglob("*.rs"))

    operation_registry = json.loads(
        text("docs/contracts/spec135/generated-contract-v1/operation-registry.json")
    )
    openapi = json.loads(
        text("docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json")
    )
    operations = operation_registry["operations"]

    route_paths = set()
    for source in rust_api_sources:
        body = ROUTE_CLASSIFIER.without_inline_test_modules(source.read_text(errors="strict"))
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
            relative_source = source.relative_to(ROOT)
            raise ValueError(
                f"{relative_source}: unresolved route path constants: {', '.join(unresolved)}"
            )
        route_paths.update(re.findall(r'\.route\(\s*"([^"]+)"', body, re.S))
        route_paths.update(string_constants[name] for name in constant_route_names)

    route_classification_path = capability_dir / "route-classification.json"
    route_classification = (
        json.loads(route_classification_path.read_text())
        if route_classification_path.exists()
        else {"routes": []}
    )
    classified_route_paths = {
        item.get("path")
        for item in route_classification.get("routes", [])
        if item.get("path")
    }
    missing_operation_docs = sorted(
        {
            item["docs_ref"]
            for item in operations
            if not (ROOT / item["docs_ref"]).exists()
        }
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
        "when its specific Focusa state or workflow surface is the narrowest tool"
        in p.read_text()
        for p in tool_docs
    )
    docs_with_examples = sum(
        bool(re.search(r"^## Example(?: usage)?$", p.read_text(), re.M | re.I))
        for p in tool_docs
    )
    docs_with_input = sum(
        bool(
            re.search(
                r"Input schema|Parameters|Required arguments", p.read_text(), re.I
            )
        )
        for p in tool_docs
    )
    docs_with_dependency = sum(
        bool(
            re.search(
                r"^## (Dependencies?|Prerequisites?|Sequence|Workflow)",
                p.read_text(),
                re.M | re.I,
            )
        )
        for p in tool_docs
    )

    validator = subprocess.run(
        ["node", "scripts/validate-focusa-tool-contracts.mjs"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    descriptor_generator = subprocess.run(
        ["bun", "scripts/generate-agent-capability-descriptors.ts", "--check"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )

    inventory_match = re.search(
        r"fn inventory_lines\(\).*?vec!\[(.*?)\n\s*\]", cli_help_src, re.S
    )
    cli_inventory_count = (
        len(re.findall(r'^\s*"focusa ', inventory_match.group(1), re.M))
        if inventory_match
        else 0
    )
    commands_match = re.search(r"enum Commands \{(.*?)\n\}", cli_main_src, re.S)
    cli_top_commands = (
        len(
            re.findall(
                r"^\s{4}(?:#\[[^\n]+\]\s*)*([A-Z]\w*)", commands_match.group(1), re.M
            )
        )
        if commands_match
        else 0
    )

    mcp_names = {
        item.get("name")
        for item in mcp_projection.get("tools", [])
        if isinstance(item, dict) and item.get("name")
    }
    expected_mcp_tools = sum(bool(item.get("api_routes")) for item in contracts)
    generated_cli_commands = cli_projection.get("commands", [])
    typebox_properties = len(
        re.findall(
            r"Type\.(?:String|Boolean|Number|Integer|Array|Object|Union|Optional)\(",
            tools_src,
        )
    )
    parameter_descriptions = len(re.findall(r"description\s*:", tools_src))
    strict_objects = len(re.findall(r"additionalProperties\s*:\s*false", tools_src))
    output_schemas = len(
        re.findall(
            r"outputSchema|output_schema|resultSchema|result_schema",
            tools_src + contract_src,
        )
    )
    next_tool_overrides = 0
    if "const TOOL_NEXT_TOOLS:" in contract_src:
        block = contract_src.split("const TOOL_NEXT_TOOLS:", 1)[1].split("\n};", 1)[0]
        next_tool_overrides = len(re.findall(r"^\s*focusa_[a-z0-9_]+:", block, re.M))

    required_agent_fields = {
        "input_schema",
        "output_schema",
        "error_schema",
        "examples",
        "anti_examples",
        "dependencies",
        "skill_refs",
        "version",
        "deprecation",
        "cost_hint",
        "latency_hint",
    }
    absent_contract_fields = sorted(
        {
            field
            for descriptor in capability_descriptors
            for field in required_agent_fields
            if field not in descriptor
        }
    )
    if not capability_descriptors:
        absent_contract_fields = sorted(required_agent_fields)
    strict_descriptor_inputs = sum(
        item.get("input_schema", {}).get("type") == "object"
        and item.get("input_schema", {}).get("additionalProperties") is False
        for item in capability_descriptors
    )
    typed_descriptor_outputs = sum(
        bool(item.get("output_schema")) for item in capability_descriptors
    )

    stale_count_docs = []
    for path in [
        "docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md",
        "docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md",
        "docs/current/PI_EXTENSION_FINAL_TOOLSET_AUDIT.md",
        "docs/current/TOOL_RELIABILITY_AUDIT.md",
    ]:
        body = text(path)
        stated = [
            int(v)
            for v in re.findall(
                r"\b(\d{2,3})\s+(?:registered\s+)?(?:Focusa\s+)?tools?\b", body, re.I
            )
        ]
        if stated and any(value != len(contracts) for value in stated):
            stale_count_docs.append(
                {"path": path, "stated_counts": sorted(set(stated))}
            )

    metrics = {
        "pi_registered_tools": len(contract_names),
        "tool_contracts": len(contracts),
        "per_tool_docs": len(tool_docs),
        "tool_families": dict(
            sorted(Counter(item["family"] for item in contracts).items())
        ),
        "contract_json_validator_passed": validator.returncode == 0,
        "capability_descriptor_generator_passed": descriptor_generator.returncode == 0,
        "capability_descriptors_v2": len(capability_descriptors),
        "capability_descriptors_with_strict_input": strict_descriptor_inputs,
        "capability_descriptors_with_output_schema": typed_descriptor_outputs,
        "agent_card_present": agent_card is not None,
        "agent_card_pi_tool_count": (agent_card or {}).get("pi_tool_count", 0),
        "agent_card_pi_tool_docs_count": (agent_card or {}).get("pi_tool_docs_count", 0),
        "agent_card_skill_count": (agent_card or {}).get("skill_count", 0),
        "agent_card_runbook_count": (agent_card or {}).get("runbook_count", 0),
        "generic_tool_docs": generic_when,
        "docs_with_examples": docs_with_examples,
        "docs_with_explicit_input_schema": docs_with_input,
        "docs_with_dependency_section": docs_with_dependency,
        "pi_typebox_nodes": typebox_properties,
        "pi_schema_description_tokens": parameter_descriptions,
        "pi_strict_object_schema_markers": strict_objects,
        "pi_output_schema_markers": output_schemas,
        "tools_with_explicit_next_tool_graph": next_tool_overrides,
        "mcp_exposed_tools": len(mcp_names),
        "mcp_expected_callable_tools": expected_mcp_tools,
        "mcp_tool_names": sorted(mcp_names),
        "cli_top_level_commands": cli_top_commands,
        "cli_machine_help_inventory_entries": cli_inventory_count,
        "cli_generated_agent_commands": len(generated_cli_commands),
        "cli_expected_agent_commands": sum(
            bool(item["cli_commands"]) for item in contracts
        ),
        "api_route_paths": len(route_paths),
        "agent_operation_registry_entries": len(operations),
        "agent_operation_openapi_paths": len(openapi.get("paths", {})),
        "classified_api_route_paths": len(classified_route_paths),
        "unclassified_api_route_paths": len(route_paths - classified_route_paths),
        "operation_schema_refs": len(schema_refs),
        "materialized_openapi_schema_refs": len(
            normalized_schema_refs & openapi_schema_names
        ),
        "missing_operation_docs_refs": len(missing_operation_docs),
        "tool_contracts_without_api_route": sum(
            not item["api_routes"] for item in contracts
        ),
        "tool_contracts_without_cli_command": sum(
            not item["cli_commands"] for item in contracts
        ),
        "installed_root_skills": skill_coverage.get("installed_root_skill_count", 0),
        "packaged_skills": skill_coverage.get("packaged_skill_count", 0),
        "skill_root_packaged_parity": skill_coverage.get("root_packaged_parity", False),
        "skill_runbook_count": skill_coverage.get("runbook_count", 0),
        "skill_runbook_coverage_complete": skill_coverage.get("runbook_coverage_complete", False),
        "latest_spec_public_alignment_count": public_alignment.get("spec_count", 0),
        "agent_conformance_passed": conformance.get("status") == "passed",
        "agent_conformance_levels": len(conformance.get("agent_levels", [])),
    }

    findings = []
    if validator.returncode:
        findings.append(
            finding(
                "AF-TOOL-001",
                "critical",
                "tool_contract_registry",
                "Generated JSON contract registry has drifted from the TypeScript authority.",
                {"validator_stderr": validator.stderr.strip(), "tools": len(contracts)},
                "Generate all projections from one canonical descriptor and fail CI on drift.",
            )
        )
    if absent_contract_fields or len(capability_descriptors) != len(contracts):
        findings.append(
            finding(
                "AF-TOOL-002",
                "critical",
                "tool_contract_registry",
                "Agent Capability Descriptor V2 is incomplete or omits invocation/composition fields.",
                {
                    "absent_fields": absent_contract_fields,
                    "descriptors": len(capability_descriptors),
                    "tools": len(contracts),
                },
                "Publish strict input/output/error schemas, examples, dependencies, skill refs, versions, deprecation, and budget hints per tool.",
            )
        )
    if typed_descriptor_outputs < len(contracts):
        findings.append(
            finding(
                "AF-TOOL-003",
                "critical",
                "pi_tools",
                "Not every Pi capability exposes a generated output schema.",
                {"tools": len(contracts), "output_schemas": typed_descriptor_outputs},
                "Add structured output schemas and validate tool_result_v1 details for every tool.",
            )
        )
    if strict_descriptor_inputs < len(contracts):
        findings.append(
            finding(
                "AF-TOOL-004",
                "high",
                "pi_tools",
                "Not every generated Pi input object schema explicitly rejects unknown properties.",
                {"tools": len(contracts), "strict_objects": strict_descriptor_inputs},
                "Generate strict schemas with additionalProperties=false and conditional requirement tests.",
            )
        )
    if (
        len(mcp_names) < expected_mcp_tools
        or "call_rest_tool" not in mcp_src
        or "listChanged" not in mcp_src
    ):
        findings.append(
            finding(
                "AF-TOOL-005",
                "critical",
                "mcp",
                "MCP does not expose the complete callable generated catalog with scoped invocation.",
                {
                    "mcp_tools": len(mcp_names),
                    "expected_callable_tools": expected_mcp_tools,
                },
                "Generate paginated MCP tools/list and tools/call from the canonical registry, including outputSchema, structuredContent, annotations, scoped REST authority, and listChanged.",
            )
        )
    if route_paths != classified_route_paths:
        findings.append(
            finding(
                "AF-TOOL-006",
                "high",
                "rest_openapi",
                "The route classification projection is missing or drifted from the Axum route inventory.",
                {
                    "api_routes": len(route_paths),
                    "classified_routes": len(classified_route_paths),
                    "unclassified": sorted(route_paths - classified_route_paths),
                    "stale": sorted(classified_route_paths - route_paths),
                },
                "Classify every route; fully contract agent-eligible routes and explicitly mark internal/operator-only routes.",
            )
        )
    if missing_operation_docs:
        findings.append(
            finding(
                "AF-TOOL-007",
                "critical",
                "operation_docs",
                "Every operation family docs_ref currently points to a missing document.",
                {
                    "missing_count": len(missing_operation_docs),
                    "refs": missing_operation_docs,
                },
                "Materialize and validate every operation docs_ref from the canonical descriptor.",
            )
        )
    if (
        generic_when
        or docs_with_input < len(tool_docs)
        or docs_with_dependency < len(tool_docs)
    ):
        findings.append(
            finding(
                "AF-TOOL-008",
                "critical",
                "tool_docs",
                "Per-tool documentation is structurally present but frequently generic and incomplete for deep agent operation.",
                {
                    "docs": len(tool_docs),
                    "generic": generic_when,
                    "with_examples": docs_with_examples,
                    "with_input_schema": docs_with_input,
                    "with_dependencies": docs_with_dependency,
                },
                "Generate specific parameter tables, positive/negative examples, failure recovery, prerequisites, dependency chains, and workflow position for every tool.",
            )
        )
    if len(generated_cli_commands) < sum(
        bool(item["cli_commands"]) for item in contracts
    ):
        findings.append(
            finding(
                "AF-TOOL-009",
                "high",
                "cli",
                "Machine-readable CLI help lacks an exhaustive generated agent command schema.",
                {
                    "top_level_commands": cli_top_commands,
                    "help_inventory": cli_inventory_count,
                    "generated_agent_commands": len(generated_cli_commands),
                    "expected_agent_commands": sum(
                        bool(item["cli_commands"]) for item in contracts
                    ),
                },
                "Generate JSON command schemas, flags, defaults, examples, effects, and migration metadata for every contract with a CLI binding.",
            )
        )
    if stale_count_docs:
        findings.append(
            finding(
                "AF-TOOL-010",
                "high",
                "internal_docs",
                "Canonical/current audit documents contain stale tool totals.",
                {"current_tools": len(contracts), "documents": stale_count_docs},
                "Replace hand-maintained totals with generated values and freshness checks.",
            )
        )
    discovery_tools = {
        "focusa_tool_search",
        "focusa_tool_describe",
        "focusa_tool_graph",
        "focusa_tool_bundle",
        "focusa_agent_card",
    }
    if not discovery_tools.issubset(contract_names):
        findings.append(
            finding(
                "AF-TOOL-011",
                "high",
                "progressive_discovery",
                "Focusa lacks one or more dedicated search/describe/graph/bundle/card surfaces for cold-loading schemas.",
                {
                    "missing_tools": sorted(discovery_tools - contract_names),
                    "generic_affordances": generic_when,
                    "total_tools": len(contracts),
                },
                "Add tool search, describe, dependency graph, namespaced bundles, digest/listChanged, and token-budgeted deferred schema loading.",
            )
        )
    if (
        not agent_card
        or len(capability_descriptors) != len(contracts)
        or descriptor_generator.returncode
    ):
        findings.append(
            finding(
                "AF-TOOL-012",
                "critical",
                "cross_harness_interop",
                "No current generated Agent Card/capability manifest projects equivalent Pi, MCP, OpenAI, CLI, REST, skill, and browser affordances.",
                {
                    "pi_tools": len(contracts),
                    "capability_descriptors": len(capability_descriptors),
                    "agent_card": bool(agent_card),
                    "generator_passed": descriptor_generator.returncode == 0,
                },
                "Generate a signed/versioned Focusa Agent Capability Manifest with protocol bindings, auth, skills, examples, compatibility, and conformance refs.",
            )
        )
    browser_tools = {
        "focusa_browser_capabilities_intake",
        "focusa_browser_workflow_plan",
        "focusa_browser_diagnostics_intake",
    }
    browser_interop_source = ROOT / "crates/focusa-api/src/routes/browser_interop.rs"
    if (
        not browser_tools.issubset(contract_names)
        or not browser_interop_source.exists()
    ):
        findings.append(
            finding(
                "AF-TOOL-013",
                "high",
                "browser_interop",
                "Focusa lacks a complete machine-readable WebMCP/UIAI capability bridge or browser workflow dependency graph.",
                {
                    "missing_tools": sorted(browser_tools - contract_names),
                    "browser_interop_route_module": browser_interop_source.exists(),
                },
                "Add UIAI/WebMCP capability discovery, session-isolated browser operation descriptors, evidence contracts, and browser-to-Workpoint workflow graphs.",
            )
        )
    if (
        conformance.get("status") != "passed"
        or len(conformance.get("agent_levels", [])) < 7
    ):
        findings.append(
            finding(
                "AF-TOOL-014",
                "high",
                "agent_evaluation",
                "Weak-to-strong cross-harness agent conformance evidence is missing or incomplete.",
                {
                    "status": conformance.get("status"),
                    "agent_levels": conformance.get("agent_levels", []),
                },
                "Add dumb-agent conformance fixtures, golden workflow tasks, invalid-call repair tests, token-cost budgets, and cross-harness behavioral parity evaluation.",
            )
        )
    if (
        not skill_coverage.get("root_packaged_parity")
        or not skill_coverage.get("runbook_coverage_complete")
        or skill_coverage.get("installed_root_skill_count", 0) < 22
        or (agent_card or {}).get("skill_count")
        != skill_coverage.get("installed_root_skill_count")
        or (agent_card or {}).get("runbook_count")
        != skill_coverage.get("runbook_count")
        or (agent_card or {}).get("pi_tool_count") != len(contracts)
        or (agent_card or {}).get("pi_tool_docs_count") != len(tool_docs)
    ):
        findings.append(
            finding(
                "AF-TOOL-015",
                "high",
                "skills_runbooks",
                "All-skill/runbook, Agent Card, every-Pi-tool, or root/package parity is incomplete.",
                {"coverage": skill_coverage},
                "Generate complete skill/runbook inventory, every-Pi-tool counts/routes, and exact root/package parity proof.",
            )
        )
    if public_alignment.get("spec_count", 0) < 15 or not public_alignment.get(
        "integrity", {}
    ).get("spec_paths_resolve"):
        findings.append(
            finding(
                "AF-TOOL-016",
                "high",
                "public_docs",
                "Rolling latest-15-spec public documentation alignment is incomplete.",
                {"alignment": public_alignment},
                "Reconcile README, docs index, llms.txt, shipped/planned truth, and latest-spec direction.",
            )
        )

    severity_counts = dict(
        sorted(Counter(item["severity"] for item in findings).items())
    )
    report = {
        "schema": "focusa.agent_first_tool_audit.v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "project_root": str(ROOT),
        "status": "gaps_found" if findings else "pass",
        "release_gate": "fail"
        if any(item["severity"] in {"critical", "high"} for item in findings)
        else "pass",
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
            lines.extend(
                [
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
                ]
            )
        lines.extend(["## External benchmark sources", ""])
        lines.extend(f"- {ref}" for ref in report["external_benchmark_refs"])
        Path(args.markdown_path).write_text("\n".join(lines) + "\n")

    if args.strict and report["release_gate"] == "fail":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
