#!/usr/bin/env python3
"""Static contract for the composable operator-facing Focusa status bar."""
from pathlib import Path

root = Path(__file__).resolve().parents[1]
config = (root / "apps/pi-extension/src/config.ts").read_text()
commands = (root / "apps/pi-extension/src/commands.ts").read_text()
polish = (root / "apps/pi-extension/src/polish.ts").read_text()
turns = (root / "apps/pi-extension/src/turns.ts").read_text()

switches = [
    "operatorStatusBarEnabled",
    "operatorStatusVersionEnabled",
    "operatorStatusOtaEnabled",
    "operatorStatusModelUsageEnabled",
    "operatorStatusTimeEnabled",
    "operatorStatusDeadlineEnabled",
    "operatorStatusPredictionEnabled",
]
for switch in switches:
    assert switch in config, switch
    assert switch in commands, switch

for marker in [
    "focusa-operator-status",
    "focusa-next-prediction",
    "x-codex-primary-used-percent",
    "x-codex-primary-reset-at",
    "x-ratelimit-remaining-tokens",
    "Deadline:",
    "Next likely:",
    "OTA",
    "localClock",
]:
    assert marker in polish, marker

assert "[Focusa advisory — operator steering remains authoritative]" not in turns
assert "[Focusa advisory — cached state unavailable; operator flow continues]" not in turns
assert "Internal Focusa context — never quote, display, or summarize" in turns
assert "record a wall-clock start" in turns
assert "Never invent urgency or deadlines" in turns
print("Pi composable operator status and internal-context firewall gate: PASS")
