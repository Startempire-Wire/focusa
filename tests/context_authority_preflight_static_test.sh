#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/CONTEXT_AUTHORITY_CURRENT.md"
SKILL="$ROOT_DIR/apps/pi-extension/skills/focusa/SKILL.md"
AUTH="$ROOT_DIR/docs/current/AUTHORITY_MODEL.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$DOC" "$SKILL" "$AUTH"; do
  rg -n 'Before risky mutation|risky mutation|Mutation boundary|mutation boundary' "$file" >/dev/null || fail "$file missing risky mutation boundary"
done
pass "docs/skill declare risky mutation boundary"

for trigger in 'binary replacement' 'daemon restart' 'deploy' 'release publish' 'git push' 'destructive file operation' 'database migration' 'broad refactor' 'cross-project file edit' 'generated code overwrite' 'secret/config change' 'live service action' 'pairing/install/update ambiguity'; do
  rg -n "$trigger" "$DOC" >/dev/null || fail "Context Authority doc missing trigger: $trigger"
done
pass "Context Authority doc lists required risky mutation triggers"

for verdict in allow block ask_operator verify_first diagnosis_only planning_only; do
  rg -n "\b$verdict\b" "$DOC" "$AUTH" >/dev/null || fail "missing verdict $verdict"
done
pass "required Context Authority verdicts documented"

rg -n 'focusa action preflight' "$DOC" >/dev/null || fail "Context Authority doc missing action preflight command"
rg -n 'ActionPreflightArgs|ActionCmd|mutation preflight' "$ROOT_DIR/crates/focusa-cli/src" >/dev/null || fail "CLI source missing action preflight"
pass "preflight command documented and present in CLI source"

echo "context authority preflight static test: PASS"
