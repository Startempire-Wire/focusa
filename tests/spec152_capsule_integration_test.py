#!/usr/bin/env python3
import pathlib, json
ROOT=pathlib.Path(__file__).resolve().parents[1]
# integration: ledger + envelope both exist
assert (ROOT/"docs/contracts/spec152-protected-boundary-ledger.v1.json").exists()
assert (ROOT/"docs/contracts/spec152-capsule-envelope.v1.json").exists()
print("capsule integration PASS")
