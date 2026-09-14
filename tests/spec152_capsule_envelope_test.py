#!/usr/bin/env python3
import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/spec152-capsule-envelope.v1.json").read_text())
assert j["schema"]=="focusa.capsule_envelope.v1"
assert j["node_binding"].startswith("sha256:")
print("capsule envelope PASS")
