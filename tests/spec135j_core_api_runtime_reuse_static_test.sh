#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md"
DELIVERY="$ROOT_DIR/docs/135-series-current-manifest.md"
DIRECTIVE="$ROOT_DIR/docs/agent/spec135-real-time-generated-ui-directive.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$SPEC" "$DELIVERY" "$DIRECTIVE"; do
  [[ -f "$file" ]] || fail "missing required Spec 135J/runtime file: $file"
done

for needle in \
  'focusa.operation_descriptor.v1' \
  'Generated Focusa Operation Registry' \
  'OpenAPI vendor extensions' \
  'focusa.ui_capability_snapshot.v1' \
  'Shared result and recovery envelope' \
  'One typed envelope implementation' \
  'SQLite canonical events' \
  'existing in-process broadcast channel' \
  'Last-Event-ID' \
  'focusa.stream_event.v1' \
  'AG-UI translation boundary' \
  'Read-model and UI-intent reuse' \
  'Surface cache and invalidation'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135J missing runtime reuse decision: $needle"
done
pass "Operation Registry, envelopes, capabilities, durable stream, and read-model reuse are explicit"

for needle in \
  'Do not hand-author A2UI action definitions' \
  'JSON Schema + x-focusa UI annotations' \
  'Deterministic UI without model calls' \
  'TanStack Query remains the web cache/refetch layer' \
  'Scaffold from fixtures' \
  'Schemathesis stateful preview/commit tests'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135J missing speed decision: $needle"
done
pass "core API acceleration and non-reinvention decisions are explicit"

for needle in \
  '135H' \
  '135I' \
  '135J' \
  '135K' \
  'No companion is optional' \
  'A nontechnical generated UI path is required for every C.R.I.S.T. stage' \
  'UIAI Engine Eval' \
  'OpenAPI 3.0.3'; do
  rg -n -F "$needle" "$DELIVERY" >/dev/null || fail "Delivery Contract missing: $needle"
done
pass "complete Spec 135 companion and delivery decisions are discoverable"

for needle in \
  'Operation Registry' \
  'Last-Event-ID' \
  'replay missed matching events from SQLite' \
  'ToolResult/error envelope' \
  'A missing generated-UI or UIAI Eval section blocks the ticket' \
  'AG-UI translates this stream for external compatibility only'; do
  rg -n -F "$needle" "$DIRECTIVE" >/dev/null || fail "generated UI directive missing runtime reuse instruction: $needle"
done
pass "agents receive durable stream and core API reuse instructions"

echo "Spec 135J core API/runtime reuse static test: PASS"
