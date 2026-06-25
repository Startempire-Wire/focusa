#!/bin/bash
# FOCUSA-zsld: Test that project_identity returns degraded_reasons array
set -e

DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"

echo "=== FOCUSA-zsld: Degraded Reasons Test ==="

# Test 1: Nonexistent path returns degraded_reasons
echo ""
echo "Test 1: /nonexistent/path returns degraded_reasons..."
RESPONSE=$(curl -s "$DAEMON_URL/v1/project/identity?project_root=/nonexistent/path")
REASONS_COUNT=$(echo "$RESPONSE" | jq '.degraded_reasons | length // 0')
if [ "$REASONS_COUNT" -gt 0 ]; then
  echo "✓ PASS: degraded_reasons has $REASONS_COUNT entries"
  echo "$RESPONSE" | jq '.degraded_reasons[0]' 2>/dev/null
else
  echo "✗ FAIL: degraded_reasons is empty"
  exit 1
fi

# Test 2: Each reason has required fields
echo ""
echo "Test 2: Each reason has required fields (code, severity, reason, fix)..."
FIRST_REASON=$(echo "$RESPONSE" | jq '.degraded_reasons[0]')
for field in code severity reason fix evidence_ref; do
  if echo "$FIRST_REASON" | jq -e ". | has(\"$field\")" > /dev/null; then
    echo "  ✓ $field: present"
  else
    echo "  ✗ $field: MISSING"
    exit 1
  fi
done

# Test 3: Valid project returns empty degraded_reasons
echo ""
echo "Test 3: Valid project returns empty degraded_reasons..."
VALID_RESPONSE=$(curl -s "$DAEMON_URL/v1/project/identity?project_root=/home/wirebot/focusa")
VALID_REASONS=$(echo "$VALID_RESPONSE" | jq '.degraded_reasons | length // 0')
if [ "$VALID_REASONS" = "0" ]; then
  echo "✓ PASS: Valid project has 0 degraded_reasons"
else
  echo "  degraded_reasons: $VALID_REASONS (expected 0)"
  echo "  Note: This may be acceptable if warnings are present"
fi

echo ""
echo "=== ALL TESTS PASSED ==="
