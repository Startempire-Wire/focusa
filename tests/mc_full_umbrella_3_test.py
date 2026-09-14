import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
j=json.loads((ROOT/"docs/contracts/mc-full.3.v1.json").read_text())
assert j["umb"]=="done"
print("umb3 PASS")
