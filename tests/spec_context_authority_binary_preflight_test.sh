#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CARGO_BIN="${CARGO:-cargo}"
ASSET="${TMPDIR:-/tmp}/focusa-fake-release-asset-$$"
OUT="${TMPDIR:-/tmp}/focusa-binary-preflight-$$.json"
trap 'rm -f "$ASSET" "$OUT"' EXIT

printf 'fake release asset marker GLIBC_99.0\n' > "$ASSET"
chmod +x "$ASSET"

"$CARGO_BIN" run -q -p focusa-cli --locked -- --json binary preflight-install \
  --asset "$ASSET" \
  --target /usr/local/bin/focusa \
  --install-role live_build_host \
  --source github_release_asset > "$OUT"

jq -e '.schema == "focusa.binary_preflight.v1"' "$OUT" >/dev/null
jq -e '.verdict == "block"' "$OUT" >/dev/null
jq -e '.conflicts[] | select(.class == "release_asset_blocked_by_environment_contract")' "$OUT" >/dev/null
jq -e '.conflicts[] | select(.class == "glibc_incompatible_asset")' "$OUT" >/dev/null
jq -e '.safe_alternative | contains("local repo")' "$OUT" >/dev/null

echo "binary preflight test passed"
