#!/usr/bin/env python3
"""Record a CI/Release/Deploy workflow_run failure into the audit trail.

Invoked by `.github/workflows/audit-recorder.yml` whenever a watched workflow
concludes with `failure`. Writes a single canonical-schema row.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

AUDIT_PATH = Path(os.environ.get("AUDIT_PATH", "release-proof/audit/audit.jsonl"))


def load_failure_classification(log_path: str) -> dict:
    if not log_path:
        return {}
    path = Path(log_path)
    if not path.exists() or path.stat().st_size == 0:
        return {}
    classifier = Path(__file__).resolve().with_name("classify-ci-failure.py")
    try:
        proc = subprocess.run(
            [sys.executable, str(classifier), str(path)],
            check=True,
            text=True,
            capture_output=True,
        )
        payload = json.loads(proc.stdout)
        if isinstance(payload, dict):
            return payload
    except Exception as exc:  # pragma: no cover - defensive CI audit fallback
        return {
            "schema": "focusa.release_failure_classification.v1",
            "failure_class": "classification_process_error",
            "retry_policy": "hard_failure_no_rerun",
            "deterministic": True,
            "safe_to_rerun_unchanged": False,
            "plain_language_error": f"Audit classifier failed: {exc}",
            "likely_root_cause": "classifier_process_error",
            "remediation_template": "Inspect scripts/classify-ci-failure.py and workflow log capture.",
            "source_refs": ["scripts/classify-ci-failure.py"],
            "signals": ["classifier_process_error"],
        }
    return {}


def main() -> int:
    run_id = os.environ.get("RUN_ID", "")
    workflow = os.environ.get("WORKFLOW", "unknown")
    head_sha = os.environ.get("HEAD_SHA", "")
    branch = os.environ.get("BRANCH", "")
    event = os.environ.get("EVENT", "")
    attempt = os.environ.get("ATTEMPT", "")
    log_url = os.environ.get("LOG_URL", "")
    failed_log_path = os.environ.get("FAILED_LOG_PATH", "")
    classification = load_failure_classification(failed_log_path)
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
        "symptom": classification.get("plain_language_error")
        or f"{workflow} run {run_id} concluded failure",
        "root_cause": classification.get("likely_root_cause") or "see workflow logs",
        "fix": classification.get("remediation_template") or "see auto-heal synthesis",
        "guard": "scripts/classify-ci-failure.py + auto-heal-audit.py",
        "test": "scripts/classify-ci-failure.py; scripts/auto-heal-audit.py",
        "linked_run": run_id,
        "head_sha": head_sha,
        "branch": branch,
        "github_event": event,
        "run_attempt": attempt,
        "log_url": log_url,
    }
    if classification:
        entry.update(
            {
                "classification_schema": classification.get("schema", ""),
                "failure_class": classification.get("failure_class", ""),
                "retry_policy": classification.get("retry_policy", ""),
                "deterministic": classification.get("deterministic"),
                "safe_to_rerun_unchanged": classification.get(
                    "safe_to_rerun_unchanged"
                ),
                "source_refs": classification.get("source_refs", []),
                "remediation_template": classification.get("remediation_template", ""),
                "classifier_signals": classification.get("signals", []),
            }
        )
    AUDIT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with AUDIT_PATH.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, separators=(",", ":")) + "\n")
    print(f"recorded {rid}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
