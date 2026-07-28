#!/usr/bin/env python3
"""Regression guard for GitHub #66 DXUX Pi argument propagation."""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
CONTRACTS = ROOT / "apps/pi-extension/src/tool-contracts.ts"
source = TOOLS.read_text()
contracts = CONTRACTS.read_text()

for name, field in (
    ("focusa_dxux_requirement", "id"),
    ("focusa_dxux_explain", "failure"),
):
    block_match = re.search(
        rf'name: "{name}"(?P<body>.*?)(?=\n\s*pi\.registerTool\(|\Z)',
        source,
        re.DOTALL,
    )
    assert block_match, f"missing {name} registration"
    block = block_match.group("body")
    assert "execute: async (_toolCallId: string, params: any)" in block, (
        f"{name} must receive Pi execute(toolCallId, params, ...) arguments"
    )
    assert f'validateNoExtraKeys("{name}", params, ["{field}"])' in block
    assert f"keyCheck.value.{field}" in block, f"{name} must forward validated {field} unchanged"
    assert f"{field}: Type.String({{" in block, f"{name} schema must require bounded {field}"
    assert re.search(rf'name: "{name}".*?api_routes: \[[^\]]+\]', contracts, re.DOTALL), (
        f"{name} generated contract must preserve route parity"
    )

assert "focusa_tool_requirement" not in source, "recovery guidance references nonexistent tool"
assert '"focusa_tool_describe"' in source, "schema recovery must resolve through registered discovery"
assert "expected_schema" in source
assert "validation_errors" in source
assert "recovery_hint" in source

print("Spec105 DXUX Pi binding regression: PASS")
