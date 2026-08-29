#!/usr/bin/env bash
set -euo pipefail
if [[ "$(id -u)" == 0 ]]; then
  exec /usr/local/bin/as-user wirebot "$0"
fi
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/PERSISTED_STATE_PRIVACY_CLASSES.md"
PRED_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_predict_record.md"
METACOG_DOC="$ROOT_DIR/docs/focusa-tools/tools/focusa_metacog_capture.md"
PI_TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

[[ -f "$DOC" ]] || { echo "missing persisted-state privacy classes doc" >&2; exit 1; }

for needle in \
  "P0 Public" \
  "P4 Secret" \
  "Workpoint" \
  "Predictions" \
  "Metacognition" \
  "Store handles/evidence refs instead of raw provider payloads" \
  "tests/security_persisted_state_privacy_static_test.sh"; do
  if ! grep -Fq "$needle" "$DOC"; then
    echo "privacy doc missing marker: $needle" >&2
    exit 1
  fi
done

for f in "$PRED_DOC" "$METACOG_DOC"; do
  if ! grep -Eiq 'raw provider payload|evidence refs|evidence_refs|handles' "$f"; then
    echo "tool doc lacks raw-payload/evidence-ref privacy guidance: $f" >&2
    exit 1
  fi
done

# Pi model-facing tools should bound evidence refs for metacog capture and describe bounded ontology context for predictions.
for marker in \
  "evidence_refs: Array.isArray(raw.evidence_refs) ? raw.evidence_refs.slice(0, 8)" \
  "ontology_context: Type.Optional" \
  "Bounded ontology refs"; do
  if ! grep -Fq "$marker" "$PI_TOOLS"; then
    echo "Pi tool privacy/bounding marker missing: $marker" >&2
    exit 1
  fi
done

# Project docs should not contain obvious raw private key blocks.
if rg -n \
  --glob '!.git/**' \
  --glob '!node_modules/**' \
  --glob '!target/**' \
  --glob '!tests/security_persisted_state_privacy_static_test.sh' \
  --glob '!crates/focusa-core/src/silent_sessions/runner_security_test.rs' \
  --glob '!docs/evidence/PUBLIC_DOCS_RELEASE_SYNC_2026-05-26.md' \
  -- '-----BEGIN .*PRIVATE KEY-----' "$ROOT_DIR" >/tmp/focusa-private-key-scan.txt 2>/dev/null; then
  echo "private key block detected in repository" >&2
  cat /tmp/focusa-private-key-scan.txt >&2
  exit 1
fi

echo "✓ persisted-state privacy classes and redaction markers present"
