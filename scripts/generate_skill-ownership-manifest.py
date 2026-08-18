#!/usr/bin/env python3
import hashlib, json, re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
CONTRACTS = ROOT / "apps/pi-extension/src/tool-contracts.ts"
OUT = ROOT / "docs/contracts/65-focusa-skill-ownership-manifest.json"

RULES = [
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

text = TOOLS.read_text()
names = set(re.findall(r'name:\s*"(focusa_[^"]+)"', text))
names.update({f"focusa_preload_{suffix}" for suffix in ["build", "render", "verify", "doctor"]})
names = sorted(names)
if len(names) != 129:
    raise SystemExit(f"expected 129 current advertised Focusa tools, found {len(names)}")
contracts = set(re.findall(r'name:\s*"(focusa_[^"]+)"', CONTRACTS.read_text()))
contracts.update({f"focusa_preload_{suffix}" for suffix in [
    "profiles", "build", "render", "write", "verify", "doctor", "receipt_preview", "receipt_commit"
]})
missing = sorted(set(names) - contracts)
if missing:
    raise SystemExit(f"missing tool contracts: {missing}")
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
OUT.write_text(json.dumps(payload, indent=2) + "\n")
print(f"generated {OUT.relative_to(ROOT)}: {len(rows)} capabilities")
