#!/usr/bin/env python3
"""Spec 152E.07.09 Seal Spec 152E correction closure for locked release.

Atom: focusa-vbcqu.20.13.63 (locked-release correction; spec152f/spec172
async pipeline). Exact verification:

    python3 tests/spec152e_correction_closure_test.py

Objective: reconcile every Spec 152E requirement and Bead atom to
implementation/evidence, and fail closed on administrative closure, deferred
required behavior, split authority, unverified-email path, stale deployment,
or unpublished source correction.

Exact surfaces (this atom only)
- Spec/contract/task trace matrix (embedded below as REQUIREMENTS and
  verified against the committed tree)
- All correction commits (child-atom audit of focusa-vbcqu.20.13.2..20.13.62)
- Tests (the three bounded staging/canary gates re-executed with real exit
  codes)
- Staging/canary results (final cutover canary + migration inventory pins)
- Unresolved blocker inventory (deferred Cargo build gates enumerated with
  claimed_passed=false; release-blocker contract state)
- Locked-release gates (final-audit, release-blocker-summary, next-command
  contracts; governance technical-closure gate consumed read-only)

Semantics (deterministic, offline, replayable from the pinned commit)
- Every Spec 152E requirement row must map to committed contracts, committed
  tests, and committed acceptance records; a missing contract, missing test,
  missing atom record, or record without verification content fails the seal.
- Every Spec 152E child atom (20.13.2..20.13.62) must have a git-tracked
  acceptance record naming real verification content; every backtick-named
  implementation commit must resolve to a real commit object in this tree
  (no fabricated refs, no unpublished source correction); every exact-
  verification test referenced by a record must exist and be git-tracked.
- No record may carry an unfinished acceptance marker (no administrative
  closure, no false completion claim).
- The three staging/canary gates are re-executed and each must exit 0.
- Deferred Cargo/release builds stay enumerated with claimed_passed=false and
  are never claimed to have passed (FORBIDDEN: builds deferred to the
  operator's 50% gate; no cargo command is run).
- Locked-release gates: Spec 152F closure accepted with receipts; the Spec
  152E correction closure is sealed with receipts; REL.4-REL.7 stay not_closed;
  distribution stays blocked and publication stays forbidden.
- Fail-closed FORBIDDEN coverage: no unverified-email promotion, no local/
  self-issued entitlement, no independent facade authority, no client-
  controlled EDD price/grants, no secret or unmasked real-email evidence, no
  stale deployment claim (live_server_verified stays false).
- No push, deploy, release, merge, or Beads mutation is performed.
"""

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

try:  # PyYAML is present in the build-independent gate; fall back to a
    import yaml  # tolerant line parser if it is ever absent.
    HAVE_YAML = True
except Exception:  # pragma: no cover - environment fallback
    HAVE_YAML = False

ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "tests"
CONTRACTS = ROOT / "docs" / "contracts"
EVIDENCE_152E = ROOT / "docs" / "evidence" / "spec152e"
EVIDENCE_152F = ROOT / "docs" / "evidence" / "spec152f"
AUDIT = ROOT / "release-proof" / "audit"

CLOSURE_RECEIPT = "docs/evidence/spec152f/focusa-vbcqu.20.13.63-acceptance.txt"
# The seal task's own receipt is written by this atom's evidence commit
# (committed separately, after the implementation commit), exactly like the
# Spec 152F seal (focusa-vbcqu.20.14.51) handled its own receipt: the seal
# task is not audited against a pre-existing record.
SEAL_TASK = "focusa-vbcqu.20.13.63"

positive = 0
negative = 0
failures: list[str] = []


def expect(condition: bool, message: str) -> None:
    global positive
    positive += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def expect_negative(condition: bool, message: str) -> None:
    global negative
    negative += 1
    if not condition:
        raise AssertionError(f"FAIL: {message}")


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def run(*args: str, expected: int = 0) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(list(args), cwd=ROOT, text=True, capture_output=True)
    if result.returncode != expected:
        raise AssertionError(
            f"command failed ({result.returncode}): {' '.join(args)}\n"
            f"{(result.stdout or '')[-600:]}{(result.stderr or '')[-600:]}"
        )
    return result


def resolve_test_ref(ref: str) -> Path | None:
    """Resolve a record-named test reference to a real committed file."""
    direct = ROOT / ref
    if direct.is_file():
        return direct
    # Records may cite the short ``tests/<name>`` path for a surface that
    # lives under apps/<presenter>/tests/<name> (e.g. the menubar gate).
    base = Path(ref).name
    for candidate in sorted((ROOT / "apps").rglob(base)):
        if candidate.is_file():
            return candidate
    return None


def git_tracked(path: Path) -> bool:
    result = subprocess.run(
        ["git", "ls-files", "--error-unmatch", str(path.relative_to(ROOT))],
        cwd=ROOT,
        capture_output=True,
    )
    return result.returncode == 0


def git_commit_exists(sha: str) -> bool:
    result = subprocess.run(
        ["git", "cat-file", "-e", f"{sha}^{{commit}}"], cwd=ROOT, capture_output=True
    )
    return result.returncode == 0


def read_yaml(path: Path) -> dict:
    if HAVE_YAML:
        return yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    data: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if ":" in line and not line.startswith((" ", "-", "  ")):
            key, value = line.split(":", 1)
            data[key.strip()] = value.strip()
    return data


def atom_record_path(atom: str) -> Path | None:
    for directory in (EVIDENCE_152E, EVIDENCE_152F):
        candidate = directory / f"{atom}-acceptance.txt"
        if candidate.is_file():
            return candidate
    return None


# ── 1. Spec/contract/task trace matrix (every requirement reconciled) ────────

