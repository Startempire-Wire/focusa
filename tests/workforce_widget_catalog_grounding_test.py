#!/usr/bin/env python3
"""WIDGET-1: widget catalog operation refs must exist in Spec135 registry."""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/focusa-core/src/widget_contracts.rs"
REGISTRY = ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"

source = SOURCE.read_text()
registry = json.loads(REGISTRY.read_text())
operation_ids = {entry["operation_id"] for entry in registry["operations"]}
block = re.search(r"const GROUNDED_OPERATION_IDS: &\[&str\] = &\[(.*?)\];", source, re.S)
assert block, "grounded operation block missing"
refs = set(re.findall(r'"(focusa\.[a-z0-9_.]+)"', block.group(1)))
missing = sorted(ref for ref in refs if ref not in operation_ids)
assert not missing, f"widget catalog contains ungrounded operation refs: {missing}"
assert "focusa.call_stack.verify" not in refs, "unverified CallGraph widget must not enter catalog"
print(f"PASS: {len(refs)} widget operation references ground in Spec135 registry")
