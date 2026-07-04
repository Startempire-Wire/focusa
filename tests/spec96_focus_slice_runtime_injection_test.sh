#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"
TYPEBOX_DIR="apps/pi-extension/node_modules/@sinclair/typebox"
if [ ! -f "$TYPEBOX_DIR/package.json" ]; then
  mkdir -p "$TYPEBOX_DIR"
  cat >"$TYPEBOX_DIR/package.json" <<'JSON'
{"type":"module","main":"index.js","exports":"./index.js"}
JSON
  cat >"$TYPEBOX_DIR/index.js" <<'JS'
export const Type = new Proxy({}, { get: (_target, prop) => (...args) => ({ kind: String(prop), args }) });
JS
  trap 'rm -rf "apps/pi-extension/node_modules/@sinclair"' EXIT
fi
bun tests/spec96_focus_slice_runtime_injection_test.mts
