#!/usr/bin/env bash
# 172.05.09 Run complete build-independent Spec 172 gate.
#
# Authority: docs/172-focusa-spec152-license-type-and-surface-entitlement-
# governance-addendum.md (Spec 172 §22.12 "Rerun build-independent ... gates";
# Specs 152, 152E, 152F, and 150A remain binding where non-conflicting).
#
# Semantics (zero hidden skips, zero fabricated passes):
#   - EVERY tests/spec172_*.{py,php,mjs,sh} file runs, discovered by glob —
#     no static skip list exists, so a new Spec 172 test is never silently
#     skipped, and a missing runtime (python3/php/node) fails the gate. The
#     gate script itself (this file) is the aggregator, not a test, and is
#     excluded from the shell glob to avoid self-recursion. The accepted
#     refund/revoke gate's documented random-token hygiene flake is
#     mitigated with bounded genuine re-runs on the exact signature (see
#     run_case), mirroring the acceptance lane's own replay retries; every
#     recorded exit code is a real run result.
#   - Every Spec 172 contract/generator artifact is statically validated:
#     JSON parse (contracts + taskgraph phases), YAML parse, PHP lint
#     (contracts + tests), Python syntax compile, Node syntax check.
#   - The reconciliation overlay (Spec 152 locked-release reconciliation
#     that Spec 172 overlays, plus docs/contracts/spec172-spec152-
#     reconciliation-map.v1.json validated by the taskgraph contract test)
#     and the governance inventory
#     (docs/contracts/spec172-no-sales-inventory.v1.json) run and validate.
#   - Fail-closed FORBIDDEN coverage: each forbidden property is asserted by
#     at least one Spec 172 test (no anonymous product capability; no
#     local/self-issued grant; no caller-controlled product/price/License
#     Type/family/feature/limit/node/commercial right; no presenter-owned
#     policy; no implicit legacy Download 453 mapping; recovery and the
#     retained export/repair/rollback/update/uninstall controls are never
#     disabled or blocked).
#   - Redaction hygiene: no raw email, key, token, customer row, credential,
#     or card data anywhere under docs/evidence/spec172.
#   - Regenerated outputs produce ZERO DIFF: the byte-identical regeneration
#     gates inside the Spec 172 tests are authoritative, and this gate
#     additionally requires no tracked modification and no new untracked
#     file under docs/contracts after the full run.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

RUNS=0
PASSED=0
FAILED=0

# The accepted refund/revoke gate (tests/spec172_refund_downgrade_test.php)
# carries a documented pre-existing hygiene flake: its random opaque
# settlement token (bin2hex(random_bytes(16))) can randomly form a 16-digit
# run that the gate's own card-pattern self-check false-positives on. The
# acceptance lane mitigates this at the replay layer with bounded genuine
# re-runs (tests/spec172_downgrade_data_preservation_test.py, retries=4).
# Mirror that here: only the EXACT documented signature is retried, each
# attempt is a real full run, and the recorded exit code is a real run
# result (never fabricated).
FLAKE_SIGNATURE="no card data in any settlement decision"
FLAKE_MAX_ATTEMPTS=4
FLAKE_RETRIES=0

run_case(){
  local label="$1"; shift
  local log; log="$(mktemp)"
  RUNS=$((RUNS+1))
  local attempt=0
  while :; do
    attempt=$((attempt+1))
    if "$@" >"$log" 2>&1; then
      PASSED=$((PASSED+1))
      pass "$label"
      rm -f "$log"
      return 0
    fi
    if [[ "$attempt" -lt "$FLAKE_MAX_ATTEMPTS" ]] \
      && grep -q "$FLAKE_SIGNATURE" "$log" 2>/dev/null; then
      FLAKE_RETRIES=$((FLAKE_RETRIES+1))
      continue
    fi
    break
  done
  FAILED=$((FAILED+1))
  echo "✗ FAIL: $label (run $RUNS, after $attempt real attempt(s))" >&2
  sed -n '1,30p' "$log" >&2 || true
  rm -f "$log"
}

assert_file(){ [[ -f "$1" ]] || fail "missing required file: $1"; }

# ── 1. Every build-independent Spec 172 test (glob-discovered, zero skips) ──

