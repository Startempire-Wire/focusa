#!/bin/bash
# Contract: Spec80 §20.2 runtime harnesses must produce executable gate decisions for latency and restore/compaction budgets.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
HARNESS="${ROOT_DIR}/scripts/spec80/perf_gate_harness.py"
TMP_DIR="$(mktemp -d /tmp/spec80-gates.XXXXXX)"
trap 'rm -rf "$TMP_DIR"' EXIT
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if [ -x "$HARNESS" ]; then
  log_pass "runtime gate harness script exists and is executable"
else
  log_fail "runtime gate harness script missing or not executable"
fi

python3 - <<'PY' "$TMP_DIR/reflection.jsonl" "$TMP_DIR/restore.jsonl"
import json, sys
ref, rst = sys.argv[1], sys.argv[2]
with open(ref, 'w') as f:
    for _ in range(200):
        f.write(json.dumps({"mode":"baseline","latency_ms":100})+"\n")
    for _ in range(200):
        f.write(json.dumps({"mode":"with_metacog","latency_ms":108})+"\n")
with open(rst, 'w') as f:
    for _ in range(200):
        f.write(json.dumps({"operation":"restore","latency_ms":300})+"\n")
    for _ in range(200):
        f.write(json.dumps({"operation":"compaction","profile":"prebranch","latency_ms":200})+"\n")
    for _ in range(200):
        f.write(json.dumps({"operation":"compaction","profile":"branch","latency_ms":260})+"\n")
PY

python3 "$HARNESS" reflection-latency --input "$TMP_DIR/reflection.jsonl" --output "$TMP_DIR/reflection-out.json" >/dev/null
if jq -e '.gate_id=="D3.1-latency" and .decision=="pass" and .added_latency_ratio <= 0.12' "$TMP_DIR/reflection-out.json" >/dev/null 2>&1; then
  log_pass "reflection/metacog runtime gate computes <=12% p95 decision"
else
  log_fail "reflection/metacog runtime gate output invalid"
fi

python3 "$HARNESS" restore-compaction --input "$TMP_DIR/restore.jsonl" --output "$TMP_DIR/restore-out.json" >/dev/null
if jq -e '.gate_id=="D3.2-restore-compaction" and .decision=="pass" and .restore.p95_ms <= 400 and .compaction.ratio <= 1.5' "$TMP_DIR/restore-out.json" >/dev/null 2>&1; then
  log_pass "restore/compaction runtime gate computes 400ms/1.5x decisions"
else
  log_fail "restore/compaction runtime gate output invalid"
fi

echo "=== SPEC80 IMPL RUNTIME GATE HARNESSES RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
