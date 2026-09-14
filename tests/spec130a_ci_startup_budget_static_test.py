#!/usr/bin/env python3
from pathlib import Path
S=(Path(__file__).resolve().parents[1]/'.github/workflows/ci.yml').read_text()
for x in ['STARTUP_BUDGET_SECONDS=300','kill -0 "$DAEMON_PID"','waiting for release daemon health','release daemon health exceeded','tail -200 /tmp/focusa-daemon-probe.log']:
 assert x in S,x
for x in ['FOCUSA_DATA_DIR: ${{ runner.temp }}/focusa-api-contract-probe-${{ github.run_id }}-${{ github.run_attempt }}','shutil.rmtree(path, ignore_errors=True)','wait "$DAEMON_PID"']:
 assert x in S,x
assert 'seq 1 120' not in S
print('Spec130A CI daemon startup budget static contract: PASS')
