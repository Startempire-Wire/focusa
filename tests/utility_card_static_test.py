#!/usr/bin/env python3
"""focusa-4jo5 utility-card/bootstrap static audit."""
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
    core = read("crates/focusa-core/src/utility_card.rs")
    for token in ["UtilityCard", "utility_card", "authority_boundary", "usefulness_bar", "bootstrap_card", "post_compaction_card", "exact_next_actions", "do_not_drift", "evidence_policy", "brevity_rules", "proof_commands"]:
        if token not in core:
            fail(f"core missing {token}")

    api = read("crates/focusa-api/src/routes/utility.rs") + read("crates/focusa-api/src/routes/agent_reminder.rs") + read("crates/focusa-api/src/routes/mod.rs") + read("crates/focusa-api/src/server.rs")
    for token in ["/v1/utility/card", "/v1/utility/bootstrap", "/v1/utility/post-compaction", "authority_boundary", "exact_next_actions", "utility_card", "routes::utility::router()", "pub mod utility"]:
        if token not in api:
            fail(f"api missing {token}")

    cli = read("crates/focusa-cli/src/main.rs") + read("crates/focusa-cli/src/commands/utility.rs") + read("crates/focusa-cli/src/commands/mod.rs")
    for token in ["Utility", "UtilityCmd", "focusa utility card", "PostCompaction", "authority_boundary", "exact_next_actions", "/v1/utility/card"]:
        if token not in cli:
            fail(f"cli missing {token}")

    contracts = json.loads(read("docs/current/focusa-tool-contracts.json"))
    names = {entry["name"] for entry in contracts["contracts"]}
    if "focusa_utility_card" not in names:
        fail("contract missing focusa_utility_card")
    if contracts.get("tool_count") != len(contracts["contracts"]):
        fail("tool_count mismatch")

    choreography = json.loads(read("docs/current/focusa-tool-choreography.json"))
    if "focusa_utility_card" not in choreography.get("per_tool_next_tools", {}):
        fail("choreography missing focusa_utility_card")

    tools_ts = read("apps/pi-extension/src/tools.ts")
    for token in ["focusa_utility_card", "/utility/card", "bootstrap", "compaction"]:
        if token not in tools_ts:
            fail(f"Pi tool missing {token}")

    if not (ROOT / "docs/focusa-tools/tools/focusa_utility_card.md").exists():
        fail("missing utility card doc")

    print("✓ PASS: utility/bootstrap/post-compaction card wired across core/api/cli/pi-contracts/docs")


if __name__ == "__main__":
    main()
