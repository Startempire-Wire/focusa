#!/usr/bin/env python3
import json
import importlib.util
import re
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/audit-agent-first-tool-surfaces.py"
RELEASE_WORKFLOW = ROOT / ".github/workflows/release.yml"
route_spec = importlib.util.spec_from_file_location(
    "route_classification", ROOT / "scripts/generate-agent-route-classification.py"
)
route_classifier = importlib.util.module_from_spec(route_spec)
route_spec.loader.exec_module(route_classifier)
subprocess.run(
    ["python3", str(ROOT / "tests/spec141_route_test_module_exclusion_test.py")],
    cwd=ROOT,
    check=True,
)

workflow = RELEASE_WORKFLOW.read_text()
# Tracking state is not installed evidence: collecting proof must not depend
# on prematurely closing the issue that requires that proof.
assert "open-issue-release-gate:" not in workflow
assert "needs: [rust-check, final-release-gap-gate, pull-request-release-gate, version-policy]" in workflow
assert "predeployment-compatibility-canary:" in workflow
assert "needs: predeployment-compatibility-canary" in workflow
deploy_workflow = (ROOT / ".github/workflows/deploy-live-daemon.yml").read_text()
proof_steps = [
    "Verify installed distribution parity",
    "Gate OTA installability against signed deployed release",
    "Settle signed release manifest after OTA acceptance",
    "Promote accepted stable release to Latest",
]
positions = [deploy_workflow.index(step) for step in proof_steps]
assert positions == sorted(positions)

with tempfile.TemporaryDirectory(prefix="focusa-spec141-") as tmp:
    report_path = Path(tmp) / "audit.json"
    markdown_path = Path(tmp) / "audit.md"
    subprocess.run(
        [
            "python3",
            str(SCRIPT),
            "--json",
            str(report_path),
            "--markdown",
            str(markdown_path),
        ],
        cwd=ROOT,
        check=True,
    )
    report = json.loads(report_path.read_text())
    subprocess.run(
        [
            "python3",
            str(ROOT / "scripts/generate-agent-route-classification.py"),
            "--check",
        ],
        cwd=ROOT,
        check=True,
    )
    classification = json.loads(
        (
            ROOT
            / "docs/contracts/spec141/generated-capability-v2/route-classification.json"
        ).read_text()
    )
    classified_paths = {item["path"] for item in classification["routes"]}
    constant_route_paths = set()
    for source in sorted((ROOT / "crates/focusa-api/src").rglob("*.rs")):
        body = route_classifier.without_inline_test_modules(source.read_text(errors="strict"))
        constants = dict(
            re.findall(
                r'^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*&str\s*=\s*"([^"]+)"\s*;',
                body,
                re.M,
            )
        )
        route_constants = set(
            re.findall(r'^\s*\.route\(\s*([A-Z][A-Z0-9_]*)\s*,', body, re.M)
        )
        assert route_constants <= constants.keys(), (
            f"{source.relative_to(ROOT)} has unresolved route constants: "
            f"{sorted(route_constants - constants.keys())}"
        )
        constant_route_paths.update(constants[name] for name in route_constants)
    assert constant_route_paths <= classified_paths, (
        "constant-backed Axum routes missing from classification: "
        f"{sorted(constant_route_paths - classified_paths)}"
    )
    assert "/v1/task-plans/mutate" in constant_route_paths

    assert report["schema"] == "focusa.agent_first_tool_audit.v1"
    assert report["status"] in {"pass", "gaps_found"}
    assert report["release_gate"] in {"pass", "fail"}
    metrics = report["metrics"]
    assert metrics["pi_registered_tools"] == metrics["tool_contracts"]
    assert metrics["tool_contracts"] == metrics["per_tool_docs"]
    assert metrics["tools_with_explicit_next_tool_graph"] == metrics["tool_contracts"]
    assert (
        metrics["materialized_openapi_schema_refs"] == metrics["operation_schema_refs"]
    )
    assert (
        metrics["agent_operation_openapi_paths"]
        == metrics["agent_operation_registry_entries"]
    )
    assert metrics["contract_json_validator_passed"] is True
    assert metrics["capability_descriptor_generator_passed"] is True
    assert metrics["capability_descriptors_v2"] == metrics["tool_contracts"]
    assert (
        metrics["capability_descriptors_with_strict_input"] == metrics["tool_contracts"]
    )
    assert (
        metrics["capability_descriptors_with_output_schema"]
        == metrics["tool_contracts"]
    )
    assert metrics["agent_card_present"] is True
    assert metrics["agent_card_pi_tool_count"] == metrics["tool_contracts"]
    assert metrics["agent_card_pi_tool_docs_count"] == metrics["per_tool_docs"]
    assert metrics["agent_card_skill_count"] == metrics["installed_root_skills"]
    assert metrics["agent_card_runbook_count"] == metrics["skill_runbook_count"]
    assert metrics["skill_runbook_coverage_complete"] is True
    assert "## Findings" in markdown_path.read_text()
    assert report["external_benchmark_refs"]

    codes = {item["code"] for item in report["findings"]}
    assert {"AF-TOOL-002", "AF-TOOL-003", "AF-TOOL-004", "AF-TOOL-012"}.isdisjoint(
        codes
    )
    if report["release_gate"] == "fail":
        assert {"AF-TOOL-014"} <= codes
        assert {
            "AF-TOOL-005",
            "AF-TOOL-007",
            "AF-TOOL-008",
            "AF-TOOL-009",
            "AF-TOOL-011",
            "AF-TOOL-013",
        }.isdisjoint(codes)
        strict = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "--strict",
                "--json",
                str(Path(tmp) / "strict.json"),
            ],
            cwd=ROOT,
            check=False,
        )
        assert strict.returncode == 1

print("Spec141 agent-first tool audit contract: PASS")
