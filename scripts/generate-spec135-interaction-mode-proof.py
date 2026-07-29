#!/usr/bin/env python3
"""Generate Spec 135K-1 Issue #53 interaction-mode proof contract."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"docs/contracts/spec135-interaction-mode-toggle.v1.json"
C={"schema":"focusa.spec135.interaction_mode_toggle.v1","acceptance_criteria":"All #53 toggle and headless acceptance scenarios pass without state loss, nags, or crashes.","modes":["canvas-guided","terminal-guided","headless"],"precedence":["temporary-session override","project preference","user preference","environment","default canvas-guided"],"properties":{"durable":True,"source_displayed":True,"scope_exact":True,"refresh_immediate":True,"resume_survives":True,"reconnect_survives":True,"headless_no_ui_calls":True,"state_loss":False,"nag_on_headless":False},"implementation_refs":["apps/pi-extension/src/config.ts","apps/pi-extension/src/commands.ts","apps/pi-extension/src/turns.ts"],"proof_refs":["apps/pi-extension/tests/mission-canvas-mode-precedence.test.mjs","tests/spec135_m2_pi_work_rail_test.py"]}
OUT.write_text(json.dumps(C,indent=2)+"\n")
print("Spec 135K-1 interaction mode proof generated")
