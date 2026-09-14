#!/usr/bin/env python3
"""Spec 152E.05.04 Rust installer activation transcript contract.

Binds the Rust installer activation surface (crates/focusa-cli/src/commands/
install.rs `authorize_installer_activation_flow` + the shared flow in
activation_flow.rs) and the install lifecycle models/orchestrator/receipts
(crates/focusa-core/src/install_lifecycle/) to the frozen Spec 152E
contracts: an interactive terminal renders the universal email → verify →
offer → checkout/poll → key/lease flow through the shared ActivationSession;
noninteractive installs keep device-code authorization; `--eval` maps to the
Spec 172 limited-access overlay (Evaluation is authority-issued only); the
lifecycle orchestrator and receipts grant product execution only from a
verified signed entitlement decision. Card data is never accepted and
nothing is self-issued.

The deterministic transcript replay executes in the same commit (the shared
flow's Rust unit tests); this test binds the installer wiring, the
lifecycle surfaces, and the frozen machine so evidence is replayable from
the pinned commit without any network.

Exact verification: python3 tests/spec152e_installer_activation_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
CLI = ROOT / "crates/focusa-cli/src/commands"
INSTALL = (CLI / "install.rs").read_text(encoding="utf-8")
FLOW = (CLI / "activation_flow.rs").read_text(encoding="utf-8")
LIFECYCLE = ROOT / "crates/focusa-core/src/install_lifecycle"
MODELS = (LIFECYCLE / "models.rs").read_text(encoding="utf-8")
ORCHESTRATOR = (LIFECYCLE / "orchestrator.rs").read_text(encoding="utf-8")
RECEIPTS = (LIFECYCLE / "receipts.rs").read_text(encoding="utf-8")
FIXTURE = json.loads(
    (ROOT / "crates/focusa-license/tests/fixtures/spec152e-activation-transcript-fixtures.v1.json")
    .read_text(encoding="utf-8")
)
INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


# ── Installer renders the universal flow through the shared client ────────

expect("authorize_installer_activation_flow" in INSTALL, "installer activation flow exists")
expect("run_activation_flow(" in INSTALL, "installer drives the shared flow")
expect("INSTALLER_FLOW" in INSTALL, "installer uses the installer presenter identity")
expect("interactive_available()" in INSTALL, "installer checks for an interactive terminal")
expect('"installer"' in FLOW and '"official_installer"' in FLOW,
       "installer presenter identity is typed in the shared flow")
expect("activation_flow::interactive_available()" in INSTALL
       or "interactive_available()" in INSTALL,
       "phase_license routes through the shared flow when interactive")
expect("acquire_installer_entitlement(&config_dir, &required_feature, args.json).await?" in INSTALL,
       "noninteractive device-code authorization remains the fallback")
expect("E_AUTHORITY_ACTIVATION_UNSETTLED" in INSTALL,
       "unsettled activation fails closed without a lease")
expect("E_AUTHORITY_INTERACTIVE_REQUIRED" in INSTALL,
       "noninteractive interactive-flow attempt fails closed")
expect("ActivationHttpClient::new" in INSTALL, "installer wires the activation HTTP transport")
expect("FOCUSA_AUTHORITY_ORIGIN" in INSTALL, "authority origin is configurable, never embedded")

# ── --eval intent maps to the Spec 172 limited-access overlay ─────────────

expect("args.eval" in INSTALL, "installer accepts Evaluation intent")
expect("ActivationJourney::LimitedAccess" in INSTALL,
       "Evaluation intent maps to limited access, never local issuance")

# ── Fail-closed installer guarantees (unchanged pins) ─────────────────────

expect("E_AUTHORITY_RAW_KEY_FORBIDDEN" in INSTALL, "raw license keys are still forbidden")
expect("E_AUTHORITY_PAID_MIGRATION_REQUIRED" in INSTALL, "paid legacy migration is still required")
expect("E_AUTHORITY_LEASE_UNUSABLE" in INSTALL, "unusable leases still fail closed")
expect("persist_eval_license" not in INSTALL, "installer never persists a local Evaluation", negative=True)
expect("DeviceAuthorizationSession::new" in INSTALL, "device-code session remains for noninteractive paths")
expect("PersistedAuthorityState::from_verified_envelopes" in INSTALL
       or "persist_delivered_lease" in FLOW,
       "delivered leases verify before persistence")

# ── Lifecycle models: entitlement binding is signed-authority-only ────────

expect("signature_verified" in MODELS, "binding carries a signature-verified flag")
expect("sha256:" in MODELS and "len() != 71" in MODELS,
       "lease digests must be sha256: with exact length")
expect("signature_verified" in MODELS and "EntitlementBindingIncomplete" in MODELS,
       "incomplete bindings fail closed")
expect("allows_product_execution_at" in MODELS, "product execution is time-bounded")
expect("receipt_class" in MODELS and "BlockedEntitlement" in MODELS,
       "receipt classes include blocked entitlement")
for state in ["Unactivated", "PendingIdentity", "PendingDeviceCode", "Expired", "Revoked", "Invalid"]:
    expect(state in MODELS, f"lifecycle entitlement state {state} exists")

# ── Lifecycle orchestrator: install/update require a signed decision ──────

expect("EntitlementRequired" in ORCHESTRATOR, "install/update require entitlement")
expect("ProductGrantRequired" in ORCHESTRATOR, "product grant is required")
expect("FeatureGrantRequired" in ORCHESTRATOR, "feature grant is required")
expect("EntitlementBlocked" in ORCHESTRATOR, "blocked entitlements cannot mutate")
expect("ArtifactTrustRequired" in ORCHESTRATOR, "install/update artifacts must be verified")
expect("PurgeConfirmationRequired" in ORCHESTRATOR, "purge requires separate confirmation")
expect("recovery_safe" in ORCHESTRATOR, "recovery-safe operations are distinguished")
expect("granted_products.contains" in ORCHESTRATOR
       or "contains(&request.selected_product)" in ORCHESTRATOR,
       "product grant check is exact")
expect("evidence_refs.is_empty()" in ORCHESTRATOR or "evidence_refs" in ORCHESTRATOR,
       "entitlement decisions carry evidence references")

# ── Lifecycle receipts: product-ready only from a verified binding ────────

expect("LimitedAccessReady" in RECEIPTS and "PaidReady" in RECEIPTS and "DevelopmentReady" in RECEIPTS,
       "receipt classes distinguish Eval/Paid/Development readiness")
expect("product_ready" in RECEIPTS, "product-ready projection exists")
expect("LifecycleReceiptError" in RECEIPTS and "not bound to a verified entitlement snapshot" in RECEIPTS,
       "unbound receipts fail closed")
expect("verify(" in RECEIPTS and "expected_previous_hash" in RECEIPTS,
       "receipts are hash-chained")

# ── Frozen machine binding for the installer channel ──────────────────────

machine = INTERNAL["registration_states"]
for denied in ("denied", "refunded", "revoked", "superseded", "expired"):
    expect(machine["transitions"][denied] == ["recovery_only"], f"{denied} settles recovery_only")
expect(machine["transitions"]["recovery_only"] == [], "recovery_only accepts nothing")
expect(INTERNAL["polling"]["terminal_states"] == ["activated", "denied", "recovery_only"],
       "poll terminal states are frozen")

# The shared flow's Rust transcript replay executes in this commit.
expect("paid_terminal_flow_renders_frozen_presenter_states_and_redacts" in FLOW,
       "Rust paid-terminal replay exists")
expect("existing_key_flow_settles_activated_without_checkout" in FLOW,
       "Rust existing-key replay exists")
expect("limited_access_spec172_overlay_settles_activated_without_checkout" in FLOW,
       "Rust limited-access replay exists")
expect("checkout_timeout_cancels_fail_closed_to_recovery_only" in FLOW,
       "Rust timeout/cancel replay exists")
expect("recovery_only_resume_never_regrants" in FLOW, "Rust recovery never-regrant replay exists")

# ── Hygiene: no raw email, secret, or card data in the installer wiring ───

email_pattern = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
installer_source = INSTALL + FLOW
for match in email_pattern.findall(installer_source):
    if match.endswith("@example.com") or match == "support@focusa.dev":
        continue
    raise AssertionError(f"unmasked email in installer surfaces: {match}")
for needle in ["card_pan", "card_expiry", "card_cvc"]:
    expect(needle not in installer_source, f"card data absent from installer surfaces: {needle}",
           negative=True)
expect("full_license_key" not in FLOW, "no full-key field in the flow presenter", negative=True)

print(json.dumps({
    "schema": "focusa.spec152e.installer_activation_transcript.v1",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "installer_presenter": "installer/official_installer",
    "interactive_flow": "run_activation_flow + ActivationSession",
    "fallback": "acquire_installer_entitlement (device-code)",
    "lifecycle_surfaces": ["models", "orchestrator", "receipts"],
    "result": "passed_fail_closed",
}, sort_keys=True))
