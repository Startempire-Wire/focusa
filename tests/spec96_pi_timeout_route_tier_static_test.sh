#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS_TS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n 'function focusaRouteTier|function timeoutFailureClassForRoute' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: Pi tools include route-tier timeout classifier"
else
  echo "✗ FAIL: route-tier timeout classifier missing" >&2
  exit 1
fi

if rg -n 'cold_path_timeout|hot_path_timeout' "$TOOLS_TS" >/dev/null \
  && rg -n 'route\.includes\("/deep"\)|\[\?&\]deep=true|closure-bundle|include_full_payload=true|mode=full' "$TOOLS_TS" >/dev/null \
  && rg -n 'context-cognition/proof|context-cognition/optimizer/artifacts|call-stack/verify' "$TOOLS_TS" >/dev/null \
  && rg -n 'return Math\.max\(configured, 2500\)' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: hot/warm/cold routes use explicit classes and low-frequency routes receive a warm/cold timeout floor"
else
  echo "✗ FAIL: timeout classes, cold route patterns, or warm timeout floor missing" >&2
  exit 1
fi

if rg -n 'details\.response.*failure_class|failure_class === "cold_path_timeout"|body\?\.failure_class' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: failure_class propagates into tool envelopes and explanations"
else
  echo "✗ FAIL: failure_class propagation missing" >&2
  exit 1
fi

if rg -n 'status: stack\.ok \? "completed" : "blocked"|Focus State hygiene is healthy' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: hygiene doctor emits semantic status instead of numeric HTTP text"
else
  echo "✗ FAIL: hygiene doctor semantic status contract missing" >&2
  exit 1
fi

echo "SPEC96 Pi timeout route-tier static test: PASS"
