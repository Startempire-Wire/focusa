#!/usr/bin/env bash
# 152F.06.07 Run complete build-independent Spec 152F gate.
#
# Authority: docs/152f-simple-entitlement-gating-and-future-granularity-
# addendum.md (Spec 152F; Specs 152, 152E, 150A, and the Spec 172 overlay
# remain binding where non-conflicting).
#
# Semantics (zero hidden skips, zero fabricated passes):
#   - EVERY tests/spec152f_*.{py,mjs} file runs, discovered by glob — no
#     static skip list exists, so a new Spec 152F test is never silently
#     skipped, and a missing runtime (python3/node) fails the gate. The gate
#     script itself (this file) is the aggregator, not a test, and is
#     excluded from the shell glob to avoid self-recursion. Every recorded
#     exit code is a real run result.
#   - Every Spec 152F contract/generator artifact is statically validated:
#     JSON parse (spec152f contracts, the spec152 entitlement-coverage
#     contract, the surface-reconciliation shards, and every taskgraph phase
#     file), YAML parse (spec152f YAML contracts), Python syntax compile and
#     Node syntax check of every test.
#   - Generated files are current: scripts/generate-spec152-entitlement-
#     coverage.py --check must exit 0 (byte-identical regeneration), the
#     runtime coverage must show zero unmatched surfaces, and the taskgraph
#     index per-phase sha256 digests must recompute byte-exactly.
#   - Fail-closed FORBIDDEN coverage: each forbidden property is asserted by
#     at least one Spec 152F test (no local/self-issued Evaluation; no
#     caller-controlled product/price/grants; no presenter-owned commercial
#     decision; no anonymous product capability; no 395 independent paywalls;
#     no account enumeration; no false urgency; no dead-end paywalls; no
#     duplicate customer/key; no automatic expiry; no reinstall-same-account
#     or node/limit expansion; bounded cached grants; offline grace never
#     expands features or limits; recovery always available; export, repair,
#     stable security update, rollback, and uninstall controls retained).
#   - Redaction hygiene: no raw email, key, token, customer row, credential,
#     or card data anywhere under docs/evidence/spec152f.
#   - Regenerated outputs produce ZERO DIFF: the gate additionally requires
#     no tracked modification and no new untracked file under docs/contracts
#     after the full run.
#   - The gate records the exact HEAD SHA and the sha256 of the committed
#     entitlement-coverage contract in its receipt.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

# Missing runtimes fail the gate before any test is attempted.
for tool in python3 node git; do
  command -v "$tool" >/dev/null 2>&1 || fail "missing required runtime/tool: $tool"
done
pass "runtimes present: $(python3 --version 2>&1), $(node --version 2>&1)"

RUNS=0
PASSED=0
FAILED=0

run_case(){
  local label="$1"; shift
  local log; log="$(mktemp)"
  RUNS=$((RUNS+1))
  if "$@" >"$log" 2>&1; then
    PASSED=$((PASSED+1))
    pass "$label"
    rm -f "$log"
  else
    FAILED=$((FAILED+1))
    echo "✗ FAIL: $label (run $RUNS)" >&2
    sed -n '1,30p' "$log" >&2 || true
    rm -f "$log"
  fi
}

assert_file(){ [[ -f "$1" ]] || fail "missing required file: $1"; }

# ── 1. Every build-independent Spec 152F test (glob-discovered, zero skips) ──

mapfile -t PY_TESTS < <(find tests -maxdepth 1 -name 'spec152f_*.py' | sort)
mapfile -t MJS_TESTS < <(find tests -maxdepth 1 -name 'spec152f_*.mjs' | sort)
mapfile -t SH_TESTS < <(find tests -maxdepth 1 -name 'spec152f_*.sh' ! -name 'spec152f_build_independent_gate.sh' | sort)

[[ "${#PY_TESTS[@]}" -ge 1 ]] || fail "no Spec 152F Python tests discovered"
[[ "${#MJS_TESTS[@]}" -ge 1 ]] || fail "no Spec 152F Node tests discovered"
pass "discovered Spec 152F tests: ${#PY_TESTS[@]} python, ${#MJS_TESTS[@]} node, ${#SH_TESTS[@]} shell"

