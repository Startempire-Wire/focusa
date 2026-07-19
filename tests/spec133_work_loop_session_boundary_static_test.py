#!/usr/bin/env python3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
spec = (root / "docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md").read_text()
assert "Silent Sessions are an execution substrate beneath governed continuous work." in spec
assert "The two must not become parallel schedulers." in spec
for phrase in [
    "Work Loop owns:",
    "* ordered work selection;",
    "* continuation decisions;",
    "* alternate-ready-work selection;",
    "Silent Session owns:",
    "* one supervised agent execution;",
    "* runtime streams;",
    "* process control;",
]:
    assert phrase in spec, phrase

for relative in [
    "crates/focusa-core/src/silent_session.rs",
    "crates/focusa-core/src/silent_session_reducer.rs",
]:
    source = (root / relative).read_text()
    for forbidden in [
        "WorkLoopPolicy", "WorkLoopStatus", "select_next", "ready_work", "dependency_scheduler",
        "alternate_ready", "current_task",
    ]:
        assert forbidden not in source, f"{relative} crosses into Work Loop scheduling: {forbidden}"

api = (root / "crates/focusa-api/src/routes/work_loop.rs").read_text()
daemon = (root / "crates/focusa-core/src/runtime/daemon.rs").read_text()
assert "transport_partition_matches" in api
assert "transport_workpoint_id" in api
assert "claim_bd_item_if_possible" not in daemon
assert "execution_work_item_id.as_deref()" in daemon
assert "agent_session_id: self.state.work_loop.transport_session_id.clone()" in daemon

print("Spec133 §19.10 boundary: Work Loop schedules; Silent Sessions supervise one execution: PASS")
