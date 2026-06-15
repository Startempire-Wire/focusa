#!/usr/bin/env bash
# Spec107 Mac pairing overclaim regression test — focusa-bwky.6
# Regression proves focusa-ui0y.15-style claim blocks until macOS native evidence exists.

set -euo pipefail

FOCUSA="${1:-}"
if [ -z "$FOCUSA" ] || [ ! -x "$FOCUSA" ]; then
  FOCUSA="/tmp/focusa-target-4jo54/release/focusa"
fi
if [ ! -x "$FOCUSA" ]; then
  echo "FAIL: focusa binary not found at $FOCUSA"
  echo "  Build with: cargo build --package focusa-cli --release"
  exit 1
fi

echo "=== Spec107 Mac pairing overclaim regression ==="
echo "Binary: $FOCUSA"

# Test 1: Surrogate evidence must be blocked
echo ""
echo "Test 1: Surrogate evidence (api:/v1/ for macOS native) -> BLOCK"
output=$("$FOCUSA" claim classify \
  --work-item-id focusa-ui0y.15 \
  --claim "Evidence citations: docs/evidence/MAC_MENUBAR_PAIRING_E2E_CHECKLIST_2026-06-15.md (class: surrogate) ; api:/v1/device/pair/list (class: partial)" \
  2>&1 || true)

if echo "$output" | grep -q "decision: block"; then
  echo "  PASS: blocked as expected"
  if echo "$output" | grep -q "evidence_class: surrogate"; then
    echo "  PASS: classified as surrogate"
  else
    echo "  FAIL: not classified as surrogate"
    echo "$output"
    exit 1
  fi
else
  echo "  FAIL: should have been blocked"
  echo "$output"
  exit 1
fi

# Test 2: No evidence citations -> blocked as missing
echo ""
echo "Test 2: No evidence citations -> BLOCK"
output=$("$FOCUSA" claim classify \
  --work-item-id focusa-ui0y.15 \
  --claim "Completed Mac menubar pairing" \
  2>&1 || true)

if echo "$output" | grep -q "decision: block"; then
  echo "  PASS: blocked as expected"
  if echo "$output" | grep -q "evidence_class: missing"; then
    echo "  PASS: classified as missing"
  else
    echo "  FAIL: not classified as missing"
    echo "$output"
    exit 1
  fi
else
  echo "  FAIL: should have been blocked"
  echo "$output"
  exit 1
fi

# Test 3: Actual evidence -> allowed
echo ""
echo "Test 3: Actual evidence (macOS screenshot path) -> ALLOW"
output=$("$FOCUSA" claim classify \
  --work-item-id focusa-ui0y.15 \
  --claim "Evidence citations: tests/mac_pairing_e2e_screenshot_test.sh ; apps/menubar/src-tauri/Cargo.toml" \
  2>&1 || true)

if echo "$output" | grep -q "decision: allow"; then
  echo "  PASS: allowed as expected"
else
  echo "  FAIL: should have been allowed"
  echo "$output"
  exit 1
fi

# Test 4: Blocked evidence with operator deferral -> allowed
echo ""
echo "Test 4: Blocked evidence + --deferred -> ALLOW"
output=$("$FOCUSA" claim classify \
  --work-item-id focusa-ui0y.15 \
  --claim "Evidence citations: apps/menubar/src-tauri/Cargo.toml (class: blocked)" \
  --deferred \
  2>&1 || true)

if echo "$output" | grep -q "decision: allow"; then
  echo "  PASS: allowed with deferral as expected"
else
  echo "  FAIL: should have been allowed with operator deferral"
  echo "$output"
  exit 1
fi

echo ""
echo "=== PASS: All Spec107 Mac pairing regression tests passed ==="
echo "Evidence: focusa-ui0y.15-style claims are blocked by the claim gate"
echo "Proof: $FOCUSA claim classify against surrogate/partial/missing evidence patterns"
