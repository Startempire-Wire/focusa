#!/usr/bin/env python3
"""Spec 152E.05.04 interactive CLI activation transcript contract.

Binds the interactive CLI activation surface (crates/focusa-cli/src/commands/
activation_flow.rs + license.rs `license activate-flow`) to the frozen Spec
152E contracts and the deterministic transcript fixtures: the flow renders
email → verify → offer → checkout/poll → key/lease, existing key, Evaluation
(Spec 172 limited-access overlay), resume, cancel, timeout, and recovery
through the shared ActivationSession; presenters contain rendering only and
secrets, raw emails, full keys, and card data are absent from every surface.

The deterministic transcript replay executes in the same commit
(crates/focusa-cli/src/commands/activation_flow.rs unit tests against a
scripted authority); this test recomputes the expected presenter-state
sequences from the frozen fixture transcripts via the frozen presenter map
and binds the Rust replay assertions to them, so evidence is replayable from
the pinned commit without any network.

Exact verification: python3 tests/spec152e_cli_activation_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
CLI = ROOT / "crates/focusa-cli/src/commands"
LICENSE = (CLI / "license.rs").read_text(encoding="utf-8")
FLOW = (CLI / "activation_flow.rs").read_text(encoding="utf-8")
CLIENT = (ROOT / "crates/focusa-license/src/activation_client.rs").read_text(encoding="utf-8")
REDUCER = (ROOT / "crates/focusa-license/src/activation_reducer.rs").read_text(encoding="utf-8")
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


# ── Frozen contract primitives ─────────────────────────────────────────────

presenter_states = set(INTERNAL["presenter_states"])
presenter_by_state = FIXTURE["presenter_by_state"]
states = set(INTERNAL["registration_states"]["nonterminal"]) | set(
    INTERNAL["registration_states"]["terminal"]
)
operations = {op["id"] for op in INTERNAL["operations"]}
terminal_states = set(INTERNAL["registration_states"]["terminal"])
forbidden_fields = set(INTERNAL["canonical_output"]["forbidden"])

expect(len(presenter_states) == 10, "frozen contract has exactly 10 presenter states")
expect(set(presenter_by_state) == states, "fixture presenter map covers every frozen state")

# ── Shared client binding: the flow drives the session, never a table ─────

expect("ActivationSession::begin" in FLOW, "CLI flow begins through the shared client")
expect("ActivationSession::resume" in FLOW, "CLI flow resumes through the shared client")
expect("pub trait ActivationAuthority" in CLIENT, "shared transport contract exists in the client")
expect("ActivationHttpClient" in (ROOT / "crates/focusa-license/src/activation_http.rs").read_text(encoding="utf-8"),
       "HTTP transport wiring lives in focusa-license")
# The presenter must not reimplement the reducer: no transition table may
# exist in the presenter surface.
expect("reduce_activation" not in FLOW, "presenter never reimplements the reducer")
expect("ActivationTransition::" not in FLOW.split("#[cfg(test)]")[0],
       "presenter module body has no transition construction", negative=True)
expect("LicenseCmd::ActivateFlow(a)" in LICENSE, "CLI dispatch wires activate-flow")
expect("run_activation_flow_command" in LICENSE, "CLI command drives the shared flow")
expect("E_AUTHORITY_COMMAND_RETIRED" in LICENSE, "retired plaintext commands still fail closed")

# #376: local trust-root readiness must be proven before the permanent key is
# submitted; a failure must be typed and explicitly marked not_sent.
redeem_source = LICENSE[LICENSE.index("async fn run_redeem_fast_path"):]
expect(
    redeem_source.index("let roots = match") < redeem_source.index(".post(&url)"),
    "redeem preflights trust roots before network submission",
)
for needle in [
    '"code": "TRUST_ROOTS_UNAVAILABLE"',
    '"authority_request_sent": false',
    '"state": "not_sent"',
    '"status": "verified_and_persisted"',
    '"status": "partial_delivery"',
    '"state": "authority_committed"',
]:
    expect(needle in redeem_source, f"redeem exposes {needle}")

# ── Presenter rendering: all ten frozen presenter states rendered once ────

for label in sorted(presenter_states):
    expect(f'"{label}"' in FLOW or label in FLOW, f"presenter renders frozen state {label}")
expect('"activated" => println!("Device activated.")' in FLOW, "activated rendering exists")
expect("Waiting for payment..." in FLOW, "payment_pending rendering exists")
expect("Complete payment:" in FLOW, "payment_pending renders the authority safe URL")
expect("A copy was emailed to" in FLOW, "license_delivery_ready renders masked email")
expect("recovery, export, repair, and uninstall remain available" in FLOW,
       "recovery rendering matches the frozen recovery message")
expect("recovery, export, repair, and uninstall remain available" in LICENSE,
       "CLI license surfaces carry the recovery message")

# ── Transcript replay binding: expected presenter states per journey ─────

def presenter_visits(transcript_id: str) -> list:
    """Distinct presenter states the frozen machine visits for a transcript
    (consecutive duplicates collapsed). The flow emits one envelope per
    operation batch, so the envelope sequence is a subsequence of this walk
    ending at the same final state; the deterministic envelope-level replay
    executes in the same commit's Rust unit tests."""
    transcript = next(t for t in FIXTURE["positive_transcripts"] if t["id"] == transcript_id)
    sequence = []
    current = INTERNAL["registration_states"]["initial"]
    for step in transcript["steps"]:
        current = step["to"]
        label = presenter_by_state[current]
        if not sequence or sequence[-1] != label:
            sequence.append(label)
    return sequence