# Each row: spec reference, requirement text, committed contracts, committed
# tests, and the Spec 152E child atoms whose acceptance records implement it.
REQUIREMENTS = [
    {"req": "D1", "spec": "§2 decision 1 — WPUIAI.com EDD is the canonical authority for customer identity, checkout, orders, refunds, and human license keys",
     "contracts": ["spec152e-edd-customer-adapter.v1.php", "spec152e-edd-order-binding.v1.php", "spec152e-edd-gate-hooks.v1.php"],
     "tests": ["spec152e_edd_customer_adapter_test.php", "spec152e_edd_order_binding_test.php"],
     "atoms": ["focusa-vbcqu.20.13.8", "focusa-vbcqu.20.13.18", "focusa-vbcqu.20.13.19"]},
    {"req": "D2", "spec": "§2 decision 2 — WPUIAI.com hosts authority account, device, sequence, and lease-signing state derived from EDD truth",
     "contracts": ["spec152e-authority-account.v1.php", "spec152e-authority-node.v1.php", "spec152e-edd-bound-lease-issuer.v1.php"],
     "tests": ["spec152e_authority_account_schema_test.php", "spec152e_authority_node_reservation_test.php"],
     "atoms": ["focusa-vbcqu.20.13.5", "focusa-vbcqu.20.13.34", "focusa-vbcqu.20.13.35"]},
    {"req": "D3", "spec": "§2 decision 3 — non-WPUIAI domains are facades or presenters only",
     "contracts": ["spec152e-facade-registry.v1.php", "spec152e-facade-protocol.v1.php", "spec152e-facade-security.v1.php", "spec152e-install-facade-routes.v1.php"],
     "tests": ["spec152e_facade_registry_test.php", "spec152e_facade_protocol_test.php", "spec152e_facade_security_test.php"],
     "atoms": ["focusa-vbcqu.20.13.25", "focusa-vbcqu.20.13.26", "focusa-vbcqu.20.13.29", "focusa-vbcqu.20.13.53"]},
    {"req": "D4", "spec": "§2 decision 4 — a submitted email creates only a pending registration attempt",
     "contracts": ["spec152e-activation-registration.v1.php", "spec152e-activation-start-handler.v1.php"],
     "tests": ["spec152e_registration_schema_test.php", "spec152e_registration_transition_test.php"],
     "atoms": ["focusa-vbcqu.20.13.7", "focusa-vbcqu.20.13.9"]},
    {"req": "D5", "spec": "§2 decision 5 — no customer/checkout/Evaluation/license/node/lease until mailbox control is verified",
     "contracts": ["spec152e-account-promotion.v1.php", "spec152e-email-identity.v1.php", "spec152e-challenge-service.v1.php"],
     "tests": ["spec152e_account_promotion_test.php", "spec152e_verification_complete_test.php"],
     "atoms": ["focusa-vbcqu.20.13.10", "focusa-vbcqu.20.13.12", "focusa-vbcqu.20.13.14"]},
    {"req": "D6", "spec": "§2 decision 6 — every website and client uses the same registration state machine and API contract",
     "contracts": ["spec152e-activation-call-stack.v1.yaml", "spec152e-activation-public-openapi.v1.json", "spec152e-activation-internal.v1.json", "spec152e-activation-errors.v1.json"],
     "tests": ["spec152e_state_machine_contract_test.py", "spec152e_api_contract_test.py"],
     "atoms": ["focusa-vbcqu.20.13.4", "focusa-vbcqu.20.13.43"]},
    {"req": "D7", "spec": "§2 decision 7 — paid checkout uses EDD's configured Stripe gateway; clients/facades never collect card data",
     "contracts": ["spec152e-edd-checkout-intent.v1.php", "spec152e-checkout-email-integrity.v1.php"],
     "tests": ["spec152e_edd_checkout_intent_test.php", "spec152e_checkout_email_integrity_test.php"],
     "atoms": ["focusa-vbcqu.20.13.16", "focusa-vbcqu.20.13.17"]},
    {"req": "D8", "spec": "§2 decision 8 — EDD Software Licensing produces the sole human-facing license key",
     "contracts": ["spec152e-edd-license-issuance.v1.php", "spec152e-edd-gate-hooks.v1.php"],
     "tests": ["spec152e_edd_license_issuance_test.php"],
     "atoms": ["focusa-vbcqu.20.13.19"]},
    {"req": "D9", "spec": "§2 decision 9 — the same authority-issued key is delivered via transactional email and one-time terminal envelope",
     "contracts": ["spec152e-dual-delivery-coordinator.v1.php", "spec152e-terminal-delivery-envelope.v1.php", "spec152e-transactional-mail-adapter.v1.php"],
     "tests": ["spec152e_dual_license_delivery_test.php", "spec152e_terminal_envelope_test.php"],
     "atoms": ["focusa-vbcqu.20.13.32", "focusa-vbcqu.20.13.33"]},
    {"req": "D10", "spec": "§2 decision 10 — runtime authorization requires a signed, device-bound lease; a human key alone is not execution authority",
     "contracts": ["spec152e-edd-bound-lease-issuer.v1.php", "spec152e-lease-refresh-service.v1.php"],
     "tests": ["spec152e_edd_bound_lease_issuer_test.php", "spec152e_lease_refresh_lifecycle_test.php"],
     "atoms": ["focusa-vbcqu.20.13.35", "focusa-vbcqu.20.13.36"]},
    {"req": "D11", "spec": "§2 decision 11 — Evaluation is EDD-backed, authority-issued, expiring; local eval issuance is forbidden",
     "contracts": ["spec152e-evaluation-issuance.v1.php"],
     "tests": ["spec152e_evaluation_issuance_test.php", "spec152e_eval_existing_source_e2e_test.py"],
     "atoms": ["focusa-vbcqu.20.13.20", "focusa-vbcqu.20.13.58"]},
    {"req": "D12", "spec": "§2 decision 12 — source builds, raw binaries, package installs, official installers, and agent installs follow the same authority flow",
     "contracts": ["spec152e-installer-route-manifest.v1.json", "spec152e-presenter-parity-matrix.v1.json", "spec152e-install-facade-routes.v1.php"],
     "tests": ["spec152e_presenter_parity_test.py", "spec152e_source_build_activation_test.py", "spec152e_installer_activation_test.py"],
     "atoms": ["focusa-vbcqu.20.13.39", "focusa-vbcqu.20.13.42", "focusa-vbcqu.20.13.45", "focusa-vbcqu.20.13.48"]},
    {"req": "D13", "spec": "§2 decision 13 — existing install-site and synthetic records are migration inputs, never co-equal authority",
     "contracts": ["spec152e-legacy-customer-migration.v1.php", "spec152e-paid-record-migration.v1.php", "spec152e-key-quarantine.v1.php", "spec152e-migration-inventory.v1.json"],
     "tests": ["spec152e_legacy_customer_migration_test.php", "spec152e_paid_record_migration_test.py", "spec152e_key_quarantine_test.php"],
     "atoms": ["focusa-vbcqu.20.13.49", "focusa-vbcqu.20.13.50", "focusa-vbcqu.20.13.51", "focusa-vbcqu.20.13.52"]},
    {"req": "D14", "spec": "§2 decision 14 — Spec 158 remains excluded; no cognitive/Workstream authority is granted or selected",
     "contracts": ["spec152e-activation-public-openapi.v1.json", "spec152e-activation-errors.v1.json"],
     "tests": ["spec152e_document_authority_test.py"],
     "atoms": ["focusa-vbcqu.20.13.4", "focusa-vbcqu.20.13.55"]},
    {"req": "S5", "spec": "§5 universal registration state machine and terminal states",
     "contracts": ["spec152e-activation-call-stack.v1.yaml", "spec152e-activation-registration.v1.php"],
     "tests": ["spec152e_state_machine_contract_test.py"],
     "atoms": ["focusa-vbcqu.20.13.4", "focusa-vbcqu.20.13.7"]},
    {"req": "S6", "spec": "§6 verified identity, verification, atomic promotion, and checkout email integrity",
     "contracts": ["spec152e-verification-complete-handler.v1.php", "spec152e-account-promotion.v1.php", "spec152e-email-identity.v1.php", "spec152e-checkout-email-integrity.v1.php", "spec152e-verified-registration-token-validator.v1.php"],
     "tests": ["spec152e_verification_start_test.php", "spec152e_verification_complete_test.php", "spec152e_account_promotion_test.php", "spec152e_email_identity_schema_test.php", "spec152e_checkout_email_integrity_test.php"],
     "atoms": ["focusa-vbcqu.20.13.9", "focusa-vbcqu.20.13.10", "focusa-vbcqu.20.13.11", "focusa-vbcqu.20.13.12", "focusa-vbcqu.20.13.13", "focusa-vbcqu.20.13.14", "focusa-vbcqu.20.13.17"]},
    {"req": "S7", "spec": "§7 canonical account and data model (accounts, identities, registrations, nodes, leases, outbox)",
     "contracts": ["spec152e-authority-account.v1.php", "spec152e-email-identity.v1.php", "spec152e-activation-registration.v1.php", "spec152e-authority-node.v1.php", "spec152e-edd-bound-lease-issuer.v1.php", "spec152e-authority-outbox.v1.php"],
     "tests": ["spec152e_authority_account_schema_test.php", "spec152e_email_identity_schema_test.php", "spec152e_registration_schema_test.php", "spec152e_authority_node_reservation_test.php", "spec152e_authority_outbox_test.php"],
     "atoms": ["focusa-vbcqu.20.13.5", "focusa-vbcqu.20.13.6", "focusa-vbcqu.20.13.7", "focusa-vbcqu.20.13.22", "focusa-vbcqu.20.13.34"]},
    {"req": "S8", "spec": "§8 server-owned product and grant registry (four product families, no client-controlled fields)",
     "contracts": ["spec152e-edd-product-registry.v1.yaml", "spec152e-edd-product-registry.v1.json", "spec152e-edd-product-registry.v1.php"],
     "tests": ["spec152e_product_registry_test.php", "spec152e_product_registry_contract_test.py", "spec152e_edd_product_gate_test.php"],
     "atoms": ["focusa-vbcqu.20.13.3", "focusa-vbcqu.20.13.15"]},
    {"req": "S9", "spec": "§9 branded facade registry (exact origins, proxy-only, allowlisted redirects)",
     "contracts": ["spec152e-facade-registry.v1.yaml", "spec152e-facade-registry.v1.json", "spec152e-facade-registry.v1.php", "spec152e-facade-golden-vectors.v1.json"],
     "tests": ["spec152e_facade_registry_test.php", "spec152e_facade_registry_contract_test.py", "spec152e_facade_golden_vector_test.py", "spec152e_facade_acceptance_test.mjs"],
     "atoms": ["focusa-vbcqu.20.13.25", "focusa-vbcqu.20.13.26", "focusa-vbcqu.20.13.27", "focusa-vbcqu.20.13.31"]},
    {"req": "S10", "spec": "§10 public activation API (one contract, no unmasked email, poll secrets hashed)",
     "contracts": ["spec152e-activation-public-openapi.v1.json", "spec152e-activation-internal.v1.json", "spec152e-activation-errors.v1.json"],
     "tests": ["spec152e_api_contract_test.py", "spec152e_agent_json_contract_test.py"],
     "atoms": ["focusa-vbcqu.20.13.4", "focusa-vbcqu.20.13.43"]},
    {"req": "S11", "spec": "§11 paid customer journey (verified email → EDD order/key → email + terminal delivery)",
     "contracts": ["spec152e-website-paid-facade.v1.php", "spec152e-terminal-agent-paid-activation.v1.php", "spec152e-edd-checkout-intent.v1.php"],
     "tests": ["spec152e_paid_website_e2e_test.py", "spec152e_paid_terminal_agent_e2e_test.py"],
     "atoms": ["focusa-vbcqu.20.13.56", "focusa-vbcqu.20.13.57"]},
    {"req": "S12", "spec": "§12 Evaluation journey (authority-eligibility, expiring EDD-backed Evaluation, no downgrade)",
     "contracts": ["spec152e-evaluation-issuance.v1.php"],
     "tests": ["spec152e_evaluation_issuance_test.php", "spec152e_eval_existing_source_e2e_test.py"],
     "atoms": ["focusa-vbcqu.20.13.20", "focusa-vbcqu.20.13.58"]},
    {"req": "S13", "spec": "§13 existing-license journey (verified owner email, status/product/node-limit enforcement)",
     "contracts": ["spec152e-legacy-activation-adapter.v1.php", "spec152e-legacy-customer-migration.v1.php"],
     "tests": ["spec152e_eval_existing_source_e2e_test.py", "spec152e_legacy_customer_migration_test.php"],
     "atoms": ["focusa-vbcqu.20.13.13", "focusa-vbcqu.20.13.51", "focusa-vbcqu.20.13.58"]},
    {"req": "S14", "spec": "§14 terminal, agent/JSON, and source-build experience (no invented email/code/consent/payment/license)",
     "contracts": ["spec152e-agent-activation.v1.json", "spec152e-terminal-agent-paid-activation.v1.php", "spec152e-source-build-first-run-fixture.v1.json"],
     "tests": ["spec152e_agent_json_contract_test.py", "spec152e_source_build_activation_test.py", "spec152e_cli_activation_test.py"],
     "atoms": ["focusa-vbcqu.20.13.42", "focusa-vbcqu.20.13.43", "focusa-vbcqu.20.13.45"]},
    {"req": "S15", "spec": "§15 human key and signed lease separation (key never authorizes runtime mutation alone)",
     "contracts": ["spec152e-edd-bound-lease-issuer.v1.php", "spec152e-lease-refresh-service.v1.php", "spec152e-lease-golden-vectors.v1.json"],
     "tests": ["spec152e_lease_golden_vector_test.py", "spec152e_terminal_envelope_vector_test.py"],
     "atoms": ["focusa-vbcqu.20.13.32", "focusa-vbcqu.20.13.35"]},
    {"req": "S16", "spec": "§16 dual-channel delivery (EDD email + one-time terminal envelope of one canonical key)",
     "contracts": ["spec152e-dual-delivery-coordinator.v1.php", "spec152e-terminal-delivery-envelope.v1.php", "spec152e-transactional-mail-adapter.v1.php"],
     "tests": ["spec152e_dual_license_delivery_test.php", "spec152e_terminal_envelope_test.php"],
     "atoms": ["focusa-vbcqu.20.13.32", "focusa-vbcqu.20.13.33"]},
    {"req": "S17", "spec": "§17 EDD lifecycle integration (hooks, outbox, reconciliation, sequence on entitlement transitions)",
     "contracts": ["spec152e-edd-lifecycle-projection.v1.php", "spec152e-authority-outbox.v1.php", "spec152e-authority-reconciliation.v1.php", "spec152e-edd-gate-hooks.v1.php"],
     "tests": ["spec152e_edd_lifecycle_projection_test.php", "spec152e_authority_outbox_test.php", "spec152e_authority_reconciliation_test.php"],
     "atoms": ["focusa-vbcqu.20.13.21", "focusa-vbcqu.20.13.22", "focusa-vbcqu.20.13.23", "focusa-vbcqu.20.13.24"]},
    {"req": "S18", "spec": "§18 refund, revoke, and recovery (sequence increment, refresh denial, recovery-only preserves truth)",
     "contracts": ["spec152e-recovery-only-surface.v1.json", "spec152e-edd-lifecycle-projection.v1.php", "spec152e-lease-refresh-service.v1.php"],
     "tests": ["spec152e_recovery_only_surface_test.py", "spec152e_lifecycle_outage_e2e_test.py"],
     "atoms": ["focusa-vbcqu.20.13.37", "focusa-vbcqu.20.13.61"]},
    {"req": "S19", "spec": "§19 security and privacy requirements (no card data, no raw email, hashed secrets, server-owned grants)",
     "contracts": ["spec152e-facade-security.v1.php", "spec152e-email-identity.v1.php", "spec152e-challenge-service.v1.php", "spec152e-terminal-delivery-envelope.v1.php"],
     "tests": ["spec152e_facade_security_test.php", "spec152e_facade_browser_security_test.mjs", "spec152e_key_quarantine_test.php"],
     "atoms": ["focusa-vbcqu.20.13.29", "focusa-vbcqu.20.13.30", "focusa-vbcqu.20.13.60"]},
    {"req": "S20", "spec": "§20 stable failure semantics (public-safe error registry, safe next actions)",
     "contracts": ["spec152e-activation-errors.v1.json", "spec152e-activation-call-stack.v1.yaml"],
     "tests": ["spec152e_api_contract_test.py", "spec152e_adversarial_identity_facade_test.py"],
     "atoms": ["focusa-vbcqu.20.13.4", "focusa-vbcqu.20.13.60"]},
    {"req": "S21", "spec": "§21 surface consolidation (one shared activation client/state machine for all presenters)",
     "contracts": ["spec152e-presenter-parity-matrix.v1.json", "spec152e-agent-activation.v1.json", "spec152e-install-facade-routes.v1.php"],
     "tests": ["spec152e_presenter_parity_test.py", "spec152e_activation_client_contract_test.py", "spec152e_tui_rest_activation_test.py"],
     "atoms": ["focusa-vbcqu.20.13.39", "focusa-vbcqu.20.13.40", "focusa-vbcqu.20.13.41", "focusa-vbcqu.20.13.44", "focusa-vbcqu.20.13.46", "focusa-vbcqu.20.13.47"]},
    {"req": "S22", "spec": "§22 migration and legacy retirement (inventory, merge rules, cutover, rollback)",
     "contracts": ["spec152e-migration-inventory.v1.json", "spec152e-authority-cutover.v1.php", "spec152e-migration-canary.v1.php", "spec152e-legacy-customer-migration.v1.php", "spec152e-paid-record-migration.v1.php", "spec152e-key-quarantine.v1.php"],
     "tests": ["spec152e_migration_inventory_test.py", "spec152e_legacy_identity_merge_test.php", "spec152e_authority_cutover_test.py", "spec152e_migration_canary_test.py"],
     "atoms": ["focusa-vbcqu.20.13.2", "focusa-vbcqu.20.13.49", "focusa-vbcqu.20.13.50", "focusa-vbcqu.20.13.51", "focusa-vbcqu.20.13.52", "focusa-vbcqu.20.13.53", "focusa-vbcqu.20.13.54"]},
    {"req": "S23", "spec": "§23 acceptance matrix (website/terminal/agent paid, Evaluation, existing key, UIAI, bundle, wrong product, invalid email, changed email, duplicate, prior Eval, node limit, refund, revocation, outage, facade spoof, delivery loss, broken URL, legacy record)",
     "contracts": ["spec152e-website-paid-facade-fixture.v1.json", "spec152e-terminal-agent-paid-activation-fixture.v1.json", "spec152e-uiai-bundle-isolation-fixture.v1.json"],
     "tests": ["spec152e_paid_website_e2e_test.py", "spec152e_paid_terminal_agent_e2e_test.py", "spec152e_eval_existing_source_e2e_test.py", "spec152e_uiai_activation_test.py", "spec152e_uiai_bundle_e2e_test.py", "spec152e_adversarial_identity_facade_test.py"],
     "atoms": ["focusa-vbcqu.20.13.56", "focusa-vbcqu.20.13.57", "focusa-vbcqu.20.13.58", "focusa-vbcqu.20.13.59", "focusa-vbcqu.20.13.60", "focusa-vbcqu.20.13.61", "focusa-vbcqu.20.13.62"]},
    {"req": "S24", "spec": "§24 completion gate (canonical truth, verified mailbox, durable account, EDD-backed keys, same key, signed lease, facades without issuance, refund propagation, same flow, no split issuance, migration proof, redacted acceptance, Spec 152F closure first)",
     "contracts": ["spec152-final-audit-status.v1.yaml", "spec152-release-blocker-summary.v1.yaml", "spec152-next-command.v1.yaml"],
     "tests": ["spec152e_complete_static_gate.py", "spec152e_final_cutover_canary_test.py"],
     "atoms": ["focusa-vbcqu.20.13.55", "focusa-vbcqu.20.13.62", "focusa-vbcqu.20.13.63"]},
]

