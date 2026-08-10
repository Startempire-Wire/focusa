// Spec 152E §14.2 Pi/agent tool envelope surface.
//
// Builds the agent-safe activation envelope (`focusa.agent_activation_envelope.v1`)
// from daemon/API activation-status payloads. It masks email and key by
// default, returns typed human-action states, carries safe checkout /
// verification links, exposes the resumable registration handle, and enforces
// the explicit customer-controlled key-reveal gate. It never invents an
// email, verification code, consent, payment confirmation, or license, and it
// never advances a human-required state itself.
//
// Deterministic and dependency-free so tests can import it directly.
export const AGENT_ENVELOPE_SCHEMA = "focusa.agent_activation_envelope.v1";
const TERMINAL_STATES = new Set(["activated", "denied", "recovery_only"]);
/** Typed human action for a presenter state; terminal states have none. */
export function humanActionForState(state) {
    switch (state) {
        case "email_required":
            return "provide_email";
        case "email_verification_pending":
            return "enter_verification_code";
        case "email_verified":
            return "select_offer";
        case "selection_required":
            return "select_offer";
        case "checkout_required":
            return "open_checkout_url";
        case "payment_pending":
            return "complete_payment_then_poll";
        case "license_delivery_ready":
            return "reveal_or_accept_license";
        default:
            return null;
    }
}
/** True when the human (not the agent) must act. Unknown states fail closed. */
export function humanActionRequired(state) {
    return humanActionForState(state) !== null || !TERMINAL_STATES.has(state);
}
/** Mask an email to `c***@example.com`; unmaskable input fails closed. */
export function maskEmail(email) {
    const trimmed = email.trim();
    const at = trimmed.indexOf("@");
    if (at <= 0 || at === trimmed.length - 1)
        return null;
    const local = trimmed.slice(0, at);
    const domain = trimmed.slice(at + 1);
    if (/\s/.test(local) || domain.includes("@") || /\s/.test(domain) || !domain.includes(".")) {
        return null;
    }
    return `${local.slice(0, 1)}***@${domain}`;
}
/** Mask a full key to its prefix group followed by `-XXXX` groups. */
export function maskKeyPrefix(fullKey) {
    const trimmed = fullKey.trim();
    if (!trimmed)
        return "XXXX-XXXX-XXXX-XXXX";
    const [head, ...rest] = trimmed.split("-");
    const groups = Math.max(rest.length, 1);
    return [head, ...Array(groups).fill("XXXX")].join("-");
}
/** Explicit customer-controlled reveal: both opt-in and confirmation needed. */
export function revealAuthorized(revealKey, revealConfirmation) {
    return revealKey === true && revealConfirmation === true;
}
/** Build the agent envelope; key material stays masked by default. */
export function buildAgentActivationEnvelope(input) {
    const humanAction = humanActionForState(input.state);
    const reveal = revealAuthorized(Boolean(input.reveal_key), Boolean(input.reveal_confirmation));
    const keyVisible = input.key_present && reveal;
    const envelope = {
        schema: AGENT_ENVELOPE_SCHEMA,
        registration_id: input.registration_id,
        state: input.state,
        terminal: Boolean(input.terminal),
        human_action_required: humanActionRequired(input.state),
        key_present: Boolean(input.key_present),
        key_visible: keyVisible,
        poll_count: input.poll_count,
        max_polls: input.max_polls,
        retry_posture: input.retry_posture ?? "none",
        // When a human action is required the agent's next action IS the typed
        // human action; the canonical presenter next action would invite the
        // agent to advance a human-required state itself.
        next_action: humanAction ?? input.next_action ?? (input.terminal ? input.state : "hand_off_to_human"),
    };
    if (humanAction)
        envelope.human_action = humanAction;
    if (input.masked_email)
        envelope.masked_email = input.masked_email;
    if (input.safe_url)
        envelope.safe_url = input.safe_url;
    if (input.retry_after_seconds != null)
        envelope.retry_after_seconds = input.retry_after_seconds;
    if (input.error)
        envelope.error = input.error;
    if (keyVisible && input.full_key_prefix) {
        envelope.masked_key_prefix = maskKeyPrefix(input.full_key_prefix);
    }
    return envelope;
}
/** Summarize an envelope for a one-line agent-facing status line. */
export function summarizeEnvelope(envelope) {
    if (envelope.terminal)
        return `activation settled: ${envelope.state}`;
    const action = envelope.human_action ?? "human_action";
    const url = envelope.safe_url ? ` (${envelope.safe_url})` : "";
    return `human action required: ${action}${url}; resume handle ${envelope.registration_id}`;
}
