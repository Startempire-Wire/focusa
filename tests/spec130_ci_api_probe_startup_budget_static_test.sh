#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CI="$ROOT/.github/workflows/ci.yml"

fail() { echo "FAIL: $*" >&2; exit 1; }
require() { rg -q "$2" "$1" || fail "missing $2 in $1"; }

require "$CI" 'name: API contract probe'
require "$CI" 'timeout-minutes: 20'
require "$CI" 'DAEMON_STARTUP_TIMEOUT_SECS: 300'
require "$CI" 'deadline=\$\(\(SECONDS \+ DAEMON_STARTUP_TIMEOUT_SECS\)\)'
require "$CI" 'curl --max-time 2 -fsS'
require "$CI" 'kill -0 "\$DAEMON_PID"'
require "$CI" 'focusa-daemon exited before API contract probe readiness'
require "$CI" 'focusa-daemon did not become ready within \$\{DAEMON_STARTUP_TIMEOUT_SECS\}s'
require "$CI" 'tail -100 /tmp/focusa-daemon-probe.log'

if rg -n 'for i in \$\(seq 1 120\)' "$CI" >/dev/null; then
  fail "API contract probe still uses the opaque 120-second loop"
fi

echo 'PASS: Spec130 CI API probe has bounded startup budget and diagnostics'
