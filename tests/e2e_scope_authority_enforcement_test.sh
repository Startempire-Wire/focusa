#!/bin/bash
# FOCUSA_FIX-r4n9: True E2E test for scope authority enforcement
# Tests that when resuming a workpoint from a different project, execution is blocked

set -e

DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
PROJECT_A="/home/wirebot/focusa"
PROJECT_B="/tmp/e2e-test-project-different"

echo "=== FOCUSA_FIX-r4n9 E2E Scope Authority Enforcement Test ==="
echo "Daemon: $DAEMON_URL"
echo ""

# Setup: Create test project directories
mkdir -p "$PROJECT_A"
mkdir -p "$PROJECT_B"

# Step 1: Create a workpoint in project A
echo "Step 1: Creating workpoint in project A ($PROJECT_A)..."
WORKPOINT_CREATE=$(curl -s -X POST "$DAEMON_URL/v1/workpoint/checkpoint" \
  -H "Content-Type: application/json" \
  -d "{
    \"mission\": \"E2E test: focusa-r4n9 scope authority enforcement\",
    \"project_root\": \"$PROJECT_A\",
    \"continuity_id\": \"e2e-test-$(date +%s)\",
    \"next_action\": \"This should be blocked if resumed from different project\"
  }")

WORKPOINT_ID=$(echo "$WORKPOINT_CREATE" | jq -r '.workpoint_id // .details.workpoint_id // empty')
echo "Created workpoint: $WORKPOINT_ID"

if [ -z "$WORKPOINT_ID" ]; then
  echo "✗ FAIL: Could not create workpoint"
  echo "Response: $WORKPOINT_CREATE"
  exit 1
fi

echo "✓ Workpoint created"

# Step 2: Try to resume from project B (different project)
echo ""
echo "Step 2: Resuming workpoint from different project ($PROJECT_B)..."
echo "Expected: scope mismatch rejection"

WORKPOINT_RESUME=$(curl -s -X POST "$DAEMON_URL/v1/workpoint/resume" \
  -H "Content-Type: application/json" \
  -d "{
    \"workpoint_id\": \"$WORKPOINT_ID\",
    \"project_root\": \"$PROJECT_B\",
    \"continuity_id\": \"e2e-test-resume-$(date +%s)\"
  }")

echo "Response structure:"
echo "$WORKPOINT_RESUME" | jq '{status, canonical, scope_found, warnings}' 2>/dev/null

# Step 3: Verify the response blocks execution
echo ""
echo "Step 3: Verifying execution is blocked..."

# Key assertions based on actual API response
STATUS=$(echo "$WORKPOINT_RESUME" | jq -r '.status // empty')
SCOPE_FOUND=$(echo "$WORKPOINT_RESUME" | jq -r '.scope_found // true')
CANONICAL=$(echo "$WORKPOINT_RESUME" | jq -r '.canonical // false')
REJECTED=$(echo "$WORKPOINT_RESUME" | jq -r '.rejected // false')

# Check for safe_recovery guidance
SAFE_RECOVERY=$(echo "$WORKPOINT_RESUME" | jq -r '.safe_recovery // empty')
NEXT_STEP_HINT=$(echo "$WORKPOINT_RESUME" | jq -r '.next_step_hint // empty')

echo "Verification:"
echo "  status: $STATUS"
echo "  scope_found: $SCOPE_FOUND"
echo "  canonical: $CANONICAL"
echo "  safe_recovery: $SAFE_RECOVERY"

# Assertions
PASS1=false
PASS2=false
PASS3=false

# 1. Status should indicate rejection
if [ "$STATUS" = "rejected_scope_mismatch" ]; then
  echo "✓ PASS: Status is rejected_scope_mismatch"
  PASS1=true
elif [ "$REJECTED" = "true" ]; then
  echo "✓ PASS: Request was rejected"
  PASS1=true
else
  echo "✗ FAIL: Status should be rejected_scope_mismatch or rejected=true"
  echo "  Got status=$STATUS, rejected=$REJECTED"
fi

# 2. scope_found should be false or canonical should be false
if [ "$SCOPE_FOUND" = "false" ] || [ "$CANONICAL" = "false" ]; then
  echo "✓ PASS: Scope mismatch detected (scope_found=$SCOPE_FOUND, canonical=$CANONICAL)"
  PASS2=true
else
  echo "✗ FAIL: Should detect scope mismatch"
fi

# 3. Should provide safe recovery guidance
if [ -n "$SAFE_RECOVERY" ] && [ "$SAFE_RECOVERY" != "null" ]; then
  echo "✓ PASS: Safe recovery guidance provided: $SAFE_RECOVERY"
  PASS3=true
else
  echo "✗ FAIL: No safe recovery guidance"
fi

# Final verdict
echo ""
echo "=== TEST RESULTS ==="
if [ "$PASS1" = true ] && [ "$PASS2" = true ] && [ "$PASS3" = true ]; then
  echo "✓ ALL CHECKS PASSED"
  echo ""
  echo "Fix verified: Scope mismatch correctly blocks execution."
  echo ""
  echo "What happens now:"
  echo "1. Agent receives rejected_scope_mismatch status"
  echo "2. PI extension's buildAttentionRecallVerdict() detects conflictReason"
  echo "3. Focus Slice renders: next_action=BLOCKED: scope conflict..."
  echo "4. Agent cannot continue without focusa_project_identity verification"
  exit 0
else
  echo "✗ SOME CHECKS FAILED"
  exit 1
fi
