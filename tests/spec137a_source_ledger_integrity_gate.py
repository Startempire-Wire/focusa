#!/usr/bin/env python3
"""Verify Spec137A source atoms are fully and deterministically represented."""
from pathlib import Path
import hashlib
import re
import yaml

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md"
LEDGER = ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml"
source_lines = SPEC.read_text().splitlines()
rows = yaml.safe_load(LEDGER.read_text())["spec137a_requirement_rows"]
assert len(rows) == 172
assert hashlib.sha256(SPEC.read_bytes()).hexdigest() == "2747f7f1ff7417c4541d7223199b3a480128ca049c9fe4a45859028af99a8419"

refs = []
for row in rows:
    path, line = row["source_clause_ref"].rsplit(":", 1)
    assert path == "docs/137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md"
    number = int(line)
    assert 1 <= number <= len(source_lines), row["requirement_id"]
    assert row["source_text_hash"] == hashlib.sha256(source_lines[number - 1].encode()).hexdigest(), row["requirement_id"]
    refs.append(number)
assert len(set(refs)) == len(refs)
assert all(row["closure_impact"] == "blocking_for_claimed_conformance" for row in rows)
print("Spec137A source ledger integrity gate: PASS (172 unique hashed source clauses)")
