#!/usr/bin/env bash
# spec152e_bundle_installer_test.sh
#
# Spec 152E.05.09 bundle installer/facade flow guard (Spec 152E §4 presenters,
# §8 product/grant registry, §9 facade registry, §21 surface consolidation,
# §22.3 cutover item 2 "repair /bundle convenience URL"; Spec 172 §7.3 shared
# operator nodes, §9 bundle composition).
#
# Positive: the /bundle convenience URL resolves to the verified deployed
# bundle installer asset (installers/install-bundle.sh, deployed_only_pinned
# sha256 16cb3944...), the install facade registers the bundle public code as
# a server-owned allowlist entry with zero customer/commerce/entitlement
# authority, the deployed-surface inventory classifies installer.bundle as a
# presenter split flow converging on universal activation, and the Unix
# bootstrapper delegates the bundle through the SAME fail-closed shared-client
# handoff as every other product (one verified account/order/key flow).
# Negative: no facade/bootstrapper owns a price, product, grant, feature,
# limit, license, or unmasked email; no bundle branch issues local entitlement.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS="$ROOT_DIR/docs/contracts"
SH="$ROOT_DIR/scripts/install-focusa.sh"
BUNDLE_RUST="$ROOT_DIR/crates/focusa-license/src/bundle_activation.rs"
CLIENT_RUST="$ROOT_DIR/crates/focusa-license/src/activation_client.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

json_get() { # json_get <file> <python expression> <description>
  python3 -c "
import json, sys
data = json.load(open(sys.argv[1], encoding='utf-8'))
try:
    value = eval(sys.argv[2], {'data': data})
    print(value)
except (KeyError, IndexError, TypeError) as exc:
    sys.stderr.write(f'lookup failed: {exc}\n')
    sys.exit(3)
" "$1" "$2" || fail "$3"
}

# ---- Route manifest: /bundle is repaired to the verified bundle installer
# (Spec 152E §22.3 item 2) and pinned deployed-only. ----
[ -f "$CONTRACTS/spec152e-installer-route-manifest.v1.json" ] \
  || fail "installer route manifest missing"

BUNDLE_TARGET="$(json_get "$CONTRACTS/spec152e-installer-route-manifest.v1.json" \
  "next(r['target'] for r in data['convenience_urls'] if r['route'] == '/bundle')" \
  "/bundle route target")"
[ "$BUNDLE_TARGET" = "/installers/install-bundle.sh" ] \
  || fail "/bundle must route to /installers/install-bundle.sh, got $BUNDLE_TARGET"

BUNDLE_KIND="$(json_get "$CONTRACTS/spec152e-installer-route-manifest.v1.json" \
  "next(r['trust']['kind'] for r in data['convenience_urls'] if r['route'] == '/bundle')" \
  "/bundle trust kind")"
[ "$BUNDLE_KIND" = "deployed_only_pinned" ] \
  || fail "/bundle trust must be deployed_only_pinned, got $BUNDLE_KIND"

BUNDLE_SHA="$(json_get "$CONTRACTS/spec152e-installer-route-manifest.v1.json" \
  "next(r['trust']['sha256'] for r in data['convenience_urls'] if r['route'] == '/bundle')" \
  "/bundle sha256")"
[ "$BUNDLE_SHA" = "16cb3944c969d5c3bd7c9cb73b3a30161ada1c2a1ab7282811f038c114904912" ] \
  || fail "/bundle sha256 not pinned"
pass "/bundle convenience URL repaired to the verified pinned bundle installer"

NO_404="$(json_get "$CONTRACTS/spec152e-installer-route-manifest.v1.json" \
  "any('no advertised URL returns 404' in i for i in data['invariants'])" \
  "no-404 invariant")"
[ "$NO_404" = "True" ] || fail "no-404 invariant missing from route manifest"
pass "route manifest proves the /bundle 404 defect is repaired"

# ---- Deployed-surface inventory: installer.bundle is a presenter split flow
# converging on universal activation, deployed-only, never re-issued. ----
[ -f "$CONTRACTS/spec152e-deployed-surface-inventory.v1.json" ] \
  || fail "deployed-surface inventory missing"

BUNDLE_CLASS="$(json_get "$CONTRACTS/spec152e-deployed-surface-inventory.v1.json" \
  "next(f['classification'] for f in data['files'] if f['id'] == 'installer.bundle')" \
  "installer.bundle classification")"
