#!/usr/bin/env python3
"""Record a CI/Release/Deploy workflow_run failure into the audit trail.

Invoked by `.github/workflows/audit-recorder.yml` whenever a watched workflow
concludes with `failure`. Writes a single canonical-schema row.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

AUDIT_PATH = Path(os.environ.get("AUDIT_PATH", "release-proof/audit/audit.jsonl"))


def main() -> int:
    run_id = os.environ.get("RUN_ID", "")
    workflow = os.environ.get("WORKFLOW", "unknown")
    head_sha = os.environ.get("HEAD_SHA", "")
    branch = os.environ.get("BRANCH", "")
    event = os.environ.get("EVENT", "")
    attempt = os.environ.get("ATTEMPT", "")
    log_url = os.environ.get("LOG_URL", "")
    ts = os.environ.get("TS", "")
    if not ts:
        from datetime import datetime, timezone
        ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    rid = f"fail-{ts.replace(':', '').replace('-', '')}-{run_id}"
    entry = {
        "id": rid,
        "ts": ts,
        "event": "failure",
        "subsystem": "ci",
        "scope": workflow,
        "category": "ci_workflow_failure",
        "symptom": f"{workflow} run {run_id} concluded failure",
        "root_cause": "see workflow logs",
        "fix": "see auto-heal synthesis",
        "guard": "auto-heal-audit.py",
        "test": "scripts/auto-heal-audit.py",
        "linked_run": run_id,
        "head_sha": head_sha,
        "branch": branch,
        "github_event": event,
        "run_attempt": attempt,
        "log_url": log_url,
    }
    AUDIT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with AUDIT_PATH.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, separators=(",", ":")) + "\n")
    print(f"recorded {rid}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())