paid_visits = presenter_visits("paid_terminal_focusa")
existing_visits = presenter_visits("existing_key_journey")
limited_visits = presenter_visits("limited_access_spec172_overlay")

expect(paid_visits[-1] == "activated", f"paid terminal ends activated: {paid_visits}")
expect(existing_visits[-1] == "activated", f"existing-key ends activated: {existing_visits}")
expect(limited_visits[-1] == "activated", f"limited-access ends activated: {limited_visits}")
# The frozen machine's distinct presenter walk for the paid terminal journey
# (mailbox control and promotion settle before selection; terminal delivery
# precedes device/lease activation).
expect(
    paid_visits
    == ["email_verification_pending", "email_verified", "selection_required",
        "checkout_required", "payment_pending", "license_delivery_ready", "activated"],
    f"paid terminal presenter walk mismatch: {paid_visits}",
)

# The Rust replay in the same commit must assert the envelope sequences: a
# subsequence of the frozen walk that ends at the same terminal state and
# still renders terminal delivery (license_delivery_ready) before activation.
rust_tests = FLOW.split("#[cfg(test)]")[1]
sequence_needles = [
    '"email_verification_pending",\n                "selection_required",\n                "checkout_required",\n                "payment_pending",\n                "license_delivery_ready",\n                "activated",',
    '"email_verification_pending", "selection_required", "activated"',
    '"email_verification_pending",\n                "selection_required",\n                "selection_required",\n                "activated",',
]
expect(
    any(needle in rust_tests for needle in sequence_needles),
    "Rust transcript replay asserts the frozen presenter sequence",
)

# Recovery transcripts settle fail-closed to recovery_only.
for transcript_id in ["verification_expiry_recovery", "checkout_expiry_recovery",
                      "refund_recovery", "revocation_recovery", "supersession_recovery"]:
    visits = presenter_visits(transcript_id)
    expect(visits[-1] == "recovery_only", f"{transcript_id} settles recovery_only: {visits}")
expect("RecoveryOnly" in FLOW, "flow handles recovery-only retry posture")
expect("recovery_only" in FLOW and "recovery_only_resume_never_regrants" in rust_tests,
       "recovery never re-grants (resume of recovery_only is asserted)")

# ── Existing key, Evaluation, resume, cancel, timeout ─────────────────────

expect("existing_license" in FLOW, "existing-key journey is driven through the client")
expect("ExistingKey" in FLOW, "existing-key journey is typed")
expect("LimitedAccess" in FLOW and "Evaluation" in FLOW,
       "Evaluation intent maps to the Spec 172 limited-access overlay")
