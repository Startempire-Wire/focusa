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

/** Typed human action an agent must hand to the human (Spec 152E §14.2). */
export type HumanActionLabel =
  | "provide_email"
  | "enter_verification_code"
  | "select_offer"
  | "open_checkout_url"
  | "complete_payment_then_poll"
  | "reveal_or_accept_license";

/** Presenter-safe agent envelope. No raw email, key, credential, or card
 * data field exists by construction. */
export interface AgentActivationEnvelopeV1 {
  schema: string;
  /** Resumable registration handle: return this id so a later invocation can
   * resume bounded polling from the protected store. */
  registration_id: string;
  state: string;
  terminal: boolean;
  /** True when a human must act before the agent may resume. */
  human_action_required: boolean;
  human_action?: HumanActionLabel;
  /** Masked email (e.g. `c***@example.com`); never the raw address. */
  masked_email?: string;
  /** Authority-owned branded checkout/verification link. */
  safe_url?: string;
  key_present: boolean;
  /** False by default; true only after explicit customer opt-in AND
   * confirmation of the one-time reveal. */
  key_visible: boolean;
  /** Masked key prefix (e.g. `FOCUSA-XXXX-XXXX-XXXX`), only under authorized
   * reveal from caller-held key knowledge. */
  masked_key_prefix?: string;
  poll_count: number;
  max_polls: number;
  retry_posture: string;
  retry_after_seconds?: number;
  next_action: string;
  error?: { code: string; next_action: string };
}

/** Inputs accepted from daemon/API activation-status payloads. */
export interface ActivationEnvelopeInput {
  registration_id: string;
  state: string;
  terminal: boolean;
  masked_email?: string | null;
  safe_url?: string | null;
  key_present: boolean;
  poll_count: number;
  max_polls: number;
  retry_posture?: string;
  retry_after_seconds?: number | null;
  next_action?: string | null;
  error?: { code: string; next_action: string } | null;
  /** Explicit customer-controlled reveal opt-in (Spec 152E §14.2). */
  reveal_key?: boolean;
  /** Explicit confirmation required alongside the opt-in. */
  reveal_confirmation?: boolean;
  /** Caller-held key knowledge (full key prefix), only used under reveal. */
  full_key_prefix?: string | null;
}

const TERMINAL_STATES = new Set(["activated", "denied", "recovery_only"]);

/** Typed human action for a presenter state; terminal states have none. */
export function humanActionForState(state: string): HumanActionLabel | null {
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
export function humanActionRequired(state: string): boolean {
  return humanActionForState(state) !== null || !TERMINAL_STATES.has(state);
}

/** Mask an email to `c***@example.com`; unmaskable input fails closed. */
export function maskEmail(email: string): string | null {
  const trimmed = email.trim();
  const at = trimmed.indexOf("@");
  if (at <= 0 || at === trimmed.length - 1) return null;
  const local = trimmed.slice(0, at);
  const domain = trimmed.slice(at + 1);
  if (/\s/.test(local) || domain.includes("@") || /\s/.test(domain) || !domain.includes(".")) {
    return null;
  }
  return `${local.slice(0, 1)}***@${domain}`;
}

/** Mask a full key to its prefix group followed by `-XXXX` groups. */
export function maskKeyPrefix(fullKey: string): string {
  const trimmed = fullKey.trim();
  if (!trimmed) return "XXXX-XXXX-XXXX-XXXX";
  const [head, ...rest] = trimmed.split("-");
  const groups = Math.max(rest.length, 1);
  return [head, ...Array(groups).fill("XXXX")].join("-");
}

/** Explicit customer-controlled reveal: both opt-in and confirmation needed. */
export function revealAuthorized(revealKey: boolean, revealConfirmation: boolean): boolean {
  return revealKey === true && revealConfirmation === true;
}

/** Build the agent envelope; key material stays masked by default. */
export function buildAgentActivationEnvelope(input: ActivationEnvelopeInput): AgentActivationEnvelopeV1 {
  const humanAction = humanActionForState(input.state);
  const reveal = revealAuthorized(Boolean(input.reveal_key), Boolean(input.reveal_confirmation));
  const keyVisible = input.key_present && reveal;
  const envelope: AgentActivationEnvelopeV1 = {
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
  if (humanAction) envelope.human_action = humanAction;
  if (input.masked_email) envelope.masked_email = input.masked_email;
  if (input.safe_url) envelope.safe_url = input.safe_url;
  if (input.retry_after_seconds != null) envelope.retry_after_seconds = input.retry_after_seconds;
  if (input.error) envelope.error = input.error;
  if (keyVisible && input.full_key_prefix) {
    envelope.masked_key_prefix = maskKeyPrefix(input.full_key_prefix);
  }
  return envelope;
}

/** Summarize an envelope for a one-line agent-facing status line. */
export function summarizeEnvelope(envelope: AgentActivationEnvelopeV1): string {
  if (envelope.terminal) return `activation settled: ${envelope.state}`;
  const action = envelope.human_action ?? "human_action";
  const url = envelope.safe_url ? ` (${envelope.safe_url})` : "";
  return `human action required: ${action}${url}; resume handle ${envelope.registration_id}`;
}
