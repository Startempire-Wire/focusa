#!/usr/bin/env python3
"""Non-mutating deploy self-heal proof drill.

Proves deploy-path self-heal behavior without installing/restarting anything:
- deploy_health_failure => rerun_once_allowed
- deterministic deploy/config/process failure => repair_required_no_rerun
- audit failure rows + synthesized self_heal rows exist
- audit summary renders remediation
- optional live /v1/health read-only check remains ok
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DECISION_DRILL = ROOT / "scripts" / "self-heal-decision-drill.py"
SUMMARY = ROOT / "scripts" / "audit-failure-summary.py"


def run_json(cmd: list[str]) -> dict:
    proc = subprocess.run(cmd, cwd=ROOT, check=True, text=True, capture_output=True)
    return json.loads(proc.stdout)


def check_health(url: str, timeout: float) -> dict:
    if url.lower() in {"skip", "none", ""}:
        return {"checked": False, "ok": None, "reason": "skipped"}
    try:
        with urllib.request.urlopen(url, timeout=timeout) as response:
            body = response.read().decode("utf-8")
        payload = json.loads(body)
        return {
            "checked": True,
            "ok": bool(payload.get("ok") or payload.get("status") == "ok"),
            "status": payload.get("status"),
            "version": payload.get("version"),
            "url": url,
        }
    except Exception as exc:
        return {"checked": True, "ok": False, "url": url, "error": str(exc)}


def case_by_fixture(payload: dict, fixture: str) -> dict:
    for case in payload.get("cases", []):
        if case.get("fixture") == fixture:
            return case
    raise SystemExit(f"fixture {fixture!r} missing from drill payload")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--health-url", default="http://127.0.0.1:8787/v1/health")
    parser.add_argument("--health-timeout", type=float, default=3.0)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    audit_path = Path(tempfile.mkdtemp(prefix="focusa-deploy-heal-drill-")) / "audit.jsonl"
    deploy_payload = run_json([
        sys.executable,
        str(DECISION_DRILL),
        "--fixture",
        "deploy_health_failure",
        "--audit",
        str(audit_path),
        "--json",
    ])
    deterministic_payload = run_json([
        sys.executable,
        str(DECISION_DRILL),
        "--fixture",
        "auto_heal_process_error",
        "--audit",
        str(audit_path),
        "--json",
    ])

    deploy_case = case_by_fixture(deploy_payload, "deploy_health_failure")
    deterministic_case = case_by_fixture(deterministic_payload, "auto_heal_process_error")
    if deploy_case["decision"]["decision"] != "rerun_once_allowed":
        raise SystemExit(f"deploy health decision mismatch: {deploy_case['decision']}")
    if not deploy_case["decision"]["rerun_allowed"]:
        raise SystemExit("deploy health failure did not allow one bounded rerun")
    if deterministic_case["decision"]["decision"] != "repair_required_no_rerun":
        raise SystemExit(f"deterministic decision mismatch: {deterministic_case['decision']}")
    if deterministic_case["decision"]["rerun_allowed"]:
        raise SystemExit("deterministic failure unexpectedly allowed rerun")

    summary = subprocess.run(
        [sys.executable, str(SUMMARY), "--audit", str(audit_path), "--limit", "20"],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    ).stdout
    for needle in (
        "deploy_health_failure",
        "auto_heal_process_error",
        "Retry once; if repeated, inspect service logs",
        "Patch Auto Heal/Watchdog process before retrying",
    ):
        if needle not in summary:
            raise SystemExit(f"audit summary missing {needle!r}")

    rows = [json.loads(raw) for raw in audit_path.read_text().splitlines() if raw.strip()]
    failure_rows = [row for row in rows if row.get("event") == "failure"]
    heal_rows = [row for row in rows if row.get("event") == "self_heal"]
    if len(failure_rows) != 2 or len(heal_rows) != 2:
        raise SystemExit(f"expected 2 failure + 2 self_heal rows, got {len(failure_rows)} + {len(heal_rows)}")

    health = check_health(args.health_url, args.health_timeout)
    if health.get("checked") and not health.get("ok"):
        raise SystemExit(f"live health check failed: {health}")

    result = {
        "schema": "focusa.deploy_self_heal_proof_drill.v1",
        "audit_path": str(audit_path),
        "failure_rows": len(failure_rows),
        "self_heal_rows": len(heal_rows),
        "deploy_health_decision": deploy_case["decision"],
        "deterministic_decision": deterministic_case["decision"],
        "health": health,
    }
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print("Deploy self-heal proof drill: PASS")
        print(f"deploy_health_failure -> {deploy_case['decision']['decision']}")
        print(f"auto_heal_process_error -> {deterministic_case['decision']['decision']}")
        print(f"failure_rows={len(failure_rows)} self_heal_rows={len(heal_rows)}")
        if health.get("checked"):
            print(f"health ok version={health.get('version')} url={health.get('url')}")
        else:
            print("health skipped")
        print(f"audit_path={audit_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
