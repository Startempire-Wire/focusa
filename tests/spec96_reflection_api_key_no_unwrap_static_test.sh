#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
REFLECTION="${ROOT_DIR}/crates/focusa-api/src/routes/reflection.rs"

if rg -n 'std::env::var\("MINIMAX_API_KEY"\)\.unwrap\(\)' "$REFLECTION" >/dev/null; then
  echo "✗ FAIL: reflection LLM path still unwraps MINIMAX_API_KEY" >&2
  exit 1
fi

if rg -n 'API key unavailable after precheck|return \(vec!\[\], vec!\[\], vec!\[\], None, false\)' "$REFLECTION" >/dev/null; then
  echo "✓ PASS: reflection LLM handles missing API key after precheck without panic"
else
  echo "✗ FAIL: reflection LLM missing deterministic no-key bypass" >&2
  exit 1
fi

echo "SPEC96 Reflection API key no-unwrap static test: PASS"