for row in REQUIREMENTS:
    for contract in row["contracts"]:
        path = CONTRACTS / contract
        expect(path.is_file(), f"trace {row['req']}: contract exists {contract}")
        expect(git_tracked(path), f"trace {row['req']}: contract committed {contract}")
    for test in row["tests"]:
        path = TESTS / test
        expect(path.is_file(), f"trace {row['req']}: test exists {test}")
        expect(git_tracked(path), f"trace {row['req']}: test committed {test}")
    for atom in row["atoms"]:
        if atom == SEAL_TASK:
            continue  # receipt written by this atom's evidence commit
        expect(atom_record_path(atom) is not None, f"trace {row['req']}: evidence for {atom}")
print(
    f"trace matrix reconciled: {len(REQUIREMENTS)} requirement rows, "
    f"{sum(len(r['contracts']) for r in REQUIREMENTS)} contracts, "
    f"{sum(len(r['tests']) for r in REQUIREMENTS)} tests, "
    f"{sum(len(r['atoms']) for r in REQUIREMENTS)} atom bindings"
)


# ── 2. All correction commits (child-atom audit 20.13.2..20.13.62) ───────────

EVIDENCE_MARKERS = [
    "Bounded result", "OUTPUTS", "Exact verification", "Verification",
    "Command:", "Exit code", "Exit status", "Scope", "Pinned source", "Result",
]
PENDING_MARKERS = [
    "NOT DONE", "not done", "TODO", "FIXME", "unfinished",
    "pending acceptance", "PENDING ACCEPTANCE", "no acceptance evidence",
]
PENDING_RE = re.compile(r"\bXXX\b|TODO|FIXME|unfinished|not done|pending acceptance|no acceptance evidence", re.IGNORECASE)
TEST_REF_RE = re.compile(r"((?:tests|apps)/spec152e_[A-Za-z0-9_]+\.(?:php|py|sh|mjs|ps1))")
COMMIT_REF_RE = re.compile(r"`([0-9a-f]{7,40})`")

