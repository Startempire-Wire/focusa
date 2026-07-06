#!/usr/bin/env bash
# Spec 111 Slices 6+7 — focusa preload CLI subcommand + Pi/tool contracts static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

CLI="$ROOT_DIR/crates/focusa-cli/src/commands/preload.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
[[ -f "$CLI" ]] || fail "CLI preload.rs missing"

for needle in \
  'pub struct PreloadArgs' \
  'pub async fn run' \
  'profiles' \
  'build' \
  'render' \
  'verify' \
  'doctor' \
  'write' \
  'receipt-preview' \
  '/v1/preload/profiles' \
  '/v1/preload/build' \
  '/v1/preload/render' \
  '/v1/preload/verify' \
  '/v1/preload/doctor' \
  '/v1/preload/write' \
  '/v1/preload/receipt-preview' \
  'idempotency_key' \
  'target_path' \
  'overwrite'; do
  grep -qF -- "$needle" "$CLI" || fail "CLI preload missing: $needle"
done
pass "CLI preload covers all spec 111 subcommands"

grep -qF 'pub mod preload;' "$MOD" || fail "commands mod missing preload export"
grep -qF 'Preload(commands::preload::PreloadArgs)' "$MAIN" || fail "main.rs missing Preload command"
grep -qF 'Commands::Preload(args) => commands::preload::run(args, cli.json).await' "$MAIN" || fail "main.rs does not dispatch preload"
pass "preload CLI command wired into focusa binary"

echo "focusa-111 preload slice6_7 static test: PASS"
