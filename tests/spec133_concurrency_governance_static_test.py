#!/usr/bin/env python3
from pathlib import Path
P=Path(__file__).resolve().parents[1]/'crates/focusa-core/src/silent_sessions/concurrency_governance.rs'; S=P.read_text()
for x in ['WorkLoop','Foreground','SilentSession','path_intents','WriterLease','heartbeat_at','expires_at','exactly one scoped writer','IsolatedWorktree','ExclusiveExisting','ReadOnlyShared','ApprovedShared','shared write mode requires explicit approval','collision_safe_name','dependencies_ready','lease_admitted','resource_admitted','select_work_loop_owned','Merge','Rebase','CherryPick','tests_ref','checkpoint_ref','diff_ref','commit_ref','preview_ref','authority_ref','integration conflict blocks mutation','unrelated dirty changes must be preserved','destructive cleanup is outside integration authority']:
    assert x in S,x
assert len(S.splitlines())<=500
print('Spec133 concurrency/worktree/integration static contract: PASS')
