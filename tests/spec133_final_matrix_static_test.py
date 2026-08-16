#!/usr/bin/env python3
from pathlib import Path
R=Path(__file__).resolve().parents[1]
F=(R/'tests/spec133_fault_fixture.py').read_text(); T=(R/'crates/focusa-core/src/silent_sessions/types.rs').read_text()+(R/'crates/focusa-core/src/silent_sessions/runner_protocol.rs').read_text()+(R/'crates/focusa-core/src/silent_sessions/capability_catalog.rs').read_text(); X=(R/'crates/focusa-core/src/silent_sessions/retention.rs').read_text()
for x in ['harness','subprocess','child-leak','prompt-wait','output-flood','model-mismatch','retry-failure','isolated-git','entitlement','runner-disconnect']: assert x in F,x
for x in ['DAEMON_RUNNER_PROTOCOL_VERSION','HARNESS_ADAPTER_PROTOCOL_VERSION','PROCESS_BACKEND_PROTOCOL_VERSION','ProtocolVersions','capabilities']: assert x in T,x
for x in ['SilentSessionPurgePlan','set_evidence_hold','purge_session','export_session_bundle','ordinary_delete_session']: assert x.lower() in X.lower(),x
print('Spec133 final fixture/protocol/retention static contract: PASS')