[ "$BUNDLE_CLASS" = "bundle_presenter_split_flow" ] \
  || fail "installer.bundle classification, got $BUNDLE_CLASS"
BUNDLE_MIGRATION="$(json_get "$CONTRACTS/spec152e-deployed-surface-inventory.v1.json" \
  "next(f['migration'] for f in data['files'] if f['id'] == 'installer.bundle')" \
  "installer.bundle migration")"
[ "$BUNDLE_MIGRATION" = "converge_to_universal_activation" ] \
  || fail "installer.bundle migration, got $BUNDLE_MIGRATION"
BUNDLE_PARITY="$(json_get "$CONTRACTS/spec152e-deployed-surface-inventory.v1.json" \
  "next(f['parity'] for f in data['files'] if f['id'] == 'installer.bundle')" \
  "installer.bundle parity")"
[ "$BUNDLE_PARITY" = "deployed_only" ] || fail "installer.bundle parity, got $BUNDLE_PARITY"
pass "deployed inventory: bundle installer is a presenter split flow converging on universal activation"

# ---- Facade registry: the install facade binds the bundle public code as a
# server-owned allowlist entry; facades have ZERO commerce/entitlement
# authority (Spec 152E §9, Spec 172 decision 15). ----
[ -f "$CONTRACTS/spec152e-facade-registry.v1.json" ] \
  || fail "facade registry missing"

BUNDLE_BOUND="$(json_get "$CONTRACTS/spec152e-facade-registry.v1.json" \
  "any('focusa_uiai_operator_bundle_lifetime_v1' in f['products'] for f in data['facades'] if f['facade_id'] == 'focusa_install_v1')" \
  "install facade bundle binding")"
[ "$BUNDLE_BOUND" = "True" ] || fail "install facade does not bind the bundle public code"
FACADE_TRUTH="$(json_get "$CONTRACTS/spec152e-facade-registry.v1.json" \
  "data['authority']['customer_or_commerce_truth']" \
  "facade commerce authority")"
[ "$FACADE_TRUTH" = "forbidden" ] || fail "facade must never own customer/commerce truth"
FACADE_ISSUE="$(json_get "$CONTRACTS/spec152e-facade-registry.v1.json" \
  "data['authority']['entitlement_issuance']" \
  "facade entitlement authority")"
[ "$FACADE_ISSUE" = "forbidden" ] || fail "facade must never issue entitlement"
NO_FACADE_EMAIL="$(json_get "$CONTRACTS/spec152e-facade-registry.v1.json" \
  "any('credentials_secrets_or_email_addresses' in i for i in data['invariants'])" \
  "facade hygiene invariant")"
[ "$NO_FACADE_EMAIL" = "True" ] || fail "facade registry hygiene invariant missing"
pass "facade registry: bundle code server-owned; facade has no commerce/entitlement authority"

# ---- Shared client surface: the bundle flows through the SAME registration
# with only the server-owned public product code; the bundle public code is a
# first-class pinned constant. ----
grep -qF 'pub fn begin(' "$CLIENT_RUST" && grep -qF 'public_product_code' "$CLIENT_RUST" \
  || fail "shared client does not accept only the public product code"
grep -qF 'PUBLIC_CODE_FOCUSA_UIAI_BUNDLE_LIFETIME_V1' "$BUNDLE_RUST" \
  || fail "bundle public code not pinned in the bundle contract"
grep -qF 'BUNDLE_GRANTS: [&str; 2]' "$BUNDLE_RUST" \
  || fail "bundle grants are not the exact two underlying grants"
grep -qF 'no_duplicate_payment' "$BUNDLE_RUST" \
  && grep -qF 'no_duplicate_license' "$BUNDLE_RUST" \
  || fail "typed recoverable partial state does not forbid duplicate payment/license"
pass "bundle flows through the shared registration; no second checkout/license route"

# ---- Unix bootstrapper (scripts/install-focusa.sh): bundle converges on the
# SAME fail-closed shared-client handoff; no local entitlement, raw key/email,
# or caller-controlled product/price/grant input anywhere. ----
[ -f "$SH" ] || fail "install-focusa.sh missing"
bash -n "$SH" || fail "install-focusa.sh: bash -n syntax check failed"
pass "install-focusa.sh: bash syntax OK"

