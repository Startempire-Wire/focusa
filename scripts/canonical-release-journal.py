#!/usr/bin/env python3
"""Publish Focusa release lifecycle measurements to agent-kb-api."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import io
import json
import os
import re
import shlex
import statistics
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = "agent-kb.release_journal.event.v1"
PROTOCOL = "focusa.release_benchmark.v1"
UNMEASURED_BENCHMARK = {
    "status": "unmeasured",
    "reason": "No execution-backed ON/OFF/candidate observations are available",
}
PROJECT_ID = "focusa"
DEFAULT_API = "http://127.0.0.1:8791"
DEFAULT_FOCUSA_API = "http://127.0.0.1:8787"
FOCUSA_PROJECT_ROOT = os.environ.get("FOCUSA_PROJECT_ROOT", "/home/wirebot/focusa")
FOCUSA_CONTINUITY_ID = os.environ.get("FOCUSA_CONTINUITY_ID", "focusa-v0.9.135-locked-14")
WORKFLOW_NAMES = ("CI", "Release", "Deploy Live Daemon")
COMMAND_DIAGNOSTIC_BYTES = 2048
RETRYABLE_READ_ERRORS = (urllib.error.URLError, ConnectionError, TimeoutError)


def utcnow() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def parse_time(value: str) -> dt.datetime:
    return dt.datetime.fromisoformat(value.replace("Z", "+00:00"))


def _bounded_command_diagnostic(value: str) -> str:
    raw = value.encode("utf-8", errors="replace")[-COMMAND_DIAGNOSTIC_BYTES:]
    tail = raw.decode("utf-8", errors="replace")
    tail = re.sub(r"(?i)(bearer\s+)[^\s]+", r"\1[REDACTED]", tail)
    tail = re.sub(
        r"(?i)((?:api[_-]?key|token|password|secret)\s*[=:]\s*)[^\s,;]+",
        r"\1[REDACTED]",
        tail,
    )
    return tail


def command(args: list[str], *, cwd: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(args, cwd=cwd, text=True, capture_output=True, check=False)
    if check and result.returncode != 0:
        failed_check = " ".join(str(part) for part in args[:2])
        diagnostic = {
            "status": "blocked",
            "failed_check": _bounded_command_diagnostic(failed_check),
            "exit_code": result.returncode,
            "stdout_tail": _bounded_command_diagnostic(result.stdout or ""),
            "stderr_tail": _bounded_command_diagnostic(result.stderr or ""),
        }
        raise RuntimeError(
            "release benchmark command failed: "
            + json.dumps(diagnostic, sort_keys=True, ensure_ascii=False)
        )
    return result


def git(*args: str) -> str:
    return command(["git", *args]).stdout.strip()


def token() -> str:
    for name in ("AGENT_KB_RELEASE_TOKEN", "AGENT_KB_TOKEN"):
        value = os.environ.get(name, "").strip()
        if value:
            return value
    for path in (Path("/etc/agent-kb/release-publisher.token"), Path("/etc/agent-kb/token")):
        try:
            value = path.read_text().strip()
        except PermissionError:
            continue
        if value:
            return value
    raise RuntimeError("agent-kb release publisher token unavailable")


class JournalApiError(RuntimeError):
    def __init__(self, status_code: int, detail: str):
        self.status_code = status_code
        try:
            response = json.loads(detail)
            self.response = response if isinstance(response, dict) else {}
        except ValueError:
            self.response = {}
        super().__init__(f"agent-kb-api {status_code}: {detail}")


def api_request(method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
    base = os.environ.get("AGENT_KB_RELEASE_API_URL", os.environ.get("AGENT_KB_API_URL", DEFAULT_API)).rstrip("/")
    body = None if payload is None else json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    request = urllib.request.Request(
        base + path,
        data=body,
        method=method,
        headers={"Authorization": f"Bearer {token()}", "Content-Type": "application/json"},
    )
    read_retries = 0
    while True:
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.loads(response.read())
        except urllib.error.HTTPError as error:
            detail = error.read().decode(errors="replace")[:1000]
            raise JournalApiError(error.code, detail) from error
        except RETRYABLE_READ_ERRORS:
            # Only idempotent reads may be retried; an uncertain POST must never replay.
            if method != "GET" or read_retries >= 2:
                raise
            read_retries += 1
            time.sleep(1)


def focusa_headers() -> dict[str, str]:
    headers = {
        "Content-Type": "application/json",
        "x-scope-project-root": FOCUSA_PROJECT_ROOT,
        "x-scope-continuity-id": FOCUSA_CONTINUITY_ID,
    }
    auth_token = os.environ.get("FOCUSA_AUTH_TOKEN", "").strip()
    if auth_token:
        headers["Authorization"] = f"Bearer {auth_token}"
    return headers


# This adapter reaches the existing authoritative loopback daemon, never starts
# a daemon/relay, and carries authorization and payload only over encrypted stdin.
FOCUSA_SSH_HTTP = """import json, sys, urllib.request, urllib.error
p = json.load(sys.stdin)
data = None if p['payload'] is None else json.dumps(p['payload'], sort_keys=True).encode()
r = urllib.request.Request(p['url'], data=data, method=p['method'], headers=p['headers'])
try:
    response = urllib.request.urlopen(r, timeout=25)
