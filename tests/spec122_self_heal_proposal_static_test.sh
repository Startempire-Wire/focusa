#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

python3 -m py_compile scripts/propose-system-fix.py scripts/auto-heal-audit.py scripts/audit-schema.py

grep -q 'scripts/propose-system-fix.py' .github/workflows/audit-recorder.yml
grep -q 'suppressing passive audit commit' .github/workflows/audit-recorder.yml
grep -q 'should_commit' .github/workflows/audit-recorder.yml
grep -q 'gh pr create' .github/workflows/audit-recorder.yml
grep -q 'pull-requests: write' .github/workflows/audit-recorder.yml

grep -q 'fail_count_30d' scripts/propose-system-fix.py
grep -q 'deliverable' scripts/propose-system-fix.py
grep -q 'intervention_rate' scripts/propose-system-fix.py
grep -q 'auto_generated.*False' scripts/propose-system-fix.py

grep -q 'propose-system-fix.py' scripts/auto-heal-audit.py

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/release-proof/audit" "$tmp/tests" "$tmp/scripts"
cp scripts/propose-system-fix.py "$tmp/scripts/"
touch "$tmp/tests/spec122_self_heal_proposal_static_test.sh" "$tmp/scripts/retry.sh" "$tmp/scripts/process-health-check.py" "$tmp/scripts/deploy-smoke-check.sh"
cat > "$tmp/release-proof/audit/audit.jsonl" <<'JSONL'
{"id":"fail-1","ts":"2099-01-01T00:00:00Z","event":"failure","category":"ci_workflow_failure","subsystem":"ci","scope":"CI","symptom":"clippy failed","root_cause":"warning","fix":"manual","guard":"open","test":"open","linked_run":"1","failure_class":"ci_clippy_failure"}
{"id":"fail-2","ts":"2099-01-02T00:00:00Z","event":"failure","category":"ci_workflow_failure","subsystem":"ci","scope":"CI","symptom":"clippy failed","root_cause":"warning","fix":"manual","guard":"open","test":"open","linked_run":"2","failure_class":"ci_clippy_failure"}
{"id":"fail-3","ts":"2099-01-03T00:00:00Z","event":"failure","category":"ci_workflow_failure","subsystem":"ci","scope":"CI","symptom":"clippy failed","root_cause":"warning","fix":"manual","guard":"open","test":"open","linked_run":"3","failure_class":"ci_clippy_failure"}
JSONL
(cd "$tmp" && python3 scripts/propose-system-fix.py --dry-run > result.json)
grep -q '"should_commit": true' "$tmp/result.json"
grep -q '"fail_count_30d": 3' "$tmp/result.json"
grep -q '"auto_generated": false' "$tmp/result.json"

cat > "$tmp/release-proof/audit/audit.jsonl" <<'JSONL'
{"id":"fail-1","ts":"2099-01-01T00:00:00Z","event":"failure","category":"ci_workflow_failure","subsystem":"ci","scope":"CI","symptom":"clippy failed","root_cause":"warning","fix":"manual","guard":"open","test":"open","linked_run":"1","failure_class":"ci_clippy_failure"}
{"id":"fail-2","ts":"2099-01-02T00:00:00Z","event":"failure","category":"ci_workflow_failure","subsystem":"ci","scope":"CI","symptom":"clippy failed","root_cause":"warning","fix":"manual","guard":"open","test":"open","linked_run":"2","failure_class":"ci_clippy_failure"}
JSONL
before_lines="$(wc -l < "$tmp/release-proof/audit/audit.jsonl")"
(cd "$tmp" && python3 scripts/propose-system-fix.py > below-threshold.json)
after_lines="$(wc -l < "$tmp/release-proof/audit/audit.jsonl")"
test "$before_lines" = "$after_lines"
grep -q '"should_commit": false' "$tmp/below-threshold.json"
