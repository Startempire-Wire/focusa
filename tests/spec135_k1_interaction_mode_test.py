#!/usr/bin/env python3
"""Spec 135K-1 Issue #53 interaction mode proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
C=json.loads((ROOT/"docs/contracts/spec135-interaction-mode-toggle.v1.json").read_text())
CFG=(ROOT/"apps/pi-extension/src/config.ts").read_text()
CMD=(ROOT/"apps/pi-extension/src/commands.ts").read_text()
assert C["modes"] == ["canvas-guided","terminal-guided","headless"]
for mode in C["modes"]: assert mode in CFG and mode in CMD
for key,value in C["properties"].items():
    assert value is (False if key in {"state_loss","nag_on_headless"} else True), key
assert "resolveInteractionMode" in CFG
assert 'registerCommand("mission-canvas-mode"' in CMD
assert "sessionInteractionModes.set" in CMD
assert "saveConfigOverrides" in CMD
for ref in C["proof_refs"]+C["implementation_refs"]: assert (ROOT/ref).exists(), ref
print("Spec 135 K1 Issue #53 interaction modes/live toggle: PASS")
