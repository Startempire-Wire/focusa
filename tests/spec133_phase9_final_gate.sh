#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"
export CARGO_INCREMENTAL=${CARGO_INCREMENTAL:-0}

# Fail closed: every prior runtime gate must pass in the same clean CI proof
# context before the exhaustive final matrix can claim Spec133 readiness.
bash tests/spec133_phase4_runtime_gate.sh
bash tests/spec133_phase5_isolation_gate.sh
bash tests/spec133_phase6_evidence_gate.sh
bash tests/spec133_phase7_operator_gate.sh
bash tests/spec133_phase8_backend_gate.sh

# Isolated-daemon Workloop proofs intentionally run outside the shared-daemon
# strict gate because they kill/restart their own supervised process trees.
DAEMON_BIN="${DAEMON_BIN:-$ROOT/target/debug/focusa-daemon}" \
  bash tests/work_loop_checkpoint_recovery_test.sh
DAEMON_BIN="${DAEMON_BIN:-$ROOT/target/debug/focusa-daemon}" \
  bash tests/work_loop_process_tree_supervision_test.sh
DAEMON_BIN="${DAEMON_BIN:-$ROOT/target/debug/focusa-daemon}" \
  bash tests/work_loop_writer_lease_fencing_test.sh

for test in tests/spec133_*static_test.py tests/spec133_*static_test.sh; do
  case "$test" in
    *.py) python3 "$test" ;;
    *.sh) bash "$test" ;;
  esac
done

python3 tests/spec133_execution_dependency_graph_test.py
python3 tests/spec133_work_item_grounding_test.py
cargo test --workspace --all-targets --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
pnpm --dir apps/pi-extension typecheck
pnpm --dir apps/pi-extension test
pnpm --dir apps/menubar check
pnpm --dir apps/menubar test

git diff --check
printf '%s\n' 'PASS: Spec133 Phase 9 exhaustive runtime, security, platform, operator, and acceptance gate'
