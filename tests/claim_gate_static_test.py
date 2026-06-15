#!/usr/bin/env python3
"""focusa-bwky anti-false-claim gate static audit."""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def read(rel: str) -> str:
    return (ROOT / rel).read_text()


def main() -> None:
    core = read("crates/focusa-core/src/claim_gate.rs")
    for token in ["CompletionClaimRequest", "CompletionClaimGateReport", "completion_claim_gate", "evidence_class", "missing_required_evidence", "overclaim_risks", "mac_pairing_api_web_only_evidence_blocks_completion"]:
        if token not in core:
            fail(f"core missing {token}")

    api = read("crates/focusa-api/src/routes/claim_gate.rs") + read("crates/focusa-api/src/routes/mod.rs") + read("crates/focusa-api/src/server.rs")
    for token in ["/v1/claim/preclose", "completion_claim_gate", "routes::claim_gate::router()", "pub mod claim_gate"]:
        if token not in api:
            fail(f"api missing {token}")

    cli = read("crates/focusa-cli/src/main.rs") + read("crates/focusa-cli/src/commands/claim.rs") + read("crates/focusa-cli/src/commands/mod.rs")
    for token in ["Claim", "ClaimCmd", "Preclose", "/v1/claim/preclose", "evidence_class"]:
        if token not in cli:
            fail(f"cli missing {token}")

    contracts = json.loads(read("docs/current/focusa-tool-contracts.json"))
    names = {entry["name"] for entry in contracts["contracts"]}
    if "focusa_claim_preclose_gate" not in names:
        fail("contract missing focusa_claim_preclose_gate")
    if contracts.get("tool_count") != len(contracts["contracts"]):
        fail("tool_count mismatch")

    choreography = json.loads(read("docs/current/focusa-tool-choreography.json"))
    if "focusa_claim_preclose_gate" not in choreography.get("per_tool_next_tools", {}):
        fail("choreography missing focusa_claim_preclose_gate")

    tools_ts = read("apps/pi-extension/src/tools.ts")
    for token in ["focusa_claim_preclose_gate", "/claim/preclose", "overclaim", "partial"]:
        if token not in tools_ts:
            fail(f"Pi tool missing {token}")

    if not (ROOT / "docs/focusa-tools/tools/focusa_claim_preclose_gate.md").exists():
        fail("missing claim gate doc")

    print("✓ PASS: anti-false-claim preclose gate wired across core/api/cli/pi-contracts/docs")


if __name__ == "__main__":
    main()