except urllib.error.HTTPError as error:
    response = error
except urllib.error.URLError as error:
    if not isinstance(error.reason, ConnectionRefusedError):
        raise
    print(json.dumps({'status': 0, 'failure_class': 'daemon_connection_refused'}))
    sys.exit(0)
with response:
    body = response.read(4194305)
    if len(body) > 4194304:
        raise ValueError('Focusa response exceeds transport limit')
    print(json.dumps({'status': response.code, 'body': body.decode()}))
"""


def focusa_http(method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
    base = os.environ.get("FOCUSA_API_URL", DEFAULT_FOCUSA_API).rstrip("/")
    url = base + path
    headers = focusa_headers()
    ssh_host = os.environ.get("FOCUSA_API_SSH_HOST", "").strip()
    if ssh_host:
        parsed = urllib.parse.urlsplit(url)
        if (not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", ssh_host)
                or parsed.scheme not in ("http", "https")
                or parsed.hostname not in ("127.0.0.1", "localhost", "::1")
                or parsed.username is not None or parsed.password is not None):
            raise ValueError("Focusa SSH transport requires an explicit host alias and loopback URL")
        result = subprocess.run(
            ["ssh", "-T", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
             "-o", "StrictHostKeyChecking=yes", ssh_host,
             "python3 -c " + shlex.quote(FOCUSA_SSH_HTTP)],
            input=json.dumps({"url": url, "method": method, "headers": headers, "payload": payload}),
            text=True, capture_output=True, timeout=35, check=False,
        )
        if result.returncode:
            raise RuntimeError(f"Focusa SSH route failed (exit {result.returncode}): "
                               + _bounded_command_diagnostic(result.stderr or ""))
        reply = json.loads(result.stdout)
        if reply.get("status") == 0 and reply.get("failure_class") == "daemon_connection_refused":
            raise urllib.error.URLError(ConnectionRefusedError("authoritative Focusa connection refused"))
        if not 200 <= reply["status"] < 300:
            raise urllib.error.HTTPError(url, reply["status"], "Focusa request rejected", None,
                                         io.BytesIO(reply["body"].encode()))
        return json.loads(reply["body"])
    request = urllib.request.Request(
        url, data=None if payload is None else json.dumps(payload, sort_keys=True).encode(),
        method=method, headers=headers,
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read())


def focusa_request(path: str, payload: dict[str, Any]) -> dict[str, Any]:
    return focusa_http("POST", path, payload)


def focusa_get(path: str) -> dict[str, Any]:
    return focusa_http("GET", path)


def query_events(release_id: str | None = None, *, view: str | None = None, limit: int = 1000) -> dict[str, Any]:
    params: dict[str, str] = {"project_id": PROJECT_ID, "limit": str(limit)}
    if release_id:
        params["release_id"] = release_id
    if view:
        params["view"] = view
    return api_request("GET", "/v1/releases/journal?" + urllib.parse.urlencode(params))


def release_id(tag: str) -> str:
    return f"{PROJECT_ID}:{tag}"


def release_events(tag: str) -> list[dict[str, Any]]:
    return query_events(release_id(tag)).get("events", [])


def next_sequence(tag: str) -> int:
    return max((int(event["sequence"]) for event in release_events(tag)), default=0) + 1


def existing_phase(tag: str, phase: str) -> dict[str, Any] | None:
    return next((row for row in release_events(tag) if row.get("phase") == phase), None)


def event(
    tag: str,
    phase: str,
    sequence: int,
    *,
    event_id: str,
    protocol: str = PROTOCOL,
    estimates: dict[str, Any] | None = None,
    measurements: dict[str, Any] | None = None,
    problems: list[dict[str, Any]] | None = None,
    comparison: dict[str, Any] | None = None,
    evidence_refs: list[str] | None = None,
    source: str = "focusa_release_automation",
    observed_at: str | None = None,
) -> dict[str, Any]:
    return {
        "schema": SCHEMA,
        "event_id": event_id,
        "release_id": release_id(tag),
        "project_id": PROJECT_ID,
        "tag": tag,
        "phase": phase,
        "sequence": sequence,
        "observed_at": observed_at or utcnow(),
        "protocol_version": protocol,
        "source": source,
        "estimates": estimates or {},
        "measurements": measurements or {},
        "problems": problems or [],
        "comparison": comparison or {},
        "evidence_refs": evidence_refs or [f"git:commit:{git('rev-parse', 'HEAD')}"],
    }


def direct_master_acceptance(payload: dict[str, Any], receipt: dict[str, Any]) -> dict[str, Any] | None:
    """Verify a primary's durable event, not a nonexistent replica queue entry."""
    if not os.environ.get("AGENT_KB_RELEASE_API_URL", "").strip():
        return None
    health = api_request("GET", "/v1/health")
    if not (health.get("status") == "ok" and health.get("source") == "master"
            and health.get("freshness") == "fresh" and health.get("stale") is False
            and health.get("fallback_used") is False and health.get("master_configured") is False):
        return None
    history = query_events(str(payload["release_id"]))
    durable = next((row for row in history.get("events", [])
                    if row.get("event_id") == payload.get("event_id")), None)
    digest = receipt.get("event_hash")
    if (history.get("status") != "ok" or not isinstance(digest, str)
            or not re.fullmatch(r"[a-f0-9]{64}", digest) or durable is None
            or durable.get("event_hash") != digest
            or any(durable.get(key) != value for key, value in payload.items())):
        raise RuntimeError("direct master acceptance requires matching durable event, payload and hash")
    return {"basis": "authenticated_direct_master_read", "event_id": payload["event_id"],
            "event_hash": digest, "index_generation": health.get("index_generation")}