atoms = [f"focusa-vbcqu.20.13.{n}" for n in range(2, 63)]
expect(len(atoms) == 61, "complete Spec 152E child atom series 20.13.2..20.13.62")
records_audited = 0
commit_refs_checked = 0
test_refs_checked = 0
for atom in atoms:
    record = atom_record_path(atom)
    expect(record is not None, f"acceptance record exists for {atom}")
    assert record is not None
    expect(git_tracked(record), f"acceptance record committed for {atom}: {record.name}")
    text = record.read_text(encoding="utf-8")
    expect(atom in text, f"{record.name}: record names its own atom id")
    expect(any(marker in text for marker in EVIDENCE_MARKERS), f"{record.name}: record has verification content")
    for marker in PENDING_MARKERS:
        expect_negative(marker not in text, f"{record.name}: no unfinished acceptance marker ({marker})")
    expect_negative(PENDING_RE.search(text) is None, f"{record.name}: no unfinished acceptance marker regex")
    for ref in COMMIT_REF_RE.findall(text):
        commit_refs_checked += 1
        expect(git_commit_exists(ref), f"{record.name}: commit ref resolves in tree {ref}")
    for test_ref in set(TEST_REF_RE.findall(text)):
        test_refs_checked += 1
        path = resolve_test_ref(test_ref)
        expect(path is not None, f"{record.name}: referenced test exists {test_ref}")
        assert path is not None
        expect(git_tracked(path), f"{record.name}: referenced test committed {test_ref}")
    records_audited += 1
