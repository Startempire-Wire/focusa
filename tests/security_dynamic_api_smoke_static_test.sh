#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT_DIR/tests/security_dynamic_api_smoke_test.sh"
DOC="$ROOT_DIR/docs/current/DYNAMIC_API_SECURITY_SMOKE.md"
GATE="$ROOT_DIR/scripts/ci/run-spec-gates.sh"
[[ -x "$SCRIPT" ]] || { echo "dynamic API smoke script missing or not executable" >&2; exit 1; }
[[ -f "$DOC" ]] || { echo "dynamic API smoke doc missing" >&2; exit 1; }
[[ -f "$GATE" ]] || { echo "spec gate script missing" >&2; exit 1; }

for marker in \
  "FOCUSA_API_MAX_BODY_BYTES=4096" \
  "127.0.0.1" \
  "/v1/health" \
  "/v1/telemetry/trace" \
  "malformed JSON" \
  "HTTP 413" \
  "DAEMON_BIN" \
  "HEALTH_FILE" \
  "schema_reject_count" \
  "/v1/workpoint/checkpoint" \
  "/v1/trajectory/define-goal" \
  "/v1/metacognition/capture" \
  "FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW" \
  "FOCUSA_API_JSON_MAX_DEPTH" \
  "shape_reject_count" \
  "excessive_depth" \
  "excessive_array" \
  "route_fuzz_count" \
  "expect_route_fuzz_reject" \
  "route_fuzzes" \
  "burst_429_count" \
  "HTTP 429"; do
  if ! grep -Fq "$marker" "$SCRIPT" "$DOC"; then
    echo "dynamic API smoke marker missing: $marker" >&2
    exit 1
  fi
done

for marker in \
  "security_dynamic_api_smoke_static_test.sh" \
  "security_dynamic_api_smoke_test.sh" \
  "DAEMON_BIN=\"\$DAEMON_BIN\""; do
  if ! grep -Fq "$marker" "$GATE"; then
    echo "dynamic API smoke CI gate marker missing: $marker" >&2
    exit 1
  fi
done

echo "✓ dynamic API security smoke static markers and CI gate wiring present"
