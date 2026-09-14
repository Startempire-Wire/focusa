import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/mc-full-b5.v1.json").read_text())
assert j["ok"]
print("b5 PASS")
