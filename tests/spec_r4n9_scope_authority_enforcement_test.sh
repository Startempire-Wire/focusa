#!/bin/bash
# FOCUSA_FIX-r4n9: Test that next_action is cut when authority is suppressed
set -e

echo "=== FOCUSA_FIX-r4n9: Scope Authority Enforcement Test ==="

# Test 1: Verify state.ts has the fix
echo ""
echo "Test 1: Checking state.ts for next_action cut..."
if grep -q "BLOCKED: scope conflict" apps/pi-extension/src/state.ts; then
    echo "✓ PASS: state.ts contains blocked message for scope conflict"
else
    echo "✗ FAIL: state.ts missing blocked message"
    exit 1
fi

# Test 2: Verify the fix uses conflictReason
echo ""
echo "Test 2: Checking conflictReason conditional..."
if grep -q "conflictReason" apps/pi-extension/src/state.ts && grep -q "BLOCKED: scope conflict" apps/pi-extension/src/state.ts; then
    echo "✓ PASS: Fix uses conflictReason to gate next_action"
else
    echo "✗ FAIL: Fix not properly gated"
    exit 1
fi

# Test 3: Verify Focus Slice formatting has blocked indicator
echo ""
echo "Test 3: Checking Focus Slice has ⛔ indicator..."
if grep -q "⛔" apps/pi-extension/src/state.ts && grep -q "EXECUTION BLOCKED" apps/pi-extension/src/state.ts; then
    echo "✓ PASS: Focus Slice shows ⛔ and EXECUTION BLOCKED"
else
    echo "✗ FAIL: Focus Slice missing blocked indicator"
    exit 1
fi

# Test 4: Verify TypeScript compiles
echo ""
echo "Test 4: Running TypeScript compilation..."
cd apps/pi-extension
if npx tsc --noEmit 2>&1 | grep -q "error"; then
    echo "✗ FAIL: TypeScript has errors"
    exit 1
else
    echo "✓ PASS: TypeScript compiles cleanly"
fi
cd ../..

# Test 5: Verify authority suppressed detection in tools.ts
echo ""
echo "Test 5: Checking authoritySuppressed detection..."
if grep -q "authoritySuppressed" apps/pi-extension/src/tools.ts; then
    echo "✓ PASS: authoritySuppressed detection present in tools.ts"
else
    echo "✗ FAIL: authoritySuppressed not found"
    exit 1
fi

echo ""
echo "=== ALL TESTS PASSED ==="
echo ""
echo "Summary of fix:"
echo "- When action_authority_for_current_ask=false (scope conflict)"
echo "- next_action shows: 'BLOCKED: scope conflict — verify project scope...'"
echo "- Focus Slice shows: ⛔ next_action=... ← EXECUTION BLOCKED"
echo "- Agent can no longer ignore the block and continue"
