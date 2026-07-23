#!/usr/bin/env python3
from pathlib import Path
R=Path(__file__).resolve().parents[1]
C=(R/'crates/focusa-core/src/silent_sessions/process_supervision.rs').read_text()
S=(R/'crates/focusa-session-runner/src/security.rs').read_text()
M=(R/'crates/focusa-session-runner/src/main.rs').read_text()
for marker in ['HarnessAbortRequested','ProcessGroupTermRequested','ProcessGroupKillRequested','ChildLeakCheckPassed','ChildLeakDetected','verify_complete']:
    assert marker in C+S, marker
assert S.index('RunnerSignal::Cancel') < S.index('RunnerSignal::ForceKill')
assert 'controlled_stop(' in M
assert 'receipt.verify_complete()' in M
print('Spec133 POSIX controlled-stop static contract: PASS')
