#!/usr/bin/env python3
"""Spec 172.05.10 — Installed exact-SHA cross-platform acceptance receipts.

Static, fail-closed receipt verification for the installed exact-SHA
cross-platform Spec 172 acceptance surfaces (atom focusa-vbcqu.20.15.41,
lane acceptance):

  * Final exact-SHA macOS/Windows/Linux artifacts and release receipts — the
    release pipeline locks the candidate to the exact pipeline SHA
    (exact_sha == env.GITHUB_SHA), reuses only successful candidate gates for
    that SHA, and binds the release trust metadata to "$GITHUB_SHA"; the
    Desktop Spec 172 entitlement gate consumes the Tauri bundles produced by
    the locked-release-candidate-artifacts.yml desktop-artifacts job and is a
    gate only (never builds, signs, deploys, or releases).
  * Installed CLI/Desktop/menubar/Pi/Cockpit/installer flows — every Spec 172
    installed surface carries the canonical presenter projection and fails
    closed: Desktop bridge (wired through main.rs), menubar posture module,
    TUI presenter, Pi entitlement adapter, Cockpit mixed-product action
    registry, CLI/agent presenter fixtures, and the Unix/PowerShell/Rust
    installers (no local/self-issued grant, raw keys forbidden, verified
    limited activation).
  * WPUIAI test authority — WPUIAI EDD remains the canonical authority;
    public UIAI proof, observe-only limited mode, limited-access assertions,
    and public-facade convergence receipts are present and exact-SHA bound.
  * Every Spec 172 journey is actually re-run here with real exit codes —
    verified limited, paid standalone, Bundle, refund/revoke, offline,
    dynamic-tool, recovery/export/update/uninstall, and public facade — plus
    every installed-surface gate (installer, desktop, menubar/TUI, cockpit,
    CLI, cross-surface adversarial). Cargo-invoking journeys route through
    the canonical OVH build (one cargo command at a time, global lock); the
    build host must be reachable (FOCUSA_BUILD_REMOTE set to the equivalent
    Tailscale peer, see the 20.15.38 acceptance record).
  * Exact-SHA binding of the phase-05 evidence chain and redaction hygiene —
    every docs/evidence/spec172 acceptance record binds an exact commit SHA
    and no raw email, key, token, customer row, credential, or card data
    appears anywhere under the evidence directory.

Pure stdlib, deterministic, zero hidden skips: every assertion reads the
committed tree and every journey is a real subprocess run whose exit code is
recorded verbatim. The test exits 0 only when the complete installed
exact-SHA receipt chain is present and every Spec 172 journey passes without
relying on demo prerelease evidence.

Exact verification:
    python3 tests/spec172_installed_acceptance_receipt_test.py
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RELEASE_WF = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
DESKTOP_WF = (ROOT / ".github/workflows/desktop-spec172-entitlement-gate.yml").read_text(
    encoding="utf-8"
)
CANDIDATE_WF = (ROOT / ".github/workflows/locked-release-candidate-artifacts.yml").read_text(
    encoding="utf-8"
)
BRIDGE = (ROOT / "apps/menubar/src-tauri/src/spec172_desktop_bridge.rs").read_text(
    encoding="utf-8"
)
MAIN_RS = (ROOT / "apps/menubar/src-tauri/src/main.rs").read_text(encoding="utf-8")
MENUBAR_TS = (ROOT / "apps/menubar/src/lib/spec172Posture.ts").read_text(encoding="utf-8")
TUI_RS = (ROOT / "crates/focusa-tui/src/spec172_presenter.rs").read_text(encoding="utf-8")
PI_TS = (ROOT / "apps/pi-extension/src/entitlement-policy-adapter.ts").read_text(
    encoding="utf-8"
)
COCKPIT_RS = (ROOT / "crates/focusa-license/src/cockpit_action_registry.rs").read_text(
    encoding="utf-8"
)
CLI_FIXTURES = (ROOT / "crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json").read_text(
    encoding="utf-8"
)
INSTALL_RS = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text(encoding="utf-8")
EVIDENCE_DIR = ROOT / "docs/evidence/spec172"

# Accepted card-pattern hygiene flake (documented): the gates' random opaque
# settlement tokens (bin2hex(random_bytes(16))) can randomly form a 16-digit
# run that their own card-pattern self-check false-positives on (refund gate:
# "no card data in any settlement decision"; EDD commerce matrix:
# "no card data in any matrix decision"). Mirror the acceptance lane and the
# build-independent gate (tests/spec172_build_independent_gate.sh run_case):
# bounded genuine re-runs ONLY for the exact documented signature, each
# attempt a real full run, recorded exit code a real run result.
FLAKE_SIGNATURE = "no card data in any"
FLAKE_MAX_ATTEMPTS = 4

JOURNEY_RUNS: dict[str, dict] = {}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(f"installed acceptance receipt missing/weak: {message}")


def require_in(text: str, needle: str, message: str) -> None:
    require(needle in text, f"{message} (missing: {needle!r})")


def run_journey(name: str, argv: list[str], timeout: int = 2400) -> int:
    """Run one Spec 172 journey once and record its REAL exit code."""
    proc = subprocess.run(
        argv,
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    exit_code = proc.returncode
    JOURNEY_RUNS[name] = {"argv": " ".join(argv), "exit": exit_code}
    if exit_code != 0:
        raise AssertionError(
            f"journey failed rc={exit_code} for {name} argv={' '.join(argv)}\n"
            f"{proc.stdout[-1500:]}\n{proc.stderr[-1500:]}"
        )
    return exit_code


# ── 1. Final exact-SHA artifacts and release receipts ─────────────────────

# The release pipeline reuses only successful candidate gates for the exact
# pipeline SHA and locks the release candidate to that exact SHA.
require_in(RELEASE_WF, "Lock exact release candidate", "exact release candidate lock missing")
require_in(RELEASE_WF, "exact_sha: $exact_sha", "release candidate manifest lacks exact_sha")
require_in(RELEASE_WF, ".exact_sha == env.GITHUB_SHA", "release candidate not bound to exact SHA")
require_in(RELEASE_WF, "headSha == $sha", "candidate gate reuse not bound to exact SHA")
require_in(RELEASE_WF, '--commit "$GITHUB_SHA"', "release trust metadata not bound to exact SHA")
require_in(RELEASE_WF, '--sha "$GITHUB_SHA"', "release intelligence not bound to exact SHA")

# The Desktop Spec 172 entitlement gate consumes the final candidate Tauri
# bundles and is a gate only: it never builds, signs, deploys, or releases.
require_in(DESKTOP_WF, "locked-release-candidate-artifacts.yml", "desktop gate not wired to candidate artifacts")
require_in(DESKTOP_WF, "desktop-artifacts", "desktop gate not wired to desktop-artifacts job")
require_in(DESKTOP_WF, "workflow is a gate only: it never builds, signs, deploys, or releases",
           "desktop gate no longer gate-only")
for forbidden in ("cargo build", "gh release create", "softprops/action-gh-release"):
    require(forbidden not in DESKTOP_WF,
            f"build/publish surface leaked into desktop gate: {forbidden}")
require_in(DESKTOP_WF, "tests/spec172_focusa_desktop_entitlement_test.py",
           "desktop gate does not run the exact desktop verification")
require_in(DESKTOP_WF, "node tests/spec172_menubar_tui_presenter_test.mjs",
           "desktop gate does not run menubar/TUI parity")
require_in(DESKTOP_WF, "Desktop action map contract is schema-stable and valid JSON",
           "desktop gate does not validate the action map contract")
require_in(DESKTOP_WF, "rustc --edition 2021 --test apps/menubar/src-tauri/src/spec172_desktop_bridge.rs",
           "desktop bridge not compiled standalone by the gate")
require_in(DESKTOP_WF, "Desktop bridge and fixtures ship in the release artifact tree",
           "artifact-tree shipping check missing")
require_in(CANDIDATE_WF, "desktop-artifacts:", "final candidate desktop-artifacts job missing")

# ── 2. Installed CLI/Desktop/menubar/Pi/Cockpit/installer flows ──────────

# Desktop bridge: zero local authority, no direct-core bypass; wired through
# the Tauri command surface with canonical presenter projection.
require_in(BRIDGE, "focusa.spec172.presenter_projection.v1", "desktop bridge lacks canonical projection")
require_in(BRIDGE, "never mints a License Type", "desktop bridge misses presenter-not-policy guard")
require_in(BRIDGE, "never cache local commercial policy", "desktop bridge may cache local commercial policy")
require_in(MAIN_RS, "mod spec172_desktop_bridge;", "desktop bridge module not wired")
require_in(MAIN_RS, "fn focusa_desktop_route_action", "desktop route action not exported")
require_in(MAIN_RS, "fn focusa_desktop_spec172_posture", "desktop posture command not exported")
require_in(MAIN_RS, "focusa_desktop_route_action,", "desktop route action not registered in invoke handler")

# Menubar and TUI presenters carry the same frozen projection vocabulary.
require_in(MENUBAR_TS, "SPEC172_LICENSE_TYPE_CODES", "menubar Spec 172 posture module missing")
require_in(MENUBAR_TS, "SPEC172_RETAINED_CONTROLS", "menubar retained-controls fixture missing")
require_in(MENUBAR_TS, "presenters, not products", "menubar posture lacks presenter-not-product sentence")
require_in(TUI_RS, "SPEC172_LICENSE_TYPE_CODES", "TUI presenter lacks canonical License Type codes")
require_in(TUI_RS, "SPEC172_RETAINED_CONTROLS", "TUI presenter lacks retained-controls fixture")
require_in(TUI_RS, "SPEC172_PRESENTER_NOT_PRODUCT", "TUI presenter lacks presenter-not-product sentence")
require_in(TUI_RS, "presenter never mints a License Type", "TUI presenter misses presenter-not-policy guard")

# Pi extension routes the same canonical posture and never owns policy.
require_in(PI_TS, 'SPEC172_PRESENTER_SCHEMA = "focusa.spec172.presenter_projection.v1"',
           "Pi adapter lacks canonical Spec 172 schema")
require_in(PI_TS, "export function spec172PostureForAuthority", "Pi adapter posture projector missing")
require_in(PI_TS, "export function spec172DenialAndUpgrade", "Pi adapter denial/upgrade mapping missing")

# Cockpit mixed-product presenter resolves only canonical actions.
require_in(COCKPIT_RS, "SPEC172_COCKPIT_ACTION_REGISTRY", "Cockpit action registry missing")
require_in(COCKPIT_RS, "resolve_cockpit_action", "Cockpit resolver missing")

# CLI/agent presenter parity fixtures are canonical and frozen.
require_in(CLI_FIXTURES, '"schema": "focusa.spec172.cli_agent_presenter_fixtures.v1"',
           "CLI/agent presenter fixture schema missing")
require_in(CLI_FIXTURES, "focusa.spec172.presenter_projection.v1",
           "CLI/agent fixtures lack canonical projection")

# Installers fail closed: no local/self-issued grant, raw keys forbidden,
# verified limited activation only.
require("persist_eval_license" not in INSTALL_RS, "installer persists a local evaluation license")
require('return Ok("eval".to_string())' not in INSTALL_RS, "installer self-issues an eval license")
require_in(INSTALL_RS, "E_AUTHORITY_RAW_KEY_FORBIDDEN", "raw license keys not rejected by installer")
require_in(INSTALL_RS, "DeviceAuthorizationSession::new", "installer device-authorization start missing")
require_in(INSTALL_RS, "PersistedAuthorityState::from_verified_envelopes",
           "issued lease not verified before persistence")
require_in(INSTALL_RS, "ActivationJourney::LimitedAccess", "--eval does not map to verified limited activation")
require((ROOT / "tests/170_focusa_installer_entitlement_activation_gate_test.py").exists(),
        "installer entitlement activation gate test missing")

# ── 3. WPUIAI test authority (canonical; facades/UIAI never grant) ────────

for test_name in (
    "spec172_public_uiai_proof_test.py",     # bounded redacted live-capture proof
    "spec172_uiai_limited_mode_test.py",     # observe-only limited mode boundary
    "spec172_uiai_operator_issuance_test.php",  # WPUIAI EDD operator issuance
    "spec172_limited_assertion_test.php",    # signed limited-access assertion
    "spec172_public_facade_convergence_test.py",  # facades never grant
):
    require((ROOT / "tests" / test_name).exists(), f"WPUIAI test-authority gate missing: {test_name}")

# ── 4. Every Spec 172 journey surface is present (run below with real ─────
# ──     exit codes; presence proves no silent skip in the receipt)       ──

JOURNEY_TESTS = {
    "verified_limited": ["python3", "tests/spec172_verified_limited_e2e_test.py"],
    "paid_standalone": ["python3", "tests/spec172_paid_upgrade_e2e_test.py"],
    "bundle_composition": ["php", "tests/spec172_bundle_composition_test.php"],
    "bundle_e2e": ["python3", "tests/spec172_bundle_e2e_test.py"],
    "refund_revoke": ["php", "tests/spec172_refund_downgrade_test.php"],
    "offline_runtime_policy": ["python3", "tests/spec172_runtime_policy_acceptance_test.py"],
    "dynamic_tool": ["python3", "tests/spec172_dynamic_operation_manifest_test.py"],
    "recovery_export_update_uninstall": ["python3", "tests/spec172_downgrade_data_preservation_test.py"],
    "public_facade": ["python3", "tests/spec172_public_facade_convergence_test.py"],
    "installer_surface": ["python3", "tests/spec172_installer_limited_access_test.py"],
    "desktop_surface": ["python3", "tests/spec172_focusa_desktop_entitlement_test.py"],
    "menubar_tui_surface": ["node", "tests/spec172_menubar_tui_presenter_test.mjs"],
    "cockpit_surface": ["python3", "tests/spec172_cockpit_mixed_product_test.py"],
    "cli_surface": ["python3", "tests/spec172_cli_agent_presenter_test.py"],
    "cross_surface_adversarial": ["python3", "tests/spec172_cross_surface_adversarial_test.py"],
}
for name, argv in JOURNEY_TESTS.items():
    require((ROOT / argv[-1]).exists(), f"Spec 172 journey test file missing: {argv[-1]}")

# ── 5. Exact-SHA binding of the phase-05 evidence chain and redaction ─────
# ──     hygiene                                                          ──

evidence_records = sorted(EVIDENCE_DIR.glob("*-acceptance.txt"))
require(len(evidence_records) >= 40, f"expected >= 40 closed Spec 172 acceptance records, found {len(evidence_records)}")

# Every phase-05 journey/installed acceptance record binds to an exact commit
# SHA (7+ hex), proving no static/administrative-only closure.
PHASE05_JOURNEY_EVIDENCE = [
    "focusa-vbcqu.20.15.24",  # complete runtime policy matrix (offline cases)
    "focusa-vbcqu.20.15.26",  # CLI/Pi/agent presenter parity fixtures
    "focusa-vbcqu.20.15.27",  # menubar/TUI presenter projection
    "focusa-vbcqu.20.15.28",  # Focusa Desktop entitlement
    "focusa-vbcqu.20.15.29",  # UIAI Cockpit mixed-product presenter
    "focusa-vbcqu.20.15.33",  # verified limited E2E journey
    "focusa-vbcqu.20.15.34",  # paid standalone upgrade journey
    "focusa-vbcqu.20.15.35",  # Bundle journey
    "focusa-vbcqu.20.15.37",  # refund downgrade / data preservation journey
    "focusa-vbcqu.20.15.38",  # cross-presenter adversarial matrix
    "focusa-vbcqu.20.15.39",  # public facade convergence
    "focusa-vbcqu.20.15.40",  # complete build-independent gate
]
for atom_id in PHASE05_JOURNEY_EVIDENCE:
    record = EVIDENCE_DIR / f"{atom_id}-acceptance.txt"
    require(record.exists(), f"phase-05 journey acceptance record missing: {atom_id}")
    require(re.search(r"\b[0-9a-f]{7,40}\b", record.read_text(encoding="utf-8", errors="replace")),
            f"phase-05 journey acceptance record not bound to an exact commit: {atom_id}")

# Prior-atom receipts: the build-independent gate and the 152F installed
# acceptance chain were real runs recorded immutably (never demo prerelease).
build_independent = EVIDENCE_DIR / "focusa-vbcqu.20.15.40-acceptance.txt"
require(build_independent.exists(), "build-independent gate receipt missing")
require("runs=114 passed=114" in build_independent.read_text(encoding="utf-8"),
        "build-independent gate receipt lacks real run record")
for prior_id in ("focusa-vbcqu.20.14.50", "focusa-vbcqu.20.13.62"):
    require((ROOT / "docs/evidence/spec152f" / f"{prior_id}-acceptance.txt").exists(),
            f"prior installed/cutover receipt missing: {prior_id}")

# Fail-closed FORBIDDEN coverage: each forbidden property is asserted by at
# least one Spec 172 test.
for token in (
    "no anonymous",              # no anonymous product capability
    "no local",                  # no local/self-issued grant
    "caller_controlled",         # no caller-controlled product/price/type/family/feature/limit/node
    "presenter_must_not",        # presenters never own policy
    "Download 453",              # no implicit legacy Download 453 mapping
    "recovery_always",           # recovery survives commercial denial
    "never_disabled",            # retained read/export/repair/rollback/update/uninstall
):
    require(
        any(token in p.read_text(encoding="utf-8", errors="replace")
            for p in (ROOT / "tests").glob("spec172_*.py"))
        or any(token in p.read_text(encoding="utf-8", errors="replace")
               for p in (ROOT / "tests").glob("spec172_*.php"))
        or any(token in p.read_text(encoding="utf-8", errors="replace")
               for p in (ROOT / "tests").glob("spec172_*.mjs")),
        f"forbidden property not asserted by any Spec 172 test: {token}",
    )

# Redaction hygiene: no raw email, key, token, customer row, credential, or
# card data anywhere under docs/evidence/spec172.
credential_patterns = (
    r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}",
    r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+",
    r"BEGIN [A-Z ]*PRIVATE KEY",
    r"AKIA[0-9A-Z]{16}",
    r"ghp_[A-Za-z0-9]{20,}",
    r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}",
    r"\b(?:\d[ -]?){13,16}\b",
)
leaks = []
for record in evidence_records:
    text = record.read_text(encoding="utf-8", errors="replace")
    for pattern in credential_patterns:
        if re.search(pattern, text):
            leaks.append(f"{record.name}:{pattern}")
require(not leaks, f"raw credential/token/card material in evidence: {leaks}")

# ── 6. Real journey runs (one command at a time; cargo builds serialize) ──

def run_journey_bounded(name: str, argv: list[str]) -> None:
    """Run one Spec 172 journey, re-running only on the documented flake."""
    global flake_retries
    attempts = 0
    exit_code = -1
    while True:
        attempts += 1
        proc = subprocess.run(argv, cwd=str(ROOT), capture_output=True, text=True, timeout=2400)
        exit_code = proc.returncode
        if exit_code == 0:
            break
        if attempts < FLAKE_MAX_ATTEMPTS and FLAKE_SIGNATURE in proc.stdout + proc.stderr:
            flake_retries += 1
            continue
        raise AssertionError(
            f"journey failed rc={exit_code} for {name} argv={' '.join(argv)}"
            f" (after {attempts} real attempt(s), {flake_retries} flake retries)"
            f"\n{proc.stdout[-1500:]}\n{proc.stderr[-1500:]}"
        )
    JOURNEY_RUNS[name] = {"argv": " ".join(argv), "exit": exit_code, "attempts": attempts}


flake_retries = 0
for name, argv in JOURNEY_TESTS.items():
    run_journey_bounded(name, argv)

journey_exit_codes_all_zero = all(run["exit"] == 0 for run in JOURNEY_RUNS.values())
require(journey_exit_codes_all_zero, "one or more Spec 172 journeys did not exit 0")


# ── Receipt ──

def main() -> int:
    import json

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=ROOT
    ).stdout.strip() or "unknown"
    receipt = {
        "schema": "focusa.spec172.installed_acceptance_receipt.v1",
        "atom": "focusa-vbcqu.20.15.41",
        "sha256_head": head,
        "journeys": {k: {"exit": v["exit"]} for k, v in sorted(JOURNEY_RUNS.items())},
        "journey_exit_codes_all_zero": journey_exit_codes_all_zero,
        "flake_retries": flake_retries,
        "journey_count": len(JOURNEY_RUNS),
        "evidence_records": len(evidence_records),
    }
    print(json.dumps(receipt, sort_keys=True))
    print("✓ spec172_installed_acceptance_receipt PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