def publish(payload: dict[str, Any]) -> dict[str, Any]:
    receipt = api_request("POST", "/v1/releases/journal", payload)
    if os.environ.get("AGENT_KB_REQUIRE_MASTER_ACK", "1") != "0":
        event_id = str(payload.get("event_id", "")).strip()
        if not event_id:
            raise RuntimeError("release journal event_id required for master acknowledgement")
        primary_acceptance = direct_master_acceptance(payload, receipt)
        if primary_acceptance is not None:
            receipt["master_acknowledged"] = True
            receipt["master_acceptance"] = primary_acceptance
            return receipt
        replication_path = (
            "/v1/releases/journal?view=replication&event_id="
            + urllib.parse.quote(event_id, safe="")
        )
        # Replication can acknowledge a durably accepted event after 45s.
        # Allow a bounded three-minute window without relaxing master proof.
        deadline = time.monotonic() + 180
        last_transport_error: str | None = None
        while time.monotonic() < deadline:
            try:
                replication = api_request("GET", replication_path)
            except JournalApiError as error:
                if error.status_code != 404 or error.response.get("summary") != "release journal replication state not found":
                    raise
                # The accepted event can precede its replication projection.
                # Absence is pending, never acknowledgment or permission to publish.
                time.sleep(1)
                continue
            except RETRYABLE_READ_ERRORS as error:
                # The POST may already be durable; retry only the idempotent read.
                last_transport_error = type(error).__name__
                time.sleep(1)
                continue
            if (
                replication.get("status") == "ok"
                and replication.get("state") == "master_accepted"
                and replication.get("master_event_hash")
            ):
                receipt["master_acknowledged"] = True
                receipt["replication"] = replication
                break
            if replication.get("status") == "conflict":
                raise RuntimeError(
                    f"agent-kb master rejected conflicting event_id {event_id}"
                )
            time.sleep(1)
        else:
            diagnostic = f" (last transport error: {last_transport_error})" if last_transport_error else ""
            raise RuntimeError(
                f"agent-kb master acknowledgement timed out for event_id {event_id}{diagnostic}"
            )
    return receipt


def median(values: list[float]) -> float | None:
    return round(statistics.median(values), 3) if values else None


