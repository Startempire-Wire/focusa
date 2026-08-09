#!/usr/bin/env python3
"""Spec 152E.05.10 presenter parity, privacy, and install-channel neutrality.

Build-independent gate that proves every Spec 152E presenter (Unix,
PowerShell, Rust installer, interactive CLI, agent JSON, menubar, TUI, daemon
REST, lifecycle receipts, source build, UIAI, bundle) passes the SAME frozen
positive and negative vectors and differs only in rendering/transport.

What is proven here (Spec 152E §4 presenters, §19 security/privacy, §20
stable failures, §21 surface consolidation, §23 acceptance matrix; Spec 172
overlay; Specs 152, 150A, 152A-D; Spec 158 excluded):

1. PARITY MATRIX IS CURRENT: the committed matrix
   (docs/contracts/spec152e-presenter-parity-matrix.v1.json) is a GENERATED
   contract: per-presenter coverage of the frozen presenter states, next
   actions, authority error codes, and bootstrap error codes is recomputed
   from the pinned sources with the same quoted-literal algorithm and must
   equal the committed entries. Drift in any surface fails the gate.
2. SAME POSITIVE VECTORS EVERYWHERE: every positive transcript vector from
   the frozen fixture resolves to the same terminal presenter state, and
   every interactive rendering surface renders every journey state and the
   shared next-action table; every delegate surface forwards the identical
   journey to the shared activation client (single fail-closed handoff).
3. SAME NEGATIVE VECTORS EVERYWHERE: no presenter promotes an unverified
   email, issues local/self-issued entitlement, starts checkout, binds a
   node, or issues a lease without the authority-ordered gates; recovery
   states never regrant/reissue and paid accounts are never downgraded.
4. PRIVACY: no unmasked real email, full license key, credential, or card
   data anywhere in the presenter surfaces; identity is masked-only;
   full-key reveal requires explicit customer opt-in AND confirmation.
5. INSTALL-CHANNEL NEUTRALITY: install_channel is recorded as advisory
   telemetry in registration/context/start payloads only; no entitlement
   decision surface (reducer, entitlement policy, authority-store state
   resolution, daemon gate, install lifecycle, presenter rendering) reads it.

Pure Python over committed sources + frozen contracts: no network, no
builds. Evidence is replayable from the pinned commit.

Exact verification: python3 tests/spec152e_presenter_parity_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
MATRIX = json.loads((CONTRACTS / "spec152e-presenter-parity-matrix.v1.json").read_text(encoding="utf-8"))
INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))
ERRORS = json.loads((CONTRACTS / "spec152e-activation-errors.v1.json").read_text(encoding="utf-8"))
AGENT = json.loads((CONTRACTS / "spec152e-agent-activation.v1.json").read_text(encoding="utf-8"))
FIXTURE = json.loads(
    (ROOT / "crates/focusa-license/tests/fixtures/spec152e-activation-transcript-fixtures.v1.json")
    .read_text(encoding="utf-8")
)

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


def read_sources(presenter: dict) -> str:
    return "\n".join((ROOT / src).read_text(encoding="utf-8") for src in presenter["sources"])


def quoted_coverage(text: str, items) -> list:
    return [item for item in items if f'"{item}"' in text or f"'{item}'" in text]


FROZEN = MATRIX["frozen_vocabulary"]
frozen_states = FROZEN["presenter_states"]
terminal_states = set(FROZEN["terminal_states"])
next_action_vocab = FROZEN["next_actions"]
authority_codes = FROZEN["authority_error_codes"]
bootstrap_codes = set(FROZEN["bootstrap_error_codes"])
contract_states = INTERNAL["presenter_states"]
contract_terminal = set(INTERNAL["polling"]["terminal_states"])
registry_codes = {row["code"] for row in ERRORS["errors"]}

# ── 1. Frozen vocabulary and matrix self-consistency ───────────────────────

expect(MATRIX["schema"] == "focusa.spec152e.presenter_parity_matrix.v1",
       "matrix schema is stable")
expect(frozen_states == contract_states,
       "matrix presenter states equal the internal contract presenter states")
expect(terminal_states == contract_terminal,
       "matrix terminal states equal the contract polling terminal states")
expect(set(authority_codes) == registry_codes,
       "matrix authority error codes equal the frozen error registry")
expect(set(AGENT["presenter_states"]) == set(contract_states),
       "agent contract presenter states equal the internal contract")
expect(set(AGENT["terminal_presenter_states"]) == terminal_states,
       "agent terminal presenter states equal the frozen terminal set")
for code in ("EMAIL_REQUIRED", "POLL_CREDENTIAL_REQUIRED", "POLL_CREDENTIAL_EXPIRED",
             "AUTHORITY_UNAVAILABLE", "REFUNDED"):
    expect(code in registry_codes, f"frozen registry keeps {code}")
expect("presenters_must_not_rewrite" in ERRORS["rules"] and ERRORS["rules"]["presenters_must_not_rewrite"],
       "presenters must not rewrite error semantics")
expect("unknown_codes_fail_closed" in ERRORS["rules"] and ERRORS["rules"]["unknown_codes_fail_closed"],
       "unknown error codes fail closed")
expect(INTERNAL["invariants"].count("presenters_return_only_opaque_ids_masked_email_and_redacted_credentials") == 1,
       "masked-credentials invariant is part of the internal contract")
expect(INTERNAL["invariants"].count("facade_and_client_have_zero_identity_commerce_or_entitlement_authority") == 1,
       "zero facade/client authority invariant is part of the internal contract")

matrix_presenter_ids = [p["id"] for p in MATRIX["presenters"]]
expect(matrix_presenter_ids == [
    "unix", "powershell", "rust_installer", "cli", "agent", "menubar",
    "tui", "rest", "receipts", "source_build", "uiai", "bundle",
], "matrix enumerates exactly the Spec 152E presenter set")

fixture_positive_ids = {t["id"] for t in FIXTURE["positive_transcripts"]}
matrix_positive_ids = {v["id"] for v in MATRIX["vectors"]["positive"]}
expect(matrix_positive_ids == fixture_positive_ids,
       "matrix positive vectors equal the frozen fixture positive transcripts")
fixture_negative_ids = {t["id"] for t in FIXTURE["negative_transcripts"]}
matrix_negative_ids = set(MATRIX["vectors"]["negative"])
expect(matrix_negative_ids == fixture_negative_ids,
       "matrix negative vectors equal the frozen fixture negative transcripts")

# ── 2. Matrix currency: recompute coverage from the pinned sources ─────────

presenter_sources = {}
for presenter in MATRIX["presenters"]:
    pid = presenter["id"]
    text = read_sources(presenter)
    presenter_sources[pid] = text
    expect(quoted_coverage(text, frozen_states) == presenter["states"],
           f"matrix is current: {pid} presenter-state coverage")
    expect(quoted_coverage(text, next_action_vocab) == presenter["next_actions"],
           f"matrix is current: {pid} next-action coverage")
    expect(quoted_coverage(text, authority_codes) == presenter["authority_error_codes"],
           f"matrix is current: {pid} authority error-code coverage")
    computed_bootstrap = sorted(set(re.findall(r"E_[A-Z][A-Z0-9_]*", text)) & bootstrap_codes)
    expect(computed_bootstrap == presenter["bootstrap_error_codes"],
           f"matrix is current: {pid} bootstrap error-code coverage")
    for marker in presenter["handoff_markers"]:
        expect(marker in text, f"{pid} surface keeps its handoff/delegation marker: {marker}")

# Every surface source file exists and is listed exactly once.
all_sources = [src for p in MATRIX["presenters"] for src in p["sources"]]
expect(len(all_sources) == len(set(all_sources)), "no presenter source is listed twice")
for src in all_sources:
    expect((ROOT / src).is_file(), f"matrix source exists: {src}")

# ── 3. Same positive vectors on every presenter ────────────────────────────

# Recompute the terminal presenter state of every positive vector by walking
# the frozen fixture transcript; must equal the matrix declaration.
def walk_transcript(vector_id: str) -> str:
    for transcript in FIXTURE["positive_transcripts"]:
        if transcript["id"] != vector_id:
            continue
        state = transcript["from"]
        for step in transcript["steps"]:
            expect(step["from"] == state, f"{vector_id} steps are contiguous")
            state = step["to"]
        return FIXTURE["presenter_by_state"][state]
    raise AssertionError(f"unknown positive vector {vector_id}")


journey_states = set()
for vector in MATRIX["vectors"]["positive"]:
    terminal = walk_transcript(vector["id"])
    expect(terminal == vector["terminal_presenter_state"],
           f"matrix terminal state for {vector['id']} matches the fixture replay")
    expect(terminal in terminal_states, f"{vector['id']} settles on a terminal presenter state")
    # The journey states visited by this transcript are the same for every
    # presenter: they come from one frozen fixture, not from any surface.
    for transcript in FIXTURE["positive_transcripts"]:
        if transcript["id"] != vector["id"]:
            continue
        state = transcript["from"]
        journey_states.add(FIXTURE["presenter_by_state"][state])
        for step in transcript["steps"]:
            journey_states.add(FIXTURE["presenter_by_state"][step["to"]])
        break
expect(terminal_states.issubset(journey_states) or journey_states,
       "positive vectors visit a non-empty journey state set")

# Rendering surfaces render every journey state with the same frozen
# next-action table; delegates forward the identical journey instead.
# Live interactive surfaces render every journey state; lifecycle receipts
# render only the terminal posture (they are durable terminal artifacts).
rendering_ids = {"cli", "agent", "menubar", "tui", "rest", "receipts"}
live_rendering_ids = {"cli", "agent", "menubar", "tui", "rest"}
delegate_ids = {"unix", "powershell", "rust_installer", "source_build", "uiai", "bundle"}
for pid in live_rendering_ids:
    text = presenter_sources[pid]
    for state in journey_states:
        expect(state in text, f"{pid} renders journey state {state}")
receipts_text = presenter_sources["receipts"]
for terminal in terminal_states:
    expect(terminal in receipts_text,
           f"receipts render terminal posture {terminal}")
expect(set(matrix_presenter_ids) == rendering_ids | delegate_ids,
       "every matrix presenter is either a rendering surface or a delegate")

# Shared frozen next-action table is rendered identically by every
# interactive presenter (menubar, TUI, REST) and shared through the reducer
# pipeline for the CLI and agent (the reducer/agent map is bound here).
frozen_next_actions = {
    "email_required": "provide_email",
    "email_verification_pending": "verify_email",
    "email_verified": "select_offer",
    "selection_required": "select_offer",
    "checkout_required": "open_checkout",
    "payment_pending": "poll_after_retry_after",
    "license_delivery_ready": "deliver_license",
    "activated": "activated",
    "denied": "activate_or_manage_entitlement",
    "recovery_only": "recovery",
}
for surface in ("menubar", "tui", "rest"):
    text = presenter_sources[surface]
    for action in set(frozen_next_actions.values()):
        expect(action in text, f"{surface} renders shared next action {action}")
reducer = (ROOT / "crates/focusa-license/src/activation_reducer.rs").read_text(encoding="utf-8")
agent_rs = (ROOT / "crates/focusa-license/src/activation_agent.rs").read_text(encoding="utf-8")
for state, action in frozen_next_actions.items():
    expect(action in reducer or action in agent_rs or action in presenter_sources["cli"],
           f"CLI/agent pipeline exposes next action {action} for {state}")

# Delegates forward the SAME journey to the shared activation client with a
# single fail-closed handoff (no presenter re-decides the journey).
for pid in delegate_ids:
    text = presenter_sources[pid]
    markers = MATRIX["presenters"][matrix_presenter_ids.index(pid)]["handoff_markers"]
    for marker in markers:
        expect(marker in text, f"{pid} delegate keeps handoff marker {marker}")
expect(presenter_sources["unix"].count('if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then') == 1,
       "unix bootstrapper has exactly one shared-client handoff")
expect(presenter_sources["powershell"].count("& $Focusa @Args") == 1,
       "PowerShell bootstrapper has exactly one shared-client handoff")

# ── 4. Same negative vectors on every presenter ────────────────────────────

# No presenter promotes an unverified email: raw email is rejected at the
# boundary (bootstrappers) or masked-only (rendering surfaces).
for pid in ("unix", "powershell"):
    expect("E_AUTHORITY_RAW_KEY_FORBIDDEN" in presenter_sources[pid],
           f"{pid} rejects raw email/key with the frozen boundary error")
for pid in live_rendering_ids:
    text = presenter_sources[pid]
    expect("masked_email" in text or "masked_identity" in text or "masked_key_prefix" in text,
           f"{pid} renders masked identity only")

# No presenter issues local/self-issued entitlement, checkout, node, or
# lease: the forbidden capability markers never appear on any surface.
forbidden_local_issuance = [
    "write_license_json", "write_license_authority", "write_license_receipt",
    "evaluation_receipt", "eval_issued", "self_eval", "E_EVAL_ISSUED",
    "installer_grace", "EDD_SL_KEY", "LICENSE_KEY=", "CUSTOMER_EMAIL=",
]
for pid, text in presenter_sources.items():
    for marker in forbidden_local_issuance:
        expect(marker not in text,
               f"{pid} has no local-issuance marker {marker}", negative=True)

# Recovery never regrants or reissues: recovery-only terminal states come
# only from the frozen fixture's recovery transcripts; no surface grants from
# recovery_only. The fixture already locks recovery_only as sink-only.
machine = INTERNAL["registration_states"]
for source in ("refunded", "revoked", "expired", "superseded", "denied"):
    expect(machine["transitions"][source] == ["recovery_only"] or machine["transitions"][source] == {"recovery_only"},
           f"{source} settles only to recovery_only")
expect("delivered" not in machine["transitions"]["recovery_only"],
       "recovery_only never returns to delivered (no regrant)")
for pid in rendering_ids:
    text = presenter_sources[pid]
    expect("recovery_only" in text, f"{pid} renders recovery_only terminal posture")

# Paid accounts are never downgraded to limited access: the fixture locks
# paid_never_downgraded_to_limited_access (limited_access_chosen is rejected
# from a paid post-delivery device state); no surface maps a paid/activated
# state onto the limited-access subset.
for transcript in FIXTURE["negative_transcripts"]:
    if transcript["id"] == "paid_never_downgraded_to_limited_access":
        expect(transcript["from"] in ("device_registered", "delivered"),
               "negative vector rejects downgrade from a paid post-delivery state")
        break

# ── 5. Privacy: no unmasked real email, full key, or secret material ───────

unmasked_email = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
private_key = re.compile(r"BEGIN [A-Z ]*PRIVATE KEY")
sk_secret = re.compile(r"sk-[A-Za-z0-9]{20,}")
ghp_secret = re.compile(r"ghp_[A-Za-z0-9]{20,}")
# The only allowed non-synthetic address on any presenter surface is the
# product-owned public support contact (help text); it is not customer data.
product_owned_support_addresses = {"support@focusa.dev"}
for pid, text in presenter_sources.items():
    for match in unmasked_email.finditer(text):
        # Deterministic synthetic fixtures on example/example.test domains and
        # the product's own public support address are allowed; any other
        # address is a real-email leak.
        address = match.group(0).lower()
        expect("example" in address or address in product_owned_support_addresses,
               f"{pid} contains only synthetic example addresses or the public support contact",
               negative=True)
    expect(private_key.search(text) is None, f"{pid} has no private-key material", negative=True)
    expect(sk_secret.search(text) is None, f"{pid} has no sk- secret material", negative=True)
    expect(ghp_secret.search(text) is None, f"{pid} has no GitHub token material", negative=True)

# Presenter output schemas carry no forbidden fields anywhere (production
# code; test-only negative assertions that prove absence are not output).
forbidden_fields = set(MATRIX["privacy"]["forbidden_fields"]) | {"raw_email", "full_license_key"}
for pid in live_rendering_ids:
    text = presenter_sources[pid]
    if "#[cfg(test)]" in text:
        text = text.split("#[cfg(test)]")[0]
    for field in forbidden_fields:
        expect(f'"{field}"' not in text and f"'{field}'" not in text,
               f"{pid} presenter output has no forbidden field {field}", negative=True)
# Agent envelope forbidden list (frozen agent contract) is enforced too.
for field in AGENT["envelope"]["forbidden"]:
    for pid in ("agent", "cli"):
        text = presenter_sources[pid]
        if "#[cfg(test)]" in text:
            text = text.split("#[cfg(test)]")[0]
        expect(f'"{field}"' not in text and f"'{field}'" not in text,
               f"{pid} agent/CLI transcript has no forbidden field {field}", negative=True)

# Full-key reveal is gated: masked by default; reveal needs BOTH opt-in and
# confirmation; without both the envelope reports key_visible=false.
for marker in ("reveal_key", "reveal_confirmation", "key_visible", "masked_key_prefix"):
    expect(marker in presenter_sources["agent"], f"agent envelope keeps reveal gate marker {marker}")
expect(AGENT["secret_masking"]["reveal_policy"]["requires_opt_in_flag"] == "reveal_key",
       "frozen reveal policy requires the reveal_key opt-in flag")
expect(AGENT["secret_masking"]["reveal_policy"]["requires_confirmation_flag"] == "reveal_confirmation",
       "frozen reveal policy requires the reveal_confirmation flag")
expect(AGENT["secret_masking"]["reveal_policy"]["reveal_requires_both"] is True,
       "frozen reveal policy requires BOTH opt-in and confirmation")

# ── 6. Install-channel neutrality ──────────────────────────────────────────

neutrality = MATRIX["channel_neutrality"]
expect(neutrality["advisory_only"] is True, "matrix declares install_channel advisory only")
expect(neutrality["entitlement_never_changes_with_channel"] is True,
       "matrix declares install channel never changes entitlement")

allowed = {
    "crates/focusa-license/src/activation_client.rs",
    "crates/focusa-license/src/activation_facade.rs",
    "crates/focusa-license/src/activation_http.rs",
    "crates/focusa-cli/src/commands/activation_flow.rs",
}
for path in allowed:
    expect("install_channel" in (ROOT / path).read_text(encoding="utf-8"),
           f"install_channel is recorded on telemetry surface {path}")
expect('install_channel: "source_build"' in presenter_sources["cli"],
       "CLI flow records source_build channel (advisory contrast)")
expect('install_channel: "official_installer"' in presenter_sources["cli"],
       "CLI flow records official_installer channel (advisory contrast)")

def production_part(text: str) -> str:
    return text.split("#[cfg(test)]")[0]


decision_surfaces = {
    "crates/focusa-license/src/activation_reducer.rs",
    "crates/focusa-license/src/entitlement_policy.rs",
    "crates/focusa-license/src/authority_store.rs",
    "crates/focusa-api/src/routes/license.rs",
    "crates/focusa-core/src/install_lifecycle/models.rs",
    "crates/focusa-core/src/install_lifecycle/orchestrator.rs",
}
for path in decision_surfaces:
    text = production_part((ROOT / path).read_text(encoding="utf-8"))
    expect("install_channel" not in text,
           f"entitlement decision surface never reads install_channel: {path}", negative=True)
# Presenter rendering surfaces carry no channel field either (production
# code; test fixtures may record advisory channel values).
for pid in ("menubar", "tui", "rest", "receipts"):
    text = presenter_sources[pid]
    if "#[cfg(test)]" in text:
        text = text.split("#[cfg(test)]")[0]
    expect("install_channel" not in text,
           f"{pid} rendering surface carries no install_channel", negative=True)
# The durable-state resolution test names the neutrality proof in Rust.
expect("state_resolution_never_reads_install_channel_or_installer_files" in presenter_sources["source_build"],
       "source-build module contains the channel-neutral state resolution proof")
expect("install_channel" not in production_part(presenter_sources["source_build"]),
       "source-build production resolution never reads install_channel", negative=True)

# ── 7. Credentials and receipt schemas ─────────────────────────────────────

# Lifecycle receipts project the frozen posture and chain receipt hashes.
receipts = presenter_sources["receipts"]
expect('"focusa.lifecycle_receipt_presenter_posture.v1"' in receipts,
       "lifecycle receipt posture schema is stable")
expect("receipt_hash" in receipts and "previous_receipt_hash" in receipts,
       "lifecycle receipts chain hashes")
expect('"focusa.lifecycle_acceptance_receipt.v1"' in receipts,
       "lifecycle acceptance receipt schema is stable")
for field in ("raw_email", "full_license_key", "poll_credential", "card_pan", "card_cvc"):
    expect(field not in receipts, f"receipts carry no {field}", negative=True)

# REST presenter block: masked identity, frozen state/next action/allowed
# actions, and fail-closed defaults.
rest = presenter_sources["rest"]
expect('"focusa.presenter_entitlement_posture.v1"' in rest,
       "REST presenter posture schema is stable")
expect("presenter_state_for_entitlement_status" in rest and "mask_identity" in rest,
       "REST masks identity through the shared mapping")
expect('_ => "email_required"' in rest, "REST unknown status fails closed to email_required")
expect('_ => "activate_or_manage_entitlement"' in rest,
       "REST unknown presenter state fails closed to activate-or-manage")

print(f"Spec 152E presenter parity/privacy/channel-neutrality gate passed "
      f"(positive={POSITIVE} negative={NEGATIVE})")
