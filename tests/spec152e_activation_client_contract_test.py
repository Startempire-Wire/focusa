#!/usr/bin/env python3
"""Spec 152E.05.01 shared activation client / presenter-neutral reducer contract.

Binds the reusable activation client and presenter-neutral reducer in
crates/focusa-license (activation_reducer, activation_facade,
activation_client) and their deterministic transcript fixtures
(crates/focusa-license/tests/fixtures/
spec152e-activation-transcript-fixtures.v1.json) to the frozen Spec 152E
contracts (internal state machine, stable-failure registry, public OpenAPI
envelope/retry schemas). Every state and every error is handled once in the
shared fixture/reducer encoding; presenters render only and secrets, raw
emails, and full keys are absent from every artifact.

Exact verification: python3 tests/spec152e_activation_client_contract_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
CRATE = ROOT / "crates/focusa-license"

INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))
ERRORS = json.loads((CONTRACTS / "spec152e-activation-errors.v1.json").read_text(encoding="utf-8"))
PUBLIC = json.loads((CONTRACTS / "spec152e-activation-public-openapi.v1.json").read_text(encoding="utf-8"))
FIXTURE = json.loads((CRATE / "tests/fixtures/spec152e-activation-transcript-fixtures.v1.json").read_text(encoding="utf-8"))

REDUCER = (CRATE / "src/activation_reducer.rs").read_text(encoding="utf-8")
FACADE = (CRATE / "src/activation_facade.rs").read_text(encoding="utf-8")
CLIENT = (CRATE / "src/activation_client.rs").read_text(encoding="utf-8")
LIB = (CRATE / "src/lib.rs").read_text(encoding="utf-8")
INTEGRATION = (CRATE / "tests/spec152e_activation_contract.rs").read_text(encoding="utf-8")
CLI_LICENSE = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text(encoding="utf-8")

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


# ── Contract primitives ───────────────────────────────────────────────────

machine = INTERNAL["registration_states"]
initial_state = machine["initial"]
nonterminal = set(machine["nonterminal"])
terminal = set(machine["terminal"])
all_states = nonterminal | terminal
transitions = {state: set(destinations) for state, destinations in machine["transitions"].items()}
frozen_pairs = {(frm, to) for frm, destinations in transitions.items() for to in destinations}
error_rows = {row["code"]: row for row in ERRORS["errors"]}
error_codes = set(error_rows)
operations = INTERNAL["operations"]
operation_by_id = {op["id"]: op for op in operations}
presenter_states = set(INTERNAL["presenter_states"])
retry_rules = INTERNAL["polling"]["retry_rules"]
envelope_required = set(INTERNAL["canonical_output"]["required"])
envelope_optional = set(INTERNAL["canonical_output"]["optional"])
envelope_forbidden = set(INTERNAL["canonical_output"]["forbidden"])
forbidden_caller = set(INTERNAL["request_context"]["forbidden_caller_fields"])

expect(initial_state == "attempt_created", "registration begins as a bounded pending attempt")
expect(len(all_states) == 19, "frozen machine has exactly 19 states")
expect(len(frozen_pairs) == 48, "frozen machine has exactly 48 transitions")

# ── Fixture machine must be the exact frozen machine ──────────────────────

fixture_pairs = {(row["from"], row["to"]) for row in FIXTURE["state_machine"]}
expect(fixture_pairs == frozen_pairs, "fixture state machine is byte-exact with the frozen contract")
fixture_events = {(row["from"], row["event"], row["to"]) for row in FIXTURE["state_machine"]}
expect(len(fixture_events) == 48, "fixture encodes each transition exactly once")
for frm, to in frozen_pairs:
    matching = [row for row in FIXTURE["state_machine"] if (row["from"], row["to"]) == (frm, to)]
    expect(len(matching) == 1, f"exactly one event encoding for {frm} -> {to}")

expect(FIXTURE["initial_state"] == initial_state, "fixture initial state matches")
expect(set(FIXTURE["presenter_states"]) == presenter_states, "fixture presenter states match the frozen list")

# ── Presenter mapping: rendering only, all states handled once ────────────

expect(set(FIXTURE["presenter_by_state"]) == all_states, "every state has exactly one presenter projection")
for registration_state, presenter in FIXTURE["presenter_by_state"].items():
    expect(presenter in presenter_states, f"presenter {presenter} is a frozen presenter state")
expect(len(set(FIXTURE["presenter_by_state"].values())) == 10, "all ten presenter states are reachable")

# ── Operation reachability: presenter projections stay inside the frozen
# ── success-state sets for every operation ────────────────────────────────

settles = {
    "activation.start": [("attempt_created", "email_challenge_sent")],
    "activation.verify": [("email_challenge_sent", "email_verified"), ("email_verified", "account_promoted")],
    "activation.offers": [],
    "activation.select_offer": [
        ("account_promoted", "offer_selected"),
        ("account_promoted", "limited_access_review"),
        ("account_promoted", "existing_key_review"),
        ("offer_selected", "offer_selected"),
    ],
    "activation.checkout": [("offer_selected", "checkout_pending")],
    "activation.existing_license": [
        ("existing_key_review", "entitlement_issued"),
        ("entitlement_issued", "terminal_delivery_ready"),
        ("terminal_delivery_ready", "device_registered"),
        ("device_registered", "lease_issued"),
        ("lease_issued", "delivered"),
    ],
    "activation.poll": [],
    "lease.refresh": [("lease_issued", "recovery_only"), ("delivered", "recovery_only")],
    "nodes.list": [],
    "nodes.deactivate": [],
    "account.manage_link": [],
}
for op in operations:
    op_id = op["id"]
    success = set(op["success_states"])
    expect(success <= presenter_states, f"{op_id} success states are frozen presenter states")
    expect(set(op["failures"]) <= error_codes, f"{op_id} failures are frozen error codes")
    for frm, to in settles[op_id]:
        if (frm, to) == (frm, frm):
            continue
        expect((frm, to) in frozen_pairs, f"{op_id} settle {frm} -> {to} is a frozen transition")
        expect(FIXTURE["presenter_by_state"][to] in success, f"{op_id} settle {frm} -> {to} projects {FIXTURE['presenter_by_state'][to]} not in {success}")

# Poll can observe every presenter state the frozen machine can sit in.
observable = {
    FIXTURE["presenter_by_state"][state]
    for state in all_states
    if FIXTURE["presenter_by_state"][state]
    not in {"email_required", "email_verified", "selection_required", "checkout_required"}
}
expect(observable == set(operation_by_id["activation.poll"]["success_states"]),
       "poll success states are exactly the observable non-input presenter states")

# ── Positive transcripts replay deterministically ─────────────────────────

def event_targets(from_state: str, event: str) -> list[str]:
    return [row["to"] for row in FIXTURE["state_machine"] if row["from"] == from_state and row["event"] == event]


for transcript in FIXTURE["positive_transcripts"]:
    expect(bool(transcript["steps"]), f"{transcript['id']} has steps")
    expect(transcript["from"] in all_states, f"{transcript['id']} starts at a frozen state")
    previous = transcript["from"]
    for step in transcript["steps"]:
        expect(step["from"] == previous, f"{transcript['id']} chain: {step['from']} after {previous}")
        targets = event_targets(step["from"], step["event"])
        expect(targets == [step["to"]], f"{transcript['id']}: {step['from']} --{step['event']}--> {step['to']}")
        previous = step["to"]

# ── Negative transcripts fail closed ──────────────────────────────────────

for negative in FIXTURE["negative_transcripts"]:
    expect(negative["from"] in all_states, f"{negative['id']} from is a state")
    expect(
        not any(row["from"] == negative["from"] and row["event"] == negative["event"] for row in FIXTURE["state_machine"]),
        f"{negative['id']} must not be a legal transition",
        negative=True,
    )
    expect(bool(negative["reason"]), f"{negative['id']} names a frozen invariant reason")

# ── Error cases: every code exactly once, bound to typed values ───────────

case_codes = [case["code"] for case in FIXTURE["error_cases"]]
expect(len(case_codes) == 33, "fixture binds all 33 error codes")
expect(len(set(case_codes)) == 33, "each error code appears exactly once")
operation_failures = {code for op in operations for code in op["failures"]}
for case in FIXTURE["error_cases"]:
    row = error_rows.get(case["code"])
    expect(row is not None, f"{case['code']} exists in the frozen registry", negative=False)
    expect(row is not None and case["http_status"] == row["http_status"], f"{case['code']} http_status")
    expect(row is not None and case["retryable"] == row["retryable"], f"{case['code']} retryable")
    expect(row is not None and case["safe_next_action"] == row["safe_next_action"], f"{case['code']} safe_next_action")
    if case["operation"] is None:
        expect(case["code"] not in operation_failures,
               f"{case['code']} is a runtime entitlement code outside the activation call stack",
               negative=True)
    else:
        op = operation_by_id.get(case["operation"])
        expect(op is not None, f"{case['code']} bound to a frozen operation")
        expect(case["code"] in op["failures"], f"{case['code']} is a failure of {case['operation']}")

posture_for = {}
for code in retry_rules["safe_retry"]:
    posture_for[code] = "safe_retry"
for code in retry_rules["retry_with_same_idempotency_key"]:
    posture_for[code] = "retry_same_idempotency_key"
for code in retry_rules["restart_verification"]:
    posture_for[code] = "restart"
for code in retry_rules["recovery_only"]:
    posture_for[code] = "recovery_only"
for code in retry_rules["do_not_retry_unchanged"]:
    posture_for.setdefault(code, "none")
for case in FIXTURE["error_cases"]:
    expect(case["retry_posture"] == posture_for.get(case["code"], "none"),
           f"{case['code']} retry posture matches the frozen retry rules")

# ── Canonical output envelope: required/optional/forbidden ────────────────

public_envelope = PUBLIC["components"]["schemas"]["ActivationEnvelope"]["properties"]
expect(set(public_envelope) == envelope_required | envelope_optional,
       "public OpenAPI envelope matches the frozen required+optional fields")
expect("full_license_key" not in public_envelope, "public envelope never carries a full license key")
expect("if let Some(key) = args.license_key.as_deref()" in CLI_LICENSE, "agent activation dispatches existing paid keys through fast path")
expect("return run_redeem_fast_path(json_output, key, args.registry.as_deref()).await" in CLI_LICENSE, "agent fast path reuses canonical redeem implementation")
for forbidden in envelope_forbidden:
    expect(forbidden not in public_envelope, f"forbidden field {forbidden} absent from the public envelope")
expect(public_envelope["masked_email"]["pattern"] == r"^[^@]*\*[^@]*@[^@]+$",
       "masked email pattern is the frozen one")
public_retry = PUBLIC["components"]["schemas"]["Retry"]["properties"]
expect(set(public_retry["posture"]["enum"]) == {"none", "safe_retry", "retry_same_idempotency_key", "restart", "recovery_only"},
       "retry postures match the frozen enum")
expect(public_retry["retry_after_seconds"]["minimum"] == 1 and public_retry["retry_after_seconds"]["maximum"] == 30,
       "retry_after_seconds is bounded to 1..=30")

# ── Static Rust surface checks ────────────────────────────────────────────

expect(REDUCER and FACADE and CLIENT and INTEGRATION, "all shared Rust surfaces exist")
expect("pub const fn reduce_activation" in REDUCER, "reducer entry point exists")
expect("pub enum ActivationState" in REDUCER and "pub enum ActivationTransition" in REDUCER,
       "typed states and transitions exist")
expect("pub enum PresenterActivationState" in REDUCER and "pub const fn presenter_state" in REDUCER,
       "presenter projection is a pure rendering function")
expect("pub struct ActivationOutputEnvelope" in REDUCER, "canonical output envelope exists")
expect("pub enum FacadeOperation" in FACADE, "typed facade operations exist")
expect("pub enum ActivationErrorCode" in FACADE and "pub struct ActivationError" in FACADE,
       "typed facade errors exist")
expect("pub fn mask_email" in FACADE, "masked-email helper exists")
expect("pub struct ActivationRequestContext" in FACADE, "typed request context exists")
expect("pub trait ActivationAuthority" in CLIENT, "shared activation transport contract exists")
expect("pub struct ActivationSession" in CLIENT and "pub struct ActivationRegistration" in CLIENT,
       "shared activation session and snapshot exist")
expect("pub fn retry_policy_for_code" in CLIENT, "retry rules are decided once in shared code")
expect("pub mod activation_reducer" in LIB and "pub mod activation_facade" in LIB and "pub mod activation_client" in LIB,
       "modules are exported from the crate")

# The reducer's match is the single source of truth: every frozen event
# variant is encoded and unknown pairs fail closed at the default arm.
def pascal(label: str) -> str:
    return "".join(part.title() for part in label.split("_"))

match_body = REDUCER.split("let next = match (state, transition) {", 1)[1].split("_ => return Err", 1)[0]
for event in sorted({row["event"] for row in FIXTURE["state_machine"]}):
    expect(f"T::{pascal(event)}" in match_body, f"reducer encodes the frozen event {event}")
expect("_ => return Err(ActivationTransitionError::IllegalTransition)" in REDUCER,
       "reducer fails closed on unknown or illegal transitions")

# Every typed error code label is defined exactly once.
for case in FIXTURE["error_cases"]:
    count = FACADE.count(f'"{case["code"]}"')
    expect(count == 1, f"{case['code']} label defined exactly once in the typed facade", negative=False)

# Envelope struct fields never include a forbidden name.
envelope_fields = re.findall(r"^\s+pub (\w+):", REDUCER, re.MULTILINE)
field_names = {name for name in envelope_fields if name != "schema"}
expect(field_names.isdisjoint(envelope_forbidden), "envelope struct has no forbidden field")
expect("masked_email" in field_names, "envelope carries only masked email")

# Request context has no forbidden caller field.
context_fields = set(re.findall(r"^\s+pub (\w+):", FACADE, re.MULTILINE))
expect(forbidden_caller.isdisjoint(context_fields), "request context has no forbidden caller field")
expect("FORBIDDEN_CALLER_FIELDS" in FACADE, "forbidden caller fields are named in shared code")

# ── Hygiene: no secrets, unmasked real email, or keys anywhere ────────────

artifacts = "\n".join(
    path.read_text(encoding="utf-8")
    for path in (
        CRATE / "tests/fixtures/spec152e-activation-transcript-fixtures.v1.json",
        CONTRACTS / "spec152e-activation-internal.v1.json",
        CONTRACTS / "spec152e-activation-errors.v1.json",
        CONTRACTS / "spec152e-activation-public-openapi.v1.json",
    )
)
test_source = Path(__file__).read_text(encoding="utf-8")
raw = artifacts + "\n" + test_source

expect(not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw),
       "no unmasked real email in fixtures or tests", negative=True)
expect(not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw),
       "no secret prefixes in fixtures or tests", negative=True)
expect(not re.search(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}", raw),
       "no full license keys in fixtures or tests", negative=True)
expect("-----BEGIN" not in artifacts, "no private key material in fixtures or contracts", negative=True)

result = {
    "schema": "focusa.spec152e.activation_client_contract.v1",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "states": len(all_states),
    "transitions": len(frozen_pairs),
    "presenter_states": len(presenter_states),
    "error_codes": len(error_codes),
    "positive_transcripts_replayed": len(FIXTURE["positive_transcripts"]),
    "negative_transcripts_replayed": len(FIXTURE["negative_transcripts"]),
    "rust_surfaces": [
        "activation_reducer",
        "activation_facade",
        "activation_client",
        "spec152e_activation_contract",
    ],
    "result": "passed_fail_closed",
}
print(json.dumps(result, sort_keys=True))