def metric_comparison(current: float | int | None, baseline: float | int | None, unit: str, lower_better: bool) -> dict[str, Any]:
    if current is None or baseline is None:
        return {"current": current, "baseline": baseline, "delta": None, "unit": unit, "direction": "not_comparable"}
    delta = round(float(current) - float(baseline), 3)
    if delta == 0:
        direction = "unchanged"
    elif (delta < 0) == lower_better:
        direction = "improved"
    else:
        direction = "degraded"
    return {"current": current, "baseline": baseline, "delta": delta, "unit": unit, "direction": direction}


def gh_json(args: list[str]) -> Any:
    result = command(["gh", *args])
    return json.loads(result.stdout)


def github_release(tag: str) -> dict[str, Any]:
    return gh_json(["api", f"repos/Startempire-Wire/focusa/releases/tags/{tag}"])


def workflow_metrics(commit_sha: str) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    data = gh_json(["api", f"repos/Startempire-Wire/focusa/actions/runs?head_sha={commit_sha}&per_page=100"])
    runs = data.get("workflow_runs", [])
    selected: dict[str, Any] = {}
    problems: list[dict[str, Any]] = []
    for name in WORKFLOW_NAMES:
        matching = [row for row in runs if row.get("name") == name]
        successful = [row for row in matching if row.get("status") == "completed" and row.get("conclusion") == "success"]
        chosen = sorted(successful, key=lambda row: row.get("updated_at", ""), reverse=True)
        if chosen:
            row = chosen[0]
            started = parse_time(row["run_started_at"])
            ended = parse_time(row["updated_at"])
            selected[name] = {
                "run_id": row["id"],
                "conclusion": row["conclusion"],
                "duration_seconds": round((ended - started).total_seconds(), 3),
                "started_at": row["run_started_at"],
                "completed_at": row["updated_at"],
                "url": row["html_url"],
            }
        for row in matching:
            if row.get("status") == "completed" and row.get("conclusion") not in ("success", "skipped", "neutral"):
                problems.append({
                    "stage": name,
                    "diagnosis": f"workflow concluded {row.get('conclusion')}",
                    "impact": "release workflow retry or recovery required",
                    "recovery": "subsequent successful exact-commit run" if successful else "unresolved",
                    "run_id": row.get("id"),
                    "evidence_ref": row.get("html_url"),
                })
    return selected, problems


def release_measurements(tag: str) -> tuple[dict[str, Any], list[dict[str, Any]], list[str]]:
    release = github_release(tag)
    commit_sha = git("rev-list", "-n1", tag)
    workflows, problems = workflow_metrics(commit_sha)
    starts = [parse_time(row["started_at"]) for row in workflows.values()]
    ends = [parse_time(row["completed_at"]) for row in workflows.values()]
    remote_seconds = round((max(ends) - min(starts)).total_seconds(), 3) if starts and ends else None
    assets = [asset["name"] for asset in release.get("assets", [])]
    measurements = {
        "release_commit": commit_sha,
        "published_at": release.get("published_at"),
        "asset_count": len(assets),
        "signed_asset_count": len([name for name in assets if name.endswith(".sig")]),
        "checksum_manifest_present": "SHA256SUMS.txt" in assets,
        "release_manifest_present": "release-manifest.json" in assets,
        "deploy_receipt_present": "deploy-success.json" in assets,
        "workflow_count": len(workflows),
        "workflows": workflows,
        "remote_pipeline_seconds": remote_seconds,
        "problems_count": len(problems),
    }
    refs = [release.get("html_url", f"github:release:{tag}")] + [row["url"] for row in workflows.values()]
    return measurements, problems, refs


def latest_final(exclude_release_id: str | None = None) -> dict[str, Any] | None:
    releases = query_events(view="releases").get("releases", [])
    finals = [row for row in releases if row.get("actuals") and row.get("release_id") != exclude_release_id]
    return finals[-1] if finals else None


def historical_comparison(measurements: dict[str, Any], baseline: dict[str, Any] | None) -> dict[str, Any]:
    actuals = (baseline or {}).get("actuals", {})
    metrics = {
        "remote_pipeline_seconds": metric_comparison(measurements.get("remote_pipeline_seconds"), actuals.get("remote_pipeline_seconds"), "seconds", True),
        "problems_count": metric_comparison(measurements.get("problems_count"), actuals.get("problems_count"), "count", True),
        "asset_count": metric_comparison(measurements.get("asset_count"), actuals.get("asset_count"), "count", False),
    }
    return {"baseline_release_id": (baseline or {}).get("release_id"), "metrics": metrics}


