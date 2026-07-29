#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCH = ROOT / "public" / "bench"

required = ["index.html", "styles.css", "app.js", "latest.json", "methodology.html"]
for name in required:
    assert (BENCH / name).is_file(), f"missing observatory artifact: {name}"

html = (BENCH / "index.html").read_text()
app = (BENCH / "app.js").read_text()
method = (BENCH / "methodology.html").read_text()
snapshot = json.loads((BENCH / "latest.json").read_text())

for marker in (
    "Focusa-vs-No-Focusa",
    "Outcome trend",
    "Improvement board",
    "TASK REPLAY THEATER",
    "Evidence bundle",
    "Honesty rail",
):
    assert marker in html, marker

assert snapshot["schema"] == "focusa.public_benchmark_snapshot.v1"
assert snapshot["publish_allowed"] is False
assert snapshot["comparison"]["uplift_score"] is None
assert snapshot["redaction_status"] == "not_required_no_raw_payload"
assert "No performance claim is published" in snapshot["empty_state_message"]
assert "publish_allowed === true" in app
assert "No public measured trend yet" in app
assert "Private prompts and raw transcripts never render" in html
assert "No public-safe measured run" in method
assert "/v1/evals" not in app, "public UI must not expose private daemon eval routes"

print("Spec114 observatory UI static preview: PASS")
