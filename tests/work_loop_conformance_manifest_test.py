#!/usr/bin/env python3
import importlib.util
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/work_loop_conformance.py"
spec = importlib.util.spec_from_file_location("work_loop_conformance", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

manifest = module.load_manifest(module.DEFAULT_MANIFEST)
records = module.must_records(manifest)
assert len(records) >= 250, "all four normative specs must contribute MUST traces"
assert {record["spec"] for record in records} == {"spec79", "spec98", "spec104", "spec133"}
assert all(record["proof_count"] > 0 for record in records)
assert all(record["coverage_status"] in module.VALID_STATUSES for record in records)

result = subprocess.run(["python3", str(SCRIPT), "--mode", "audit", "--json"], cwd=ROOT, capture_output=True, text=True)
assert result.returncode == 0, result.stderr
report = json.loads(result.stdout)
assert report["schema"] == "focusa.work_loop_conformance_report.v1"
assert report["normative_must_total"] == len(records)
assert report["release_ready"] is False
assert report["pending_must_total"] > 0

release = subprocess.run(["python3", str(SCRIPT), "--mode", "release"], cwd=ROOT, capture_output=True, text=True)
assert release.returncode == 3
assert "RELEASE BLOCKED" in release.stderr

with tempfile.TemporaryDirectory() as tmp:
    invalid = Path(tmp) / "invalid.json"
    invalid.write_text('{"schema":"unknown","specs":[]}')
    rejected = subprocess.run(
        ["python3", str(SCRIPT), "--manifest", str(invalid)], cwd=ROOT, capture_output=True, text=True
    )
    assert rejected.returncode == 2
    assert "unsupported conformance manifest schema" in rejected.stderr

print(f"work-loop conformance manifest traced {len(records)} normative MUST statements and failed release closed")
