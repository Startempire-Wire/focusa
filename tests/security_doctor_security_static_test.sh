#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCTOR="$ROOT_DIR/crates/focusa-cli/src/commands/doctor.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
DOC="$ROOT_DIR/docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md"
RESOURCE_DOC="$ROOT_DIR/docs/current/API_RESOURCE_LIMITS.md"

for marker in   "DoctorCommand::Security"   "security_posture_payload"   "FOCUSA_API_MAX_BODY_BYTES"   "FOCUSA_API_MUTATION_RATE_LIMIT_PER_WINDOW"   "FOCUSA_API_JSON_MAX_DEPTH"   "FOCUSA_AUTH_TOKEN"   "Reverse-proxy rate-limit guidance" \
  "/home/wirebot/focusa"; do
  if ! grep -Fq "$marker" "$DOCTOR"; then
    echo "doctor security implementation missing marker: $marker" >&2
    exit 1
  fi
done

for marker in   "Doctor(commands::doctor::DoctorArgs)"   "commands::doctor::run(cli.json, args)"; do
  if ! grep -Fq "$marker" "$MAIN"; then
    echo "CLI doctor security parser missing marker: $marker" >&2
    exit 1
  fi
done

for marker in   "focusa doctor security"   "focusa --json doctor security"   "deployment/API security report"   "not a license-plan or entitlement command"; do
  if ! grep -Fq "$marker" "$DOC" "$RESOURCE_DOC"; then
    echo "doctor security docs missing marker: $marker" >&2
    exit 1
  fi
done

echo "✓ doctor security posture static markers present"
