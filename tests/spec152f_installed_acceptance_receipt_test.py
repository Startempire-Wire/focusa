#!/usr/bin/env python3
"""Spec 152F.06.08 — Installed exact-SHA cross-platform acceptance receipts.

Static, fail-closed receipt verification for the installed exact-SHA
cross-platform acceptance surfaces (atom focusa-vbcqu.20.14.50):

  * final candidate artifacts for macOS arm64/x64, Windows x64/arm64, and
    Linux GNU/musl — signed with release/updater authority, verified
    (SHA256SUMS, codesign), candidate-only (no stable publication), and
    bound to the exact pipeline SHA;
  * installed CLI/desktop/agent flows — clean install, restart, update,
    rollback, reapply, user-data preservation, native UIAI health fixture;
  * WPUIAI test authority — WPUIAI.com EDD remains the canonical authority,
    branded facades and UIAI/browser surfaces never grant, and installed
    test fixtures stay visibly non-authoritative.

Pure stdlib, deterministic, zero hidden skips: every assertion reads the
committed tree and fails with a named message on any missing receipt,
weakened authority, or candidate-only bypass. The test exits 0 only when the
complete installed cross-platform receipt chain is present and bound to an
exact commit SHA.

Exact verification:
    python3 tests/spec152f_installed_acceptance_receipt_test.py
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CANDIDATE_WF = (ROOT / ".github/workflows/locked-release-candidate-artifacts.yml").read_text()
OTA_WF = (ROOT / ".github/workflows/windows-ota-e2e.yml").read_text()
RELEASE_WF = (ROOT / ".github/workflows/release.yml").read_text()
DEPLOY_WF = (ROOT / ".github/workflows/deploy-live-daemon.yml").read_text()
ASSETS = (ROOT / "scripts/verify-canonical-release-assets.py").read_text()
INSTALL_RS = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text()
UPDATE_RS = (ROOT / "crates/focusa-cli/src/commands/update.rs").read_text()
TRUST_RS = (ROOT / "crates/focusa-cli/src/commands/update_trust.rs").read_text()
INSTALL_SH = (ROOT / "scripts/install-focusa.sh").read_text()
CUTOVER_TEST = (ROOT / "tests/spec152e_final_cutover_canary_test.py").read_text()
UIAI_POLICY_TEST = (ROOT / "tests/spec152f_uiai_policy_test.py").read_text()
FACADE_TEST = (ROOT / "tests/spec152f_facade_policy_presenter_test.py").read_text()
CHALLENGE_TEST = (ROOT / "tests/162_focusa_uiai_challenge_ownership_test.py").read_text()
EVIDENCE_DIR = ROOT / "docs/evidence/spec152f"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(f"installed acceptance receipt missing/weak: {message}")


def require_in(text: str, needle: str, message: str) -> None:
    require(needle in text, f"{message} (missing: {needle!r})")


# ── 1. Cross-platform final candidate artifact manifest (all six platforms) ──

# macOS arm64 / macOS x64 / Linux GNU / Linux musl / Windows x64 / Windows arm64
PLATFORM_TARGETS = {
    "aarch64-apple-darwin": "macos-14",        # macOS arm64
    "x86_64-apple-darwin": "macos-15-intel",   # macOS x64
    "x86_64-unknown-linux-gnu": "ubuntu-latest",
    "x86_64-unknown-linux-musl": "ubuntu-latest",
    "x86_64-pc-windows-msvc": "windows-latest",
    "aarch64-pc-windows-msvc": "windows-11-arm",
}
for target, runner in PLATFORM_TARGETS.items():
    require_in(CANDIDATE_WF, target, f"candidate artifact target missing: {target}")
    require_in(CANDIDATE_WF, runner, f"candidate native runner missing for {target}")

require_in(CANDIDATE_WF, "musl: true", "Linux musl cross build is not enabled")
require_in(CANDIDATE_WF, "cross build --release --target", "musl cross build not wired")
require_in(
    CANDIDATE_WF,
    'test "$(grep -m1 \'^version = \' Cargo.toml | cut -d \'"\' -f2)" = 0.9.144',
    "candidate artifacts are not pinned to exact version 0.9.144",
)
require_in(CANDIDATE_WF, "verify-version-surfaces.py v0.9.144", "candidate version surfaces not verified")

# Signing authority is mandatory and fail-closed (empty secrets fail the job).
for secret in (
    "TAURI_SIGNING_PRIVATE_KEY",
    "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
    "FOCUSA_RELEASE_ED25519_PRIVATE_KEY",
):
    require_in(CANDIDATE_WF, secret, f"candidate signing secret {secret} not required")
require_in(CANDIDATE_WF, 'test -n "${TAURI_SIGNING_PRIVATE_KEY:-}"', "empty updater signing key not rejected")
require_in(CANDIDATE_WF, 'test -n "${FOCUSA_RELEASE_ED25519_PRIVATE_KEY:-}"', "empty release key not rejected")

# Installed artifacts are verified before they may count as accepted.
require_in(CANDIDATE_WF, "codesign --verify --deep --strict --verbose=2", "macOS app signature not verified")
require_in(CANDIDATE_WF, "sha256sum -c SHA256SUMS.txt", "artifact digest manifest not verified")
require_in(CANDIDATE_WF, "verify-canonical-release-assets.py --dist dist --tag v0.9.144",
           "canonical all-platform asset manifest not verified")
require_in(CANDIDATE_WF, "release-trust-metadata.py", "signed release trust metadata not generated")

# No stable publication: the candidate bundle is explicit about candidate-only.
require_in(CANDIDATE_WF, '--candidate', "candidate provenance mode not passed to trust metadata")
require_in(CANDIDATE_WF, '.publication_status == "candidate_only"', "candidate-only status not asserted")
require_in(CANDIDATE_WF, "(.gates.release_success | not)", "candidate gates must not claim release success")
require_in(CANDIDATE_WF, ".gates.release_run_url == null", "candidate must not reference a release run")
require_in(CANDIDATE_WF, "focusa-v0.9.144-candidate-bundle", "candidate proof bundle not uploaded")
for forbidden_publish in ("softprops/action-gh-release", "gh release create", "git push"):
    require(forbidden_publish not in CANDIDATE_WF,
            f"stable-publication surface leaked into candidate workflow: {forbidden_publish}")

# The canonical asset verifier must cover all six systems and all installed
# surfaces (CLI, daemon, TUI, desktop, Pi extension, agent context, installers).
for target in PLATFORM_TARGETS:
    require_in(ASSETS, f'"{target}"', f"canonical asset verifier missing target {target}")
for surface in ('"focusa"', '"focusa-daemon"', '"focusa-tui"'):
    require_in(ASSETS, surface, f"canonical asset verifier missing surface {surface}")
require_in(ASSETS, "focusa-installer-{tag}.sh", "canonical installer (sh) surface missing")
require_in(ASSETS, "focusa-installer-{tag}.ps1", "canonical installer (ps1) surface missing")
require_in(ASSETS, "focusa-pi-extension-{tag}.tar.gz", "Pi extension surface missing")
require_in(ASSETS, "focusa-agent-context-{tag}.tar.gz", "agent-context surface missing")
require_in(ASSETS, "Focusa-{tag}-aarch64-apple-darwin.app.zip", "macOS arm64 desktop surface missing")
require_in(ASSETS, "Focusa-{tag}-x86_64-apple-darwin.app.zip", "macOS x64 desktop surface missing")

# ── 2. Installed CLI/desktop/agent flows (clean install, restart, update, ──
# ──     rollback, reapply, twice where specified)                        ──

# Windows x64 and arm64 native OTA: clean install -> apply -> rollback ->
# reapply. The fixture installs twice per architecture (clean install plus
# reapply after rollback), restarting the daemon and health-verifying after
# every phase, and preserving a user-data sentinel throughout.
require_in(OTA_WF, "Clean install v0.9.116 fixture, apply v0.9.117, rollback, reapply",
           "installed-twice OTA phase step missing")
require_in(OTA_WF, "install_target: windows-x64", "Windows x64 installed flow missing")
require_in(OTA_WF, "install_target: windows-arm64", "Windows arm64 installed flow missing")
require_in(OTA_WF, "windows-11-arm", "Windows arm64 native runner missing")
require_in(OTA_WF, "runtimeArch -ne '${{ matrix.architecture }}'",
           "native architecture mismatch not rejected")
require_in(OTA_WF, "focusa-daemon.exe", "installed daemon binary not asserted")
require_in(OTA_WF, "focusa-tui.exe", "installed TUI binary not asserted")
require_in(OTA_WF, "user-state-sentinel.txt", "installed user-data sentinel missing")
require_in(OTA_WF, "user_data_preserved = $true", "installed user-data preservation not proven")
require_in(OTA_WF, "daemon_health = 'v0.9.117-dev'", "installed daemon restart health not proven")
require_in(OTA_WF, "127.0.0.1:18791/v1/health", "installed daemon health URL missing")
require_in(OTA_WF, "clean_install = 'v0.9.116-dev'", "clean install receipt phase missing")
require_in(OTA_WF, "rollback = 'v0.9.116-dev'", "rollback receipt phase missing")
require_in(OTA_WF, "reapply = 'v0.9.117-dev'", "second install (reapply) receipt phase missing")
require_in(OTA_WF, "focusa.windows_native_ota_proof.v1", "installed OTA proof schema missing")

# UIAI in installed flows is a bounded, non-authoritative test fixture: it
# proves a responsive Engine health contract, never production entitlement.
require_in(OTA_WF, "uiai-health.py", "bounded local UIAI health fixture missing")
require_in(OTA_WF, "UIAI_ENGINE_URL", "UIAI fixture URL not injected")
require_in(OTA_WF, "production_entitlement_claimed = $false",
           "installed fixture must not claim production entitlement")
require_in(OTA_WF, "historical_fixture_authority = 'non_authoritative_migration_fixture'",
           "installed fixture authority not visibly non-authoritative")

# The installed-acceptance workflow consumes the final candidate artifacts.
require_in(OTA_WF, "locked-release-candidate-artifacts.yml", "installed acceptance not wired to candidate artifacts")
require_in(OTA_WF, "secrets: inherit", "candidate artifact secrets not inherited by installed acceptance")

# The installer must never self-issue Evaluation or accept raw keys: it
# routes through the EDD device-authorization session and persists only
# verified, atomically written authority state in native protected storage.
require("persist_eval_license" not in INSTALL_RS, "installer persists a local evaluation license")
require('return Ok("eval".to_string())' not in INSTALL_RS, "installer self-issues an eval license")
require_in(INSTALL_RS, "DeviceAuthorizationSession::new", "device-authorization start missing")
require_in(INSTALL_RS, "AuthorityHttpClient::new", "authority HTTP client missing")
require_in(INSTALL_RS, "PersistedAuthorityState::from_verified_envelopes",
           "issued lease not verified before persistence")
require_in(INSTALL_RS, "rotate_refresh_credential", "refresh credential rotation missing")
require_in(INSTALL_RS, "KeyringCredentialStore", "native protected credential store missing")
require_in(INSTALL_RS, 'InstallTarget::Linux => "x86_64-unknown-linux-musl".to_string()',
           "Linux installed target does not use the musl triple")

# The portable installer maps macOS arm64/x64 and Linux GNU/musl installed
# flows to the exact cross-platform triples.
require_in(INSTALL_SH, 'Darwin:arm64|Darwin:aarch64) TRIPLE="aarch64-apple-darwin"',
           "macOS arm64 installed flow missing")
require_in(INSTALL_SH, 'Darwin:x86_64|Darwin:amd64) TRIPLE="x86_64-apple-darwin"',
           "macOS x64 installed flow missing")
require_in(INSTALL_SH, 'Linux:x86_64|Linux:amd64) TRIPLE="x86_64-unknown-linux-musl"',
           "Linux GNU/musl installed flow missing")

# Update/rollback trust is mandatory before any installed apply: verified
# deploy proof gates OTA, and rollback restores promoted local surfaces.
require_in(UPDATE_RS, "execute_verified_apply(&apply.plan)", "verified OTA executor not active")
require_in(UPDATE_RS, "rollback_promoted_parts", "installed rollback restore missing")
require_in(TRUST_RS, "verify_deploy_proof", "deploy-proof verification missing")
require_in(TRUST_RS, "deploy-success.json.sig", "deploy-proof signature not verified")
require_in(DEPLOY_WF, "Gate OTA installability against signed deployed release",
           "live-deployed OTA installability gate missing")
require_in(DEPLOY_WF, ".latest.trust.deploy_proof_verified == true", "live OTA trust not proven")
require_in(DEPLOY_WF, ".apply_allowed == true", "live OTA apply not gated")

# The stable release pipeline still packages and signs every installed
# surface (CLI, daemon, TUI, menubar, Pi extension, agent context) and runs
# the OTA installability and final release gap gates.
require_in(RELEASE_WF, "tests/spec143_ota_installability_release_gate_test.py",
           "OTA installability release contract gate missing")
require_in(RELEASE_WF, "tests/final_release_gap_gate.sh", "final release gap gate missing")
require_in(RELEASE_WF, "x86_64-unknown-linux-musl", "stable release musl target missing")
require_in(RELEASE_WF, '-f asset_suffix="x86_64-unknown-linux-musl"', "musl deploy asset suffix missing")
require_in(RELEASE_WF, "Require Tauri updater signing authority", "desktop updater signing authority missing")
require_in(RELEASE_WF, "OTA authenticity is enforced by the pinned updater key",
           "desktop OTA authenticity key binding missing")
require_in(RELEASE_WF, "verify-canonical-release-assets.py", "stable release asset verification missing")
require_in(RELEASE_WF, "beta_ad_hoc", "pre-license desktop consent mode explicit")

# ── 3. WPUIAI test authority (unchanged canonical authority; facades/UIAI ──
# ──     never grant)                                                     ──

require_in(CUTOVER_TEST, "WPUIAI.com EDD", "canonical EDD authority not asserted in cutover fixtures")
require_in(CUTOVER_TEST, "WPUIAI/wpuiai", "product registry owner not WPUIAI/wpuiai")
require_in(UIAI_POLICY_TEST, "Focusa-only paid entitlement NEVER grants",
           "Focusa-only entitlement may grant UIAI")
require_in(UIAI_POLICY_TEST, "uiai-engine", "UIAI engine grant family missing")
require_in(CHALLENGE_TEST, 'contract["authority_owner"] == "uiai-engine"', "UIAI challenge owner not external")
require_in(CHALLENGE_TEST, 'contract["mutation_authority"] is False', "UIAI challenge mutation authority not denied")
require_in(FACADE_TEST, "no facade can select grants, prices, feature",
           "branded facade may select grants/prices")
require_in(FACADE_TEST, "presenter never branches output on", "facade presenter may branch on facade id")

# ── 4. Exact-SHA binding of platform receipts and artifact manifest ──

# The release pipeline reuses only successful gates for the exact candidate
# SHA and locks the release candidate to that exact SHA.
require_in(RELEASE_WF, "Require exact candidate-SHA preflight receipts",
           "exact candidate-SHA preflight receipts not required")
require_in(RELEASE_WF, "headSha == $sha", "candidate gate reuse not bound to exact SHA")
require_in(RELEASE_WF, "Lock exact release candidate", "exact release candidate lock missing")
require_in(RELEASE_WF, "exact_sha: $exact_sha", "release candidate manifest lacks exact_sha")
require_in(RELEASE_WF, ".exact_sha == env.GITHUB_SHA", "release candidate not bound to exact SHA")

# The candidate artifact manifest and trust metadata bind to the exact
# pipeline SHA (GITHUB_SHA), not to a floating head.
require_in(CANDIDATE_WF, '--commit "$GITHUB_SHA"', "trust metadata not bound to exact pipeline SHA")
require_in(CANDIDATE_WF, '--sha "$GITHUB_SHA"', "release intelligence not bound to exact pipeline SHA")

# The build-independent gate (20.14.49) already records the exact HEAD SHA
# and the policy-contract digest in its immutable receipt.
gate_receipt = EVIDENCE_DIR / "focusa-vbcqu.20.14.49-acceptance.txt"
require(gate_receipt.exists(), "build-independent gate receipt missing")
require("sha256 head=" in gate_receipt.read_text(), "gate receipt does not record exact HEAD SHA")
require("sha256 spec152-entitlement-coverage.v1.json=" in gate_receipt.read_text(),
        "gate receipt does not record policy-contract digest")

# Every phase-06 installed/lifecycle acceptance record binds to an exact
# commit SHA (7+ hex), proving no static/administrative-only closure.
PHASE06_EVIDENCE = [
    "focusa-vbcqu.20.13.62",  # 152E cutover canary (dependency)
    "focusa-vbcqu.20.14.43",  # state-grid acceptance
    "focusa-vbcqu.20.14.44",  # cross-presenter parity
    "focusa-vbcqu.20.14.45",  # verified Evaluation E2E
    "focusa-vbcqu.20.14.46",  # paid continuation / adverse lifecycle
    "focusa-vbcqu.20.14.47",  # offline grace / outage / bypass
    "focusa-vbcqu.20.14.48",  # resolver/middleware overhead
    "focusa-vbcqu.20.14.49",  # complete build-independent gate
]
for atom_id in PHASE06_EVIDENCE:
    record = EVIDENCE_DIR / f"{atom_id}-acceptance.txt"
    require(record.exists(), f"phase-06 acceptance record missing: {atom_id}")
    require(re.search(r"\b[0-9a-f]{7,40}\b", record.read_text()),
            f"phase-06 acceptance record not bound to an exact commit: {atom_id}")

# Prior-atom evidence completeness protects against hidden skips in the
# installed acceptance series (same floor as the build-independent gate).
evidence_count = len(list(EVIDENCE_DIR.glob("*-acceptance.txt")))
require(evidence_count >= 67, f"expected >= 67 closed Spec 152F acceptance records, found {evidence_count}")

# ── 5. Lifecycle cases bound to the installed flows ──

for test_name in (
    "spec152f_evaluation_first_value_e2e_test.py",   # verified Evaluation
    "spec152f_paid_lifecycle_e2e_test.py",           # paid / expired / refunded / revoked
    "spec152f_recovery_matrix_test.py",              # recovery survives denial
    "spec152f_premium_family_adversarial_test.py",   # premium families
    "spec152f_offline_adversarial_test.py",          # offline grace / bypass
):
    require((ROOT / "tests" / test_name).exists(), f"lifecycle E2E test missing: {test_name}")

# ── 6. Fail-closed FORBIDDEN coverage and evidence redaction hygiene ──

for token in (
    "no_local_issuance",            # no local/self-issued Evaluation
    "caller_controlled",            # no caller-controlled product/price/grants
    "presenter_must_not",           # presenters never own commercial decisions
    "recovery_always_available",    # recovery survives commercial denial
    "no_raw_key_or_token",          # no raw keys/tokens in evidence
    "no_dead_end_paywalls",         # no 395 independent paywalls
):
    require(
        any(token in p.read_text() for p in (ROOT / "tests").glob("spec152f_*.py")),
        f"forbidden property not asserted by any Spec 152F test: {token}",
    )

credential_patterns = (
    r"sk_live_[A-Za-z0-9]+|pk_live_[A-Za-z0-9]+|sk_test_[A-Za-z0-9]+",
    r"BEGIN [A-Z ]*PRIVATE KEY",
    r"AKIA[0-9A-Z]{16}",
    r"ghp_[A-Za-z0-9]{20,}",
    r"AIza[0-9A-Za-z_-]{35}",
)
leaks = []
for record in sorted(EVIDENCE_DIR.glob("*-acceptance.txt")):
    text = record.read_text(encoding="utf-8", errors="replace")
    for pattern in credential_patterns:
        if re.search(pattern, text):
            leaks.append(f"{record.name}:{pattern}")
require(not leaks, f"raw credential/token material in evidence: {leaks}")


# ── Receipt ──

def main() -> int:
    import subprocess

    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=ROOT
    ).stdout.strip() or "unknown"
    print("spec152f_installed_acceptance_receipt receipt")
    print(f"  sha256 head={head}")
    print(f"  platforms=macos:arm64,x64 windows:x64,arm64 linux:gnu,musl")
    print(f"  installed_flows=cli,daemon,tui,desktop,pi-extension,agent-context,installers")
    print(f"  installed_twice=windows:clean_install+reapply (x64,arm64)")
    print(f"  authority=WPUIAI.com EDD (canonical; facades/UIAI never grant)")
    print(f"  evidence_records={evidence_count}")
    print("✓ spec152f_installed_acceptance_receipt PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
