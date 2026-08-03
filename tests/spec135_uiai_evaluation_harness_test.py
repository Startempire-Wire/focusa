#!/usr/bin/env python3
"""P10 governed UIAI evaluation and thirteen-proof contract gate."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
server = (ROOT / "apps/pi-extension/tests/mission-canvas-uiai-server.mjs").read_text()
runner = (ROOT / "apps/pi-extension/tests/uiai-eval-harness.test.mjs").read_text()
evidence = (ROOT / "docs/evidence/spec135-p10-uiai-evaluation.md").read_text()
proofs = json.loads((ROOT / "tests/fixtures/spec135-thirteen-no-dead-chrome-proofs.json").read_text())
responsive = json.loads((ROOT / "tests/fixtures/spec135-responsive-evaluations.json").read_text())

assert len(proofs["proofs"]) == 13
assert [proof["id"] for proof in proofs["proofs"]] == [f"NDC-{index:02d}" for index in range(1, 14)]
assert {proof["scenario"] for proof in proofs["proofs"]} == {"populated", "empty-optionals", "single-queue", "zero-queues"}
for surface in ["__fixture/reset", "__fixture/state", "evidence_ref", "receipt_ref", "omission_diagnostics", "layout_tree"]:
    assert surface in server, surface
for proof in ["candidate_contribution_ids", "omitted", "layout", "deepEqual", "single-queue", "zero-queues"]:
    assert proof in runner, proof
assert {item["viewport"]["platform"] for item in responsive} == {"macOS", "Windows", "Linux"}
assert "uiai-diagnostics:session=k1G5cirJ:seq=6" in evidence
assert "docs/evidence/spec135-uiai-reference-comparison.png" in evidence
assert "console errors: `0`" in evidence
assert "JavaScript exceptions: `0`" in evidence
assert "prior loopback-policy blocker is resolved" in evidence
assert (ROOT / "docs/evidence/spec135-uiai-reference-comparison.png").exists()

print("Spec 135 governed UIAI evaluation harness and thirteen proofs: PASS")
