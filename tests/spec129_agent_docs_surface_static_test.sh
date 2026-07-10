#!/usr/bin/env bash
# Spec129 — agent-internal public-safe docs surface guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/129-focusa-agent-internal-docs-and-knowledge-surface-spec.md"
DOC="$ROOT_DIR/docs/agent/01-focusa-agent-docs-index.md"
AGENTS="$ROOT_DIR/AGENTS.md"
INDEX="$ROOT_DIR/docs/INDEX.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$SPEC" ] || fail "Spec129 missing"
[ -f "$DOC" ] || fail "agent docs index missing"

grep -q 'docs/agent/01-focusa-agent-docs-index.md' "$AGENTS" || fail "AGENTS.md missing agent docs entry point"
grep -q 'agent/01-focusa-agent-docs-index.md' "$INDEX" || fail "docs/INDEX.md missing agent docs entry"

for required in \
  'What Focusa is' \
  'Architecture map' \
  'Canonical command surface' \
  'API and daemon rules' \
  'Workpoints, Evidence, and Trajectory' \
  'Update and release policy' \
  'Public/private boundary rules' \
  'Software layout checklist'; do
  grep -q "$required" "$DOC" || fail "agent docs missing section: $required"
done

for required in \
  'focusa help all' \
  'focusa first-mission' \
  'focusa workpoint checkpoint' \
  'POST /v1/workpoint/resume' \
  'GET /v1/telemetry/snapshot' \
  'scripts/guard-public-surface.sh'; do
  grep -q "$required" "$DOC" || fail "agent docs missing required reference: $required"
done

if rg -n '/home/wirebot|/root/|wpuiai.com/wp-admin|signalos\.pro|raw transcript|raw transcripts|docs/evidence/transcripts|\.focusa-private' "$DOC" "$SPEC"; then
  fail "agent docs/spec contain private path/admin URL/transcript leakage"
fi

pass "Spec129 agent docs surface static guard passed"
