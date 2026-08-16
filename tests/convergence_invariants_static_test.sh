#!/usr/bin/env bash
# Convergence invariants static test (#101): the converged-surfaces rules
# must hold in source — developer-origin resolver, OTA transaction, parity
# release gate, canonical marker service, and completion notification.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$ROOT_DIR/crates/focusa-core/src/license_developer_origin.rs" ] || fail "developer-origin resolver missing (#307)"
[ -f "$ROOT_DIR/crates/focusa-core/src/project_marker.rs" ] || fail "canonical marker service missing (#243)"
[ -f "$ROOT_DIR/crates/focusa-core/src/remote_workspace.rs" ] || fail "RemoteWorkspaceBinding core missing (#89)"
[ -f "$ROOT_DIR/crates/focusa-core/src/workstream_root.rs" ] || fail "WorkstreamRoot core missing (#125)"
[ -f "$ROOT_DIR/crates/focusa-core/src/compaction_policy.rs" ] || fail "compaction policy controller missing (#112)"
[ -f "$ROOT_DIR/crates/focusa-core/src/silent_session_completion_events.rs" ] || fail "completion events ledger missing (#311)"
[ -f "$ROOT_DIR/crates/focusa-core/src/runtime/event_retention.rs" ] || fail "event retention engine missing"
[ -f "$ROOT_DIR/crates/focusa-cli/src/commands/pi_package.rs" ] || fail "Pi package transaction missing (#309)"
pass "converged core surfaces present"

rg -n -F "distribution parity drift blocks this release" "$ROOT_DIR/scripts/create-dev-release-tag.sh" >/dev/null || fail "release parity gate missing (#260)"
pass "release parity gate present"

rg -n -F "one canonical Focusa Pi package" "$ROOT_DIR/AGENTS.md" >/dev/null || fail "one-canonical rule missing from AGENTS.md"
pass "one-canonical-Pi-package rule present"

echo "convergence invariants static test: PASS"
