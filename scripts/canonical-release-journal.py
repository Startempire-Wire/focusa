#!/usr/bin/env python3
"""Publish Focusa release lifecycle measurements to agent-kb-api."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
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
PROJECT_ID = "focusa"
DEFAULT_API = "http://127.0.0.1:8791"
DEFAULT_FOCUSA_API = "http://127.0.0.1:8787"
FOCUSA_PROJECT_ROOT = os.environ.get("FOCUSA_PROJECT_ROOT", "/home/wirebot/focusa")
FOCUSA_PROJECT_FINGERPRINT = os.environ.get("FOCUSA_PROJECT_FINGERPRINT", "project-fnv1a64:c435b14d4fb3ab67")
FOCUSA_CONTINUITY_ID = os.environ.get("FOCUSA_CONTINUITY_ID", "focusa-v0.9.135-locked-14")
WORKFLOW_NAMES = ("CI", "Release", "Deploy Live Daemon")


def utcnow() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def parse_time(value: str) -> dt.datetime:
    return dt.datetime.fromisoformat(value.replace("Z", "+00:00"))


def command(args: list[str], *, cwd: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=cwd, text=True, capture_output=True, check=check)


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


def api_request(method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
    base = os.environ.get("AGENT_KB_API_URL", DEFAULT_API).rstrip("/")
    body = None if payload is None else json.dumps(payload, sort_keys=True, separators=(",", ":")).encode()
    request = urllib.request.Request(
        base + path,
        data=body,
        method=method,
        headers={"Authorization": f"Bearer {token()}", "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.loads(response.read())
    except urllib.error.HTTPError as error:
        detail = error.read().decode(errors="replace")[:1000]
        raise RuntimeError(f"agent-kb-api {error.code}: {detail}") from error


def focusa_request(path: str, payload: dict[str, Any]) -> dict[str, Any]:
    base = os.environ.get("FOCUSA_API_URL", DEFAULT_FOCUSA_API).rstrip("/")
    request = urllib.request.Request(
        base + path,
        data=json.dumps(payload, sort_keys=True).encode(),
        method="POST",
        headers={
            "Content-Type": "application/json",
            "x-scope-project-root": FOCUSA_PROJECT_ROOT,
            "x-scope-continuity-id": FOCUSA_CONTINUITY_ID,
        },
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read())


def focusa_get(path: str) -> dict[str, Any]:
    base = os.environ.get("FOCUSA_API_URL", DEFAULT_FOCUSA_API).rstrip("/")
    with urllib.request.urlopen(base + path, timeout=30) as response:
        return json.loads(response.read())


def prediction_scope() -> dict[str, Any]:
    return {
        "root_scope": {
            "scope_kind": "project",
            "scope_id": "focusa",
            "root_path": FOCUSA_PROJECT_ROOT,
            "canonical_name": "focusa",
            "fingerprint": FOCUSA_PROJECT_FINGERPRINT,
        },
        "continuity_id": FOCUSA_CONTINUITY_ID,
    }


def retrieve_release_lessons(tag: str) -> dict[str, Any]:
    response = focusa_request(
        "/v1/metacognition/retrieve",
        {
            "current_ask": f"Plan {tag} without repeating canonical release failures or slowdowns",
            "scope_tags": ["release", "canonical-release", "failure-prevention"],
            "k": 10,
            "project_root": FOCUSA_PROJECT_ROOT,
            "continuity_id": FOCUSA_CONTINUITY_ID,
        },
    )
    candidates = response.get("candidates", [])[:10]
    return {
        "candidate_count": len(candidates),
        "lessons": [
            {
                "capture_id": row.get("capture_id"),
                "kind": row.get("kind"),
                "strategy_class": row.get("strategy_class"),
                "summary": row.get("summary"),
                "confidence": row.get("confidence"),
            }
            for row in candidates
        ],
    }


def record_release_predictions(tag: str) -> dict[str, str]:
    predictions = {}
    query = urllib.parse.urlencode(
        {
            "scope_kind": "project",
            "scope_id": "focusa",
            "root_path": FOCUSA_PROJECT_ROOT,
            "canonical_name": "focusa",
            "fingerprint": FOCUSA_PROJECT_FINGERPRINT,
            "continuity_id": FOCUSA_CONTINUITY_ID,
            "limit": 100,
        }
    )
    recent = focusa_get("/v1/predictions/recent?" + query).get("data", {}).get("predictions", [])
    stages = {
        "benchmark": "candidate benchmark passes every required release protocol check",
        "candidate-ci": "exact stamped candidate CI passes before immutable tagging",
        "release": "GitHub Release completes with signed complete assets",
        "deploy": "production Deploy completes and post-install OTA trust resolves",
        "final": "release finalizes within its journal estimate with production and learning proof",
    }
    for stage, outcome in stages.items():
        predicted_outcome = f"{tag}: {outcome}"
        existing = next(
            (
                row for row in recent
                if row.get("prediction", {}).get("prediction_type") == f"release_{stage}_success"
                and row.get("prediction", {}).get("predicted_outcome") == predicted_outcome
                and row.get("prediction", {}).get("evaluated_at") is None
            ),
            None,
        )
        if existing:
            predictions[stage] = existing["record_id"]
            continue
        response = focusa_request(
            "/v1/predictions",
            {
                "scope": prediction_scope(),
                "prediction_type": f"release_{stage}_success",
                "context_refs": [f"release:{tag}"],
                "predicted_outcome": predicted_outcome,
                "confidence": 0.9,
                "recommended_action": f"Run and evidence the {stage} guard before settlement",
                "why": "Prior release problems are now explicit recurrence guards in the measured release cycle",
            },
        )
        prediction_id = response.get("data", {}).get("record", {}).get("record_id")
        if prediction_id:
            predictions[stage] = prediction_id
    return predictions


def capture_release_lesson(stage: str, diagnosis: str, recovery: str, evidence_refs: list[str]) -> str | None:
    try:
        response = focusa_request(
            "/v1/metacognition/capture",
            {
                "kind": "release_failure_lesson",
                "content": f"{stage}: {diagnosis}",
                "rationale": recovery,
                "evidence_refs": evidence_refs,
                "confidence": 1.0,
                "strategy_class": f"release_{stage.replace('-', '_')}",
                "project_root": FOCUSA_PROJECT_ROOT,
                "continuity_id": FOCUSA_CONTINUITY_ID,
            },
        )
        return response.get("capture_id") or response.get("id")
    except Exception:
        return None


def prediction_for_stage(tag: str, stage: str) -> str | None:
    plan = next((row for row in release_events(tag) if row.get("phase") == "plan"), {})
    return plan.get("measurements", {}).get("learning", {}).get("predictions", {}).get(stage)


def evaluate_stage_prediction(tag: str, stage: str, outcome: str, score: float, lesson_ref: str | None = None) -> dict[str, Any] | None:
    prediction_id = prediction_for_stage(tag, stage)
    if not prediction_id:
        return None
    try:
        return focusa_request(
            f"/v1/predictions/{prediction_id}/evaluate",
            {
                "scope": prediction_scope(),
                "actual_outcome": outcome,
                "score": score,
                "learning_signal_ref": lesson_ref,
            },
        )
    except Exception:
        return None


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


def publish(payload: dict[str, Any]) -> dict[str, Any]:
    receipt = api_request("POST", "/v1/releases/journal", payload)
    if os.environ.get("AGENT_KB_REQUIRE_MASTER_ACK", "1") != "0":
        event_id = str(payload.get("event_id", "")).strip()
        if not event_id:
            raise RuntimeError("release journal event_id required for master acknowledgement")
        replication_path = (
            "/v1/releases/journal?view=replication&event_id="
            + urllib.parse.quote(event_id, safe="")
        )
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            replication = api_request("GET", replication_path)
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
            raise RuntimeError(
                f"agent-kb master acknowledgement timed out for event_id {event_id}"
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
        "agent_intelligence_minimum_score": 0.8,
        "estimate_source": "historical_median_plus_explicit_1200s_local_preparation",
        "historical_samples": len(actuals),
    }


def cmd_plan(args: argparse.Namespace) -> dict[str, Any]:
    rid = release_id(args.tag)
    existing = existing_phase(args.tag, "plan")
    if existing:
        return {"status": "already_present", "event_id": existing["event_id"], "event_hash": existing["event_hash"]}
    lessons = retrieve_release_lessons(args.tag)
    predictions = record_release_predictions(args.tag)
    guard_config = json.loads((ROOT / "config/release-learning-guards.json").read_text())
    guards = {
        "status": "planned",
        "guards_total": len(guard_config.get("guards", [])),
        "guards": [
            {"failure_class": row["failure_class"], "lesson_ref": row["lesson_ref"]}
            for row in guard_config.get("guards", [])
        ],
    }
    learning_refs = [
        f"metacog:{row['capture_id']}" for row in lessons["lessons"] if row.get("capture_id")
    ] + [f"prediction:{value}" for value in predictions.values()]
    payload = event(
        args.tag, "plan", 1, event_id=f"{rid}:plan:v1",
        estimates=estimate_from_history(),
        measurements={
            "candidate_commit": git("rev-parse", "HEAD"),
            "channel": args.channel,
            "risks": [
                "stable version-surface mismatch",
                "benchmark regression",
                "exact-tag CI failure",
                "release asset or signature incompleteness",
                "production version mismatch",
            ],
            "learning": {
                "retrieved_lessons": lessons,
                "predictions": predictions,
                "recurrence_guards": guards,
            },
        },
        evidence_refs=[f"git:commit:{git('rev-parse', 'HEAD')}", "agent-kb-api:view=releases", "config:release-learning-guards", *learning_refs],
    )
    return publish(payload)


def run_benchmark(tag: str) -> dict[str, Any]:
    started = time.monotonic()
    agent_run = command(["bash", "scripts/run-agent-intelligence-evals.sh"])
    cases = json.loads((ROOT / "tests/evals/agent_intelligence_cases.json").read_text())
    aggregate = sum(float(row["score"]) for row in cases["cases"]) / len(cases["cases"])
    live_run = command([sys.executable, "scripts/spec135-live-performance-proof.py"])
    live = json.loads(live_run.stdout)
    gap = command(["bash", "tests/final_release_gap_gate.sh"])
    gate = command([sys.executable, "scripts/release-gate.py"])
    version = command([sys.executable, "scripts/verify-version-surfaces.py", tag])
    gate_score_match = re.search(r"score=(\d+)", gate.stdout + gate.stderr)
    return {
        "status": "passed",
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "agent_intelligence": {
            "status": "passed", "case_count": len(cases["cases"]),
            "category_count": len(cases["required_categories"]),
            "aggregate_score": round(aggregate, 6), "threshold": cases["aggregate_threshold"],
            "result": agent_run.stdout.strip()[-240:],
        },
        "live_performance": live,
        "final_release_gap_gate": {"status": "passed", "result": gap.stdout.strip()[-240:]},
        "release_gate": {"status": "passed", "score": int(gate_score_match.group(1)) if gate_score_match else None},
        "version_surfaces": {"status": "passed", "tag": tag, "result": version.stdout.strip()[-240:]},
    }


def cmd_benchmark(args: argparse.Namespace) -> dict[str, Any]:
    existing = existing_phase(args.tag, "benchmark")
    if existing:
        return {"status": "already_present", "event_id": existing["event_id"], "event_hash": existing["event_hash"], "measurements": existing["measurements"]}
    measurements = run_benchmark(args.tag)
    artifact = Path(args.artifact or f"/tmp/focusa-{args.tag.removeprefix('v')}-benchmark.json")
    artifact.write_text(json.dumps(measurements, indent=2, sort_keys=True) + "\n")
    rid = release_id(args.tag)
    payload = event(
        args.tag, "benchmark", next_sequence(args.tag), event_id=f"{rid}:benchmark:v1",
        measurements=measurements,
        comparison={"baseline_release_id": None, "direction": "not_comparable", "reason": "first focusa.release_benchmark.v1 measurement"},
        evidence_refs=[f"artifact:{artifact}", "scripts/run-agent-intelligence-evals.sh", "scripts/spec135-live-performance-proof.py", "tests/final_release_gap_gate.sh", "scripts/release-gate.py", "scripts/verify-version-surfaces.py"],
    )
    receipt = publish(payload)
    prediction_evaluation = evaluate_stage_prediction(
        args.tag, "benchmark", f"{args.tag} benchmark passed protocol v1", 1.0
    )
    return {"status": "completed", "artifact": str(artifact), "receipt": receipt, "measurements": measurements, "prediction_evaluation": prediction_evaluation}


def cmd_progress(args: argparse.Namespace) -> dict[str, Any]:
    sequence = next_sequence(args.tag)
    evidence_refs = args.evidence_ref or [f"release-stage:{args.stage}"]
    evaluation = None
    if args.status == "completed" and args.stage in {"candidate-ci", "release", "deploy"}:
        evaluation = evaluate_stage_prediction(
            args.tag, args.stage, f"{args.tag} {args.stage} completed", 1.0
        )
    payload = event(
        args.tag, "progress", sequence,
        event_id=f"{release_id(args.tag)}:progress:{args.stage}:{sequence}",
        measurements={
            "stage": args.stage,
            "stage_status": args.status,
            "details": args.details or "",
            "learning": {"prediction_evaluation": evaluation},
        },
        evidence_refs=evidence_refs,
    )
    return publish(payload)


def cmd_problem(args: argparse.Namespace) -> dict[str, Any]:
    sequence = next_sequence(args.tag)
    evidence_refs = args.evidence_ref or [f"release-problem:{args.stage}"]
    lesson_ref = capture_release_lesson(args.stage, args.diagnosis, args.recovery, evidence_refs)
    evaluation = evaluate_stage_prediction(
        args.tag,
        args.stage,
        f"{args.tag} {args.stage} failed: {args.diagnosis}",
        0.0,
        f"metacog:{lesson_ref}" if lesson_ref else None,
    )
    problem = {
        "stage": args.stage, "diagnosis": args.diagnosis, "impact": args.impact,
        "recovery": args.recovery, "added_duration_seconds": args.added_duration_seconds,
        "failure_fingerprint": f"{args.stage}:{hashlib.sha256(args.diagnosis.encode()).hexdigest()[:16]}",
    }
    payload = event(
        args.tag, "problem", sequence,
        event_id=f"{release_id(args.tag)}:problem:{args.stage}:{sequence}",
        measurements={
            "stage": args.stage,
            "learning": {
                "metacog_capture_id": lesson_ref,
                "prediction_evaluation": evaluation,
            },
        },
        problems=[problem],
        evidence_refs=[*evidence_refs, *([f"metacog:{lesson_ref}"] if lesson_ref else [])],
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
    with urllib.request.urlopen("http://127.0.0.1:8787/v1/health", timeout=10) as response:
        return str(json.loads(response.read()).get("version", ""))


def cmd_finalize(args: argparse.Namespace) -> dict[str, Any]:
    rid = release_id(args.tag)
    current_events = query_events(rid).get("events", [])
    existing = next((row for row in current_events if row.get("phase") == "final"), None)
    if existing:
        return {"status": "already_present", "event_id": existing["event_id"], "event_hash": existing["event_hash"], "actuals": existing["measurements"], "comparison": existing["comparison"]}
    if not current_events:
        raise RuntimeError("release plan is missing")
    plan = next((row for row in current_events if row["phase"] == "plan"), None)
    benchmark = next((row for row in current_events if row["phase"] == "benchmark"), None)
    if plan is None or benchmark is None:
        raise RuntimeError("plan and benchmark events are required")
    actuals, discovered_problems, refs = release_measurements(args.tag)
    expected_version = args.tag.removeprefix("v")
    actuals["production_version"] = production_version()
    actuals["total_elapsed_seconds"] = round((parse_time(utcnow()) - parse_time(plan["observed_at"])).total_seconds(), 3)
    actuals["benchmark"] = benchmark.get("measurements", {})
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
    final_lesson_ref = capture_release_lesson(
        "finalized",
        f"{args.tag} completed with {len(all_problems)} recorded problems",
        "Reuse evaluated recurrence guards and timing deltas in the next release plan",
        refs,
    )
    prediction_evaluations = {
        stage: evaluate_stage_prediction(
            args.tag,
            stage,
            f"{args.tag} {stage} succeeded with canonical evidence",
            1.0,
            f"metacog:{final_lesson_ref}" if final_lesson_ref else None,
        )
        for stage in ("release", "deploy", "final")
    }
    plan_learning = plan.get("measurements", {}).get("learning", {})
    guard_artifact = Path(f"/tmp/focusa-{args.tag.removeprefix('v')}-learning-guards.json")
    guard_result = json.loads(guard_artifact.read_text()) if guard_artifact.exists() else {"guards": []}
    actuals["learning"] = {
        "retrieved_lesson_count": plan_learning.get("retrieved_lessons", {}).get("candidate_count", 0),
        "recurrence_guards_total": guard_result.get("guards_total", 0),
        "recurrence_guards_passed": len([
            row for row in guard_result.get("guards", []) if row.get("status") == "passed"
        ]),
        "final_metacog_capture_id": final_lesson_ref,
        "prediction_evaluations": prediction_evaluations,
    }
    evidence_refs = refs + ["production:http://127.0.0.1:8787/v1/health"]
    if final_lesson_ref:
        evidence_refs.append(f"metacog:{final_lesson_ref}")
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
    lesson_ref = capture_release_lesson(args.stage, args.corrected_fact, args.recovery, evidence_refs)
    if lesson_ref:
        evidence_refs.append(lesson_ref)
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
        if name == "benchmark":
            item.add_argument("--artifact")
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