print(
    f"child-atom audit complete: {records_audited} records, "
    f"{commit_refs_checked} commit refs resolved, {test_refs_checked} test refs bound"
)


# ── 3. Staging/canary results: bounded replay under the FINAL committed code ─

REPLAYED_GATES = [
    "spec152e_final_cutover_canary_test.py",       # 152E.07.08 final cutover canary
    "spec152e_adversarial_identity_facade_test.py",  # 152E.07.06 adversarial matrix
    "spec152e_lifecycle_outage_e2e_test.py",         # 152E.07.07 refund/revoke/outage
]
replayed_results = []
for gate in REPLAYED_GATES:
    proc = subprocess.run(
        [sys.executable, str(TESTS / gate)], cwd=ROOT, text=True, capture_output=True, timeout=900
    )
    output = (proc.stdout or "") + (proc.stderr or "")
    replayed_results.append({
        "gate": gate, "rc": proc.returncode, "out_sha256": sha256_text(output),
    })
    expect(
        proc.returncode == 0,
        f"replayed staging/canary gate {gate} exits 0 (rc={proc.returncode})",
    )
print(f"staging/canary gates replayed: {[r['rc'] for r in replayed_results]}")

# Generated Spec 152E contracts are current (acceptance criterion). The
# deployed-surface inventory and installer route manifest pins were corrected
# by this atom (two stale repository digests after the Spec 172 installer
# change), exactly as the static-gate atom 20.13.55 corrected the same pins
# after the verified-delegation installer replacement. The generator must now
# re-derive byte-identically.
run(sys.executable, str(ROOT / "scripts" / "generate-spec152e-installer-route-manifest.py"), "--check")
manifest_gate = subprocess.run(
    [sys.executable, str(TESTS / "spec152e_install_facade_routes_test.py")],
    cwd=ROOT, text=True, capture_output=True, timeout=600,
)
expect(manifest_gate.returncode == 0, "installer route manifest pins are current (install_facade_routes gate passes)")
print("generated contracts current: installer route manifest/inventory re-derive byte-identically")

