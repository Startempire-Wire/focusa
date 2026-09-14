#!/usr/bin/env bash
# spec152e_installer_shell_static_test.sh
#
# Spec 152E (§4 presenters, §12 Evaluation, §19 security/privacy, §21 surface
# consolidation, §22.3 cutover) verified-delegation guard for the Unix shell
# installer (scripts/install-focusa.sh).
#
# Positive: the bootstrapper is a pure presenter — artifact verification,
# rollback, preserve-by-default uninstall, and safe delegation to the shared
# activation client (Rust installer) survive; official and raw download paths
# converge on exactly one fail-closed handoff carrying allowlisted args only.
# Negative: no shell branch can issue Evaluation or entitlement; no local
# validation, JSON issuance, self-Eval, raw email/key logging, or unmasked
# secret material.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SH="$ROOT_DIR/scripts/install-focusa.sh"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }
require() { grep -qF -- "$1" "$SH" || fail "$2"; }
forbid() { ! grep -qE -- "$1" "$SH" || fail "$2"; }

[ -f "$SH" ] || fail "install-focusa.sh missing"
bash -n "$SH" || fail "install-focusa.sh: bash -n syntax check failed"
pass "install-focusa.sh: bash syntax OK"

# ---- No shell branch can issue Evaluation or entitlement (Spec 152E §12,
# §22.3 item 8): local validation / JSON issuance / self-Eval / key+email
# storage are forbidden everywhere in the bootstrapper. ----
forbid 'write_license_json' 'bootstrapper still writes local license JSON'
forbid 'write_license_authority' 'bootstrapper still creates local license authority'
forbid 'write_license_receipt' 'bootstrapper still writes local license receipts'
forbid 'evaluation_receipt|eval_issued|self_eval|E_EVAL_ISSUED' \
  'bootstrapper still issues local Evaluation state'
forbid 'grace_license|installer_grace' 'bootstrapper still creates installer grace licenses'
forbid 'key_hash\(' 'bootstrapper still transforms raw license keys'
forbid 'LICENSE_KEY=' 'bootstrapper still stores raw license keys'
forbid 'CUSTOMER_EMAIL=' 'bootstrapper still stores customer email'
forbid 'wpuiai-ai-cloud/v1/license/validate' \
  'bootstrapper still validates keys against the legacy registry'
forbid 'edd_sl_key|EDD_SL_KEY' 'bootstrapper still binds EDD keys locally'
pass "no local validation, JSON issuance, self-Eval, or raw key/email storage"

# ---- Shared activation client delegation (Spec 152E §4/§21): the
# bootstrapper forwards intent only; identity/product/payment/Evaluation/
# license/node/lease decisions stay in the shared client. ----
require 'raw credentials and legacy registry overrides are forbidden' \
  'raw credential rejection missing'
require 'signed authority device authorization' \
  'help does not direct users to signed authority device authorization'
require 'authority-issued only' \
  'help does not disclose that Evaluation is authority-issued only'
require 'ARGS=(install --target="$RUST_TARGET"' \
  'Rust install target handoff missing'
require 'if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then' \
  'single fail-closed shared-client handoff missing'
require 'restore_bootstrap_stash' 'rollback recovery hint missing'
require 'trap cleanup EXIT INT TERM' 'bounded temporary cleanup trap missing'
require 'uninstall_args+=(--keep-data)' 'uninstall is not preserve-by-default'
pass "activation intent delegated to shared client; rollback/uninstall preserved"

# Exactly one handoff: every download path (official GitHub release and raw
# --release-base-url mirror) converges on the same verified shared client.
HANDOFFS="$(grep -cF 'if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then' "$SH")"
[ "$HANDOFFS" -eq 1 ] || fail "expected exactly one shared-client handoff, found $HANDOFFS"
require 'github.com/%s/releases/download/%s/%s' \
  'official GitHub release download path missing'
