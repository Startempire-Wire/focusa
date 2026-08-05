#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALLER="$ROOT_DIR/scripts/install-focusa.sh"

bash -n "$INSTALLER"

fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }
require() { grep -Fq -- "$1" "$INSTALLER" || fail "$2"; }
forbid() { ! grep -Fq -- "$1" "$INSTALLER" || fail "$2"; }

forbid 'write_license_json' 'bootstrapper still writes local license JSON'
forbid 'write_license_authority' 'bootstrapper still creates local license authority'
forbid 'write_license_receipt' 'bootstrapper still creates local license receipts'
forbid 'key_hash()' 'bootstrapper still transforms raw license keys'
forbid 'LICENSE_KEY=' 'bootstrapper still stores raw license keys'
forbid 'CUSTOMER_EMAIL=' 'bootstrapper still stores customer email'
forbid 'eval: true' 'bootstrapper still creates self-issued evaluation state'

require 'raw credentials and legacy registry overrides are forbidden' \
  'legacy credential arguments are not rejected'
require 'ARGS=(install --target="$RUST_TARGET"' \
  'bootstrapper does not delegate to canonical Rust installer'
require 'stable install requires valid Cosign signature metadata; SHA256 alone is insufficient' \
  'stable bootstrap does not fail closed on missing signatures'
require 'restore_bootstrap_stash' 'Rust failure recovery path is absent'
require 'uninstall_args+=(--keep-data)' 'uninstall is not preserve-by-default'

plan="$(bash "$INSTALLER" --dry-run --eval --target=linux)"
grep -Fq 'entitlement: signed authority lease' <<<"$plan" \
  || fail 'dry-run does not disclose signed entitlement requirement'
grep -Fq 'mutations: none' <<<"$plan" || fail 'dry-run is not explicitly non-mutating'

secret='never-print-this-license-key'
set +e
rejection="$(bash "$INSTALLER" "--license-key=$secret" 2>&1)"
status=$?
set -e
[ "$status" -ne 0 ] || fail 'raw license key was accepted'
! grep -Fq "$secret" <<<"$rejection" || fail 'raw license key leaked into output'

printf 'Spec152 installer shell authority contract: PASS\n'
