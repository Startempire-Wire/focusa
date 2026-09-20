#!/usr/bin/env python3
"""Fail closed when critical executor mutations lose daemon North Star admission."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

EXPECTED = {
    "crates/focusa-api/src/routes/project.rs": [
        "require_north_star_mutation_admission",
        "require_scoped_north_star_mutation_admission",
        "NORTH_STAR_ADMISSION_BLOCKED",
    ],
    "crates/focusa-api/src/routes/background_jobs.rs": ["background_job_create"],
    "crates/focusa-api/src/routes/workpoint.rs": [
        "workpoint_evidence_link",
        "workpoint_rollover_target_materialize",
    ],
    "crates/focusa-api/src/routes/work_loop.rs": [
        "work_loop_enable",
        "work_loop_resume",
        "work_loop_select_next",
        "work_loop_driver_start",
    ],
    "crates/focusa-api/src/routes/task_plans.rs": [
        "task_plan_approve",
        "task_plan_materialize_beads",
    ],
    "crates/focusa-api/src/routes/session.rs": ["session_start", "session_resume"],
    "crates/focusa-api/src/routes/worksets.rs": [
        "workset_definition_upsert",
        "workset_append_event",
    ],
    "crates/focusa-api/src/routes/callgraph.rs": [
        "callgraph_definition_upsert",
        "callgraph_run_create",
        "callgraph_dispatch_entry_frontier",
        "callgraph_frame_settle",
        "callgraph_flowmesh_execute",
        "callgraph_evidence_link",
        "require_callgraph_run_admission",
    ],
    "crates/focusa-api/src/routes/silent_sessions_create.rs": ["silent_session_create"],
    "crates/focusa-api/src/routes/silent_sessions_lifecycle.rs": ["silent_session_start"],
    "crates/focusa-api/src/routes/silent_sessions_control.rs": ["silent_session_resume"],
    "crates/focusa-api/src/routes/silent_sessions_restart.rs": ["silent_session_restart"],
    "crates/focusa-api/src/routes/silent_sessions_adopt.rs": ["silent_session_adopt"],
    "crates/focusa-api/src/routes/silent_sessions_config_mutation.rs": [
        "silent_session_config_revise",
        "silent_session_config_rollback",
    ],
    "crates/focusa-api/src/routes/silent_sessions_approvals.rs": [
        "silent_session_approval_create"
    ],
    "crates/focusa-api/src/routes/silent_sessions_input.rs": [
        "silent_session_input",
        "silent_session_steer",
        "silent_session_follow_up",
        "silent_session_keys",
    ],
    "crates/focusa-core/src/callgraph_store.rs": [
        "pub project_root: String",
        "pub continuity_id: String",
        "ensure_schema_migrates_callgraph_run_scope_columns",
    ],
}

missing: list[str] = []
for relative, tokens in EXPECTED.items():
    text = (ROOT / relative).read_text(encoding="utf-8")
    for token in tokens:
        if token not in text:
            missing.append(f"{relative}: {token}")

if missing:
    raise SystemExit("North Star admission regression:\n" + "\n".join(missing))

print(f"Spec 618 North Star admission static gate: PASS ({len(EXPECTED)} surfaces)")
