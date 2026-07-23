#!/usr/bin/env python3
"""Spec101 Bloatgaurd budgets, tokenbloat, gate modes, profiles/routines/rollout static audit."""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOMAINS = [
    "output-firewall",
    "tool-call-compression",
    "docs-diet",
    "test-diet",
    "prompt-context-diet",
    "rust-first-core",
    "dead-code-safety",
    "adaptive-router",
]
TOKENBLOAT = ["tokenbloat-control", "tool-call-history-elision"]
GATE_MODES = ["advisory", "warning", "fail-candidate"]
PROFILES = ["daily_driver", "beast_mode", "speedy", "neat_freak", "tightwad"]
ROUTINES = [
    "patrol",
    "brief",
    "squeezer",
    "librarian",
    "janitor",
    "pantry",
    "gatekeeper",
    "xray",
    "deep_dive",
    "scout",
]
TOOLS = [
    "focusa_bloatgaurd_report",
    "focusa_bloatgaurd_domain",
    "focusa_bloatgaurd_tokenbloat_report",
    "focusa_bloatgaurd_tokenbloat_domain",
    "focusa_bloatgaurd_gate_modes",
    "focusa_bloatgaurd_gate_mode",
    "focusa_bloatgaurd_profiles",
    "focusa_bloatgaurd_profile",
    "focusa_bloatgaurd_routines",
    "focusa_bloatgaurd_routine",
    "focusa_bloatgaurd_rollout",
]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def read(rel: str) -> str:
    return (ROOT / rel).read_text()


def main() -> None:
    core = read("crates/focusa-core/src/bloatgaurd.rs")
    for token in [
        "BloatgaurdBudget",
        "BloatgaurdDomainState",
        "BloatgaurdReport",
        "TokenbloatControl",
        "TokenbloatReport",
        "tokenbloat_report",
        "tokenbloat_control",
        "BloatgaurdGateMode",
        "BloatgaurdGateModesReport",
        "BloatgaurdGateThresholds",
        "bloatgaurd_gate_modes_report",
        "bloatgaurd_gate_mode",
        "BloatgaurdProfile",
        "BloatgaurdProfilesReport",
        "bloatgaurd_profiles_report",
        "bloatgaurd_profile",
        "BloatgaurdRoutine",
        "BloatgaurdRoutinesReport",
        "bloatgaurd_routines_report",
        "bloatgaurd_routine",
        "BloatgaurdRolloutReport",
        "bloatgaurd_rollout_report",
        "full_payload_requires_opt_in",
        "deletion_requires_human_review",
        "recommended_gate_mode",
        "explicit_ci_allowlist",
    ]:
        if token not in core:
            fail(f"core missing {token}")
    for token in DOMAINS + TOKENBLOAT + GATE_MODES + PROFILES + ROUTINES:
        if token not in core:
            fail(f"core missing expected token {token}")

    api = read("crates/focusa-api/src/routes/bloatgaurd.rs")
    server = read("crates/focusa-api/src/server.rs")
    routes_mod = read("crates/focusa-api/src/routes/mod.rs")
    for token in [
        "/v1/bloatgaurd/report",
        "/v1/bloatgaurd/domain/{name}",
        "/v1/bloatgaurd/tokenbloat/report",
        "/v1/bloatgaurd/tokenbloat/domain/{name}",
        "/v1/bloatgaurd/gate-modes/report",
        "/v1/bloatgaurd/gate-modes/mode/{name}",
        "/v1/bloatgaurd/profiles/report",
        "/v1/bloatgaurd/profiles/profile/{name}",
        "/v1/bloatgaurd/routines/report",
        "/v1/bloatgaurd/routines/routine/{name}",
        "/v1/bloatgaurd/rollout/report",
        "bloatgaurd_profiles_report",
        "bloatgaurd_routines_report",
        "bloatgaurd_rollout_report",
    ]:
        if token not in api:
            fail(f"api missing {token}")
    if (
        "routes::bloatgaurd::router()" not in server
        or "pub mod bloatgaurd;" not in routes_mod
    ):
        fail("api router not registered")

    cli = (
        read("crates/focusa-cli/src/commands/bloatgaurd.rs")
        + read("crates/focusa-cli/src/main.rs")
        + read("crates/focusa-cli/src/commands/mod.rs")
    )
    for token in [
        "BloatgaurdCmd",
        "Profiles",
        "Profile",
        "Routines",
        "Routine",
        "Rollout",
        "/v1/bloatgaurd/profiles/report",
        "/v1/bloatgaurd/profiles/profile/{name}",
        "/v1/bloatgaurd/routines/report",
        "/v1/bloatgaurd/routines/routine/{name}",
        "/v1/bloatgaurd/rollout/report",
    ]:
        if token not in cli:
            fail(f"cli missing {token}")

    contracts = json.loads(read("docs/current/focusa-tool-contracts.json"))
    names = {entry["name"] for entry in contracts["contracts"]}
    for name in TOOLS:
        if name not in names:
            fail(f"tool contract missing {name}")
    if contracts.get("tool_count") != len(contracts["contracts"]):
        fail("tool_count mismatch")

    choreo = json.loads(read("docs/current/focusa-tool-choreography.json"))
    per_tool = choreo.get("per_tool_next_tools", {})
    for name in TOOLS:
        if name not in per_tool:
            fail(f"choreography missing {name}")

    menubar = read("apps/menubar/src/lib/components/GatePanel.svelte")
    for token in (
        [
            "BLOATGAURD BUDGETS",
            "bloatgaurdReport",
            "budget-pill",
            "/v1/bloatgaurd/domain",
        ]
        + DOMAINS
        + TOKENBLOAT
        + GATE_MODES
    ):
        if token not in menubar:
            fail(f"menubar GatePanel missing {token}")

    for rel in [
        "docs/focusa-tools/tools/focusa_bloatgaurd_report.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_domain.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_tokenbloat_report.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_tokenbloat_domain.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_gate_modes.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_gate_mode.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_profiles.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_profile.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_routines.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_routine.md",
        "docs/focusa-tools/tools/focusa_bloatgaurd_rollout.md",
    ]:
        if not (ROOT / rel).exists():
            fail(f"missing doc {rel}")

    print(
        "✓ PASS: Spec101 Bloatgaurd MVP surfaces wired across core/api/cli/pi-contracts/choreography/menubar/docs"
    )


if __name__ == "__main__":
    main()
