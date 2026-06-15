#!/usr/bin/env python3
"""Spec105 Agent DX/UX static audit."""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REQS = [f"DXUX-{i:03d}" for i in range(1, 13)]
TOOLS = ["focusa_dxux_report", "focusa_dxux_requirement", "focusa_dxux_explain", "focusa_dxux_digest"]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def read(rel: str) -> str:
    return (ROOT / rel).read_text()


def main() -> None:
    core = read("crates/focusa-core/src/dxux.rs")
    for token in ["DxuxRequirement", "DxuxReport", "DxuxExplain", "dxux_report", "dxux_requirement", "dxux_explain", "preflight_commands", "digest_fields"] + REQS:
        if token not in core:
            fail(f"core missing {token}")

    api = read("crates/focusa-api/src/routes/dxux.rs") + read("crates/focusa-api/src/routes/mod.rs") + read("crates/focusa-api/src/server.rs")
    for token in ["/v1/dxux/report", "/v1/dxux/requirement/{id}", "/v1/dxux/explain/{failure}", "/v1/dxux/digest", "routes::dxux::router()", "pub mod dxux"]:
        if token not in api:
            fail(f"api missing {token}")

    cli = read("crates/focusa-cli/src/main.rs") + read("crates/focusa-cli/src/commands/dxux.rs") + read("crates/focusa-cli/src/commands/mod.rs")
    for token in ["Preflight", "Explain", "Dxux", "focusa preflight", "cargo clippy --workspace -- -D warnings", "DxuxCmd", "Requirement", "Digest"]:
        if token not in cli:
            fail(f"cli missing {token}")

    contracts = json.loads(read("docs/current/focusa-tool-contracts.json"))
    names = {entry["name"] for entry in contracts["contracts"]}
    for name in TOOLS:
        if name not in names:
            fail(f"contract missing {name}")
    if contracts.get("tool_count") != len(contracts["contracts"]):
        fail("tool_count mismatch")

    choreography = json.loads(read("docs/current/focusa-tool-choreography.json"))
    per_tool = choreography.get("per_tool_next_tools", {})
    for name in TOOLS:
        if name not in per_tool:
            fail(f"choreography missing {name}")

    tools_ts = read("apps/pi-extension/src/tools.ts")
    for name in TOOLS:
        if name not in tools_ts:
            fail(f"Pi tool missing {name}")

    for rel in ["docs/focusa-tools/tools/focusa_dxux_report.md", "docs/focusa-tools/tools/focusa_dxux_requirement.md", "docs/focusa-tools/tools/focusa_dxux_explain.md", "docs/focusa-tools/tools/focusa_dxux_digest.md"]:
        if not (ROOT / rel).exists():
            fail(f"missing doc {rel}")

    print("✓ PASS: Spec105 DX/UX real surfaces wired across core/api/cli/pi-contracts/choreography/docs")


if __name__ == "__main__":
    main()
