#!/usr/bin/env python3
import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/mc-full-b1-crist-profile.v1.json").read_text())
assert j["schema"]=="focusa.mc.crist_profile.v1"
print("mc-full b1 PASS")
