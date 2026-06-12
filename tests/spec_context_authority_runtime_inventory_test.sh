#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CARGO_BIN="${CARGO:-cargo}"
OUT="${TMPDIR:-/tmp}/focusa-runtime-inventory-$$.json"
trap 'rm -f "$OUT"' EXIT

"$CARGO_BIN" run -q -p focusa-cli --locked -- --json runtime inventory --owner wirebot > "$OUT"

jq -e '.schema == "focusa.runtime_inventory.v1"' "$OUT" >/dev/null
jq -e '(.cli.version | length) > 0' "$OUT" >/dev/null
jq -e '(.daemon.running | type) == "boolean"' "$OUT" >/dev/null
jq -e '(.daemon.bind | length) > 0' "$OUT" >/dev/null
jq -e '.hygiene.status == "ok" or .hygiene.status == "degraded"' "$OUT" >/dev/null

echo "runtime inventory test passed"
