#!/usr/bin/env python3
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
S=(ROOT/'crates/focusa-core/src/silent_sessions/pi_rpc_adapter.rs').read_text()
for marker in ['ProjectIdentity barrier denied','Trajectory barrier denied','Workpoint barrier denied','Context packet barrier denied','writer lease barrier denied','model preflight barrier denied','Context Authority is stale','project mutation blocked before AgentBootstrap','requested and effective models differ','observed model differs','PiRpcTransport','DeterministicPiRpcTransport','ResumeNativeSession','QueryUsage','native_session_ref','PiRpcEvent','ToolCall','ToolResult','Usage','Turn','Message']:
    assert marker in S, marker
assert S.index('self.barrier.authorize_project_mutation') < S.index('self.transport.call(request)?', S.index('pub fn mutate'))
assert len(S.splitlines()) <= 500
print('Spec133 governed Pi RPC bootstrap static contract: PASS')
