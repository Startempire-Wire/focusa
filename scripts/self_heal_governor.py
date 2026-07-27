#!/usr/bin/env python3
"""Durable, bounded self-healing policy for Master Release Cycle providers."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import re
import sys
import uuid
from contextlib import contextmanager
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Iterator

FAILURE_SCHEMA = "focusa.self_heal.failure.v1"
DECISION_SCHEMA = "focusa.self_heal.decision.v1"
CLAIM_SCHEMA = "focusa.self_heal.claim.v1"
SETTLEMENT_SCHEMA = "focusa.self_heal.settlement.v1"
TRANSIENT_CLASSES = {
    "cache_corruption",
    "github_transient",
    "network_transient",
    "runner_interruption",
    "runner_saturation",
}
DETERMINISTIC_CLASSES = {
    "authority_failure",
    "code_failure",
    "deterministic_test_failure",
    "security_failure",
    "spec_failure",
}
SETTLED_STATUSES = {"healed", "rolled_back", "exhausted", "operator_review"}


def parse_time(value: str) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def now_text(value: datetime) -> str:
    return value.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")


def bounded(value: Any, field: str, limit: int = 512) -> str:
    if not isinstance(value, str) or not value.strip() or len(value) > limit:
        raise ValueError(f"{field} must be a non-empty bounded string")
    return value.strip()


def normalize(value: str) -> str:
    return re.sub(r"[^a-z0-9._:/-]+", "-", value.strip().lower()).strip("-")


def validate_failure(failure: dict[str, Any]) -> dict[str, Any]:
    if failure.get("schema") != FAILURE_SCHEMA:
        raise ValueError("unsupported failure schema")
    result = dict(failure)
    for field in ("repository", "workflow", "failure_class", "exact_sha", "action_scope"):
        result[field] = bounded(result.get(field), field)
    if len(result["exact_sha"]) < 7:
        raise ValueError("exact_sha is too short")
    result["failure_class"] = normalize(result["failure_class"])
    result["deterministic"] = bool(result.get("deterministic"))
    retry_policy = result.get("retry_policy")
    if retry_policy is not None:
        retry_policy = bounded(retry_policy, "retry_policy", 80)
        if retry_policy not in {"rerun_once", "hard_failure_no_rerun", "operator_review"}:
            raise ValueError("unsupported retry_policy")
        result["retry_policy"] = retry_policy
    artifact = result.get("artifact_set_id")
    if artifact is not None:
        result["artifact_set_id"] = bounded(artifact, "artifact_set_id")
    return result


def failure_fingerprint(failure: dict[str, Any]) -> str:
    item = validate_failure(failure)
    canonical = "\x00".join(
        normalize(item[field])
        for field in ("repository", "workflow", "failure_class", "exact_sha", "action_scope")
    )
    return hashlib.sha256(canonical.encode()).hexdigest()


def load_rows(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    rows: list[dict[str, Any]] = []
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        row = json.loads(line)
        if not isinstance(row, dict):
            raise ValueError(f"ledger row {number} is not an object")
        rows.append(row)
    return rows


@contextmanager
def locked_ledger(path: Path) -> Iterator[list[dict[str, Any]]]:
    path.parent.mkdir(parents=True, exist_ok=True)
    lock_path = path.with_suffix(path.suffix + ".lock")
    with lock_path.open("a+", encoding="utf-8") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        yield load_rows(path)


def append_row(path: Path, row: dict[str, Any]) -> None:
    encoded = json.dumps(row, sort_keys=True, separators=(",", ":"))
    with path.open("a", encoding="utf-8") as handle:
        handle.write(encoded + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def select_action(failure: dict[str, Any]) -> tuple[str, str]:
    cls = failure["failure_class"]
    retry_policy = failure.get("retry_policy")
    if (
        failure["deterministic"]
        or cls in DETERMINISTIC_CLASSES
        or retry_policy in {"hard_failure_no_rerun", "operator_review"}
    ):
        return "operator_review", "deterministic_failure_requires_patch"
    if cls == "artifact_unavailable" and failure.get("artifact_set_id"):
        return "redispatch_deploy", "immutable_artifact_recovery"
    if cls in TRANSIENT_CLASSES or retry_policy == "rerun_once":
        return "rerun_failed_jobs", "bounded_transient_retry"
    return "propose_patch", "concrete_deliverable_required"


def decide(
    failure: dict[str, Any],
    rows: list[dict[str, Any]],
    observed_at: datetime,
    *,
    max_attempts: int = 1,
    mutation_budget: int = 1,
    window_seconds: int = 3600,
    cooldown_seconds: int = 3600,
) -> dict[str, Any]:
    failure = validate_failure(failure)
    fingerprint = failure_fingerprint(failure)
    action, reason = select_action(failure)
    claims = [row for row in rows if row.get("schema") == CLAIM_SCHEMA and row.get("fingerprint") == fingerprint]
    settlements = [
        row
        for row in rows
        if row.get("schema") == SETTLEMENT_SCHEMA and row.get("fingerprint") == fingerprint
    ]
    active_claim_ids = {row.get("claim_id") for row in claims} - {
        row.get("claim_id") for row in settlements
    }
    prior_attempts = len(claims)
    allowed = action != "operator_review"

    if active_claim_ids:
        allowed, reason = False, "active_fingerprint_claim"
    elif settlements and settlements[-1].get("status") in SETTLED_STATUSES:
        allowed, reason = False, f"fingerprint_{settlements[-1]['status']}"
    elif prior_attempts >= max_attempts:
        allowed, reason = False, "attempt_budget_exhausted"

    window_start = observed_at - timedelta(seconds=window_seconds)
    recent_claims = [
        row
        for row in rows
        if row.get("schema") == CLAIM_SCHEMA
        and parse_time(row["claimed_at"]) >= window_start
    ]
    if allowed and len(recent_claims) >= mutation_budget:
        allowed, reason = False, "repository_mutation_budget_exhausted"

    if allowed and claims:
        last_claim = max(parse_time(row["claimed_at"]) for row in claims)
        if observed_at < last_claim + timedelta(seconds=cooldown_seconds):
            allowed, reason = False, "fingerprint_cooldown_active"

    return {
        "schema": DECISION_SCHEMA,
        "fingerprint": fingerprint,
        "repository": failure["repository"],
        "workflow": failure["workflow"],
        "failure_class": failure["failure_class"],
        "exact_sha": failure["exact_sha"],
        "action_scope": failure["action_scope"],
        "action": action,
        "allowed": allowed,
        "reason": reason,
        "attempts_used": prior_attempts,
        "max_attempts": max_attempts,
        "mutation_budget": mutation_budget,
        "window_seconds": window_seconds,
        "cooldown_seconds": cooldown_seconds,
        "observed_at": now_text(observed_at),
        "branch_ref": f"self-heal/fp-{fingerprint[:20]}",
        "idempotency_key": f"self-heal:{fingerprint}:{prior_attempts + 1}",
    }


def claim(
    failure: dict[str, Any],
    ledger: Path,
    observed_at: datetime,
    approval_ref: str,
    **policy: int,
) -> tuple[dict[str, Any], dict[str, Any] | None]:
    approval_ref = bounded(approval_ref, "approval_ref")
    if not approval_ref.startswith(("operator:", "policy:")):
        raise ValueError("claim requires operator or policy approval")
    with locked_ledger(ledger) as rows:
        decision = decide(failure, rows, observed_at, **policy)
        if not decision["allowed"]:
            return decision, None
        row = {
            "schema": CLAIM_SCHEMA,
            "claim_id": str(uuid.uuid4()),
            "fingerprint": decision["fingerprint"],
            "idempotency_key": decision["idempotency_key"],
            "action": decision["action"],
            "exact_sha": decision["exact_sha"],
            "approval_ref": approval_ref,
            "claimed_at": now_text(observed_at),
        }
        append_row(ledger, row)
        return decision, row


def settle(
    ledger: Path,
    fingerprint: str,
    claim_id: str,
    status: str,
    evidence_ref: str,
    observed_at: datetime,
    rollback_ref: str | None,
) -> dict[str, Any]:
    if status not in SETTLED_STATUSES:
        raise ValueError("invalid settlement status")
    fingerprint = bounded(fingerprint, "fingerprint", 64)
    claim_id = bounded(claim_id, "claim_id", 128)
    evidence_ref = bounded(evidence_ref, "evidence_ref", 2048)
    with locked_ledger(ledger) as rows:
        matching = [
            row
            for row in rows
            if row.get("schema") == CLAIM_SCHEMA
            and row.get("fingerprint") == fingerprint
            and row.get("claim_id") == claim_id
        ]
        if not matching:
            raise ValueError("settlement claim does not exist")
        if any(
            row.get("schema") == SETTLEMENT_SCHEMA and row.get("claim_id") == claim_id
            for row in rows
        ):
            raise ValueError("claim is already settled")
        row = {
            "schema": SETTLEMENT_SCHEMA,
            "claim_id": claim_id,
            "fingerprint": fingerprint,
            "status": status,
            "evidence_refs": [evidence_ref],
            "rollback_ref": rollback_ref,
            "settled_at": now_text(observed_at),
        }
        append_row(ledger, row)
        return row


def parse_failure(path: str) -> dict[str, Any]:
    body = sys.stdin.read() if path == "-" else Path(path).read_text()
    value = json.loads(body)
    if not isinstance(value, dict):
        raise ValueError("failure input must be an object")
    return value


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    sub = root.add_subparsers(dest="command", required=True)
    for name in ("decide", "claim"):
        cmd = sub.add_parser(name)
        cmd.add_argument("--failure", required=True)
        cmd.add_argument("--ledger", required=True)
        cmd.add_argument("--now")
        cmd.add_argument("--max-attempts", type=int, default=1)
        cmd.add_argument("--mutation-budget", type=int, default=1)
        cmd.add_argument("--window-seconds", type=int, default=3600)
        cmd.add_argument("--cooldown-seconds", type=int, default=3600)
        if name == "claim":
            cmd.add_argument("--approval-ref", required=True)
    done = sub.add_parser("settle")
    done.add_argument("--ledger", required=True)
    done.add_argument("--fingerprint", required=True)
    done.add_argument("--claim-id", required=True)
    done.add_argument("--status", required=True)
    done.add_argument("--evidence-ref", required=True)
    done.add_argument("--rollback-ref")
    done.add_argument("--now")
    return root


def main() -> int:
    args = parser().parse_args()
    observed = parse_time(args.now) if args.now else datetime.now(timezone.utc)
    try:
        if args.command == "settle":
            result: Any = settle(
                Path(args.ledger),
                args.fingerprint,
                args.claim_id,
                args.status,
                args.evidence_ref,
                observed,
                args.rollback_ref,
            )
        else:
            failure = parse_failure(args.failure)
            policy = {
                "max_attempts": args.max_attempts,
                "mutation_budget": args.mutation_budget,
                "window_seconds": args.window_seconds,
                "cooldown_seconds": args.cooldown_seconds,
            }
            if args.command == "decide":
                result = decide(failure, load_rows(Path(args.ledger)), observed, **policy)
            else:
                decision, claim_row = claim(
                    failure, Path(args.ledger), observed, args.approval_ref, **policy
                )
                result = {"decision": decision, "claim": claim_row}
        print(json.dumps(result, sort_keys=True, separators=(",", ":")))
        return 0
    except (ValueError, OSError, json.JSONDecodeError) as exc:
        print(f"self-heal-governor: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
