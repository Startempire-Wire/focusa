#!/usr/bin/env python3
import argparse, hashlib, json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "docs/current/focusa-tool-contracts.json"
SKILLS = ROOT / "config/agent-skills-v2.json"
CONTRACTS = ROOT / "apps/pi-extension/src/tool-contracts.ts"
OUT = ROOT / "docs/contracts/65-focusa-skill-ownership-manifest.json"

RULES = [
    ("focusa-security-auth-licensing", ("sms_", "instruction_", "agent_artifact_", "prompt_variant_", "runtime_constitution", "agent_runtime_")),
    ("focusa-device-pairing", ("device_pair",)),
    ("focusa-context-cognition", ("context_cognition",)),
    ("focusa-project-card", ("project_card",)),
    ("focusa-trajectory", ("trajectory", "hlt_history")),
    ("focusa-workpoint", ("workpoint", "active_object")),
    ("focusa-work-loop", ("work_loop",)),
    ("focusa-focus-state", ("state_hygiene", "focus_state")),
    ("focusa-lineage", ("tree_", "lineage", "li_tree")),
    ("focusa-dxux-recovery", ("dxux",)),
    ("focusa-metacognition", ("metacog",)),
    ("predictive-power", ("predict", "prediction_authority")),
    ("focusa-release-proof", ("bloatgaurd", "call_stack")),
    ("focusa-resource-performance", ("resource_mode", "traverse")),
    ("focusa-session-recovery", ("session_transfer", "preload")),
    ("focusa-browser-uiai", ("browser",)),
    ("focusa-project-scope", ("project_identity", "project_verify", "project_bootstrap", "project_genesis")),
    ("focusa-evidence-outcomes", ("evidence",)),
    ("focusa-temporal-authority", ("temporal",)),
    ("focusa-silent-sessions", ("silent_sessions",)),
    ("focusa-tool-discovery", ("tool_search", "tool_describe", "tool_graph", "tool_bundle", "agent_card")),
]

def owner(name: str) -> str:
    stem = name.removeprefix("focusa_")
    for skill, needles in RULES:
        if any(needle in stem for needle in needles):
            return skill
    return "focusa"

catalog = json.loads(TOOLS.read_text())
names = sorted(row["name"] for row in catalog["contracts"])
if not names or len(names) != len(set(names)) or len(names) != catalog["tool_count"]:
    raise SystemExit("tool catalog must be nonempty, have unique names and match its count")
known_skills = {row["name"] for row in json.loads(SKILLS.read_text())["skills"]}
invalid_owners = sorted({owner(name) for name in names} - known_skills)
if invalid_owners:
    raise SystemExit(f"unknown owner skills: {invalid_owners}")
rows = [{
    "tool": name,
    "owner_skill": owner(name),
    "authority_boundary": "tool contract and daemon remain canonical",
    "failure_handoff": "focusa-troubleshooting",
} for name in names]
payload = {
    "schema": "focusa.skill_ownership_manifest.v1",
    "advertised_capability_count": len(rows),
    "tool_contract_sha256": hashlib.sha256(CONTRACTS.read_bytes()).hexdigest(),
    "generated": True,
    "capabilities": rows,
}
args = argparse.ArgumentParser()
args.add_argument("--check", action="store_true")
body = json.dumps(payload, indent=2) + "\n"
if args.parse_args().check:
    if not OUT.exists() or OUT.read_text() != body:
        raise SystemExit(f"{OUT.relative_to(ROOT)} is stale; regenerate with {Path(__file__).name}")
else:
    OUT.write_text(body)
print(f"{OUT.relative_to(ROOT)}: {len(rows)} capabilities")
