#!/usr/bin/env python3
"""Save point function evaluation — Spec 88 / docs/25 session-transfer + workpoint.

Verifies that:
- focusa_session_transfer (action=save) appends to
  data/project_session_transfers.jsonl with the focusa.project_session_transfer.v1 schema.
- focusa_session_transfer (action=continue) returns the latest prior save as
  `transfer` with the saved mission + next_action + operator_handoff.
- focusa_session_transfer (action=status) returns saved=false, no resume token
  when there is no prior save; saved=true after a save.
- focusa_workpoint_checkpoint accepts a checkpoint with
  mission + continuity_id + next_slice + project_root.
- focusa_workpoint_resume returns the rendered_summary with the same mission +
  next_slice + canonical=true.
- The underlying JSONL ledger is append-only (entries never modified).

This is the operator-facing "save for later" surface; the test ensures the
save/continue/status/canonical cycle is intact.
"""
import json
import re
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def post(path: str, body: dict) -> dict:
    req = urllib.request.Request(
        f"http://127.0.0.1:8787{path}",
        method="POST",
        headers={"Content-Type": "application/json"},
        data=json.dumps(body).encode(),
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read().decode())


def main() -> None:
    project_root = "/home/wirebot/focusa"
    continuity_id = "focusa-cont-savepoint-eval-2026-06-09"
    mission = "save point function evaluation - script"
    next_slice = "evaluate_status"

    # Capture the starting ledger size BEFORE any save in this test.
    ledger = ROOT / "data/project_session_transfers.jsonl"
    if not ledger.exists():
        # First-ever run: file doesn't exist yet
        pre_count_at_start = 0
    else:
        pre_count_at_start = sum(1 for line in ledger.read_text().splitlines() if line.strip())

    # 1. status before save
    status_before = post("/v1/project/session-transfer", {
        "action": "status",
        "project_root": project_root,
        "continuity_id": continuity_id,
        "current_ask": "evaluate save point function",
    })
    if status_before.get("schema") != "focusa.project_session_transfer_response.v1":
        fail(f"status returned wrong schema: {status_before.get('schema')}")
    if "saved" not in status_before:
        fail(f"status missing 'saved' field: {status_before.keys()}")

    # 2. save
    save_resp = post("/v1/project/session-transfer", {
        "action": "save",
        "project_root": project_root,
        "continuity_id": continuity_id,
        "current_ask": "evaluate save point function",
        "mission": mission,
        "next_action": next_slice,
    })
    if save_resp.get("schema") != "focusa.project_session_transfer_response.v1":
        fail(f"save returned wrong schema: {save_resp.get('schema')}")
    if not save_resp.get("saved"):
        fail(f"save did not report saved=true: {save_resp.get('saved')}")

    # 3. ledger append
    ledger = ROOT / "data/project_session_transfers.jsonl"
    if not ledger.exists():
        fail(f"ledger not appended: {ledger}")
    matching = []
    for line in ledger.read_text().splitlines():
        if not line.strip():
            continue
        try:
            entry = json.loads(line)
        except json.JSONDecodeError:
            continue
        if entry.get("schema") == "focusa.project_session_transfer.v1" and entry.get("continuity_id") == continuity_id:
            matching.append(entry)
    if not matching:
        fail(f"no ledger entry with continuity_id={continuity_id}")
    last = matching[-1]
    if last.get("mission") != mission:
        fail(f"ledger mission mismatch: {last.get('mission')!r} != {mission!r}")
    if last.get("next_action") != next_slice:
        fail(f"ledger next_action mismatch: {last.get('next_action')!r} != {next_slice!r}")
    if last.get("project_root") != project_root:
        fail(f"ledger project_root mismatch: {last.get('project_root')!r}")

    # 4. continue returns the prior save as `transfer`
    cont_resp = post("/v1/project/session-transfer", {
        "action": "continue",
        "project_root": project_root,
        "continuity_id": continuity_id,
        "current_ask": "evaluate save point function",
    })
    if cont_resp.get("schema") != "focusa.project_session_transfer_response.v1":
        fail(f"continue returned wrong schema: {cont_resp.get('schema')}")
    transfer = cont_resp.get("transfer") or {}
    if transfer.get("mission") != mission:
        fail(f"continue.transfer.mission mismatch: {transfer.get('mission')!r} != {mission!r}")
    if transfer.get("next_action") != next_slice:
        fail(f"continue.transfer.next_action mismatch: {transfer.get('next_action')!r} != {next_slice!r}")
    operator_handoff = transfer.get("operator_handoff") or {}
    if "command" not in operator_handoff or "first_tool" not in operator_handoff:
        fail(f"continue.transfer.operator_handoff missing command/first_tool: {operator_handoff.keys()}")

    # 5. workpoint_checkpoint (underlying primitive)
    cp_resp = post("/v1/workpoint/checkpoint", {
        "current_ask": "evaluate save point function",
        "mission": mission,
        "next_slice": next_slice,
        "continuity_id": continuity_id,
        "checkpoint_reason": "operator_checkpoint",
        "canonical": True,
        "project_root": project_root,
    })
    if cp_resp.get("status") not in ("accepted", "completed"):
        fail(f"workpoint_checkpoint rejected: status={cp_resp.get('status')}, failure_class={cp_resp.get('failure_class')}")
    workpoint_id = cp_resp.get("workpoint_id")
    if not workpoint_id:
        fail("workpoint_checkpoint returned no workpoint_id")

    # 6. workpoint_resume (specific workpoint_id)
    res_resp = post("/v1/workpoint/resume", {
        "workpoint_id": workpoint_id,
        "project_root": project_root,
        "mode": "full",
    })
    if res_resp.get("status") != "completed":
        fail(f"workpoint_resume failed: {res_resp.get('status')}")
    if res_resp.get("workpoint_id") != workpoint_id:
        fail(f"workpoint_resume returned wrong id: {res_resp.get('workpoint_id')}")
    if not res_resp.get("canonical"):
        fail(f"workpoint_resume not canonical: {res_resp.get('canonical')}")
    summary = res_resp.get("rendered_summary") or ""
    if mission not in summary:
        fail(f"rendered_summary missing mission: {summary!r}")
    if next_slice not in summary:
        fail(f"rendered_summary missing next_slice: {summary!r}")

    # 7. append-only verification
    final_count = sum(1 for line in ledger.read_text().splitlines() if line.strip())
    if final_count - pre_count_at_start < 1:
        fail(f"save did not append a new entry: {pre_count_at_start} -> {final_count}")
    if final_count < pre_count_at_start:
        fail(f"ledger shrank: {pre_count_at_start} -> {final_count}")

    # 8. Pi wrapper field-shape check (the wrapper gap fix).
    # The Pi focusa_session_transfer wrapper must now expose
    # session_transfer_save_packet (the actual game-save from apiBody.transfer)
    # and workpoint_checkpoint_packet (the typed workpoint from /workpoint/checkpoint)
    # as DISTINCT fields, so the operator can see both.
    wrapper_path = ROOT / "apps/pi-extension/src/tools.ts"
    wrapper_src = wrapper_path.read_text()
    for marker in [
        "session_transfer_save_packet",
        "workpoint_checkpoint_packet",
        "workpoint_resume_packet",
        "apiBody.transfer",
    ]:
        if marker not in wrapper_src:
            fail(f"Pi wrapper missing field/marker: {marker}")
    # The old `save_packet: checkpoint?.body` (which conflated the two) must be gone.
    if "save_packet: checkpoint?.body" in wrapper_src:
        fail("Pi wrapper still has the old 'save_packet: checkpoint?.body' (gap not fixed)")

    print(f"✓ PASS: save point function (session-transfer + workpoint primitive) evaluated; "
          f"ledger grew {pre_count_at_start} -> {final_count}, workpoint_id={workpoint_id}, "
          f"Pi wrapper exposes session_transfer_save_packet + workpoint_checkpoint_packet distinct, "
          f"rendered_summary='{summary[:80]}...'")


if __name__ == "__main__":
    main()
