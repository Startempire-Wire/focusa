#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/release.yml"
SCRIPT="$ROOT/scripts/release-trust-metadata.py"
KEYS="$ROOT/config/focusa-trusted-release-keys.json"
fail() { echo "FAIL: $*" >&2; exit 1; }

for file in "$WORKFLOW" "$SCRIPT" "$KEYS"; do
  [[ -s "$file" ]] || fail "missing release trust surface: $file"
done

grep -q 'FOCUSA_RELEASE_ED25519_PRIVATE_KEY' "$WORKFLOW" \
  || fail 'release workflow does not consume the signing secret'
grep -q 'scripts/release-trust-metadata.py' "$WORKFLOW" \
  || fail 'release workflow does not generate trusted metadata'
for artifact in 'dist/*.sig' 'dist/SHA256SUMS.txt' 'dist/release-manifest.json' \
  'dist/release-provenance.json' 'dist/focusa-trusted-release-keys.json'; do
  grep -Fq "$artifact" "$WORKFLOW" || fail "release upload missing $artifact"
done
jq -e '
  .schema=="focusa.trusted_release_keys.v1" and
  (.keys|length)==1 and
  .keys[0].signing_algorithm=="ed25519" and
  (.keys[0].public_key_base64|type=="string") and
  (.keys[0].public_key_fingerprint|test("^[0-9a-f]{64}$")) and
  .keys[0].revoked_at==null
' "$KEYS" >/dev/null || fail 'trusted release key metadata is invalid'

grep -q 'focusa.release_manifest.v1' "$SCRIPT" \
  || fail 'manifest schema missing from generator'
grep -q 'focusa.release_provenance.v1' "$SCRIPT" \
  || fail 'provenance schema missing from generator'
grep -q 'private signing key does not match trusted public key metadata' "$SCRIPT" \
  || fail 'private/public key binding check missing'

echo 'PASS: release workflow publishes per-asset signatures, checksums, manifest, provenance, and trusted key metadata'
