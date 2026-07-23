#!/usr/bin/env python3
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/audit-agent-first-tool-surfaces.py"
RELEASE_WORKFLOW = ROOT / ".github/workflows/release.yml"

workflow = RELEASE_WORKFLOW.read_text()
assert "open-issue-release-gate:" in workflow
assert 'startswith("release-gate:")' in workflow
assert "needs: [rust-check, open-issue-release-gate]" in workflow

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
    assert report["schema"] == "focusa.agent_first_tool_audit.v1"
    assert report["status"] in {"pass", "gaps_found"}
    assert report["release_gate"] in {"pass", "fail"}
    metrics = report["metrics"]
    assert metrics["pi_registered_tools"] == metrics["tool_contracts"]
    assert metrics["tool_contracts"] == metrics["per_tool_docs"]
    assert metrics["tools_with_explicit_next_tool_graph"] == metrics["tool_contracts"]
    assert metrics["materialized_openapi_schema_refs"] == metrics["operation_schema_refs"]
    assert metrics["agent_operation_openapi_paths"] == metrics["agent_operation_registry_entries"]
    assert "## Findings" in markdown_path.read_text()
    assert report["external_benchmark_refs"]

    codes = {item["code"] for item in report["findings"]}
    if report["release_gate"] == "fail":
        assert {
            "AF-TOOL-002",
            "AF-TOOL-003",
            "AF-TOOL-005",
            "AF-TOOL-007",
            "AF-TOOL-008",
            "AF-TOOL-012",
            "AF-TOOL-013",
            "AF-TOOL-014",
        } <= codes
        strict = subprocess.run(
            ["python3", str(SCRIPT), "--strict", "--json", str(Path(tmp) / "strict.json")],
            cwd=ROOT,
            check=False,
        )
        assert strict.returncode == 1

print("Spec141 agent-first tool audit contract: PASS")