def cmd_backfill(args: argparse.Namespace) -> dict[str, Any]:
    receipts = []
    for tag in args.tags:
        rid = release_id(tag)
        if query_events(rid).get("events"):
            receipts.append({"release_id": rid, "status": "already_present"})
            continue
        measurements, problems, refs = release_measurements(tag)
        release = github_release(tag)
        baseline = latest_final(rid)
        payload = event(
            tag, "final", 1, event_id=f"{rid}:historical-backfill:v1",
            protocol="focusa.release_metadata.v1", measurements=measurements,
            problems=problems, comparison=historical_comparison(measurements, baseline),
            evidence_refs=refs, source="historical_backfill", observed_at=release["published_at"],
        )
        receipts.append(publish(payload))
    return {"status": "completed", "events": receipts}


def estimate_from_history() -> dict[str, Any]:
    releases = query_events(view="releases").get("releases", [])
    actuals = [row.get("actuals", {}) for row in releases if row.get("actuals")]
    workflow_values: dict[str, list[float]] = {name: [] for name in WORKFLOW_NAMES}
    for row in actuals:
        for name in WORKFLOW_NAMES:
            value = row.get("workflows", {}).get(name, {}).get("duration_seconds")
            if isinstance(value, (int, float)):
                workflow_values[name].append(float(value))
    remote = [float(row["remote_pipeline_seconds"]) for row in actuals if isinstance(row.get("remote_pipeline_seconds"), (int, float))]
    assets = [float(row["asset_count"]) for row in actuals if isinstance(row.get("asset_count"), (int, float))]
    problems = [float(row["problems_count"]) for row in actuals if isinstance(row.get("problems_count"), (int, float))]
    remote_estimate = median(remote) or 1800
    return {
        "total_elapsed_seconds": round(remote_estimate + 1200, 3),
        "local_preparation_seconds": 1200,
        "remote_pipeline_seconds": remote_estimate,
        "workflow_seconds": {name: median(values) for name, values in workflow_values.items()},
        "asset_count": round(median(assets) or 60),
        "problems_count": round(median(problems) or 0),
        "required_workflow_count": len(WORKFLOW_NAMES),
        "estimate_source": "historical_median_plus_explicit_1200s_local_preparation",
        "historical_samples": len(actuals),
    }


def cmd_plan(args: argparse.Namespace) -> dict[str, Any]:
    rid = release_id(args.tag)
    existing = existing_phase(args.tag, "plan")
    if existing:
        return {"status": "already_present", "event_id": existing["event_id"], "event_hash": existing["event_hash"]}
    guard_config = json.loads((ROOT / "config/release-learning-guards.json").read_text())
    guards = {
        "status": "planned",
        "guards_total": len(guard_config.get("guards", [])),
        "guards": [
            {"failure_class": row["failure_class"], "lesson_ref": row["lesson_ref"]}
            for row in guard_config.get("guards", [])
        ],
    }
    payload = event(
        args.tag, "plan", 1, event_id=f"{rid}:plan:v1",
        estimates=estimate_from_history(),
        measurements={
            "candidate_commit": git("rev-parse", "HEAD"),
            "channel": args.channel,
            "benchmark": dict(UNMEASURED_BENCHMARK),
            "risks": [
                "stable version-surface mismatch",
                "exact-tag CI failure",
                "release asset or signature incompleteness",
                "production version mismatch",
            ],
            "learning": {
                "status": "deferred",
                "reason": "learning_runs_outside_publication",
                "recurrence_guards": guards,
            },
        },
        evidence_refs=[f"git:commit:{git('rev-parse', 'HEAD')}", "agent-kb-api:view=releases", "config:release-learning-guards"],
    )
    return publish(payload)


def run_benchmark(tag: str) -> None:
    # This explicit diagnostic cannot record a score from scenario definitions.
    command(["bash", "scripts/run-agent-intelligence-evals.sh"])
    # Even a zero exit from an older runner is not comparative evidence.
    raise RuntimeError(
        "UNMEASURED: agent-intelligence runner completion is not execution-backed "
        "ON/OFF/candidate evidence; no verified benchmark score or release proof is available"
    )