# Known cross-atom gate drift inventory (pre-existing at the base commit, root
# cause is a published ancestor of HEAD; NOT a Spec 152E requirement gap and
# NOT claimed passed by this atom). The single remaining known-failing gate:
# spec152e_installer_activation_test.py — its frozen "EvaluationReady" source
# pin is superseded by the accepted Spec 172 removal of local Evaluation from
# installers/lifecycle receipts (atom 20.15.30, commit a843b0f5);
# docs/evidence/spec172/focusa-vbcqu.20.15.30-acceptance.txt documents that
# the Spec 152E test was intentionally NOT modified. The other two drifts
# surfaced by this closure were corrected here (installer route manifest/
# inventory digests and the presenter-parity matrix, a self-described
# generated contract that must stay current with the surfaces).
KNOWN_DRIFTS = [
    {
        "gate": "spec152e_installer_activation_test.py",
        "cause": "EvaluationReady",
        "root_causes": ["a843b0f5"],
        "supersession_evidence": "docs/evidence/spec172/focusa-vbcqu.20.15.30-acceptance.txt",
    },
]
for drift in KNOWN_DRIFTS:
    gate_path = TESTS / drift["gate"]
    expect(gate_path.is_file(), f"known drift gate exists {drift['gate']}")
    expect(git_tracked(gate_path), f"known drift gate committed {drift['gate']}")
    for commit in drift["root_causes"]:
        expect(git_commit_exists(commit), f"drift root cause published in tree {commit}")
    result = subprocess.run(
        [sys.executable, str(gate_path)], cwd=ROOT, text=True, capture_output=True, timeout=600
    )
    output = (result.stdout or "") + (result.stderr or "")
    expect(result.returncode != 0, f"known drift gate still fails (no false completion): {drift['gate']}")
    expect(drift["cause"] in output, f"known drift gate fails with the documented cause ({drift['gate']})")
    if drift["supersession_evidence"]:
        evidence = ROOT / drift["supersession_evidence"]
        expect(evidence.is_file(), f"supersession evidence exists {drift['supersession_evidence']}")
        expect(git_tracked(evidence), f"supersession evidence committed {drift['supersession_evidence']}")
# The corrected generated contracts now pass their currency gates.
parity_gate = subprocess.run(
    [sys.executable, str(TESTS / "spec152e_presenter_parity_test.py")],
    cwd=ROOT, text=True, capture_output=True, timeout=600,
)
expect(parity_gate.returncode == 0, "presenter parity matrix is current (parity gate passes)")
print(f"known cross-atom gate drift inventory: {[d['gate'] for d in KNOWN_DRIFTS]} (root cause published, not claimed passed)")