require 'release_asset_url' 'shared asset-URL resolver missing'
require '--release-base-url' 'raw mirror download path missing'
pass "official and raw download paths converge on one shared-client handoff"

# ---- Argument / help / telemetry: allowlisted presenter args only; no
# client-controlled product/price/grant/feature/credential input (Spec 152E
# §8, §19.7). ----
forbid '--product=' 'client-controlled product input remains'
forbid '--price=' 'client-controlled price input remains'
forbid '--grant' 'client-controlled grant input remains'
forbid '--feature=' 'client-controlled feature input remains'
forbid '--limits=' 'client-controlled limit input remains'
forbid '--commercial' 'client-controlled commercial input remains'
pass "handoff args are allowlisted; no caller-controlled grants/price/features"

# ---- Privacy hygiene (Spec 152E §19.2, §19.3): no unmasked email, full
# license key, or private-key material in the bootstrapper surface. ----
forbid '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' \
  'unmasked email pattern present'
forbid 'BEGIN [A-Z ]*PRIVATE KEY' 'private-key material present'
forbid 'sk-[A-Za-z0-9]{8,}' 'secret prefix present'
forbid 'ghp_[A-Za-z0-9]{8,}' 'GitHub token prefix present'
pass "no unmasked email, license keys, or secret material in the surface"

# ---- Behavioral checks (executable, not source markers only) ----
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT

# Unknown options fail closed before any state (Spec 112 §15A).
set +e
HOME="$FIXTURE/home" bash "$SH" --not-a-real-option >"$FIXTURE/out" 2>&1
RC=$?
set -e
[ "$RC" -eq 64 ] || fail "install-focusa.sh: unknown option exited $RC, expected 64"
[ ! -e "$FIXTURE/home/.focusa" ] || fail "invalid option mutated install state"
pass "unknown option fails closed (exit 64) without mutation"

# Raw credentials are rejected before any state and never echoed.
SECRET='never-print-this-license-key-152e'
set +e
REJECTION="$(HOME="$FIXTURE/home" bash "$SH" "--license-key=$SECRET" 2>&1)"
RC=$?
set -e
[ "$RC" -ne 0 ] || fail "raw license key was accepted"
grep -Fq 'E_AUTHORITY_RAW_KEY_FORBIDDEN' <<<"$REJECTION" \
  || fail "raw credential rejection code missing"
! grep -Fq "$SECRET" <<<"$REJECTION" || fail "raw license key leaked into output"
[ ! -e "$FIXTURE/home/.focusa" ] || fail "raw credential path mutated install state"
pass "raw license key rejected without echo or mutation"

set +e
REJECTION_EMAIL="$(HOME="$FIXTURE/home" bash "$SH" "--email=admin@example.com" 2>&1)"
RC=$?
set -e
[ "$RC" -ne 0 ] || fail "raw email was accepted"
! grep -Fq 'admin@example.com' <<<"$REJECTION_EMAIL" || fail "raw email leaked into output"
pass "raw email rejected without echo"

# --eval is intent forwarding: dry-run discloses authority-issued Evaluation
# and creates no local state (Spec 152E §12: no local --eval records).
set +e
PLAN="$(HOME="$FIXTURE/home" bash "$SH" --dry-run --eval --target=linux)"
RC=$?
set -e
[ "$RC" -eq 0 ] || fail "--dry-run --eval exited $RC, expected 0"
grep -Fq 'entitlement: signed authority lease' <<<"$PLAN" \
  || fail "dry-run does not disclose signed entitlement requirement"
grep -Fq 'evaluation: authority-issued only' <<<"$PLAN" \
  || fail "dry-run does not disclose authority-only Evaluation"
grep -Fq 'mutations: none' <<<"$PLAN" \
  || fail "dry-run is not explicitly non-mutating"
[ ! -e "$FIXTURE/home/.focusa" ] || fail "--dry-run --eval mutated install state"
pass "--eval intent disclosed as authority-issued; dry-run mutates nothing"

echo "✓ Spec 152E installer shell verified-delegation checks passed"
