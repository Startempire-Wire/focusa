#!/usr/bin/env bash
# Spec 124 / focusa-ux2qx.7 — bounded newcomer root help.
set -euo pipefail
cd "$(dirname "$0")/.."

fail() { echo "FAIL: $*" >&2; exit 1; }

cargo build -q -p focusa-cli --bin focusa
BIN="$PWD/target/debug/focusa"
ROOT_HELP="$($BIN --help 2>&1)"

printf '%s\n' "$ROOT_HELP" | grep -qF 'FOCUSA QUICK HELP' \
  || fail "root help missing quick-help heading"
for command in \
  'focusa about' \
  'focusa project' \
  'focusa first-mission' \
  'focusa status' \
  'focusa deck' \
  'focusa doctor' \
  'focusa help all' \
  'focusa help migration' \
  'focusa <command> --help'; do
  printf '%s\n' "$ROOT_HELP" | grep -qF "$command" \
    || fail "root help missing newcomer route: $command"
done

line_count="$(printf '%s\n' "$ROOT_HELP" | wc -l)"
[[ "$line_count" -le 45 ]] \
  || fail "root help regressed to $line_count lines (maximum 45)"
if printf '%s\n' "$ROOT_HELP" | grep -qF 'Cache management'; then
  fail "root help leaked advanced full-inventory command list"
fi

INSTALL_HELP="$($BIN install --help 2>&1)"
for flag in '--preflight' '--dry-run' '--target' '--json'; do
  printf '%s\n' "$INSTALL_HELP" | grep -q -- "$flag" \
    || fail "install subcommand help lost flag: $flag"
done

echo "PASS: root help is bounded for newcomers and subcommand help remains complete"
