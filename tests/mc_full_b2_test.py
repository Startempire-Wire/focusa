#!/usr/bin/env python3
import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/mc-full-b2-context-ingestion.v1.json").read_text())
assert j["linked"]==True
print("mc b2 PASS")
