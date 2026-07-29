#!/usr/bin/env python3
"""Spec 135C-2 UIAI research and source bridge proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md").read_text()
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.research_diagnostics_packet.v1.json").read_text()
)
CONTRACT = json.loads(
    (ROOT / "docs/contracts/spec135-research-bridge-origin-isolation.v1.json").read_text()
)
BRIDGE = (ROOT / "apps/pi-extension/src/research-bridge.ts").read_text()
INDEX = (ROOT / "apps/pi-extension/src/index.ts").read_text()
RUNTIME = (ROOT / "apps/pi-extension/tests/research-bridge.test.mjs").read_text()

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa Research Diagnostics Packet v1"
for field in ("session_origin", "browser_context_ref", "attachment_id", "origin_merge_prohibited"):
    assert field in SCHEMA["required"], field
assert SCHEMA["properties"]["origin_merge_prohibited"] == {"const": True}

assert CONTRACT["acceptance_criteria"] == "Research workflow produces cited durable artifacts without merging browser/session origins."
assert CONTRACT["schema"] == "focusa.spec135.research_bridge_origin_isolation.v1"
assert "Each cited artifact retains its exact session_origin and browser_context_ref" in CONTRACT["origin_isolation_invariants"]
assert "Isolation may not be inferred from separate tabs alone" in CONTRACT["origin_isolation_invariants"]

for token in (
    "ResearchDiagnosticsPacket",
    "validateOriginIsolation",
    "buildResearchPacket",
    "ensureNoOriginMerge",
    "origin_merge_prohibited: true",
    "registerResearchBridge",
    "cleanup_posture",
):
    assert token in BRIDGE, token

assert "registerResearchBridge(pi)" in INDEX
assert 'registerMessageRenderer("focusa-research-packet"' in INDEX
assert "no origin merge" in RUNTIME
assert "Origin merge prohibited: true" in RUNTIME

for spec_text in (
    "ResearchDiagnosticsPacket",
    "search",
    "source Markdown",
    "diagnostics",
    "session/context/target origin refs",
    "Shared browser contexts require explicit visible selection",
):
    assert spec_text in SPEC, spec_text

print("Spec 135 C2 UIAI research and source bridge: PASS")