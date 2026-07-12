#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVENT="$ROOT/crates/focusa-terminal-ui/src/install/event.rs"
CAP="$ROOT/crates/focusa-terminal-ui/src/capabilities.rs"
PRES="$ROOT/crates/focusa-terminal-ui/src/install/presenter.rs"
for token in PhaseStarted PhaseSucceeded PhaseWarning PhaseFailed AssetProgress InstallFinished RollbackStarted RollbackSucceeded; do
  grep -Fq "$token" "$EVENT" || { echo "FAIL: missing event $token" >&2; exit 1; }
done
grep -Fq 'validate_environment' "$CAP"
grep -Fq 'presenter_for_mode' "$PRES"
grep -Fq 'PlainPresenter' "$PRES"
grep -Fq 'SilentPresenter' "$PRES"
echo "Spec 132 installer animation event/presenter contract: PASS"