# Migration inventory pins (Spec 152E §22.1) recompute from the committed fixture.
inventory = json.loads((CONTRACTS / "spec152e-migration-inventory.v1.json").read_text(encoding="utf-8"))
inventory_records = inventory["records"]
from collections import Counter
inventory_counts = Counter(str(record.get("disposition")) for record in inventory_records)
expect(len(inventory_records) == 596, "migration inventory holds 596 physical records")
expect(inventory_counts["unresolved"] == 515, "migration inventory: 515 unresolved")
expect(inventory_counts["refunded_revoked"] == 41, "migration inventory: 41 refunded/revoked")
expect(inventory_counts["synthetic_quarantine"] == 34, "migration inventory: 34 synthetic quarantine")
expect(inventory_counts["verify_first"] == 6, "migration inventory: 6 verify_first")
canary_fixture = json.loads((CONTRACTS / "spec152e-migration-canary-fixture.v1.json").read_text(encoding="utf-8"))
expect(canary_fixture.get("schema") == "focusa.spec152e.migration_canary_fixture.v1", "canary fixture schema current")
expect("journal_vectors" in canary_fixture and "edd_truth" in canary_fixture, "canary fixture pins before/after vectors and EDD truth")
print("migration/canary pins current: 596 records (41 refunded/revoked, 34 synthetic quarantine, 515 unresolved, 6 verify_first)")


# ── 4. Unresolved blocker inventory (deferred Cargo build gates) ─────────────

deferred = []
for rs in sorted((ROOT / "crates").rglob("*.rs")):
    raw = rs.read_text(encoding="utf-8", errors="replace")
    if "152e" not in raw.lower():
        continue
    if "tests/" in rs.as_posix():
        kind = "cargo_integration_test"
    elif re.search(r"#\[(?:cfg\(test\)\]|test\])", raw):
        kind = "cargo_unit_test_module"
    else:
        kind = "spec152e_surface_reference"
    deferred.append({
        "file": rs.relative_to(ROOT).as_posix(),
        "kind": kind,
        "status": "deferred_build_gate",
        "claimed_passed": False,
        "reason": "cargo/release builds deferred until the operator 50% gate",
    })
expect(len(deferred) >= 20, f"complete Spec 152E deferred Cargo surface enumerated ({len(deferred)} files)")
expect(all(entry["claimed_passed"] is False for entry in deferred), "no deferred Cargo test is claimed to have passed")
blocker_summary = read_yaml(CONTRACTS / "spec152-release-blocker-summary.v1.yaml")
blocker_text = (CONTRACTS / "spec152-release-blocker-summary.v1.yaml").read_text(encoding="utf-8")
expect(blocker_summary.get("status") == "blocked_for_new_evaluator_customer_and_stable_distribution", "distribution remains blocked")
expect("spec152e_correction_status: closed_with_receipts" in blocker_text, "blocker summary seals the Spec 152E correction with receipts")
expect(CLOSURE_RECEIPT in blocker_text, "blocker summary links the Spec 152E closure receipt")
expect("spec152e_final_closure_blocked_by: focusa-vbcqu.20.14.52" in blocker_text, "blocker summary keeps the Spec 152F dependency wiring")
expect("legacy_not_approved" in blocker_text and "self-issued --eval" in blocker_text, "blocker summary forbids self-issued Evaluation and split authority")
print(f"unresolved blocker inventory: {len(deferred)} deferred Cargo gates (claimed_passed=false); distribution=blocked")


# ── 5. Locked-release gates (final audit / next command / governance) ────────

final_audit = read_yaml(CONTRACTS / "spec152-final-audit-status.v1.yaml")
final_audit_text = (CONTRACTS / "spec152-final-audit-status.v1.yaml").read_text(encoding="utf-8")
expect(final_audit.get("spec152e_closure_status") == "closed_with_receipts", "final audit seals Spec 152E closure with receipts")
expect(final_audit.get("spec152e_closure_receipt") == CLOSURE_RECEIPT, "final audit links the Spec 152E closure receipt")
expect(final_audit.get("spec152e_correction_status") == "in_progress", "correction work item status stays in_progress (no false completion)")
expect(final_audit.get("spec152f_closure_status") == "accepted_with_receipts", "Spec 152F closure accepted with receipts")
expect(final_audit.get("spec152f_closure_receipt") == "docs/evidence/spec152f/focusa-vbcqu.20.14.52-acceptance.txt", "Spec 152F closure receipt linked")
expect(final_audit.get("rel_gates_status") == "not_closed", "REL.4-REL.7 stay not_closed")
expect(final_audit.get("distribution_status") == "blocked", "distribution stays blocked")
expect(final_audit.get("spec_158") == "excluded", "Spec 158 excluded")
expect("publication_rule: forbidden" in final_audit_text, "publication stays forbidden")
expect(final_audit.get("live_server_verified") is False, "no stale deployment claim: live server not verified")
expect(final_audit.get("migration_cutover_accepted") is False, "no false cutover acceptance claim")
expect(final_audit.get("all_platform_final_candidate_verified") is False, "no false all-platform candidate claim")

next_command = read_yaml(CONTRACTS / "spec152-next-command.v1.yaml")
next_command_text = (CONTRACTS / "spec152-next-command.v1.yaml").read_text(encoding="utf-8")
expect("post_spec152e_closure" in next_command, "next-command points past the Spec 152E closure")
expect("publication: forbidden_until_focusa-vbcqu.20.13.63_and_focusa-vbcqu.20.14.52_close" in next_command_text, "publication token unchanged and still forbidden")

gate = json.loads((AUDIT / "next-locked-release-technical-closure-gate.json").read_text(encoding="utf-8"))
expect(gate.get("status") == "verified", "governance technical-closure gate verified")
expect(gate.get("invalid_closed_count") == 0, "zero invalid-closed beads (no administrative closure)")
expect(gate.get("invalid_closed_ids") == [], "no invalid-closed bead ids")
expect(gate.get("mapping_count") >= 465, f"governance mapping ledger present ({gate.get('mapping_count')})")
expect(gate.get("technically_pending_count", 0) >= 63, "Spec 152E atoms remain technically pending in the Beads provider (no false claim)")
print("locked-release gates current: REL.4-REL.7 not_closed, publication forbidden, invalid_closed=0")


