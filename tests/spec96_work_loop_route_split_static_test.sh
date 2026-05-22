#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_LOOP="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
DOC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"
CLI_MAIN="${ROOT_DIR}/crates/focusa-cli/src/main.rs"
CLI_DOCTOR="${ROOT_DIR}/crates/focusa-cli/src/commands/doctor.rs"
CLI_CONTINUE="${ROOT_DIR}/crates/focusa-cli/src/commands/continue_work.rs"

if rg -n 'route\("/v1/work-loop/health", get\(health\)\)|route\("/v1/work-loop/status/deep", get\(status_deep\)\)' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: explicit work-loop health/deep routes are registered"
else
  echo "✗ FAIL: explicit work-loop health/deep routes missing" >&2
  exit 1
fi

if rg -n 'route_tier": "hot"|summary_only": true|deep_status_route": "/v1/work-loop/status/deep"|cold_omitted' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop hot summary exposes route tier and omitted cold fields"
else
  echo "✗ FAIL: work-loop hot summary route metadata missing" >&2
  exit 1
fi

if rg -n 'route_tier": "cold"|summary_only": false|cold_omitted": \[\]' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop deep route exposes cold route metadata"
else
  echo "✗ FAIL: work-loop deep route metadata missing" >&2
  exit 1
fi

if rg -n 'GET /v1/work-loop/health|GET /v1/work-loop/status/deep' "$DOC" >/dev/null; then
  echo "✓ PASS: Spec96 route contract remains documented"
else
  echo "✗ FAIL: Spec96 work-loop route contract missing from docs" >&2
  exit 1
fi

if rg -n '/v1/work-loop/status\?summary_only=true' "$CLI_MAIN" "$CLI_DOCTOR" "$CLI_CONTINUE" >/dev/null; then
  echo "✓ PASS: CLI status/doctor/continue use hot summary work-loop route by default"
else
  echo "✗ FAIL: CLI still uses deep work-loop status by default" >&2
  exit 1
fi

echo "SPEC96 work-loop route split static test: PASS"
