#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_FILE="$ROOT_DIR/crates/focusa-core/src/runtime/daemon.rs"
ADAPTER_FILE="$ROOT_DIR/crates/focusa-core/src/work_item/adapter.rs"
BD_ADAPTER_FILE="$ROOT_DIR/crates/focusa-core/src/work_item/adapters/bd.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -q 'async fn transition\(' "$ADAPTER_FILE" \
  || fail "provider-neutral lifecycle transition contract missing"
rg -q 'transition_work_item' "$ADAPTER_FILE" \
  || fail "unsupported provider transitions must fail closed"
rg -q 'can_transition' "$ROOT_DIR/crates/focusa-core/src/work_item/types.rs" \
  || fail "provider transition capability is not declared"
rg -q 'async fn transition\(' "$BD_ADAPTER_FILE" \
  || fail "Beads lifecycle transition projection missing"
rg -q '"update"' "$BD_ADAPTER_FILE" \
  || fail "Beads transition does not invoke provider update"
! rg -q '"--notes"' "$BD_ADAPTER_FILE" \
  || fail "Beads lifecycle projection must not overwrite provider notes"
pass "provider-neutral lifecycle transition projects through the Beads adapter"

rg -q 'async fn transition_current_work_item' "$DAEMON_FILE" \
  || fail "daemon scoped work-item transition helper missing"
rg -Fq 'capabilities().can_transition' "$DAEMON_FILE" \
  || fail "daemon does not enforce provider transition capability"
rg -Fq 'adapter.transition(&work_item, status, reason).await' "$DAEMON_FILE" \
  || fail "daemon does not invoke the provider-neutral transition contract"
! rg -q 'Command::new("bd")' "$DAEMON_FILE" \
  || fail "daemon must not bypass the provider adapter with direct Beads execution"
pass "daemon transitions work items only through the provider adapter"

rg -q 'outcome_status == WorkLoopOutcomeStatus::Blocked' "$DAEMON_FILE" \
  || fail "blocked outcomes are not projected to the provider"
rg -q 'WorkItemStatus::Blocked' "$DAEMON_FILE" \
  || fail "blocked outcome transition lacks typed blocked status"
rg -q 'complete_work_item_via_lifecycle' "$DAEMON_FILE" \
  || fail "completed outcomes do not use the guarded closure lifecycle"
! rg -q 'WorkItemStatus::Closed' "$DAEMON_FILE" \
  || fail "completion must not bypass closure authority through a generic transition"
rg -q 'transition_closure_requires_lifecycle' "$BD_ADAPTER_FILE" \
  || fail "Beads adapter does not reject generic closure transitions"
pass "blocked outcomes use typed transitions and completion remains lifecycle-authorized"

rg -q 'Action::DeferContinuousWorkItem' "$DAEMON_FILE" \
  || fail "alternate-ready defer action missing"
rg -q 'deferred by Work Loop' "$DAEMON_FILE" \
  || fail "alternate-ready defer lacks provider transition reason"
pass "alternate-ready defer projects through the scoped lifecycle transition"

rg -q 'build_bd_closure_certificate' "$DAEMON_FILE" \
  || fail "completion path lacks closure certificate gate"
rg -q 'run_secondary_adversarial_closure_audit' "$DAEMON_FILE" \
  || fail "completion path lacks secondary closure verifier gate"
pass "completion transition remains behind existing closure authority gates"
