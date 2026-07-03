#!/usr/bin/env bash
# spec_audit_cmd_static_test.sh
# Static guard for focusa-audit-cmd.
# Backward compatibility: new top-level command only; reuses existing /v1/events/recent.
# Scope enforcement: no alternate audit store or project-bound mutation path.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUDIT="$ROOT_DIR/crates/focusa-cli/src/commands/audit.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$AUDIT" ] || fail "missing audit.rs"
grep -q 'pub mod audit;' "$MOD" || fail "commands/mod.rs missing pub mod audit"
grep -q 'Audit(commands::audit::AuditArgs)' "$MAIN" || fail "main.rs missing Audit command variant"
grep -q 'commands::audit::run(args, cli.json).await' "$MAIN" || fail "main.rs missing Audit dispatch"
pass "audit command wired into CLI"

for field in 'pub since: Option<String>' 'pub beads_issue: Option<String>' 'pub workpoint_id: Option<String>' 'pub limit: usize'; do
  grep -q "$field" "$AUDIT" || fail "AuditArgs missing field: $field"
done
pass "audit args include --since/--beads-issue/--workpoint-id/--limit"

grep -q '/v1/events/recent?limit=' "$AUDIT" || fail "audit must use existing /v1/events/recent route"
grep -q 'event_mentions' "$AUDIT" || fail "audit missing client-side filter helper"
grep -q 'source": "/v1/events/recent"' "$AUDIT" || fail "audit envelope must disclose source route"
pass "audit reuses existing durable events route"

if grep -qE 'post\(|delete\(|std::fs::write|File::create' "$AUDIT"; then
  fail "audit command must remain read-only"
fi
pass "audit command is read-only"

grep -q 'serde_json::to_string_pretty' "$AUDIT" || fail "audit missing --json output path"
grep -q 'print_human' "$AUDIT" || fail "audit missing human output path"
pass "audit supports JSON and human output"

echo "✓ All audit command static checks passed"