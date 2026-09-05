#!/usr/bin/env python3
"""Static contract for the signed, isolated predeployment compatibility canary."""

from __future__ import annotations

import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]


def text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(source: str, token: str, message: str) -> None:
    if token not in source:
        raise AssertionError(message)


def main() -> int:
    generator = text("scripts/release-trust-metadata.py")
    trust = text("crates/focusa-cli/src/commands/update_trust.rs")
    update = text("crates/focusa-cli/src/commands/update.rs")
    install = text("crates/focusa-cli/src/commands/install.rs")
    release = text(".github/workflows/release.yml")
    deploy = text(".github/workflows/deploy-live-daemon.yml")
    runner = text("scripts/run-predeployment-compatibility-canary.sh")
    receipt = text("scripts/compatibility-canary-receipt.py")
    deploy_proof = text("scripts/release-deploy-proof.py")
    parity_audit = text("scripts/audit-distribution-parity.mjs")

    require(generator, '"baseline_release": compatibility_baseline_input', "current signer must bind the complete baseline")
    require(trust, "signed compatibility baseline binding is missing", "missing signed baseline must fail closed")
    require(install, "compatibility canary install requires current-signer frozen asset digests", "installer must require authenticated baseline bytes")
    require(install, "binding.expected_checksum(asset)?", "signed digests must override mutable checksum downloads")
    require(update, "baseline_asset_digests", "bootstrap and rollback must consume verified baseline authority")
    require(runner, "update compatibility-bootstrap", "current CLI must own baseline installation")
    assert "PRIOR_CLI" not in runner and "bootstrap/prior" not in runner, "old installer execution is forbidden"
    assert runner.index("unset GITHUB_TOKEN GH_TOKEN") < runner.index("update compatibility-bootstrap")

    for token in (
        "focusa.compatibility_canary_authorization.v1",
        '"environment": "isolated_preproduction"',
        '"allowed_install_scope": "non_root_ephemeral_home"',
        '"production_apply_authorized": False',
        '"system_install_authorized": False',
        '"service_mutation_authorized": False',
        '"automatic_apply_authorized": False',
    ):
        require(generator, token, f"signed candidate manifest omits {token}")

    for token in (
        "ReleaseMetadataMode::CompatibilityCanary",
        "verify_compatibility_canary_authorization",
        'publication_status.as_deref() != Some("candidate_only")',
        'contains_key("distribution-manifest.json")',
        "release_compatibility_canary_proof_not_verified",
    ):
        require(trust + update, token, f"canary trust gate omits {token}")

    for token in (
        "compatibility_canary_root",
        "compatibility_canary_automatic_apply_forbidden",
        "FOCUSA_COMPATIBILITY_CANARY_PARENT",
        ".focusa-compatibility-canary-scope.json",
        "compatibility canary must run as a non-root user",
        "signed lease fixture is missing",
        "legacy database fixture is missing",
        "user sentinel is missing",
    ):
        require(update, token, f"isolated update boundary omits {token}")

    for token in (
        "LEGACY_RELEASE_BINARIES",
        "release_binaries_for_tag",
        "allow_verified_rollback",
    ):
        require(install, token, f"legacy rollback compatibility omits {token}")

    checksums_at = release.index("  checksums:")
    canary_at = release.index("  predeployment-compatibility-canary:")
    dispatch_at = release.index("  dispatch-deploy-live-daemon:")
    if not checksums_at < canary_at < dispatch_at:
        raise AssertionError("production dispatch is not ordered after the signed canary")
    for token in (
        "runs-on: [self-hosted, linux, x64, focusa-deploy]",
        "--compatibility-from-tag v0.9.177",
        "scripts/run-predeployment-compatibility-canary.sh",
        "compatibility-canary-success.json.sig",
        "--previous-tag v0.9.177",
        "needs: predeployment-compatibility-canary",
    ):
        require(release, token, f"release DAG omits {token}")

    canary_gate_at = deploy.index("Require signed predeployment compatibility canary")
    first_mutation_at = min(
        deploy.index("Safe disk cleanup preflight"),
        deploy.index("Sync public macOS/Linux and Windows bootstrappers"),
        deploy.index("Install and restart live daemon"),
    )
    if canary_gate_at > first_mutation_at:
        raise AssertionError("production can mutate before signed canary verification")
    require(
        deploy,
        "scripts/compatibility-canary-receipt.py verify",
        "production deploy does not verify the signed canary receipt",
    )
    require(
        deploy,
        "--previous-tag v0.9.177",
        "production deploy does not bind the exact compatibility baseline",
    )

    sequence = [
        'verify_phase "$PREVIOUS_TAG" 0 prior-initial',
        'apply_candidate\nverify_phase "$RELEASE_TAG" 1 candidate-first',
        'update rollback \\\n  --part all',
        'verify_phase "$PREVIOUS_TAG" 0 prior-rollback',
        'apply_candidate\nverify_phase "$RELEASE_TAG" 1 candidate-reapply',
    ]
    cursor = 0
    for token in sequence:
        position = runner.find(token, cursor)
        if position < 0:
            raise AssertionError(f"canary runtime sequence omits or reorders {token}")
        cursor = position + len(token)
    for token in (
        "production_fingerprint",
        "signed lease fixture changed",
        "user sentinel changed",
        "legacy database canary copy failed quick_check",
        "compatibility-canary-database-inventory.py",
        "FOCUSA_COMPATIBILITY_CANARY_FAULT=after_asset_download",
        "prior-interrupted-recovery",
        "audit-distribution-parity.mjs",
        "FOCUSA_PI_EXTENSION_PACKAGE_JSON",
        "FOCUSA_BIND=\"127.0.0.1:$DAEMON_PORT\"",
    ):
        require(runner, token, f"canary runtime evidence omits {token}")
    if "sudo " in runner or "systemctl restart" in runner:
        raise AssertionError("isolated canary contains a production mutation primitive")
    for token in ("FOCUSA_INSTALL_ROOT", "FOCUSA_DAEMON_HEALTH_URL"):
        require(parity_audit, token, f"parity audit omits isolated override {token}")

    require(receipt, "focusa.compatibility_canary_success.v1", "receipt schema missing")
    require(receipt, "database_evidence", "receipt database evidence gate missing")
    require(receipt, "distribution_parity", "receipt distribution parity gate missing")
    require(receipt, "--previous-tag", "receipt is not bound to exact prior release")
    require(
        receipt,
        'for command in ("sign", "verify")',
        "receipt sign/verify modes are missing",
    )
    require(
        deploy_proof,
        "superseded_by_production_deploy",
        "production deployment does not revoke canary-only authority",
    )

    print("predeployment compatibility canary static contract: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
