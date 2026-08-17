#!/usr/bin/env bash
# Spec129 — agent-internal public-safe docs surface guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/129-focusa-agent-internal-docs-and-knowledge-surface-spec.md"
DOC="$ROOT_DIR/docs/agent/01-focusa-agent-docs-index.md"
AGENTS="$ROOT_DIR/AGENTS.md"
INDEX="$ROOT_DIR/docs/INDEX.md"
README="$ROOT_DIR/README.md"
LLMS="$ROOT_DIR/docs/llms.txt"
ONBOARDING="$ROOT_DIR/docs/current/FOCUSA_FRIENDLY_ONBOARDING.md"
AGENT_CARD="$ROOT_DIR/docs/contracts/spec141/generated-capability-v2/agent-card.json"
TOOL_CONTRACTS="$ROOT_DIR/docs/current/focusa-tool-contracts.json"

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
  'Software layout checklist' \
  'Current authority and recovery model' \
  'All-Pi-tool and skill discovery' \
  'Silent Sessions' \
  'Mission Canvas' \
  'worktrees' \
  'automatic rollover' \
  'preserves user data'; do
  grep -q "$required" "$DOC" || fail "agent docs missing section: $required"
done

for required in \
  'focusa help all' \
  'focusa first-mission' \
  'focusa workpoint checkpoint' \
  'POST /v1/workpoint/resume' \
  'GET /v1/telemetry/snapshot' \
  'scripts/guard-public-surface.sh' \
  'focusa_agent_card' \
  'focusa_tool_search' \
  'generated-capability-v2/pi-tools.json' \
  'docs/focusa-tools/tools/focusa_<name>.md' \
  'focusa silent --help' \
  'focusa update --help' \
  'focusa uninstall --dry-run --keep-data'; do
  grep -q "$required" "$DOC" || fail "agent docs missing required reference: $required"
done

for required in \
  'Current agent-readiness fast path' \
  'all Focusa Pi tools' \
  'daemon-native Silent Sessions' \
  'uninstall with user data preserved'; do
  grep -qi "$required" "$AGENTS" || fail "AGENTS.md missing release-current boundary: $required"
done

for required in \
  'All 146 Focusa Pi tools' \
  'Daemon-native Silent Sessions' \
  'Mission Canvas and Work Rail' \
  'adaptive generated UI' \
  'uninstall with user data preserved'; do
  grep -qi "$required" "$README" || fail "README missing release-current public coverage: $required"
done

for required in 'Silent Sessions' 'worktrees' 'proactive compaction' 'connectors' 'uninstall with user data preserved'; do
  grep -qi "$required" "$LLMS" || fail "llms.txt missing machine-readable architecture coverage: $required"
done

for required in 'first-agent walkthrough' 'focusa_agent_card' 'all Focusa Pi tools' 'Customer lifecycle walkthrough' '--purge-data'; do
  grep -q -- "$required" "$ONBOARDING" || fail "onboarding missing current walkthrough coverage: $required"
done

TOOL_COUNT="$(jq -r '.tool_count' "$TOOL_CONTRACTS")"
jq -e --argjson tool_count "$TOOL_COUNT" \
  '.pi_tool_count == $tool_count and .pi_tool_docs_count == $tool_count and .skill_count >= 22 and .runbook_count >= .skill_count' \
  "$AGENT_CARD" >/dev/null \
  || fail "Agent Card lacks complete Pi tool/skill/runbook inventory"

if rg -n '/home/wirebot|/root/|wpuiai.com/wp-admin|signalos\.pro|raw transcript|raw transcripts|docs/evidence/transcripts|\.focusa-private' "$DOC" "$SPEC"; then
  fail "agent docs/spec contain private path/admin URL/transcript leakage"
fi

pass "Spec129 agent docs surface static guard passed"
