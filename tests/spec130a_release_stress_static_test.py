#!/usr/bin/env python3
from pathlib import Path
R=Path(__file__).resolve().parents[1]; P=(R/'apps/pi-extension/src/persistence.ts').read_text(); T=(R/'tests/spec130a_release_stress_runtime_test.mts').read_text()
for x in ['prepare','write','fsync','checksum','manifest','target-create','resume-verify','commit','injectPersistenceFault','persistence commit verification failed']: assert x in P,x
for x in ['1_000_000','10_000','unchangedAppends === 0','memory slope','PersistenceFaultBoundary','source mutated','idempotent recovery','"pi", "claude", "codex-opencode", "pi"','handoff lost']: assert x in T,x
print('Spec130A release stress fixture static contract: PASS')
