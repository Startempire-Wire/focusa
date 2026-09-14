#!/usr/bin/env python3
"""Exercise runner delegation/fail-closed behavior; not a cryptographic lease proof."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "scripts/run-predeployment-compatibility-canary.sh").read_text()
BODY = SOURCE.split("verify_authority_fixture() {", 1)[1].split("\n}\n", 1)[0]
FUNCTION = "verify_authority_fixture() {" + BODY + "\n}\n"


def exercise(state="active", node="enrolled-test-node", missing=None, tamper=False, corrupt=False, cli_error=False, missing_file=None):
    with tempfile.TemporaryDirectory(prefix="focusa-authority-fixture-test-") as temporary:
        root = Path(temporary)
        config = root / ".config/focusa"
        config.mkdir(parents=True)
        (root / "evidence").mkdir()
        (root / "bootstrap/candidate").mkdir(parents=True)
        (config / "node-identity.json").write_text(json.dumps({"schema": "focusa.node_identity.v1", "product": "focusa", "node_id": "enrolled-test-node"}))
        (config / "authority-lease.json").write_text("test fixture only; not a signed lease\n")
        sums = "".join(f"{hashlib.sha256((config / name).read_bytes()).hexdigest()}  {name}\n" for name in ("authority-lease.json", "node-identity.json"))
        (root / "evidence/authority-fixture.sha256").write_text(sums)
        authority = {"state": state, "node_id": node, "lease_id": "test-lease", "lease_digest": "test-digest"}
        if missing:
            authority.pop(missing)
        (root / "verdict.json").write_text(json.dumps({"authority": authority}))
        updater = root / "bootstrap/candidate/focusa-updater"
        updater.write_text('#!/bin/bash\nset -eu\n[[ "$*" == "license status --json" ]]\n'
                           + ('exit 7\n' if cli_error else '')
                           + ('printf tampered >> "$CANARY_ROOT/.config/focusa/authority-lease.json"\n' if tamper else '')
                           + 'cat "$CANARY_ROOT/verdict.json"\n')
        updater.chmod(0o700)
        if missing_file:
            (config / missing_file).unlink()
        if corrupt:
            (config / "node-identity.json").write_text("altered-node\n")
        # The interrupted-install path deliberately disables errexit: rejection
        # must survive that calling context, not depend on shell defaults.
        result = subprocess.run(["bash", "-c", "set +e\nset -o pipefail\n" + FUNCTION + "verify_authority_fixture\nexit $?\n"],
                                env={**os.environ, "CANARY_ROOT": str(root)}, capture_output=True)
        return result.returncode == 0


assert exercise()
for invalid in ("recoveryonly", "recovery_only", "expired", "offlinegrace", "unactivated", "revoked"):
    assert not exercise(state=invalid), invalid
assert not exercise(node="foreign-node")
assert not exercise(missing="lease_id")
assert not exercise(missing="lease_digest")
assert not exercise(corrupt=True)
assert not exercise(tamper=True)
assert not exercise(cli_error=True)
assert not exercise(missing_file="authority-lease.json")
assert not exercise(missing_file="node-identity.json")
assert "run_candidate_apply() {\n  verify_authority_fixture || return 1" in SOURCE
# A failed authority check must stop phase work even when a caller disables
# errexit or invokes the phase in a conditional context.
phase_guard = SOURCE.split("verify_phase() {", 1)[1].split("  local expected_tag=", 1)[0]
with tempfile.TemporaryDirectory(prefix="focusa-authority-phase-test-") as temporary:
    marker = Path(temporary) / "phase-entered"
    script = ("set +e\nverify_authority_fixture() { return 1; }\n"
              + "verify_phase() {" + phase_guard
              + '\n  printf unsafe > "$MARKER"\n}\nverify_phase\nexit $?\n')
    result = subprocess.run(["bash", "-c", script],
                            env={**os.environ, "MARKER": str(marker)}, capture_output=True)
    assert result.returncode != 0 and not marker.exists(), "rejected authority reached phase work"
assert SOURCE.index("verify_authority_fixture\n\n# Only the current") < SOURCE.index("update compatibility-bootstrap")
print("PASS: authority delegation rejects inactive, missing, foreign, altered and failed verifications even with errexit disabled")
