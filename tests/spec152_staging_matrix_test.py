#!/usr/bin/env python3
import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/spec152-staging-matrix.v1.json").read_text())
assert len(j["cases"])==14
print("staging matrix PASS 14 cases redacted")
