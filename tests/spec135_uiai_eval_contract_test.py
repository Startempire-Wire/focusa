#!/usr/bin/env python3
"""Validate F11 UIAI Engine Eval contracts and the first real browser artifact."""

from __future__ import annotations

import json
from pathlib import Path
from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
scenario_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_scenario.v1.schema.json").read_text()
)
result_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_result.v1.schema.json").read_text()
)
scenario = json.loads(
    (BUNDLE / "uiai-eval.alpha0-generated-ui.scenario.json").read_text()
)
result = json.loads((BUNDLE / "uiai-eval.alpha0-generated-ui.result.json").read_text())
proof = (ROOT / "packages/a2ui-renderer/proof/main.ts").read_text()
proof_html = (ROOT / "packages/a2ui-renderer/proof/index.html").read_text()
renderer_package = json.loads(
    (ROOT / "packages/a2ui-renderer/package.json").read_text()
)
lock_text = (ROOT / "packages/a2ui-renderer/package-lock.json").read_text().lower()

Draft202012Validator.check_schema(scenario_schema)
Draft202012Validator.check_schema(result_schema)
Draft202012Validator(scenario_schema, format_checker=FormatChecker()).validate(scenario)
Draft202012Validator(result_schema, format_checker=FormatChecker()).validate(result)

assert scenario["scenario_id"] == result["scenario_id"]
assert result["status"] == "passed"
assert len(scenario["steps"]) == len(result["step_results"]) == 8
assert {step["step_id"] for step in scenario["steps"]} == {
    step["step_id"] for step in result["step_results"]
}
assert all(step["status"] == "passed" for step in result["step_results"])
assert len(result["screenshots"]) == 2
assert len(result["focusa_evidence_refs"]) >= 3
assert result["accessibility_report_ref"]
assert result["failure_class"] is None and result["recovery_action"] is None
assert any("console_errors=0" in item["summary"] for item in result["diagnostics"])
assert any("failed_requests=0" in item["summary"] for item in result["diagnostics"])
assert "browser-diagnostics:2026-07-20T09:21:34.287Z" in result["focusa_evidence_refs"]

for marker in (
    'allowedActionNames: new Set(["context.review"])',
    'name: "context.review"',
    'name: "unknown.mutate"',
    'component: "UntrustedGeneratedWidget"',
    'project_root: "/example/focusa"',
    'continuity_id: "focusa-cont-alpha0-eval"',
    'attachment_id: "attachment:context-alpha0"',
):
    assert marker in proof
assert 'aria-live="polite"' in proof_html
assert 'aria-label="Focusa generated surface"' in proof_html
assert renderer_package["scripts"]["proof:build"].startswith("vite build")
assert renderer_package["devDependencies"]["vite"] == "6.4.1"
assert "playwright" not in lock_text

print(
    "Spec 135 F11 UIAI Eval: PASS (typed contracts, action/recovery, responsive/a11y, bounded diagnostics evidence)"
)