def cmd_benchmark(args: argparse.Namespace) -> dict[str, Any]:
    existing = existing_phase(args.tag, "benchmark")
    if existing:
        # Preserve append-only history, but do not re-accept fixture-derived v1 scores.
        raise RuntimeError(
            "UNMEASURED: historical benchmark records are preserved, not accepted "
            "as execution-backed ON/OFF/candidate evidence"
        )
    run_benchmark(args.tag)


def cmd_progress(args: argparse.Namespace) -> dict[str, Any]:
    sequence = next_sequence(args.tag)
    evidence_refs = args.evidence_ref or [f"release-stage:{args.stage}"]
    payload = event(
        args.tag, "progress", sequence,
        event_id=f"{release_id(args.tag)}:progress:{args.stage}:{sequence}",
        measurements={
            "stage": args.stage,
            "stage_status": args.status,
            "details": args.details or "",
        },
        evidence_refs=evidence_refs,
    )
    return publish(payload)


def cmd_problem(args: argparse.Namespace) -> dict[str, Any]:
    sequence = next_sequence(args.tag)
    evidence_refs = args.evidence_ref or [f"release-problem:{args.stage}"]
    problem = {
        "stage": args.stage, "diagnosis": args.diagnosis, "impact": args.impact,
        "recovery": args.recovery, "added_duration_seconds": args.added_duration_seconds,
        "failure_fingerprint": f"{args.stage}:{hashlib.sha256(args.diagnosis.encode()).hexdigest()[:16]}",
    }
    payload = event(
        args.tag, "problem", sequence,
        event_id=f"{release_id(args.tag)}:problem:{args.stage}:{sequence}",
        measurements={"stage": args.stage},
        problems=[problem],
        evidence_refs=evidence_refs,
    )
    return publish(payload)


def estimate_deltas(actuals: dict[str, Any], estimates: dict[str, Any]) -> dict[str, Any]:
    return {
        "total_elapsed_seconds": metric_comparison(actuals.get("total_elapsed_seconds"), estimates.get("total_elapsed_seconds"), "seconds", True),
        "remote_pipeline_seconds": metric_comparison(actuals.get("remote_pipeline_seconds"), estimates.get("remote_pipeline_seconds"), "seconds", True),
        "asset_count": metric_comparison(actuals.get("asset_count"), estimates.get("asset_count"), "count", False),
        "problems_count": metric_comparison(actuals.get("problems_count"), estimates.get("problems_count"), "count", True),
    }


def production_version() -> str:
    return str(focusa_get("/v1/health").get("version", ""))


def cmd_finalize(args: argparse.Namespace) -> dict[str, Any]:
    rid = release_id(args.tag)
    current_events = query_events(rid).get("events", [])
    existing = next((row for row in current_events if row.get("phase") == "final"), None)
    if existing:
        return {"status": "already_present", "event_id": existing["event_id"], "event_hash": existing["event_hash"], "actuals": existing["measurements"], "comparison": existing["comparison"]}
    if not current_events:
        raise RuntimeError("release plan is missing")
    plan = next((row for row in current_events if row["phase"] == "plan"), None)
    if plan is None:
        raise RuntimeError("release plan is missing")
    actuals, discovered_problems, refs = release_measurements(args.tag)
    expected_version = args.tag.removeprefix("v")
    actuals["production_version"] = production_version()
    actuals["total_elapsed_seconds"] = round((parse_time(utcnow()) - parse_time(plan["observed_at"])).total_seconds(), 3)
    # Legacy benchmark records are append-only history, not verified observations.
    actuals["benchmark"] = dict(UNMEASURED_BENCHMARK)
    prior_problems = [problem for row in current_events for problem in row.get("problems", [])]
    all_problems = prior_problems + discovered_problems
    actuals["problems_count"] = len(all_problems)
    if actuals["production_version"] != expected_version:
        raise RuntimeError(f"production version {actuals['production_version']} != {expected_version}")
    if actuals["workflow_count"] != len(WORKFLOW_NAMES):
        raise RuntimeError("required successful workflow evidence is incomplete")
    if not all((actuals["checksum_manifest_present"], actuals["release_manifest_present"], actuals["deploy_receipt_present"])):
        raise RuntimeError("required release assets are incomplete")
    baseline = latest_final(rid)
    comparison = {
        "against_estimate": estimate_deltas(actuals, plan.get("estimates", {})),
        "against_previous_release": historical_comparison(actuals, baseline),
    }
    guard_artifact = Path(
        os.environ.get("FOCUSA_LEARNING_GUARDS_ARTIFACT", "").strip()
        or f"/tmp/focusa-{os.getuid()}-{args.tag.removeprefix('v')}-learning-guards.json"
    )
    guard_result = json.loads(guard_artifact.read_text()) if guard_artifact.exists() else {"guards": []}
    actuals["learning"] = {
        "status": "deferred",
        "recurrence_guards_total": guard_result.get("guards_total", 0),
        "recurrence_guards_passed": len([
            row for row in guard_result.get("guards", []) if row.get("status") == "passed"
        ]),
    }
    evidence_refs = refs + ["production:http://127.0.0.1:8787/v1/health"]
    payload = event(
        args.tag, "final", next_sequence(args.tag), event_id=f"{rid}:final:v1",
        estimates=plan.get("estimates", {}), measurements=actuals, problems=all_problems,
        comparison=comparison, evidence_refs=evidence_refs,
    )
    receipt = publish(payload)
    return {"status": "completed", "receipt": receipt, "actuals": actuals, "comparison": comparison}


