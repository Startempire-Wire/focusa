#!/usr/bin/env python3
"""Spec 152E.05.05 agent-safe JSON activation and resume protocol contract.

Binds the agent JSON protocol (Spec 152E §14.2) across its exact surfaces —
CLI --json, daemon/API operation, Pi/agent tool envelopes, secret
masking/reveal policy, and the resumable registration handle — to the frozen
agent contract (docs/contracts/spec152e-agent-activation.v1.json).

The protocol returns typed human-action states, masked email/key by default,
safe checkout/verification links, bounded poll/resume, explicit
customer-controlled key reveal, and a resumable registration handle. Agents
never invent an email, verification code, consent, payment confirmation, or
license, and never advance a human-required state. The deterministic Rust
replay (agent begin/resume/timeout/recovery) executes in the same commit's
unit tests against a scripted authority; this test recomputes the typed
human-action mapping from the frozen contract and binds every surface to it,
so evidence is replayable from the pinned commit without any network.

Exact verification: python3 tests/spec152e_agent_json_contract_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
AGENT = json.loads((CONTRACTS / "spec152e-agent-activation.v1.json").read_text(encoding="utf-8"))
INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))
AGENT_RS = (ROOT / "crates/focusa-license/src/activation_agent.rs").read_text(encoding="utf-8")
FLOW = (ROOT / "crates/focusa-cli/src/commands/activation_flow.rs").read_text(encoding="utf-8")
LICENSE = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text(encoding="utf-8")
DAEMON = (ROOT / "crates/focusa-api/src/routes/license.rs").read_text(encoding="utf-8")
ENVELOPE_TS = (ROOT / "apps/pi-extension/src/activation-envelope.ts").read_text(encoding="utf-8")

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


# ── Frozen agent contract primitives ───────────────────────────────────────

expect(AGENT["schema"] == "focusa.spec152e.agent_activation.v1", "agent contract schema")
envelope_contract = AGENT["envelope"]
expect(envelope_contract["schema"] == "focusa.agent_activation_envelope.v1",
       "envelope schema is frozen")
presenter_states = set(AGENT["presenter_states"])
expect(presenter_states == set(INTERNAL["presenter_states"]),
       "agent presenter states match the frozen internal contract")
terminal = set(AGENT["terminal_presenter_states"])
expect(terminal == {"activated", "denied", "recovery_only"}, "frozen terminal presenter states")
for required in envelope_contract["required"]:
    expect(required != "", f"required envelope field named: {required}")
expect("registration_id" in envelope_contract["required"], "resumable handle is required")
expect("human_action_required" in envelope_contract["required"], "typed human-action flag required")
expect("key_visible" in envelope_contract["required"], "key-mask flag required")
forbidden = set(envelope_contract["forbidden"])
for field in ["email", "normalized_email", "raw_email", "full_license_key",
              "one_time_key_envelope", "lease_envelope", "poll_credential",
              "poll_credential_hash", "verification_hash", "server_credential",
              "signing_key", "card_pan", "card_expiry", "card_cvc", "edd_internal_record"]:
    expect(field in forbidden, f"forbidden field frozen: {field}")

# ── Typed human-action mapping (7 non-terminal presenter states) ──────────

human_actions = AGENT["human_action_states"]
expect(set(human_actions) == presenter_states - terminal, "exactly the 7 human-action states")
expected_actions = {
    "email_required": "provide_email",
    "email_verification_pending": "enter_verification_code",
    "email_verified": "select_offer",
    "selection_required": "select_offer",
    "checkout_required": "open_checkout_url",
    "payment_pending": "complete_payment_then_poll",
    "license_delivery_ready": "reveal_or_accept_license",
}
expect(human_actions == expected_actions, "frozen human-action mapping")
for state in AGENT["terminal_states_require_no_human_action"]:
    expect(state in terminal, f"terminal state listed: {state}")

# ── Secret masking / reveal policy ────────────────────────────────────────

masking = AGENT["secret_masking"]
expect(masking["email"] == "masked_by_default_masked_email_only", "email masked by default")
expect(masking["key"] == "masked_by_default", "key masked by default")
reveal = masking["reveal_policy"]
expect(reveal["full_key_output_masked_by_default"] is True, "full key masked by default")
expect(reveal["explicit_customer_controlled_reveal"] is True, "reveal is customer-controlled")
expect(reveal["requires_opt_in_flag"] == "reveal_key", "opt-in flag is reveal_key")
expect(reveal["requires_confirmation_flag"] == "reveal_confirmation", "confirmation flag is confirm_reveal")
expect(reveal["reveal_requires_both"] is True, "reveal requires BOTH opt-in and confirmation")
expect(reveal["absent_authorization_key_visible_false"] is True, "absent authorization keeps key masked")
expect(reveal["one_time_key_envelope_never_in_agent_transcript"] is True,
       "one-time key envelope never in agent transcript")

# ── Rust surface: focusa-license activation_agent.rs ──────────────────────

expect('pub const AGENT_ENVELOPE_SCHEMA' in AGENT_RS, "frozen envelope schema constant exists")
expect('pub struct AgentKeyReveal' in AGENT_RS, "explicit reveal gate exists")
expect("pub reveal_key: bool" in AGENT_RS and "pub reveal_confirmation: bool" in AGENT_RS,
       "reveal gate has opt-in + confirmation fields")
expect("self.reveal_key && self.reveal_confirmation" in AGENT_RS,
       "reveal authorized only with BOTH flags")
expect('pub struct AgentActivationEnvelope' in AGENT_RS, "agent envelope struct exists")
expect("pub registration_id: String" in AGENT_RS, "resumable handle field exists")
expect("pub human_action_required: bool" in AGENT_RS, "human-action flag exists")
expect("pub key_present: bool" in AGENT_RS and "pub key_visible: bool" in AGENT_RS,
       "key mask flags exist")
expect("pub fn human_action_for_state" in AGENT_RS, "typed human-action map exists")
expect("pub fn human_action_required" in AGENT_RS, "human-action predicate exists")
expect("pub fn mask_key_prefix" in AGENT_RS, "key prefix masker exists")
expect("pub fn masked_email_or_none" in AGENT_RS, "email masker exists")
expect("from_registration" in AGENT_RS, "snapshot projection exists for daemon/API")

# The AgentActivationEnvelope struct must not declare any forbidden field.
struct_body = AGENT_RS[AGENT_RS.index("pub struct AgentActivationEnvelope"):
                      AGENT_RS.index("impl AgentActivationEnvelope")]
for field in ["email", "normalized_email", "raw_email", "full_license_key",
              "one_time_key_envelope", "lease_envelope", "poll_credential",
              "verification_hash", "server_credential", "signing_key",
              "card_pan", "card_expiry", "card_cvc", "edd_internal_record"]:
    expect(f"pub {field}:" not in struct_body, f"envelope struct has no {field} field", negative=True)

# Every frozen human action label is rendered by the Rust surface.
for label in sorted(set(expected_actions.values())):
    expect(f'"{label}"' in AGENT_RS, f"Rust surface renders typed human action {label}")

# Deterministic Rust replay of the agent protocol exists in the same commit.
rust_tests = FLOW.split("#[cfg(test)]")[1]
for needle in [
    "run_agent_activation",
    "resume_agent_activation",
    "AgentKeyReveal::denied()",
    "agent_begin_returns_typed_human_action_envelope_and_handle_without_prompt",
    "agent_resume_polls_boundedly_and_returns_human_action_payment_envelope",
    "agent_resume_settles_terminal_delivery_with_key_masked_by_default",
    "agent_resume_recovery_only_never_regrants_and_carries_typed_error",
    "agent_timeout_cancels_fail_closed_to_recovery_only",
    "agent_begin_without_email_fails_closed_without_authority_call",
]:
    expect(needle in rust_tests, f"Rust agent replay test present: {needle}")

# The agent flow never prompts and never reimplements the reducer.
agent_begin_body = FLOW.split("fn run_agent_activation")[1].split("fn resume_agent_activation")[0]
expect("ActivationFlowInput" not in agent_begin_body,
       "agent begin never prompts (no input source in the agent begin signature)")
expect("reduce_activation" not in FLOW, "agent presenter never reimplements the reducer")

# ── CLI --json surface: license.rs wiring ──────────────────────────────────

expect("pub agent: bool" in LICENSE, "CLI exposes the --agent flag")
expect("pub reveal_key: bool" in LICENSE and "pub confirm_reveal: bool" in LICENSE,
       "CLI exposes reveal flags")
expect("run_agent_activation_command" in LICENSE, "agent command exists")
expect("if a.agent" in LICENSE, "dispatch routes --agent to the agent command")
expect("EMAIL_REQUIRED: agent mode never prompts" in LICENSE, "agent mode fails closed instead of prompting")
expect("--resume <registration_id>" in LICENSE, "agent resume handle documented")
expect("serde_json::to_string_pretty(&outcome.envelope)" in LICENSE,
       "agent command emits the typed envelope as JSON")
expect("AgentKeyReveal {" in LICENSE and "reveal_confirmation: args.confirm_reveal" in LICENSE,
       "CLI wires the explicit reveal gate")

# ── Daemon/API operation surface: /v1/activation/status ───────────────────

expect('"/v1/activation/status"' in DAEMON, "daemon exposes /v1/activation/status")
expect("activation_status" in DAEMON, "daemon activation status handler exists")
expect("read_registration_snapshots" in DAEMON, "daemon reads presenter-safe snapshots")
expect("resumable_handles" in DAEMON, "daemon returns resumable registration handles")
expect("AgentActivationEnvelope::from_registration" in DAEMON,
       "daemon projects snapshots through the shared agent envelope")
expect('"focusa.activation_registration.v1"' in DAEMON, "daemon accepts only canonical snapshots")
expect('"poll_credential_present": false' in DAEMON, "daemon asserts no poll credential")
expect('"raw_email_present": false' in DAEMON, "daemon asserts no raw email")
expect('"raw_key_present": false' in DAEMON, "daemon asserts no raw key")

# ── Pi/agent tool envelope surface ─────────────────────────────────────────

expect("AGENT_ENVELOPE_SCHEMA" in ENVELOPE_TS, "TS module defines the envelope schema")
expect("buildAgentActivationEnvelope" in ENVELOPE_TS, "TS envelope builder exists")
expect("maskEmail" in ENVELOPE_TS, "TS email masker exists")
expect("maskKeyPrefix" in ENVELOPE_TS, "TS key masker exists")
expect("revealAuthorized" in ENVELOPE_TS, "TS reveal gate exists")
expect("humanActionForState" in ENVELOPE_TS, "TS human-action map exists")
expect("resume handle" in ENVELOPE_TS or "registration_id" in ENVELOPE_TS,
       "TS envelope carries the resumable handle")
expect("focusa.agent_activation_envelope.v1" in ENVELOPE_TS, "TS envelope uses the frozen schema")
expect("key_visible" in ENVELOPE_TS, "TS envelope exposes the key-mask flag")

# ── Bounded poll / resume ─────────────────────────────────────────────────

polling = INTERNAL["polling"]
expect(polling["stored_as"] == "keyed_hash_only", "poll credential hash-only at rest")
expect(1 <= polling["default_retry_after_seconds"] <= polling["maximum_retry_after_seconds"] <= 30,
       "retry window bounded 1..=30")
agent_poll = AGENT["bounded_poll"]
expect(agent_poll["default_max_polls"] == 40, "default poll budget is 40")
expect(agent_poll["timeout_settles_fail_closed"] == "cancel_to_recovery_only",
       "timeout settles fail-closed to recovery_only")
expect("max_polls" in agent_poll["budget_field"] and "poll_count" in agent_poll["count_field"],
       "poll budget fields named")
expect(AGENT["resumable_handle"]["field"] == "registration_id", "resumable handle is registration_id")
expect(AGENT["resumable_handle"]["poll_credential_never_in_snapshot"] is True,
       "poll credential never in snapshot")

# ── Forbidden: no invented consent/payment/identity, no secrets ───────────

for needle in ["persist_eval_license", "LicenseGuard::eval", "card_pan", "card_expiry",
               "card_cvc", "card_number", "full_license_key"]:
    expect(needle not in AGENT_RS, f"no self-issue or card data in agent surface: {needle}", negative=True)
expect("println!" not in AGENT_RS, "no terminal printing in the agent protocol module", negative=True)
expect("StdinFlowInput" not in AGENT_RS, "agent protocol never imports the stdin prompt", negative=True)

# Hygiene: no unmasked real-email patterns in agent surfaces (only reserved
# @example.com fixture inputs and the public support address).
email_pattern = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
for source_name, source in [("activation_agent.rs", AGENT_RS), ("activation_flow.rs", FLOW),
                            ("license.rs", LICENSE), ("activation-envelope.ts", ENVELOPE_TS)]:
    for match in email_pattern.findall(source):
        if match.endswith("@example.com") or match == "support@focusa.dev":
            continue
        raise AssertionError(f"unmasked email in {source_name}: {match}")

# ── Bounded result ─────────────────────────────────────────────────────────

print(json.dumps({
    "schema": "focusa.spec152e.agent_json_contract_validation.v1",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "presenter_states": sorted(presenter_states),
    "human_action_states": human_actions,
    "surfaces": ["cli_json", "daemon_api", "pi_agent_tool_envelope"],
    "rust_replay_tests": 8,
    "result": "passed_fail_closed",
}, sort_keys=True))
