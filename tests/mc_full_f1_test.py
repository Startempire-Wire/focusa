import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/mc-full-f1-ontology-core.v1.json").read_text())
assert j["core"]=="domain-general"
print("f1 PASS")