def cmd_correction(args: argparse.Namespace) -> dict[str, Any]:
    rid = release_id(args.tag)
    problem = {
        "stage": args.stage,
        "diagnosis": args.corrected_fact,
        "impact": args.impact,
        "recovery": args.recovery,
        "added_duration_seconds": None,
    }
    evidence_refs = args.evidence_ref or [f"release:{args.tag}:correction"]
    payload = event(
        args.tag,
        "correction",
        next_sequence(args.tag),
        event_id=f"{rid}:correction:{args.stage}:v1",
        measurements={"status": "corrected", "corrected_fact": args.corrected_fact},
        problems=[problem],
        comparison={"supersedes_event_id": args.supersedes_event_id},
        evidence_refs=evidence_refs,
    )
    return publish(payload)


def cmd_history(args: argparse.Namespace) -> dict[str, Any]:
    if args.release_id:
        return query_events(args.release_id, limit=args.limit)
    return query_events(view="releases", limit=args.limit)


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    sub = root.add_subparsers(dest="command", required=True)
    backfill = sub.add_parser("backfill")
    backfill.add_argument("--tags", nargs="+", required=True)
    backfill.set_defaults(func=cmd_backfill)
    for name, func in (("plan", cmd_plan), ("benchmark", cmd_benchmark), ("finalize", cmd_finalize)):
        item = sub.add_parser(name)
        item.add_argument("--tag", required=True)
        item.add_argument("--channel", default="stable")
        item.set_defaults(func=func)
    progress = sub.add_parser("progress")
    progress.add_argument("--tag", required=True)
    progress.add_argument("--stage", required=True)
    progress.add_argument("--status", default="completed")
    progress.add_argument("--details")
    progress.add_argument("--evidence-ref", action="append")
    progress.set_defaults(func=cmd_progress)
    problem = sub.add_parser("problem")
    problem.add_argument("--tag", required=True)
    problem.add_argument("--stage", required=True)
    problem.add_argument("--diagnosis", required=True)
    problem.add_argument("--impact", default="release progress affected")
    problem.add_argument("--recovery", default="pending")
    problem.add_argument("--added-duration-seconds", type=float)
    problem.add_argument("--evidence-ref", action="append")
    problem.set_defaults(func=cmd_problem)
    correction = sub.add_parser("correction")
    correction.add_argument("--tag", required=True)
    correction.add_argument("--stage", required=True)
    correction.add_argument("--supersedes-event-id", required=True)
    correction.add_argument("--corrected-fact", required=True)
    correction.add_argument("--impact", default="prior final event required factual correction")
    correction.add_argument("--recovery", default="publish an immutable successor release")
    correction.add_argument("--evidence-ref", action="append")
    correction.set_defaults(func=cmd_correction)
    history = sub.add_parser("history")
    history.add_argument("--project-id", default=PROJECT_ID, choices=[PROJECT_ID])
    history.add_argument("--release-id")
    history.add_argument("--limit", type=int, default=50)
    history.set_defaults(func=cmd_history)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        result = args.func(args)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except (RuntimeError, subprocess.CalledProcessError, urllib.error.URLError, json.JSONDecodeError) as error:
        print(json.dumps({"status": "blocked", "error": str(error)[:1000]}, sort_keys=True), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
