#!/usr/bin/env python3
"""Spec98/99 Phase C: Work-loop execution state exposes work-item/writer partition contract."""

from pathlib import Path
import re
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.23-work-loop-execution-partition.yaml"
WORK_LOOP = ROOT / "crates/focusa-api/src/routes/work_loop.rs"
SERVER = ROOT / "crates/focusa-api/src/server.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def function_body(text: str, name: str) -> str:
    for marker in (f"async fn {name}(", f"fn {name}("):
        start = text.find(marker)
        if start >= 0:
            break
    else:
        fail(f"function missing: {name}")
    brace = text.find("{", start)
    depth = 0
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[brace + 1 : i]
    fail(f"function body unterminated: {name}")
    return ""


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.work_loop_execution_partition_contract.v1":
        fail("unexpected contract schema_version")
    if data.get("status") != "writer_and_work_item_partition_contract_defined":
        fail("contract status is not writer_and_work_item_partition_contract_defined")
    keys = set((data.get("partition_keys") or {}).keys())
    for key in ["ProjectRootKey", "WorkstreamKey", "WorkItemKey", "WriterKey"]:
        if key not in keys:
            fail(f"partition_keys missing {key}")

    text = WORK_LOOP.read_text()
    if 'const WRITER_HEADER: &str = "x-focusa-writer-id";' not in text:
        fail("writer header constant missing")
    if "fn work_loop_execution_partition_payload" not in text:
        fail("execution partition payload helper missing")
    helper = function_body(text, "work_loop_execution_partition_payload")
    for required in [
        "focusa.work_loop_execution_partition.v2",
        "work_item_key",
        "writer_key",
        "writer_claim_key",
        "legacy_active_writer_global",
        "partition_status",
    ]:
        if required not in helper:
            fail(f"execution partition payload missing {required}")
    if not re.search(
        r"wl\s*\.current_task\s*\.as_ref\(\)\s*\.map\(\|task\|\s*task\.work_item_id\.clone\(\)\)",
        helper,
    ):
        fail(
            "execution partition must derive WorkItemKey from current_task.work_item_id"
        )
    if (
        'legacy_active_writer_global": true' in helper
        or "pending_route_scope" in helper
    ):
        fail(
            "execution partition must not retain legacy global writer or pending route scope"
        )

    for route in ["health", "status", "status_deep"]:
        body = function_body(text, route)
        if "execution_partition" not in body:
            fail(f"{route} response must include execution_partition")

    for gated in ["enable", "resume", "select_next", "checkpoint", "heartbeat"]:
        body = function_body(text, gated)
        if "ensure_writer_claim" not in body:
            fail(f"{gated} must require writer claim")
    context = function_body(text, "set_decision_context")
    if "ensure_claimed_writer_matches_for_context" not in context:
        fail("context writes must require matching claimed writer")

    server = SERVER.read_text()
    if "pub writer_claims: Arc<TokioRwLock<HashMap<String, String>>>" not in server:
        fail("server must store scoped writer_claims map")
    if "pub active_writer: Arc<TokioRwLock<Option<String>>>" in server:
        fail("legacy daemon-global active_writer storage must be removed")
    proofs = set(data.get("proof_requirements") or [])
    for proof in [
        "static Work-loop contract validation",
        "static status renders execution_partition",
        "static writer claim functions remain mutation gates",
    ]:
        if proof not in proofs:
            fail(f"contract missing proof requirement: {proof}")
    print("✓ PASS: Work-loop execution partition contract/status guard is present")


if __name__ == "__main__":
    main()