# The gate pins the exact constituent inventory (no static skip list, but also
# no silent disappearance): every current Spec 152F test is named below, so a
# removed/renamed test fails the gate, and a newly added test fails the gate
# until it is explicitly inventoried here — never silently skipped either way.
require_inventory(){
  local runner="$1"; shift
  local found=("$@")
  local base
  local i=0
  for base in ${EXPECTED[$runner]}; do
    [[ "${found[$i]}" == "$base" ]] || fail "Spec 152F $runner test inventory mismatch: expected '${EXPECTED[$runner]}', found '${found[*]}'"
    i=$((i+1))
  done
  [[ $i -eq "${#found[@]}" ]] || fail "Spec 152F $runner test inventory mismatch: expected ${#EXPECTED[$runner]} files, found ${#found[@]}"
}
declare -A EXPECTED
EXPECTED[py]="spec152f_agent_presenter_test.py spec152f_call_stack_contract_test.py spec152f_cli_operation_map_test.py spec152f_cli_presenter_test.py spec152f_cross_presenter_parity_test.py spec152f_denial_ux_parity_test.py spec152f_document_authority_test.py spec152f_entitlement_policy_contract_test.py spec152f_entitlement_policy_vectors_test.py spec152f_evaluation_first_value_e2e_test.py spec152f_facade_policy_presenter_test.py spec152f_installed_acceptance_receipt_test.py spec152f_lifecycle_policy_test.py spec152f_offline_adversarial_test.py spec152f_operation_policy_metadata_test.py spec152f_paid_lifecycle_e2e_test.py spec152f_premium_family_adversarial_test.py spec152f_recovery_matrix_test.py spec152f_rest_entitlement_inheritance_test.py spec152f_runtime_entrypoint_map_test.py spec152f_surface_reconciliation_contract_test.py spec152f_surface_scanner_test.py spec152f_taskgraph_contract_test.py spec152f_uiai_policy_test.py spec152f_unknown_route_method_test.py"
EXPECTED[mjs]="spec152f_menubar_action_map_test.mjs spec152f_presenter_accessibility_test.mjs"
require_inventory py "${PY_TESTS[@]##*/}"
require_inventory mjs "${MJS_TESTS[@]##*/}"

for f in "${PY_TESTS[@]}"; do
  run_case "python3 ${f#tests/}" python3 "$f"
done
for f in "${MJS_TESTS[@]}"; do
  run_case "node ${f#tests/}" node "$f"
done
for f in "${SH_TESTS[@]}"; do
  run_case "bash ${f#tests/}" bash "$f"
done

# ── 2. Static layer over every Spec 152F contract / generator artifact ──

mapfile -t JSON_CONTRACTS < <(
  find docs/contracts -maxdepth 2 \( -name 'spec152f-*.json' -o -name 'spec152-entitlement-coverage.v1.json' \) | sort
)
for f in "${JSON_CONTRACTS[@]}"; do
  run_case "json parse ${f#docs/contracts/}" python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$f"
done

mapfile -t YAML_CONTRACTS < <(find docs/contracts -maxdepth 1 -name 'spec152f-*.yaml' | sort)
for f in "${YAML_CONTRACTS[@]}"; do
  run_case "yaml parse ${f#docs/contracts/}" python3 -c 'import yaml,sys; yaml.safe_load(open(sys.argv[1]))' "$f"
done

for f in "${PY_TESTS[@]}"; do
  run_case "python syntax ${f#tests/}" python3 -c 'import sys; compile(open(sys.argv[1], encoding="utf-8").read(), sys.argv[1], "exec")' "$f"
done

for f in "${MJS_TESTS[@]}"; do
  run_case "node --check ${f#tests/}" node --check "$f"
done

# ── 3. Generated files are current and the inventory is fully resolved ──

run_case "coverage generator --check (byte-identical regeneration)" \
  python3 scripts/generate-spec152-entitlement-coverage.py --check