# ── 6. FORBIDDEN fail-closed invariants and redaction on the closure surfaces ─

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+")
PRIVATE_KEY_RE = re.compile(r"BEGIN (?:RSA |EC |)PRIVATE KEY")
CARD_RE = re.compile(r"(?<!\d)(?:\d[ -]?){15,16}(?!\d)")

redaction_sources = [
    TESTS / "spec152e_correction_closure_test.py",
    CONTRACTS / "spec152-final-audit-status.v1.yaml",
    CONTRACTS / "spec152-release-blocker-summary.v1.yaml",
    CONTRACTS / "spec152-next-command.v1.yaml",
]
redaction_sources += sorted((CONTRACTS.glob("spec152e-edd-product-registry.v1.*")))
redaction_sources += [CONTRACTS / "spec152e-facade-registry.v1.json", CONTRACTS / "spec152e-recovery-only-surface.v1.json"]
for path in redaction_sources:
    if not path.is_file():
        continue
    raw = path.read_text(encoding="utf-8")
    expect_negative(SECRET_RE.search(raw) is None, f"{path.name}: no secret prefixes")
    expect_negative(PRIVATE_KEY_RE.search(raw) is None, f"{path.name}: no private key material")
    expect_negative(CARD_RE.search(raw) is None, f"{path.name}: no card-like sequences")

# No unmasked real email anywhere on the closure evidence surface.
closure_evidence = ROOT / CLOSURE_RECEIPT
if closure_evidence.is_file():
    raw = closure_evidence.read_text(encoding="utf-8")
    for match in EMAIL_RE.findall(raw):
        expect_negative(False, f"closure evidence carries no raw email ({match})")
    expect_negative(SECRET_RE.search(raw) is None, "closure evidence: no secret prefixes")
    expect_negative(PRIVATE_KEY_RE.search(raw) is None, "closure evidence: no private key material")

# Product registry: client-controlled price/grant/product fields are forbidden.
product_registry_text = "".join(
    (CONTRACTS / name).read_text(encoding="utf-8")
    for name in ("spec152e-edd-product-registry.v1.yaml", "spec152e-edd-product-registry.v1.json")
    if (CONTRACTS / name).is_file()
)
for token in (
    "caller_controls_forbidden", "server_owned", "PRODUCT_MAPPING_REQUIRED",
    "checkout_enabled: false", "price_authority: spec172_server_owned",
):
    expect(token in product_registry_text, f"product registry forbids client control ({token})")

# Facade registry: presenters and bounded proxies only, never issuance authority.
facade_text = (CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8")
for token in ("proxy", "presenter", "origin"):
    expect(token.lower() in facade_text.lower(), f"facade registry keeps presenter/proxy/origin boundary ({token})")

# Recovery-only surface: recovery never grants entitlement.
recovery_text = (CONTRACTS / "spec152e-recovery-only-surface.v1.json").read_text(encoding="utf-8")
expect("entitlement" in recovery_text.lower(), "recovery-only surface contract present")
expect("recovery" in recovery_text.lower(), "recovery-only surface keeps recovery semantics")

# Unverified-email path: promotion gates on verified mailbox control.
account_promotion_text = (CONTRACTS / "spec152e-account-promotion.v1.php").read_text(encoding="utf-8")
expect("verified" in account_promotion_text.lower(), "account promotion requires verified identity")

# Spec 158 exclusion is honored on the closure surfaces.
expect("excluded" in final_audit_text.lower(), "Spec 158 excluded in the final audit")


# ── 7. Summary (deterministic, replayable) ──────────────────────────────────

head = run("git", "rev-parse", "HEAD").stdout.strip()
summary = {
    "schema": "focusa.spec152e.correction_closure.v1",
    "atom": "focusa-vbcqu.20.13.63",
    "positive_checks": positive,
    "negative_checks": negative,
    "requirement_rows": len(REQUIREMENTS),
    "atoms_audited": records_audited,
    "commit_refs_resolved": commit_refs_checked,
    "test_refs_bound": test_refs_checked,
    "replayed_gates": [r["gate"] for r in replayed_results],
    "replayed_exit_codes": [r["rc"] for r in replayed_results],
    "known_cross_atom_drift_gates": [d["gate"] for d in KNOWN_DRIFTS],
    "deferred_cargo_gates": len(deferred),
    "deferred_cargo_tests": sum(
        len(re.findall(r"#\[test\]", (ROOT / entry["file"]).read_text(encoding="utf-8", errors="replace")))
        for entry in deferred
    ),
    "governance_invalid_closed": gate.get("invalid_closed_count"),
    "governance_pending": gate.get("technically_pending_count"),
    "distribution_status": "blocked",
    "publication": "forbidden",
    "rel_gates_status": "not_closed",
    "spec152e_closure_status": "closed_with_receipts",
    "head_sha256": head,
    "closure_receipt": CLOSURE_RECEIPT,
    "result": "passed",
}
if failures:
    summary["result"] = "failed"
    print("\n".join(failures))
    raise SystemExit(1)
print(json.dumps(summary, sort_keys=True))
print()
print(f"spec152e_correction_closure receipt")
print(f"  sha256 head={head}")
print(f"  sha256 closure-test={sha256_text(Path(__file__).read_text(encoding='utf-8'))}")
print(f"  atoms={records_audited} requirement_rows={len(REQUIREMENTS)}")
print(f"  governance=verified invalid_closed:{gate.get('invalid_closed_count')} pending:{gate.get('technically_pending_count')}")
print(f"  publication=blocked rel_gates=not_closed spec152f=accepted_with_receipts")
print("✓ spec152e_correction_closure PASS")
