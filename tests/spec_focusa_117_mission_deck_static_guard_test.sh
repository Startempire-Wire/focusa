#!/usr/bin/env bash
# Spec 117 — Mission Deck static guard.
# Phase 0 enforcement: any Mission Deck implementation bead must not close
# unless docs/117-mission-deck-onboarding-recall-pwa-spec.md exists AND
# contains the signature keywords below. This is the spec-first lifecycle
# gate for Spec 117 per docs/117 §21.7 Claim-Discipline Gate and §27
# Completion Criteria item 1.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/117-mission-deck-onboarding-recall-pwa-spec.md"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$SPEC" ]] || fail "spec 117 missing: $SPEC"

for needle in \
  "Focusa Mission Deck" \
  "Mission Ladder" \
  "Proof Meter" \
  "RecallCard" \
  "Walkthrough" \
  "Beginner Mode" \
  "Operator Mode" \
  "Browser/PWA Mission Deck" \
  "release readiness" \
  "five-minute" \
  "Claim-Discipline Gate" ; do
  grep -qF -- "$needle" "$SPEC" || fail "spec 117 missing signature marker: $needle"
done
pass "spec 117 exists and contains the canonical signature keywords"

cd "$ROOT_DIR" && python3 - <<'PY'
from pathlib import Path
text = Path('docs/117-mission-deck-onboarding-recall-pwa-spec.md').read_text()
assert 'schema_version: focusa.walkthrough.v1' in text
assert 'First Mission' in text
assert 'No Proof, No Done' in text
assert 'Mission Recall' in text
assert 'focusa deck' in text
PY
pass "spec 117 references walkthrough schema, required walkthroughs, Mission Recall, and focusa deck alias"

echo "focusa-117 spec-static-guard: PASS"