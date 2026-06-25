#!/bin/bash
# FOCUSA_FIX GitHub #2: Test stale Focusa carryover hard-stops cross-project continuation
set -e

DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"

echo "=== GitHub #2: Stale Carryover Hard-Stop Test ==="

# Step 1: Create workpoint in project A
PROJECT_A="/home/wirebot/focusa"
PROJECT_B="/home/focusadev/perpetua"

echo "Step 1: Creating workpoint in $PROJECT_A..."
WP_CREATE=$(curl -s -X POST "$DAEMON_URL/v1/workpoint/checkpoint" \
  -H "Content-Type: application/json" \
  -d "{
    \"mission\": \"GitHub #2 test: stale carryover hard-stop\",
    \"project_root\": \"$PROJECT_A\",
    \"continuity_id\": \"github-issue-2-test-$(date +%s)\",
    \"next_action\": \"Continue work on focusa\"
  }")

WORKPOINT_ID=$(echo "$WP_CREATE" | jq -r '.workpoint_id // empty')
echo "Created: $WORKPOINT_ID"

# Step 2: Try to resume from a completely different project (simulating stale carryover)
echo ""
echo "Step 2: Attempting resume from $PROJECT_B (stale carryover simulation)..."
RESUME=$(curl -s -X POST "$DAEMON_URL/v1/workpoint/resume" \
  -H "Content-Type: application/json" \
  -d "{
    \"workpoint_id\": \"$WORKPOINT_ID\",
    \"project_root\": \"$PROJECT_B\",
    \"continuity_id\": \"github-issue-2-resume-$(date +%s)\"
  }")

STATUS=$(echo "$RESUME" | jq -r '.status // empty')
CANONICAL=$(echo "$RESUME" | jq -r 'if has("canonical") then .canonical else "missing" end')
SCOPE_FOUND=$(echo "$RESUME" | jq -r 'if has("scope_found") then .scope_found else "missing" end')
NEXT_STEP=$(echo "$RESUME" | jq -r '.next_step_hint // empty')

echo ""
echo "Full response:"
echo "$RESUME" | jq '.' 2>/dev/null | head -40

echo "Status: $STATUS"
echo "Canonical: $CANONICAL"
echo "Scope Found: $SCOPE_FOUND"
echo "Next Step Hint: $NEXT_STEP"

# Assertions
PASS=false
if [ "$STATUS" = "rejected_scope_mismatch" ] && [ "$CANONICAL" = "false" ] && [ "$SCOPE_FOUND" = "false" ]; then
  echo ""
  echo "✓ PASS: Stale carryover correctly hard-stopped"
  echo "  - Status: rejected_scope_mismatch"
  echo "  - Canonical: false"
  echo "  - Execution blocked: agent must verify scope first"
  PASS=true
else
  echo ""
  echo "✗ FAIL: Stale carryover not properly blocked"
  echo "  Expected: rejected_scope_mismatch, false, false"
  echo "  Got: $STATUS, $CANONICAL, $SCOPE_FOUND"
fi

if [ "$PASS" = true ]; then
  exit 0
else
  exit 1
fi