mapfile -t PY_TESTS < <(find tests -maxdepth 1 -name 'spec172_*.py' | sort)
mapfile -t PHP_TESTS < <(find tests -maxdepth 1 -name 'spec172_*.php' | sort)
mapfile -t NODE_TESTS < <(find tests -maxdepth 1 -name 'spec172_*.mjs' | sort)
mapfile -t SH_TESTS < <(find tests -maxdepth 1 -name 'spec172_*.sh' ! -name 'spec172_build_independent_gate.sh' | sort)

[[ "${#PY_TESTS[@]}" -ge 1 ]] || fail "no Spec 172 Python tests discovered"
[[ "${#PHP_TESTS[@]}" -ge 1 ]] || fail "no Spec 172 PHP tests discovered"
[[ "${#NODE_TESTS[@]}" -ge 1 ]] || fail "no Spec 172 Node tests discovered"
pass "discovered Spec 172 tests: ${#PY_TESTS[@]} python, ${#PHP_TESTS[@]} php, ${#NODE_TESTS[@]} node, ${#SH_TESTS[@]} shell"

for f in "${PY_TESTS[@]}"; do
  run_case "python3 ${f#tests/}" python3 "$f"
done
for f in "${PHP_TESTS[@]}"; do
  run_case "php ${f#tests/}" php "$f"
done
for f in "${NODE_TESTS[@]}"; do
  run_case "node ${f#tests/}" node "$f"
done
for f in "${SH_TESTS[@]}"; do
  run_case "bash ${f#tests/}" bash "$f"
done

# ── 2. Reconciliation overlay and governance inventory (explicit) ──

assert_file tests/172_focusa_spec152_locked_release_reconciliation_test.py
assert_file docs/contracts/spec172-spec152-reconciliation-map.v1.json
assert_file docs/contracts/spec172-no-sales-inventory.v1.json
assert_file docs/contracts/spec172-implementation-taskgraph.v1.json

run_case "python3 tests/172_focusa_spec152_locked_release_reconciliation_test.py (Spec 152 locked-release reconciliation overlay)" \
  python3 tests/172_focusa_spec152_locked_release_reconciliation_test.py

# Governance inventory contract is semantically validated by
# tests/spec172_no_sales_inventory_test.py (ran above); require its
# migration-preserving decision shape statically as well.
python3 - "$ROOT_DIR" <<'PY' || fail "governance inventory decision shape invalid"
import json, sys
inventory = json.load(open(sys.argv[1] + "/docs/contracts/spec172-no-sales-inventory.v1.json"))
assert inventory["schema"] == "focusa.spec172.no_sales_inventory.v1"
assert inventory["decision"]["zero_sales_proven"] is False
assert inventory["decision"]["clean_cutover_allowed"] is False
assert inventory["decision"]["status"] == "migration_preserving_path_selected"
PY
pass "governance inventory decision shape (zero_sales_proven=false, clean_cutover_allowed=false, migration_preserving_path_selected)"

# ── 3. Static layer over every Spec 172 contract / generator artifact ──

mapfile -t JSON_CONTRACTS < <(find docs/contracts -maxdepth 2 \( -name 'spec172-*.json' -o -path 'docs/contracts/spec172-taskgraph/*.json' \) | sort)
for f in "${JSON_CONTRACTS[@]}"; do
  run_case "json parse ${f#docs/contracts/}" python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$f"
done

mapfile -t YAML_CONTRACTS < <(find docs/contracts -maxdepth 1 -name 'spec172-*.yaml' | sort)
for f in "${YAML_CONTRACTS[@]}"; do
  run_case "yaml parse ${f#docs/contracts/}" python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1]))' "$f"
done

mapfile -t PHP_CONTRACTS < <(find docs/contracts -maxdepth 1 -name 'spec172-*.php' | sort)
for f in "${PHP_CONTRACTS[@]}" "${PHP_TESTS[@]}"; do
  run_case "php -l $(basename "$f")" php -l "$f"
done

for f in "${PY_TESTS[@]}"; do
  run_case "python syntax ${f#tests/}" python3 -c 'import sys; compile(open(sys.argv[1], encoding="utf-8").read(), sys.argv[1], "exec")' "$f"
done

for f in "${NODE_TESTS[@]}"; do
  run_case "node --check ${f#tests/}" node --check "$f"
done