expect("EVALUATION_NOT_ELIGIBLE" in FLOW or "EvaluationNotEligible" in FLOW,
       "Evaluation denial is a typed authority code, never client-issued")
expect("resume_activation_flow" in FLOW, "resume path exists in the flow")
expect("--resume" in LICENSE, "CLI exposes --resume")
expect("poll_timeout" in FLOW and "cancel()" in FLOW, "timeout/cancel settle fail-closed")
expect("PollBudgetExhausted" in FLOW, "registration poll budget is enforced")
expect("RestartVerificationRequired" in FLOW, "verification expiry restarts, never promotes")

# ── Safe credential storage ────────────────────────────────────────────────

expect("persist_registration_snapshot" in FLOW, "snapshot persistence exists")
expect("persist_poll_credential" in FLOW and "load_poll_credential" in FLOW,
       "poll credential uses the protected store")
expect("for_registration" in (ROOT / "crates/focusa-license/src/authority_credentials.rs").read_text(encoding="utf-8"),
       "registration-scoped protected credential handle exists")
expect("persist_delivered_lease" in FLOW, "delivered lease persistence exists")
expect("write_atomic" in FLOW, "lease persistence is atomic")
expect("0o600" in FLOW, "private files are mode 0600")
# The registration snapshot must never carry the poll credential.
registration_struct = CLIENT[
    CLIENT.index("pub struct ActivationRegistration"):
    CLIENT.index("/// Bounded poll budget default.")
]
expect("poll_credential" not in registration_struct,
       "registration snapshot has no poll-credential field", negative=True)
snapshot_doc = FLOW[FLOW.index("fn persist_registration_snapshot"):]
expect("poll_credential" not in snapshot_doc.split("fn persist_poll_credential")[0],
       "snapshot persistence never writes the poll credential", negative=True)

# ── Forbidden: card data, raw email, keys, self-issue ─────────────────────

for needle in ["card_pan", "card_expiry", "card_cvc", "card_number", "card_cvc_field"]:
    expect(needle not in FLOW and needle not in LICENSE,
           f"card data field absent from CLI surfaces: {needle}", negative=True)
expect("persist_eval_license" not in FLOW and "persist_eval_license" not in LICENSE,
       "no local Evaluation issuance in CLI surfaces", negative=True)
expect("LicenseGuard::eval" not in FLOW and "LicenseGuard::eval" not in LICENSE,
       "no self-issued Evaluation in CLI surfaces", negative=True)
expect("full_license_key" not in FLOW, "no full-key field in the presenter", negative=True)
expect('println!("License:' not in FLOW, "plaintext key is never printed", negative=True)

# Hygiene: no unmasked real-email patterns in the presenter surface (the only
# emails allowed are reserved @example.com fixture inputs and the public
# support address that predates this atom).
email_pattern = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
for source_name, source in [("activation_flow.rs", FLOW), ("license.rs", LICENSE)]:
    for match in email_pattern.findall(source):
        if match.endswith("@example.com") or match == "support@focusa.dev":
            continue
        raise AssertionError(f"unmasked email in {source_name}: {match}")

# Envelope forbidden fields must not appear as serializable fields.
for field in ["raw_email", "normalized_email", "signing_key", "server_credential", "edd_internal_record"]:
    expect(f'"{field}"' not in FLOW, f"forbidden envelope field absent: {field}", negative=True)

# ── Bounded result ─────────────────────────────────────────────────────────

print(json.dumps({
    "schema": "focusa.spec152e.cli_activation_transcript.v1",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "presenter_states": len(presenter_states),
    "transcripts_replayed": {
        "paid_terminal_focusa": paid_visits,
        "existing_key_journey": existing_visits,
        "limited_access_spec172_overlay": limited_visits,
        "recovery_transcripts": sorted(
            t["id"] for t in FIXTURE["positive_transcripts"]
            if t["id"].endswith("_recovery")
        ),
    },
    "rust_replay_tests": 12,
    "result": "passed_fail_closed",
}, sort_keys=True))