forbid_patterns=(
  'write_license_json' 'bootstrapper still writes local license JSON'
  'evaluation_receipt|self_eval|E_EVAL_ISSUED' 'bootstrapper still issues local Evaluation state'
  'LICENSE_KEY=' 'bootstrapper still stores raw license keys'
  'CUSTOMER_EMAIL=' 'bootstrapper still stores customer email'
  'wpuiai-ai-cloud/v1/license/validate' 'bootstrapper still validates keys against the legacy registry'
  '--product=' 'client-controlled product input remains'
  '--price=' 'client-controlled price input remains'
  '--grant' 'client-controlled grant input remains'
  '--feature=' 'client-controlled feature input remains'
  '--limits=' 'client-controlled limit input remains'
  '--commercial' 'client-controlled commercial input remains'
)
i=0
while [ "$i" -lt "${#forbid_patterns[@]}" ]; do
  pattern="${forbid_patterns[$i]}"; message="${forbid_patterns[$((i + 1))]}"
  if grep -qE -- "$pattern" "$SH"; then fail "$message"; fi
  i=$((i + 2))
done
pass "bootstrapper has no local entitlement, raw key/email, or caller-controlled grants"

require_patterns=(
  'if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then' 'single fail-closed shared-client handoff missing'
  'github.com/%s/releases/download/%s/%s' 'official GitHub release download path missing'
  '--release-base-url' 'raw mirror download path missing'
  'raw credentials and legacy registry overrides are forbidden' 'raw credential rejection missing'
  'uninstall_args+=(--keep-data)' 'uninstall is not preserve-by-default'
)
i=0
while [ "$i" -lt "${#require_patterns[@]}" ]; do
  pattern="${require_patterns[$i]}"; message="${require_patterns[$((i + 1))]}"
  if ! grep -qF -- "$pattern" "$SH"; then fail "$message"; fi
  i=$((i + 2))
done
HANDOFFS="$(grep -cF 'if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then' "$SH")"
[ "$HANDOFFS" -eq 1 ] || fail "expected exactly one shared-client handoff, found $HANDOFFS"
pass "all products (incl. bundle) converge on one shared-client handoff"

# Privacy hygiene on the shell surface and the frozen manifests.
if grep -qE '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' "$SH"; then
  fail 'unmasked email pattern present in install-focusa.sh'
fi
if grep -qE 'BEGIN [A-Z ]*PRIVATE KEY|sk-[A-Za-z0-9]{8,}|ghp_[A-Za-z0-9]{8,}' "$SH"; then
  fail 'secret material present in install-focusa.sh'
fi
for manifest in \
  "$CONTRACTS/spec152e-installer-route-manifest.v1.json" \
  "$CONTRACTS/spec152e-facade-registry.v1.json" \
  "$CONTRACTS/spec152e-deployed-surface-inventory.v1.json"; do
  if grep -qE 'BEGIN [A-Z ]*PRIVATE KEY|sk-[A-Za-z0-9]{8,}|ghp_[A-Za-z0-9]{8,}' "$manifest"; then
    fail "secret material present in $(basename "$manifest")"
  fi
done
pass "no unmasked email or secret material on the bundle installer/facade surfaces"

# ---- Behavioral checks (executable, not source markers only) ----
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT

set +e
HOME="$FIXTURE/home" bash "$SH" --not-a-real-option >"$FIXTURE/out" 2>&1
RC=$?
set -e
[ "$RC" -eq 64 ] || fail "install-focusa.sh: unknown option exited $RC, expected 64"
[ ! -e "$FIXTURE/home/.focusa" ] || fail "invalid option mutated install state"
pass "unknown option fails closed (exit 64) without mutation"

set +e
PLAN="$(HOME="$FIXTURE/home" bash "$SH" --dry-run --eval --target=linux)"
RC=$?
set -e
[ "$RC" -eq 0 ] || fail "--dry-run --eval exited $RC, expected 0"
grep -Fq 'mutations: none' <<<"$PLAN" || fail "--dry-run is not explicitly non-mutating"
[ ! -e "$FIXTURE/home/.focusa" ] || fail "--dry-run --eval mutated install state"
pass "--dry-run mutates nothing; Evaluation intent stays authority-issued only"

echo "✓ Spec 152E bundle installer/facade flow checks passed"