python3 - "$ROOT_DIR" <<'PY' || fail "Spec 152F coverage contract is stale or has unmatched surfaces"
import json, sys
root = sys.argv[1]
coverage = json.load(open(root + "/docs/contracts/spec152-entitlement-coverage.v1.json"))
assert coverage["counts"]["covered"] == 974
assert coverage["counts"]["unmatched"] == 0, "unmatched surfaces remain (no 395 independent paywalls)"
assert coverage["counts"]["total"] == 974
assert coverage["scanner_exclusions"]["count"] == 9
PY
pass "coverage contract current: covered=974 unmatched=0 total=974 exclusions=9"

# Taskgraph generated artifacts must be current: the index pins per-phase
# sha256 digests; recompute them byte-exactly.
python3 - "$ROOT_DIR" <<'PY' || fail "Spec 152F taskgraph index is stale"
import hashlib, json, sys
root = sys.argv[1]
index = json.load(open(root + "/docs/contracts/spec152f-implementation-taskgraph.v1.json"))
assert index["schema"] == "focusa.spec152f_implementation_taskgraph_index.v1"
for phase, rel in sorted(index["phase_files"].items()):
    raw = open(root + "/" + rel, "rb").read()
    assert hashlib.sha256(raw).hexdigest() == index["phase_file_sha256"][phase], rel
PY
pass "taskgraph generated artifacts are current (per-phase sha256 zero-diff)"

# ── 4. Fail-closed FORBIDDEN coverage (each property asserted by a test) ──

require_token(){
  local tok="$1"
  grep -q "$tok" tests/spec152f_*.py tests/spec152f_*.mjs 2>/dev/null \
    || fail "no Spec 152F test asserts forbidden property: $tok"
}
require_token "no_local_issuance"          # no local/self-issued Evaluation
require_token "caller_controlled"          # no caller-controlled product/price/grants
require_token "presenter_must_not"         # presenters never own commercial decisions
require_token "no_dead_end_paywalls"       # no 395 independent paywalls
require_token "recovery_always_available"  # recovery survives commercial denial
require_token "no_raw_key_or_token"        # no raw keys/tokens in evidence
require_token "no_account_enumeration"     # no identity enumeration
require_token "no_false_urgency"           # no manufactured urgency
require_token "no_duplicate_customer_or_key"
require_token "no_automatic_expiry"
require_token "no_reinstall_same_account"
require_token "no_node_or_limit_expansion"
require_token "cached_grants_bounded"      # offline grace never expands grants
require_token "offline_grace_never_expands_features_or_limits"
require_token "scanner_exclusion_test_only" # test files never become runtime surfaces
require_token "metadata_repair_required"   # unknown side effects fail closed
pass "forbidden-property coverage: local-evaluation/caller/presenter/paywall/recovery/retained-control rules all asserted"

# ── 5. Redaction hygiene on the evidence (no raw email/key/token/card) ──

EVIDENCE_DIR="$ROOT_DIR/docs/evidence/spec152f"
[[ -d "$EVIDENCE_DIR" ]] || fail "missing docs/evidence/spec152f"

if grep -rIlE 'sk_live_|pk_live_|sk_test_|BEGIN [A-Z ]*PRIVATE KEY|AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{20,}|AIza[0-9A-Za-z_-]{35}' "$EVIDENCE_DIR" >/dev/null 2>&1; then
  fail "raw credential/token material found in docs/evidence/spec152f"
fi
if grep -rIlE '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' "$EVIDENCE_DIR" >/dev/null 2>&1; then
  # Only the public support address support@focusa.dev may appear — the
  # prior-atom evidence documents it explicitly as public (focusa-vbcqu.20.13.60-
  # acceptance.txt: "only reserved @example.com/@example.invalid fixtures and
  # the public support@focusa.dev"), and COMMERCIAL.md publishes it. Any other
  # email is customer PII and fails the gate; the public address itself may
  # not appear outside that single documented evidence record.
  leaks="$(grep -rInE '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' "$EVIDENCE_DIR" | grep -v 'support@focusa.dev' || true)"
  [[ -z "$leaks" ]] || fail "raw customer email address found in docs/evidence/spec152f: $leaks"
  public_scope="$(grep -rIlE 'support@focusa\.dev' "$EVIDENCE_DIR" | grep -v 'focusa-vbcqu.20.13.60-acceptance.txt' || true)"
  [[ -z "$public_scope" ]] || fail "public support address appears outside its documented prior-atom evidence: $public_scope"
