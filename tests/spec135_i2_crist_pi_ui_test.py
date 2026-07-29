#!/usr/bin/env python3
"""Spec 135I-2 real-time generated C.R.I.S.T. Pi UI proof."""
import json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SRC=(ROOT/"apps/pi-extension/src/crist-canvas.ts").read_text()
INDEX=(ROOT/"apps/pi-extension/src/index.ts").read_text()
REG=json.loads((ROOT/"docs/contracts/spec135/generated-contract-v1/operation-registry.json").read_text())
ops={o["operation_id"] for o in REG["operations"]}
assert 'registerCommand("focusa-crist"' in SRC
assert "registerCristCanvas(pi)" in INDEX
assert 'registerMessageRenderer("focusa-crist-stage"' in INDEX
for stage in ("Context","Role","Interview","Spec","Tasks"):
    assert f'stage: "{stage}"' in SRC, stage
bound=set(re.findall(r'(?:readOperation|mutateOperation): "([^"]+)"',SRC))
assert len(bound)==10
assert bound <= ops, sorted(bound-ops)
for command in ("/focusa-context","/focusa-role","/focusa-interview","/mission-canvas","/focusa-rail"):
    assert command in SRC
assert "canonical reducers remain authoritative" in SRC
assert "getActiveWorkpointPacket()" in SRC
assert "invalidationKeys" in SRC
print("Spec 135 I2 real-time generated C.R.I.S.T. Pi UI: PASS")