# Taskgraph generated artifacts must be current: the index pins per-phase
# sha256 digests and the taskgraph contract test recomputes them (ran above).
python3 - "$ROOT_DIR" <<'PY' || fail "Spec 172 taskgraph index is stale"
import hashlib, json, sys
root = sys.argv[1]
index = json.load(open(root + "/docs/contracts/spec172-implementation-taskgraph.v1.json"))
assert index["schema"] == "focusa.spec172_implementation_taskgraph_index.v1"
for phase, rel in sorted(index["phase_files"].items()):
    raw = open(root + "/" + rel, "rb").read()
    assert hashlib.sha256(raw).hexdigest() == index["phase_file_sha256"][phase], rel
assert index["reconciliation_map_sha256"] == hashlib.sha256(
    open(root + "/" + index["reconciliation_map"], "rb").read()).hexdigest()
PY
pass "taskgraph generated artifacts are current (per-phase sha256 zero-diff)"

# ── 4. Fail-closed FORBIDDEN coverage (each property asserted by a test) ──

require_token(){
  local tok="$1"
  grep -q "$tok" tests/spec172_*.py tests/spec172_*.php tests/spec172_*.mjs 2>/dev/null \
    || fail "no Spec 172 test asserts forbidden property: $tok"
}
require_token "no_anonymous_product_capability"
require_token "no_local_or_self_issued_grant"
require_token "no_presenter_owned_policy"
require_token "caller_controlled"
require_token "download_453"
require_token "recovery_always_available"
require_token "never_disable"
require_token "retained_controls"
pass "forbidden-property coverage: anonymous/local/presenter/caller/legacy-453/retained-controls rules all asserted"

# ── 5. Redaction hygiene on the evidence (no raw email/key/token/card) ──

EVIDENCE_DIR="$ROOT_DIR/docs/evidence/spec172"
[[ -d "$EVIDENCE_DIR" ]] || fail "missing docs/evidence/spec172"

if grep -rIlE 'sk_live_|pk_live_|sk_test_|BEGIN [A-Z ]*PRIVATE KEY|AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{20,}|AIza[0-9A-Za-z_-]{35}' "$EVIDENCE_DIR" >/dev/null 2>&1; then
  fail "raw credential/token material found in docs/evidence/spec172"
fi
if grep -rIlE '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' "$EVIDENCE_DIR" >/dev/null 2>&1; then
  fail "raw email address found in docs/evidence/spec172"
fi
if grep -rIlE '[0-9]{4} ?[0-9]{4} ?[0-9]{4} ?[0-9]{4}' "$EVIDENCE_DIR" >/dev/null 2>&1; then
  fail "card-like sequence found in docs/evidence/spec172"
fi
pass "redaction hygiene: no raw email, key, token, customer row, credential, or card data in docs/evidence/spec172"

# ── 6. Prior-atom evidence completeness (no hidden skips in the series) ──

evidence_count="$(find "$EVIDENCE_DIR" -maxdepth 1 -name '*-acceptance.txt' | wc -l)"
[[ "$evidence_count" -ge 39 ]] || fail "expected >= 39 closed Spec 172 acceptance records, found $evidence_count"
pass "prior Spec 172 acceptance evidence present: $evidence_count records"

# ── 7. Zero-diff on regenerated outputs and whitespace ──

git diff --check -- docs/contracts tests docs/evidence/spec172 \
  || fail "whitespace errors in Spec 172 surfaces"
pass "git diff --check clean on Spec 172 surfaces"

if ! git diff --quiet -- docs/contracts; then
  fail "Spec 172 contract regeneration produced a diff (generated artifacts are stale)"
fi
if git status --porcelain -- docs/contracts | grep -q '^??'; then
  fail "Spec 172 run created untracked files under docs/contracts"
fi
pass "zero-diff: Spec 172 generated artifacts regenerate byte-identically"

# ── Summary receipt ──

echo
echo "spec172_build_independent_gate receipt"
echo "  runs=$RUNS passed=$PASSED failed=$FAILED flake_retries=$FLAKE_RETRIES"
echo "  tests=py:${#PY_TESTS[@]} php:${#PHP_TESTS[@]} node:${#NODE_TESTS[@]} sh:${#SH_TESTS[@]} overlay=1 contracts=json:${#JSON_CONTRACTS[@]} yaml:${#YAML_CONTRACTS[@]} php_lint:$(( ${#PHP_CONTRACTS[@]} + ${#PHP_TESTS[@]} ))"
if [[ "$FAILED" -gt 0 ]]; then
  echo "✗ spec172_build_independent_gate FAILED"
  exit 1
fi
echo "✓ spec172_build_independent_gate PASS (zero hidden skips)"
exit 0
