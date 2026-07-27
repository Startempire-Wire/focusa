#!/usr/bin/env python3
"""Bounded GitHub provider adapter for the Master Release Cycle.

The adapter reads one ReleasePluginEnvelope from stdin and writes one
ReleaseOperationReceipt to stdout. Plan mode is the default. Observe mode may
query exact-SHA workflow evidence. Provider mutation remains fail-closed unless
an operation explicitly opts into execute mode and carries approval evidence.
"""

from __future__ import annotations

import json
import os
import pwd
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

SCHEMA = "focusa.release_plugin_envelope.v1"
ARTIFACT_STAGES = {
    "built",
    "packaged",
    "provenanced",
    "draft_published",
    "canary_deployed",
    "verified",
    "promoted",
}
MUTATING_ACTION_PREFIXES = ("release.", "Deploy Live Daemon/", "Release/")


def fail(message: str) -> None:
    raise ValueError(message)


def bounded_text(value: Any, field: str, limit: int = 512) -> str:
    if not isinstance(value, str) or not value.strip() or len(value) > limit:
        fail(f"{field} must be a non-empty bounded string")
    return value.strip()


def find_gh() -> str:
    for path in ("/usr/bin/gh", "/usr/local/bin/gh"):
        if Path(path).is_file():
            return path
    found = shutil.which("gh")
    if found and Path(found).is_absolute():
        return found
    fail("absolute gh executable is unavailable")


def provider_env() -> dict[str, str]:
    """Create a minimal environment; credentials stay in the user's gh store."""
    home = pwd.getpwuid(os.getuid()).pw_dir
    return {
        "HOME": home,
        "PATH": "/usr/local/bin:/usr/bin:/bin",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
    }


def run_gh(args: list[str], timeout: int) -> dict[str, Any] | list[Any]:
    proc = subprocess.run(
        [find_gh(), *args],
        text=True,
        capture_output=True,
        env=provider_env(),
        timeout=max(1, min(timeout, 1800)),
        check=False,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout).strip()[:300]
        fail(f"GitHub provider command failed ({proc.returncode}): {detail}")
    body = proc.stdout.strip()
    return json.loads(body) if body else {}


def exact_sha_run(operation: dict[str, Any], request: dict[str, Any]) -> tuple[str, str]:
    inputs = operation.get("inputs") or {}
    repository = bounded_text(inputs.get("repository"), "operation.inputs.repository")
    workflow = bounded_text(inputs.get("workflow"), "operation.inputs.workflow")
    rows = run_gh(
        [
            "run",
            "list",
            "--repo",
            repository,
            "--workflow",
            workflow,
            "--commit",
            request["exact_sha"],
            "--status",
            "success",
            "--limit",
            "1",
            "--json",
            "databaseId,url,headSha,conclusion",
        ],
        int(operation["timeout_seconds"]),
    )
    if not isinstance(rows, list) or not rows:
        fail("no successful exact-SHA provider run is available")
    row = rows[0]
    if row.get("headSha") != request["exact_sha"] or row.get("conclusion") != "success":
        fail("provider evidence does not match exact SHA and success outcome")
    return str(row["databaseId"]), bounded_text(row["url"], "provider run URL", 2048)


def receipt(envelope: dict[str, Any]) -> dict[str, Any]:
    if envelope.get("schema") != SCHEMA:
        fail("unsupported release plugin envelope schema")
    operation = envelope.get("operation")
    request = envelope.get("request")
    if not isinstance(operation, dict) or not isinstance(request, dict):
        fail("operation and request objects are required")

    operation_id = bounded_text(operation.get("operation_id"), "operation_id")
    executor_id = bounded_text(operation.get("executor_id"), "executor_id")
    exact_sha = bounded_text(request.get("exact_sha"), "exact_sha", 128)
    idempotency_key = bounded_text(request.get("idempotency_key"), "idempotency_key", 256)
    stage = bounded_text(operation.get("stage"), "stage", 80)
    if stage != request.get("stage"):
        fail("operation and request stages differ")
    timeout = operation.get("timeout_seconds")
    if not isinstance(timeout, int) or timeout < 1 or timeout > 7200:
        fail("timeout_seconds is outside the adapter bound")

    inputs = operation.get("inputs") or {}
    mode = inputs.get("provider_mode", "plan")
    if mode not in {"plan", "observe", "execute"}:
        fail("provider_mode must be plan, observe, or execute")

    mutates = bool(operation.get("mutates"))
    action = bounded_text(operation.get("action"), "action")
    evidence = f"github-{mode}:{operation_id}:{exact_sha}:{idempotency_key}"
    outcome = "passed"
    reasons: list[str] = []

    if mode == "observe" and inputs.get("workflow"):
        run_id, url = exact_sha_run(operation, request)
        evidence = f"github-run:{run_id}:{url}"
    elif mode == "execute":
        approvals = request.get("approval_refs") or []
        if not mutates:
            fail("execute mode is reserved for mutation-declared operations")
        if not any(str(ref).startswith(("operator:", "approval:")) for ref in approvals):
            fail("execute mode requires explicit operator or durable approval")
        # Existing workflows remain compatibility providers until they expose a
        # synchronous typed completion endpoint. Never mistake dispatch for
        # completed release evidence.
        outcome = "blocked"
        reasons = ["provider_execution_binding_required"]
        evidence = f"github-execute-blocked:{action}:{exact_sha}:{idempotency_key}"

    artifact_id = f"artifact:sha256:{exact_sha}" if stage in ARTIFACT_STAGES else None
    rollback_ref = f"github-rollback:{exact_sha}" if stage in {"promoted", "rolled_back"} else None
    return {
        "operation_id": operation_id,
        "executor_id": executor_id,
        "exact_sha": exact_sha,
        "outcome": outcome,
        "observed_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "evidence_refs": [evidence],
        "artifact_set_id": artifact_id,
        "rollback_ref": rollback_ref,
        "elapsed_ms": 0,
        "queue_ms": 0,
        "retry_ms": 0,
        "reason_codes": reasons,
    }


def main() -> int:
    try:
        envelope = json.load(sys.stdin)
        json.dump(receipt(envelope), sys.stdout, separators=(",", ":"), sort_keys=True)
        sys.stdout.write("\n")
        return 0
    except (ValueError, json.JSONDecodeError, OSError, subprocess.SubprocessError) as exc:
        print(f"master-release-github-adapter: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
