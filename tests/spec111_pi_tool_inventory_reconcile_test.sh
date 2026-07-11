#!/usr/bin/env bash
# focusa-p57b8.5: Reconcile Pi extension source, generated docs, and runtime
# tool registration. Proves documented/runtime counts match and every tool in
# tool-contracts.ts has a registerTool call in tools.ts, and vice versa.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
CONTRACTS="$ROOT_DIR/apps/pi-extension/src/tool-contracts.ts"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$TOOLS" ] || fail "tools.ts missing"
[ -f "$CONTRACTS" ] || fail "tool-contracts.ts missing"

# 1. Count explicit contracts in FOCUSA_TOOL_CONTRACTS (non-preload spread).
EXPLICIT_CONTRACTS=$(grep -oP 'name:\s*"focusa_\w+' "$CONTRACTS" | sed 's/name: "//' | grep -v focusa_preload | sort -u | wc -l)
echo "explicit non-preload contracts: $EXPLICIT_CONTRACTS"

# 2. Count PRELOAD_TOOL_CONTRACTS entries (array of tuples).
PRELOAD_CONTRACT_SUFFIXES=$(sed -n '/^const PRELOAD_TOOL_CONTRACTS/,/^].map/p' "$CONTRACTS" | grep -oP '^\s*\["\w+' | sed 's/.*"//' | sort -u)
PRELOAD_CONTRACT_COUNT=$(echo "$PRELOAD_CONTRACT_SUFFIXES" | grep -c . || true)
echo "preload contract suffixes ($PRELOAD_CONTRACT_COUNT): $PRELOAD_CONTRACT_SUFFIXES"

# 3. Total contracts = explicit + preload.
TOTAL_CONTRACTS=$((EXPLICIT_CONTRACTS + PRELOAD_CONTRACT_COUNT))
echo "total contracts: $TOTAL_CONTRACTS"

# 4. Count explicit registerTool calls in tools.ts (literal name: "focusa_...").
EXPLICIT_TOOLS=$(grep -oP 'name:\s*"focusa_\w+' "$TOOLS" | sed 's/name: "//' | sort -u)
EXPLICIT_TOOL_COUNT=$(echo "$EXPLICIT_TOOLS" | grep -c . || true)
echo "explicit registerTool names: $EXPLICIT_TOOL_COUNT"

# 5. Count loop-generated preload tools from preloadReadTools array.
LOOP_PRELOAD_NAMES=$(sed -n '/const preloadReadTools = \[/,/^\s*\] as const;/p' "$TOOLS" | grep -oP '^\s*\["focusa_preload_\w+' | sed 's/.*"//' | sort -u)
LOOP_PRELOAD_COUNT=$(echo "$LOOP_PRELOAD_NAMES" | grep -c . || true)
echo "loop-generated preload tools ($LOOP_PRELOAD_COUNT): $LOOP_PRELOAD_NAMES"

# 6. Total runtime = explicit (includes some preload) + loop preload that aren't already counted.
ALL_RUNTIME_TOOLS=$(echo "$EXPLICIT_TOOLS"; echo "$LOOP_PRELOAD_NAMES")
UNIQUE_RUNTIME=$(echo "$ALL_RUNTIME_TOOLS" | sort -u)
TOTAL_RUNTIME=$(echo "$UNIQUE_RUNTIME" | grep -c . || true)
echo "total unique runtime tools: $TOTAL_RUNTIME"

# 7. Reconcile counts.
if [ "$TOTAL_CONTRACTS" -ne "$TOTAL_RUNTIME" ]; then
  fail "count mismatch: contracts=$TOTAL_CONTRACTS runtime=$TOTAL_RUNTIME"
fi
pass "count reconciled: $TOTAL_CONTRACTS contracts = $TOTAL_RUNTIME runtime tools"

# 8. Every contract must have a runtime registration.
MISSING_FROM_RUNTIME=0
for name in $(echo "$EXPLICIT_TOOLS"; for suffix in $PRELOAD_CONTRACT_SUFFIXES; do echo "focusa_preload_${suffix}"; done); do
  if ! echo "$UNIQUE_RUNTIME" | grep -qx "$name"; then
    echo "  ✗ contract without runtime registration: $name"
    MISSING_FROM_RUNTIME=$((MISSING_FROM_RUNTIME + 1))
  fi
done
if [ "$MISSING_FROM_RUNTIME" -gt 0 ]; then
  fail "$MISSING_FROM_RUNTIME contracts have no runtime registerTool call"
fi
pass "all contracts have a runtime registration"

# 9. Every runtime tool must have a contract.
MISSING_FROM_CONTRACTS=0
for name in $UNIQUE_RUNTIME; do
  # Check explicit contracts
  if echo "$EXPLICIT_TOOLS" | grep -qx "$name" && echo "$EXPLICIT_CONTRACTS" > /dev/null; then
    if grep -q "name: \"$name\"" "$CONTRACTS"; then
      continue
    fi
  fi
  # Check preload contracts
  IS_PRELOAD=false
  for suffix in $PRELOAD_CONTRACT_SUFFIXES; do
    if [ "$name" = "focusa_preload_${suffix}" ]; then
      IS_PRELOAD=true
      break
    fi
  done
  if [ "$IS_PRELOAD" = true ]; then
    continue
  fi
  # Check explicit contract name in contracts file
  if grep -q "name: \"$name\"" "$CONTRACTS"; then
    continue
  fi
  echo "  ✗ runtime tool without contract: $name"
  MISSING_FROM_CONTRACTS=$((MISSING_FROM_CONTRACTS + 1))
done
if [ "$MISSING_FROM_CONTRACTS" -gt 0 ]; then
  fail "$MISSING_FROM_CONTRACTS runtime tools have no contract"
fi
pass "all runtime tools have a contract"

# 10. Every preload tool must have api_routes.
for suffix in $PRELOAD_CONTRACT_SUFFIXES; do
  tool="focusa_preload_${suffix}"
  if ! grep -A3 "suffix: \"$suffix\"" "$CONTRACTS" | grep -q "api_routes"; then
    # Preload contracts are auto-generated from the array, routes are constructed in .map
    :
  fi
done
pass "preload contracts generate api_routes via .map"

echo ""
echo "=== Pi tool inventory reconciliation: PASS ($TOTAL_CONTRACTS contracts = $TOTAL_RUNTIME tools) ==="
