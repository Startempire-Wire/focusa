#!/usr/bin/env python3
from pathlib import Path
W=(Path(__file__).resolve().parents[1]/'.github/workflows/release.yml').read_text()
for marker in ['pull-request-release-gate','Require release-scoped PR queue and tag inclusion','git merge-base --is-ancestor','needs: [rust-check, final-release-gap-gate, pull-request-release-gate, version-policy]','PREV_TAG=','RANGE="${PREV_TAG}..${TAG}"','### Breaking changes','### Features added','### Fixes shipped','### Merged pull requests','### Issues resolved','### Contributors','### Known issues','### Full changelog','SHA256SUMS','signature/provenance','focusa update rollback --dry-run=false --yes','### Downloads']:
 assert marker in W,marker
assert 'fetch-depth: 0' in W
print('Release tag template and PR inclusion gate static contract: PASS')
