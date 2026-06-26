#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/114-public-benchmark-flywheel-spec.md"
OBS="$ROOT_DIR/docs/current/FOCUSA_PUBLIC_BENCHMARK_OBSERVATORY.md"
PROMO="$ROOT_DIR/docs/current/FOCUSA_EVAL_PROMOTION_POLICY.md"
TEL="$ROOT_DIR/docs/31-telemetry-api.md"
SCHEMA="$ROOT_DIR/docs/30-telemetry-schema.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$SPEC" ] || fail "Spec 114 missing"
[ -f "$OBS" ] || fail "public benchmark observatory doc missing"
[ -f "$PROMO" ] || fail "eval promotion policy doc missing"

for needle in \
  'Focusa-vs-No-Focusa' \
  'bench.focusa.dev' \
  'evals.focusa.dev' \
  'proof.focusa.dev' \
  '/v1/evals/*' \
  'Eval Ledger' \
  'public-safe snapshots' \
  'Perpetua hybrid pattern' \
  '127.0.0.1:8090' \
  'X-Agent-Key' \
  'focusadev' \
  'fix-user-perms focusadev'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 114 missing marker: $needle"
done
pass "Spec 114 preserves Focusa-vs-No-Focusa, public domains, eval ledger, and observed infra pattern"

for needle in \
  'Focusa-vs-No-Focusa' \
  'WordPress on LiteSpeed/cPanel' \
  'Perpetua hybrid pattern' \
  'local Go API' \
  '127.0.0.1:8090' \
  'X-Agent-Key' \
  'internal Eval Ledger endpoints remain private' \
  'fix-user-perms focusadev'; do
  rg -n -F "$needle" "$OBS" >/dev/null || fail "observatory doc missing marker: $needle"
done
pass "observatory doc captures cPanel/WP default plus Perpetua-style Go API exception"

for needle in \
  'promote | hold | rollback | needs_more_runs' \
  'Focusa-vs-No-Focusa' \
  'public snapshot gate' \
  'Eval Ledger hash chain'; do
  rg -n -F "$needle" "$PROMO" >/dev/null || fail "promotion policy missing marker: $needle"
done
pass "promotion policy captures evidence-backed release gate"

for needle in \
  'POST /v1/evals/runs' \
  'POST /v1/evals/runs/{run_id}/events' \
  'GET  /v1/evals/compare' \
  'Telemetry is queryable, never mutable'; do
  rg -n -F "$needle" "$TEL" >/dev/null || fail "telemetry API boundary missing marker: $needle"
done
pass "telemetry API keeps /v1/telemetry read-only with eval ledger exception"

for needle in \
  'focusa.eval_event.v1' \
  'model_provider' \
  'model_version' \
  'scenario_id' \
  'pricing_snapshot'; do
  rg -n -F "$needle" "$SCHEMA" >/dev/null || fail "telemetry schema missing eval/model marker: $needle"
done
pass "telemetry schema includes eval ledger event metadata"

echo "spec114 public benchmark flywheel static test: PASS"
