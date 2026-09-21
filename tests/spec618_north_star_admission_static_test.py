#!/usr/bin/env python3
"""Fail closed when critical executor mutations lose daemon North Star admission."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

EXPECTED = {
    "crates/focusa-api/src/routes/project.rs": [
        "require_north_star_mutation_admission",
        "require_scoped_north_star_mutation_admission",
        "NORTH_STAR_ADMISSION_BLOCKED",
        "focusa.north_star_projection.v1",
        "omitted_coverage",
        "same_versioned_records",
    ],
    "crates/focusa-api/src/routes/background_jobs.rs": ["background_job_create"],
    "crates/focusa-api/src/routes/attachments.rs": ["thread_attach"],
    "crates/focusa-api/src/routes/ecs.rs": ["ecs_artifact_store"],
    "crates/focusa-api/src/routes/ontology.rs": ["ontology_action_execute"],
    "crates/focusa-api/src/routes/proxy.rs": [
        "proxy_openai_dispatch",
        "proxy_messages_dispatch",
        "proxy_acp_dispatch",
    ],
    "crates/focusa-api/src/routes/semantic_integrity_executor.rs": [
        "semantic_integrity_event_append",
        "semantic_integrity_migration_run",
    ],
    "crates/focusa-api/src/routes/instances.rs": ["instance_connect"],
    "crates/focusa-api/src/routes/role_profiles.rs": ["role_profile_approve"],
    "crates/focusa-api/src/routes/spec_workbench.rs": ["spec_workbench_final_approve"],
    "crates/focusa-api/src/routes/commands.rs": [
        "command_session_start",
        "command_visual_evidence_store",
        "command_instance_connect",
    ],
    "crates/focusa-api/src/routes/visual_workflow.rs": ["visual_evidence_store"],
    "crates/focusa-api/src/routes/workpoint.rs": [
        "workpoint_checkpoint_promote",
        "workpoint_evidence_link",
        "workpoint_rollover_target_materialize",
    ],
    "crates/focusa-api/src/routes/work_loop.rs": [
        "work_loop_enable",
        "work_loop_resume",
        "work_loop_select_next",
        "work_loop_driver_start",
        "work_loop_driver_prompt",
        "work_loop_cycle_dispatch",
        "work_loop_blocked_auto_advance",
        "work_loop_delegate_authorship",
        "work_loop_attach_session",
        "work_loop_transport_ingest",
        "work_loop_pause_release",
    ],
    "crates/focusa-api/src/routes/task_plans.rs": [
        "task_plan_approve",
        "task_plan_materialize_beads",
    ],
    "crates/focusa-api/src/routes/session.rs": ["session_start", "session_resume"],
    "crates/focusa-api/src/routes/work_rail.rs": ["work_rail_mutation"],
    "crates/focusa-api/src/routes/worksets.rs": [
        "workset_definition_upsert",
        "workset_append_event",
    ],
    "crates/focusa-api/src/routes/work_items.rs": ["work_item_closure_submit"],
    "crates/focusa-api/src/routes/work_item_temporal.rs": [
        "temporal_work_item_create",
        "temporal_work_item_start",
        "temporal_work_item_resume",
        "temporal_work_item_complete",
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

def require_ordered(
    relative: str, start: str, end: str, markers: list[str], description: str
) -> None:
    text = (ROOT / relative).read_text(encoding="utf-8")
    if start not in text or end not in text.split(start, 1)[1]:
        missing.append(f"{relative}: cannot inspect {description}")
        return
    section = text.split(start, 1)[1].split(end, 1)[0]
    positions = [section.find(marker) for marker in markers]
    if min(positions) < 0 or positions != sorted(positions):
        missing.append(f"{relative}: {description}")


require_ordered(
    "crates/focusa-api/src/routes/work_item_temporal.rs",
    "async fn transition(",
    "async fn start(",
    [
        "return Ok(completed(schema",
        "if let Some(action) = admission_action",
        "let now = Utc::now()",
    ],
    "idempotent replay must precede admission and mutation",
)
require_ordered(
    "crates/focusa-api/src/routes/semantic_integrity_executor.rs",
    "pub async fn execute(",
    "type ExecutorValue",
    [
        "append_event_phase(&state.persistence, request, false)",
        "semantic_integrity_event_append",
        "append_event_phase(&state.persistence, request, true)",
        "run_migration(&state.persistence, request, false)",
        "semantic_integrity_migration_run",
        "run_migration(&state.persistence, request, true)",
    ],
    "semantic mutations must validate or replay before admission and persist only after admission",
)
require_ordered(
    "crates/focusa-api/src/routes/proxy.rs",
    "async fn chat_completions(",
    "async fn messages_proxy(",
    ["api_key(&headers)", "proxy_openai_dispatch", "ensure_session(&state)"],
    "OpenAI auth must precede admission and session/upstream effects",
)
require_ordered(
    "crates/focusa-api/src/routes/proxy.rs",
    "async fn messages_proxy(",
    "async fn acp_proxy(",
    ["messages_auth(&headers)", "proxy_messages_dispatch", "ensure_session(&state)"],
    "Messages auth must precede admission and session/upstream effects",
)
require_ordered(
    "crates/focusa-api/src/routes/proxy.rs",
    "async fn acp_proxy(",
    "pub fn router()",
    ["acp::parse_message", "proxy_acp_dispatch", "state.focusa.read()"],
    "ACP validation must precede admission and state/upstream effects",
)

if missing:
    raise SystemExit("North Star admission regression:\n" + "\n".join(missing))

print(f"Spec 618 North Star admission static gate: PASS ({len(EXPECTED)} surfaces)")