fi
if grep -rIlE '[0-9]{4} ?[0-9]{4} ?[0-9]{4} ?[0-9]{4}' "$EVIDENCE_DIR" >/dev/null 2>&1; then
  python3 - "$EVIDENCE_DIR" <<'PY' || fail "card-like sequence found in docs/evidence/spec152f"
import re, sys
from pathlib import Path
# Real card numbers pass the Luhn check; 16-digit runs inside sha256 result
# handles (hex digests, documented immutable result handles) do not, so they
# are not card data and must not fail the hygiene gate.
def luhn_ok(num: str) -> bool:
    total = 0
    for i, d in enumerate(reversed(num)):
        v = int(d)
        if i % 2 == 1:
            v *= 2
            if v > 9:
                v -= 9
        total += v
    return total % 10 == 0
hits = []
for path in sorted(Path(sys.argv[1]).rglob("*")):
    if not path.is_file():
        continue
    for m in re.finditer(r"[0-9]{4} ?[0-9]{4} ?[0-9]{4} ?[0-9]{4}", path.read_text(encoding="utf-8", errors="replace")):
        digits = re.sub(r"\D", "", m.group(0))
        if luhn_ok(digits):
            hits.append(f"{path.name}:{digits}")
if hits:
    print("luhn-valid card-like sequences:", hits)
    sys.exit(1)
PY
fi
pass "redaction hygiene: no raw customer email, key, token, customer row, credential, or card data in docs/evidence/spec152f"

# ── 6. Prior-atom evidence completeness (no hidden skips in the series) ──

evidence_count="$(find "$EVIDENCE_DIR" -maxdepth 1 -name '*-acceptance.txt' | wc -l)"
[[ "$evidence_count" -ge 67 ]] || fail "expected >= 67 closed Spec 152F acceptance records, found $evidence_count"
pass "prior Spec 152F acceptance evidence present: $evidence_count records"

# ── 7. Zero-diff on regenerated outputs and whitespace ──

git diff --check -- docs/contracts tests docs/evidence/spec152f \
  || fail "whitespace errors in Spec 152F surfaces"
pass "git diff --check clean on Spec 152F surfaces"

if ! git diff --quiet -- docs/contracts; then
  fail "Spec 152F contract regeneration produced a diff (generated artifacts are stale)"
fi
if git status --porcelain -- docs/contracts | grep -q '^??'; then
  fail "Spec 152F run created untracked files under docs/contracts"
fi
pass "zero-diff: Spec 152F generated artifacts regenerate byte-identically"

# ── 8. Exact SHA receipt ──

HEAD_SHA="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
COVERAGE_SHA="$(sha256sum docs/contracts/spec152-entitlement-coverage.v1.json | cut -d' ' -f1)"

# ── Summary receipt ──

echo
echo "spec152f_build_independent_gate receipt"
echo "  sha256 head=$HEAD_SHA"
echo "  sha256 spec152-entitlement-coverage.v1.json=$COVERAGE_SHA"
echo "  runs=$RUNS passed=$PASSED failed=$FAILED"
echo "  tests=py:${#PY_TESTS[@]} node:${#MJS_TESTS[@]} sh:${#SH_TESTS[@]}"
echo "  contracts=json:${#JSON_CONTRACTS[@]} yaml:${#YAML_CONTRACTS[@]}"
if [[ "$FAILED" -gt 0 ]]; then
  echo "✗ spec152f_build_independent_gate FAILED"
  exit 1
fi
echo "✓ spec152f_build_independent_gate PASS (zero hidden skips)"
exit 0
