#!/usr/bin/env python3
"""Spec98 focusa-877z.1: ActiveTurn runtime-only/action-routed guard."""
from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.1-active-turn-runtime-separation.yaml"
TYPES = ROOT / "crates/focusa-core/src/types.rs"
TURN = ROOT / "crates/focusa-api/src/routes/turn.rs"
PROXY = ROOT / "crates/focusa-api/src/routes/proxy.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.active_turn_runtime_separation_contract.v1":
        fail("unexpected .1 contract schema")
    if data.get("status") != "active_turn_runtime_only_action_routed":
        fail("unexpected .1 contract status")
    types = TYPES.read_text()
    if "Runtime-only active turn from Mode A adapter" not in types:
        fail("FocusaState.active_turn must be documented runtime-only")
    if "#[serde(default, skip_serializing, skip_deserializing)]\n    pub active_turn: Option<ActiveTurn>" not in types:
        fail("FocusaState.active_turn must be skipped during snapshot serialization/deserialization")
    if "Raw input and assembled prompt are correlation buffers, not canonical cognition authority" not in types:
        fail("ActiveTurn struct must document runtime-correlation authority class")
    for field in ["raw_user_input: Option<String>", "assembled_prompt: Option<String>"]:
        if field not in types:
            fail(f"ActiveTurn missing compatibility field {field}")
    if "UpdateActiveTurnRuntime" not in types:
        fail("Action::UpdateActiveTurnRuntime must exist for runtime-only active turn updates")
    for path in [TURN, PROXY]:
        text = path.read_text()
        if "Action::UpdateActiveTurnRuntime" not in text:
            fail(f"{path.name} must route active turn prompt/chunk updates through Action::UpdateActiveTurnRuntime")
        forbidden = [
            "turn.raw_user_input =",
            "turn.assembled_prompt =",
            "let existing = turn.assembled_prompt.take()",
        ]
        for needle in forbidden:
            if needle in text:
                fail(f"{path.name} still directly mutates ActiveTurn buffer via {needle}")
    remaining = "\n".join(data.get("remaining_migration") or [])
    if "runtime cache" not in remaining or "diagnostics handle" not in remaining:
        fail("contract must retain runtime-cache/diagnostics-handle follow-up gap")
    print("✓ PASS: ActiveTurn snapshot persistence is skipped and API buffer updates are action-routed")


if __name__ == "__main__":
    main()
