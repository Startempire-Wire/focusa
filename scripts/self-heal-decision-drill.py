#!/usr/bin/env python3
"""Safe self-heal failure-injection drill.

This is a dry-run proof harness: it uses classifier fixtures to exercise the
same classifier/audit/summary path as release self-heal, but writes only to a
temporary audit ledger unless --audit is explicitly supplied.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests" / "fixtures" / "self-heal-classifier"
CLASSIFIER = ROOT / "scripts" / "classify-ci-failure.py"
RECORDER = ROOT / "scripts" / "record-workflow-failure.py"
AUTO_HEAL = ROOT / "scripts" / "auto-heal-audit.py"
SUMMARY = ROOT / "scripts" / "audit-failure-summary.py"


def classify(log_path: Path) -> dict:
    proc = subprocess.run(
        [sys.executable, str(CLASSIFIER), str(log_path)],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    )
    return json.loads(proc.stdout)


def decision_for(classification: dict) -> dict:
    retry_policy = classification.get("retry_policy", "")
    deterministic = bool(classification.get("deterministic"))
    if retry_policy == "hard_failure_no_rerun":
        return {
            "decision": "repair_required_no_rerun",
            "rerun_allowed": False,
            "repair_required": True,
            "reason": "deterministic failure must be patched before another run",
        }
    if retry_policy == "rerun_once":
        return {
            "decision": "rerun_once_allowed",
            "rerun_allowed": True,
            "repair_required": False,
            "reason": "transient/non-deterministic class permits one bounded rerun",
        }
    return {
        "decision": "unknown_policy_stop",
        "rerun_allowed": False,
        "repair_required": deterministic,
        "reason": f"unrecognized retry_policy={retry_policy!r}",
    }


def fixture_names(selected: str) -> list[str]:
    names = sorted(
        path.name.removesuffix(".expected.json")
        for path in FIXTURES.glob("*.expected.json")
    )
    if selected == "all":
        return names
    if selected not in names:
        raise SystemExit(f"unknown fixture {selected!r}; available: {', '.join(names)}")
    return [selected]


def record_failure(audit_path: Path, fixture_name: str, log_path: Path) -> None:
    env = os.environ.copy()
    env.update(
        {
            "AUDIT_PATH": str(audit_path),
            "FAILED_LOG_PATH": str(log_path),
            "RUN_ID": f"drill-{fixture_name}",
            "WORKFLOW": "Self-heal Failure Injection Drill",
            "HEAD_SHA": "dry-run",
            "BRANCH": "dry-run",
            "EVENT": "workflow_dispatch",
            "ATTEMPT": "1",
            "LOG_URL": f"fixture://self-heal-classifier/{fixture_name}.log",
            "TS": "2026-07-05T00:00:00Z",
        }
    )
    subprocess.run(
        [sys.executable, str(RECORDER)],
        cwd=ROOT,
        env=env,
        check=True,
        stdout=subprocess.DEVNULL,
    )


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    if not path.exists():
        return rows
    for raw in path.read_text().splitlines():
        raw = raw.strip()
        if raw:
            rows.append(json.loads(raw))
    return rows


def run_summary(audit_path: Path) -> str:
    proc = subprocess.run(
        [sys.executable, str(SUMMARY), "--audit", str(audit_path), "--limit", "50"],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    )
    return proc.stdout


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", default="all", help="Fixture name or 'all'")
    parser.add_argument(
        "--audit", help="Optional audit JSONL output path; defaults to temp file"
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON drill report")
    args = parser.parse_args(argv)

    audit_path = (
        Path(args.audit)
        if args.audit
        else Path(tempfile.mkdtemp(prefix="focusa-self-heal-drill-")) / "audit.jsonl"
    )
    names = fixture_names(args.fixture)
    cases: list[dict] = []
    for name in names:
        log_path = FIXTURES / f"{name}.log"
        expected_path = FIXTURES / f"{name}.expected.json"
        expected = json.loads(expected_path.read_text())
        classification = classify(log_path)
        for key in (
            "failure_class",
            "retry_policy",
            "deterministic",
            "source_refs",
            "signals",
            "remediation_template",
        ):
            if classification.get(key) != expected.get(key):
                raise SystemExit(
                    f"{name}: classifier {key} drifted: {classification.get(key)!r} != {expected.get(key)!r}"
                )
        decision = decision_for(classification)
        record_failure(audit_path, name, log_path)
        cases.append(
            {"fixture": name, "classification": classification, "decision": decision}
        )

    result_path = audit_path.with_name("self-heal-result.json")
    subprocess.run(
        [
            sys.executable,
            str(AUTO_HEAL),
            str(audit_path),
            "--result",
            str(result_path),
        ],
        cwd=ROOT,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    rows = load_jsonl(audit_path)
    failures = [row for row in rows if row.get("event") == "failure"]
    heals = [row for row in rows if row.get("event") == "self_heal"]
    summary = run_summary(audit_path)

    for case in cases:
        cls = case["classification"]["failure_class"]
        if cls not in summary:
            raise SystemExit(f"summary did not render failure_class {cls}")
        remediation = case["classification"].get("remediation_template", "")
        if remediation and remediation not in summary:
            raise SystemExit(f"summary did not render remediation for {cls}")

    payload = {
        "schema": "focusa.self_heal_failure_injection_drill.v1",
        "audit_path": str(audit_path),
        "case_count": len(cases),
        "failure_rows": len(failures),
        "self_heal_rows": len(heals),
        "cases": cases,
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(f"Self-heal failure injection drill: PASS ({len(cases)} cases)")
        for case in cases:
            cls = case["classification"]["failure_class"]
            decision = case["decision"]["decision"]
            print(f"- {case['fixture']}: {cls} -> {decision}")
        print(f"audit_path={audit_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
