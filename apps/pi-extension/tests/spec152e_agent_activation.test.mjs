// Spec 152E.05.05 Pi/agent tool envelope — deterministic activation protocol test.
//
// Binds apps/pi-extension/src/activation-envelope.ts (the Pi/agent tool
// envelope surface, Spec 152E §14.2) to the frozen agent contract
// (docs/contracts/spec152e-agent-activation.v1.json) and to the daemon/API
// route it consumes (crates/focusa-api/src/routes/license.rs
// `GET /v1/activation/status`). The envelope returns typed human-action
// states, masked email/key by default, safe checkout/verification links,
// bounded poll/resume, explicit customer-controlled key reveal, and a
// resumable registration handle; it never invents consent, payment, or
// identity, and never advances a human-required state.
//
// Exact verification: node apps/pi-extension/tests/spec152e_agent_activation.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const moduleSource = readFileSync(
  fileURLToPath(new URL("../src/activation-envelope.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(moduleSource, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const envelopeModule = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const { buildAgentActivationEnvelope, maskEmail, maskKeyPrefix, revealAuthorized, humanActionForState } =
  envelopeModule;

// Frozen contract binding (docs/contracts/spec152e-agent-activation.v1.json).
const root = fileURLToPath(new URL("../../..", import.meta.url));
const agentContract = JSON.parse(
  readFileSync(`${root}/docs/contracts/spec152e-agent-activation.v1.json`, "utf8")
);
const internalContract = JSON.parse(
  readFileSync(`${root}/docs/contracts/spec152e-activation-internal.v1.json`, "utf8")
);
const daemonRoute = readFileSync(`${root}/crates/focusa-api/src/routes/license.rs`, "utf8");

assert.equal(agentContract.schema, "focusa.spec152e.agent_activation.v1");
assert.equal(agentContract.envelope.schema, envelopeModule.AGENT_ENVELOPE_SCHEMA);
assert.equal(envelopeModule.AGENT_ENVELOPE_SCHEMA, "focusa.agent_activation_envelope.v1");
assert.deepEqual(
  new Set(agentContract.presenter_states),
  new Set(internalContract.presenter_states),
  "agent presenter states must match the frozen internal contract"
);

// ── Typed human-action states ─────────────────────────────────────────────
const terminalStates = new Set(["activated", "denied", "recovery_only"]);
for (const [state, action] of Object.entries(agentContract.human_action_states)) {
  assert.equal(humanActionForState(state), action, `${state} -> ${action}`);
}
for (const state of terminalStates) {
  assert.equal(humanActionForState(state), null, `${state} requires no human action`);
}
assert.equal(humanActionForState("unknown_state"), null, "unknown states have no typed action");
assert.equal(
  envelopeModule.humanActionRequired("unknown_state"),
  true,
  "unknown states fail closed as human-required"
);
assert.equal(envelopeModule.humanActionRequired("payment_pending"), true);
assert.equal(envelopeModule.humanActionRequired("recovery_only"), false);

// ── Secret masking by default ─────────────────────────────────────────────
const masked = maskEmail("customer@example.com");
assert.equal(masked, "c***@example.com");
assert.equal(maskEmail("not-an-email"), null);
assert.equal(maskEmail(""), null);
assert.equal(maskKeyPrefix("FOCUSA-ABCD-EFGH-IJKL-MNOP"), "FOCUSA-XXXX-XXXX-XXXX-XXXX");
assert.equal(maskKeyPrefix(""), "XXXX-XXXX-XXXX-XXXX");

// Key masked by default: key_present true, key_visible false, no prefix.
const maskedEnvelope = buildAgentActivationEnvelope({
  registration_id: "registration-0001",
  state: "license_delivery_ready",
  terminal: false,
  masked_email: "c***@example.com",
  key_present: true,
  poll_count: 1,
  max_polls: 40,
  reveal_key: false,
  reveal_confirmation: false,
});
assert.equal(maskedEnvelope.schema, "focusa.agent_activation_envelope.v1");
assert.equal(maskedEnvelope.key_present, true);
assert.equal(maskedEnvelope.key_visible, false);
assert.equal(maskedEnvelope.masked_key_prefix, undefined);
assert.equal(maskedEnvelope.human_action_required, true);
assert.equal(maskedEnvelope.human_action, "reveal_or_accept_license");
assert.equal(maskedEnvelope.next_action, "reveal_or_accept_license");
assert.equal(maskedEnvelope.registration_id, "registration-0001");
const maskedBody = JSON.stringify(maskedEnvelope);
assert.equal(maskedBody.includes("customer@example.com"), false, "no raw email");
assert.equal(maskedBody.includes("full_license_key"), false);
assert.equal(maskedBody.includes("one_time_key_envelope"), false);
assert.equal(maskedBody.includes("poll_credential"), false);
assert.equal(maskedBody.includes("card_"), false);

// ── Explicit customer-controlled key reveal ───────────────────────────────
assert.equal(revealAuthorized(false, false), false);
assert.equal(revealAuthorized(true, false), false);
assert.equal(revealAuthorized(false, true), false);
assert.equal(revealAuthorized(true, true), true);

const revealedEnvelope = buildAgentActivationEnvelope({
  registration_id: "registration-0001",
  state: "license_delivery_ready",
  terminal: false,
  masked_email: "c***@example.com",
  key_present: true,
  poll_count: 1,
  max_polls: 40,
  reveal_key: true,
  reveal_confirmation: true,
  full_key_prefix: "FOCUSA-ABCD-EFGH-IJKL-MNOP",
});
assert.equal(revealedEnvelope.key_visible, true, "explicit opt-in + confirmation reveals");
assert.equal(revealedEnvelope.masked_key_prefix, "FOCUSA-XXXX-XXXX-XXXX-XXXX");
const revealedBody = JSON.stringify(revealedEnvelope);
assert.equal(revealedBody.includes("FOCUSA-ABCD-EFGH-IJKL-MNOP"), false, "reveal is prefix-masked only");
assert.equal(revealedBody.includes("one_time_key_envelope"), false, "envelope never in transcript");

// ── Bounded poll / resumable handle ───────────────────────────────────────
const paymentEnvelope = buildAgentActivationEnvelope({
  registration_id: "registration-0002",
  state: "payment_pending",
  terminal: false,
  masked_email: "c***@example.com",
  safe_url: "https://install.focusa.dev/pay/opaque-token",
  key_present: false,
  poll_count: 3,
  max_polls: 40,
  retry_posture: "safe_retry",
  retry_after_seconds: 3,
});
assert.equal(paymentEnvelope.human_action, "complete_payment_then_poll");
assert.equal(paymentEnvelope.next_action, "complete_payment_then_poll");
assert.equal(paymentEnvelope.safe_url, "https://install.focusa.dev/pay/opaque-token");
assert.equal(paymentEnvelope.poll_count, 3);
assert.equal(paymentEnvelope.max_polls, 40);
assert.equal(paymentEnvelope.retry_posture, "safe_retry");
assert.equal(paymentEnvelope.retry_after_seconds, 3);
assert.equal(paymentEnvelope.registration_id, "registration-0002");

// Terminal envelopes carry no human action and keep the canonical next action.
const terminalEnvelope = buildAgentActivationEnvelope({
  registration_id: "registration-0003",
  state: "recovery_only",
  terminal: true,
  key_present: false,
  poll_count: 0,
  max_polls: 40,
  next_action: "recovery",
});
assert.equal(terminalEnvelope.terminal, true);
assert.equal(terminalEnvelope.human_action_required, false);
assert.equal(terminalEnvelope.human_action, undefined);
assert.equal(terminalEnvelope.next_action, "recovery");

// ── Daemon/API surface it consumes ────────────────────────────────────────
assert.match(daemonRoute, /"\/v1\/activation\/status"/, "daemon exposes /v1/activation/status");
assert.match(daemonRoute, /resumable_handles/, "daemon returns resumable registration handles");
assert.match(daemonRoute, /AgentActivationEnvelope::from_registration/, "daemon projects agent envelopes");
assert.match(daemonRoute, /poll_credential_present.: false/, "daemon asserts no poll credential");
assert.match(daemonRoute, /raw_email_present.: false/, "daemon asserts no raw email");

// ── Forbidden in the module surface ───────────────────────────────────────
for (const forbidden of [
  "full_license_key",
  "one_time_key_envelope",
  "lease_envelope",
  "poll_credential",
  "verification_hash",
  "server_credential",
  "signing_key",
  "card_pan",
  "card_expiry",
  "card_cvc",
]) {
  assert.equal(
    new RegExp(`\\b${forbidden}\\b`).test(moduleSource),
    false,
    `module surface must not declare forbidden field ${forbidden}`
  );
}

console.log("Spec 152E agent activation envelope test passed");
