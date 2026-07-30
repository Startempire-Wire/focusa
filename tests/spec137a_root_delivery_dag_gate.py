#!/usr/bin/env python3
"""Verify every accepted Spec137/137A row is explicit in one acyclic root delivery DAG."""
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]
LEDGER = yaml.safe_load((ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml").read_text())
DAG = yaml.safe_load((ROOT / "docs/contracts/spec137a-root-delivery-dag.v1.yaml").read_text())
expected = {r["requirement_id"] for r in LEDGER["requirements"]} | {r["requirement_id"] for r in LEDGER["spec137a_requirement_rows"]}
ids = [n["requirement_id"] for n in DAG["nodes"]]
assert DAG["node_count"] == 258 == len(ids)
assert len(ids) == len(set(ids))
assert set(ids) == expected
tranches = {t["tranche_id"]: t for t in DAG["tranches"]}
assert len(tranches) == 9
for tranche in tranches.values():
    assert all(dep in tranches for dep in tranche["depends_on"]), tranche["tranche_id"]
for node in DAG["nodes"]:
    rid = node["requirement_id"]
    assert node["tranche"] in tranches, rid
    assert node["owner"], rid
    for field in ("implementation_refs", "test_refs", "evidence_refs", "receipt_refs"):
        assert node[field], f"{rid}: missing {field}"
    assert node["parent_closure_impact"] == "blocking_for_claimed_conformance", rid
# bounded DFS proves tranche graph acyclic
visiting, visited = set(), set()
def visit(name):
    assert name not in visiting, f"cycle at {name}"
    if name in visited: return
    visiting.add(name)
    for dep in tranches[name]["depends_on"]: visit(dep)
    visiting.remove(name); visited.add(name)
for name in tranches: visit(name)
print("Spec137A root delivery DAG gate: PASS (258 nodes, 9 acyclic tranches, zero omissions)")
