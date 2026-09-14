#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/release.yml"
DEPLOY_WORKFLOW="$ROOT/.github/workflows/deploy-live-daemon.yml"
SCRIPT="$ROOT/scripts/release-trust-metadata.py"
DEPLOY_PROOF_SCRIPT="$ROOT/scripts/release-deploy-proof.py"
KEYS="$ROOT/config/focusa-trusted-release-keys.json"
fail() { echo "FAIL: $*" >&2; exit 1; }

for file in "$WORKFLOW" "$DEPLOY_WORKFLOW" "$SCRIPT" "$DEPLOY_PROOF_SCRIPT" "$KEYS"; do
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
  (.keys|length)>=1 and
  ([.keys[] | select(.revoked_at==null)]|length)==1 and
  all(.keys[];
    .signing_algorithm=="ed25519" and
    (.public_key_base64|type=="string") and
    (.public_key_fingerprint|test("^[0-9a-f]{64}$"))
  )
' "$KEYS" >/dev/null || fail 'trusted release key metadata is invalid'

python3 - "$KEYS" <<'PY' || fail 'trusted release key fingerprint is invalid'
import base64
import hashlib
import json
from pathlib import Path
import sys

metadata = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
for key in metadata["keys"]:
    public_key = base64.b64decode(key["public_key_base64"], validate=True)
    if len(public_key) != 32:
        raise SystemExit(f"release public key must contain 32 bytes: {key['key_id']}")
    if hashlib.sha256(public_key).hexdigest() != key["public_key_fingerprint"]:
        raise SystemExit(f"release public key fingerprint mismatch: {key['key_id']}")
PY

grep -q 'focusa.release_manifest.v1' "$SCRIPT" \
  || fail 'manifest schema missing from generator'
grep -q 'focusa.release_provenance.v1' "$SCRIPT" \
  || fail 'provenance schema missing from generator'
grep -q 'private signing key does not match trusted public key metadata' "$SCRIPT" \
  || fail 'private/public key binding check missing'
grep -q "workflow_id: 'release.yml'" "$DEPLOY_WORKFLOW" \
  || fail 'deploy workflow does not accept successful full release gate as CI-equivalent proof'
grep -q 'scripts/release-deploy-proof.py' "$DEPLOY_WORKFLOW" \
  || fail 'deploy workflow does not generate signed deploy-success evidence'
grep -q -- '--distribution-parity /tmp/focusa-release/distribution-parity.json' "$DEPLOY_WORKFLOW" \
  || fail 'deploy-success proof is not bound to installed distribution parity'
grep -q 'deploy-success.json.sig' "$DEPLOY_WORKFLOW" \
  || fail 'deploy workflow does not upload detached deploy-success signature'
grep -q 'focusa.deploy_success.v1' "$DEPLOY_PROOF_SCRIPT" \
  || fail 'deploy-success proof schema missing'
grep -q 'release manifest detached signature is invalid' "$DEPLOY_PROOF_SCRIPT" \
  || fail 'deploy proof does not validate signed release manifest'
grep -q 'installed distribution parity proof is not accepted' "$DEPLOY_PROOF_SCRIPT" \
  || fail 'deploy proof does not fail closed on installed parity drift'

echo 'PASS: release/deploy workflows publish signatures, checksums, manifest, provenance, trust metadata, and signed deploy proof'
