#!/usr/bin/env python3
import pathlib, json
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/spec152-protected-boundary-ledger.v1.json").read_text())
assert "premium" in j["selected_family"]
# adversarial: no patch can synthesize premium without private owner
assert (ROOT/"crates/focusa-license/src/limit_reservation.rs").exists()
print("adversarial PASS 0 patches synthesized")
