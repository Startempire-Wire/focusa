/**
 * focusa-facade-policy-presenter.mjs
 *
 * Spec 152F facade policy presenter contract for branded facade pages and the
 * Focusa website presenters (§7 branded-facade row; P5, P9; §11 conversion
 * principles). Exact verification:
 *   python3 tests/spec152f_facade_policy_presenter_test.py
 *
 * Every registered branded facade binds to the SAME projection of the
 * canonical authority decision: the capability family (base product or one of
 * the four premium families), the Evaluation/purchase/recovery action, and a
 * safe masked status from authority output. A facade cannot select grants,
 * prices, feature activation, or runtime policy, and it cannot turn dormant
 * future-granularity or premium dimensions on or off — the projection has no
 * such fields, exposes no setters, and fails closed on any spoofed envelope.
 * Exact-origin, session, and redirect rules remain Spec 152E-owned
 * (facade registry, facade security, activation call stack).
 */

const REGISTERED_FACADES = Object.freeze([
  'focusa_install_v1',
  'focusa_marketing_v1',
  'focusa_forge_v1',
  'focusa_arena_v1',
  'uiai_engine_v1',
  'wpuiai_public_v1',
]);

/** Canonical facade-presentable family labels. Internal maintenance has no
 * facade-presentable family (it inherits the initiating operation's family,
 * resolved by the authority, never by a page). */
const FACADE_FAMILIES = Object.freeze({
  base_focusa: 'Base Focusa',
  automation: 'Automation',
  team_remote: 'Team and remote',
  release_proof: 'Release proof',
  premium_updates: 'Premium updates',
  always_reachable: 'Always reachable',
});

/** Canonical commercial postures (Spec 172 overlay vocabulary). */
const FACADE_POSTURES = Object.freeze(['allow', 'read', 'base', 'feature', 'deny']);

/** Authority status labels a facade may present as its masked status; unknown
 * or spoofed status strings fail closed (never rendered verbatim). */
const FACADE_STATUS_ALLOWLIST = Object.freeze([
  'pending_unverified',
  'verified_no_license',
  'active_paid',
  'offline_grace',
  'expired',
  'refunded_or_revoked',
  'missing_or_corrupt',
  'email_verification_pending',
  'email_verified',
  'selection_required',
  'checkout_required',
  'payment_pending',
  'license_delivery_ready',
  'activated',
  'denied',
  'recovery_only',
]);

/** Exactly what a branded facade may present. Anything else is a
 * presenter-owned commercial decision and is forbidden by construction. */
const FACADE_PRESENTER_FIELDS = Object.freeze([
  'family',
  'posture',
  'action',
  'action_label',
  'explanation',
  'recovery_action',
  'masked_status',
  'always_reachable',
]);

/** Caller/authority-envelope fields a facade presenter must never accept or
 * emit (Spec 152F P9, §9, §10): no grants, prices, feature activation,
 * runtime policy, dormant granularity dimensions, or product/limit selection. */
const FACADE_PRESENTER_FORBIDDEN_FIELDS = Object.freeze([
  'grants',
  'prices',
  'price',
  'feature_activation',
  'runtime_policy',
  'dormant',
  'product_selection',
  'product_code',
  'limit_bucket',
  'limits',
  'lease',
  'tokens',
  'keys',
  'customer_email',
  'raw_status',
  'redirect_url',
]);

/** Frozen always-reachable surface families (Spec 152F P6) shared with the
 * menubar and TUI presenters. */
const ALWAYS_REACHABLE = Object.freeze([
  'navigation',
  'status',
  'account',
  'read',
  'export',
  'recovery',
  'repair',
  'update',
  'uninstall',
]);

const EXPLANATIONS = Object.freeze({
  base_allow: 'A verified Evaluation or paid Focusa entitlement enables the complete base Focusa value loop.',
  base_read: 'Read-only projection is available for existing local data.',
  base_base: 'A valid Evaluation, Active paid lease, or valid Offline Grace enables the complete base Focusa value loop.',
  base_deny: 'A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work.',
  premium_feature: 'This optional premium family requires an authority-issued entitlement; this branded page cannot grant it.',
  premium_deny: 'This optional premium family is not available in the current entitlement state.',
  always_reachable: 'Account recovery, read, export, repair, and uninstall remain available when execution is locked.',
});

const ACTION_LABELS = Object.freeze({
  evaluate: 'Evaluate Focusa',
  purchase: 'Purchase or renew entitlement',
  recovery: 'Continue recovery',
  manage: 'Manage entitlement',
});

function isRecord(value) {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function maskedEmail(value) {
  if (typeof value !== 'string') return false;
  const at = value.lastIndexOf('@');
  if (at <= 0 || at === value.length - 1) return false;
  const local = value.slice(0, at);
  const domain = value.slice(at + 1);
  if (!local.includes('*') || domain.includes('@') || domain.includes('*') || domain.includes(' ')) {
    return false;
  }
  return true;
}

/** Canonical posture-to-action projection (mirrors the Rust contract
 * `facade_next_action_for_posture` and the frozen menubar/TUI action
 * vocabulary). Deterministic for every facade — never re-decided per page.
 * `status` only overrides to the recovery action when the authority reports
 * recovery_only. */
function actionFor(posture, status) {
  if (status === 'recovery_only') return 'recovery';
  if (posture === 'deny') return 'evaluate';
  if (posture === 'feature') return 'purchase';
  return 'manage';
}

function explanationFor(family, posture) {
  if (family === 'base_focusa') {
    if (posture === 'allow') return EXPLANATIONS.base_allow;
    if (posture === 'read') return EXPLANATIONS.base_read;
    if (posture === 'base') return EXPLANATIONS.base_base;
    return EXPLANATIONS.base_deny;
  }
  if (Object.hasOwn(FACADE_FAMILIES, family) && family !== 'always_reachable') {
    return posture === 'feature' ? EXPLANATIONS.premium_feature : EXPLANATIONS.premium_deny;
  }
  if (family === 'always_reachable') return EXPLANATIONS.always_reachable;
  throw new Error('FACADE_POLICY_DENIED');
}

/**
 * Project one canonical authority decision envelope into the frozen
 * facade-presentable view. `facadeId` only gates the registered origin; the
 * projection itself is facade-independent, so every facade explains the same
 * authority decision for the same envelope.
 *
 * Allowed envelope keys: family, posture, reason, status, masked_email,
 * next_action. Any other key — in particular any forbidden commercial or
 * dormant-granularity selector — fails closed with FACADE_POLICY_DENIED.
 */
export function projectFacadePolicyDecision(facadeId, envelope) {
  if (!REGISTERED_FACADES.includes(facadeId)) throw new Error('FACADE_ORIGIN_DENIED');
  if (!isRecord(envelope)) throw new Error('FACADE_POLICY_DENIED');

  const envelopeKeys = Object.keys(envelope);
  if (envelopeKeys.some((key) => FACADE_PRESENTER_FORBIDDEN_FIELDS.includes(key))) {
    throw new Error('FACADE_POLICY_DENIED');
  }
  const allowedKeys = ['family', 'posture', 'reason', 'status', 'masked_email', 'next_action'];
  if (envelopeKeys.some((key) => !allowedKeys.includes(key))) {
    throw new Error('FACADE_POLICY_DENIED');
  }

  const family = envelope.family;
  if (typeof family !== 'string' || !Object.hasOwn(FACADE_FAMILIES, family)) {
    throw new Error('FACADE_POLICY_DENIED');
  }
  const posture = envelope.posture;
  if (typeof posture !== 'string' || !FACADE_POSTURES.includes(posture)) {
    throw new Error('FACADE_POLICY_DENIED');
  }
  const reason = typeof envelope.reason === 'string' ? envelope.reason : '';
  const status = typeof envelope.status === 'string' &&
    FACADE_STATUS_ALLOWLIST.includes(envelope.status) ? envelope.status : undefined;
  const masked = typeof envelope.masked_email === 'string' && maskedEmail(envelope.masked_email)
    ? envelope.masked_email
    : undefined;
  if (typeof envelope.masked_email === 'string' && masked === undefined) {
    // A raw or unmaskable identity is never rendered.
    throw new Error('FACADE_POLICY_DENIED');
  }

  const action = actionFor(posture, status);
  // Unknown/spoofed statuses never render: the masked-status region exists
  // only when the authority status label is recognized (fail closed).
  const masked_status = status === undefined
    ? undefined
    : Object.freeze({ status, masked_email: masked });
  return Object.freeze({
    family,
    posture,
    action,
    action_label: ACTION_LABELS[action],
    explanation: explanationFor(family, posture),
    recovery_action: typeof envelope.next_action === 'string' && envelope.next_action.length <= 240
      ? envelope.next_action
      : 'license_status',
    masked_status,
    always_reachable: ALWAYS_REACHABLE,
  });
}

/** Frozen contract artifact that tests and pages bind against. */
export const facadePolicyContract = Object.freeze({
  schema: 'focusa.spec152f.facade_policy_presenter.v1',
  authority: 'WPUIAI.com EDD',
  role: 'presenter_only',
  facades: REGISTERED_FACADES,
  fields: FACADE_PRESENTER_FIELDS,
  forbidden_fields: FACADE_PRESENTER_FORBIDDEN_FIELDS,
  families: Object.freeze(Object.keys(FACADE_FAMILIES)),
  postures: FACADE_POSTURES,
  status_allowlist: FACADE_STATUS_ALLOWLIST,
  always_reachable: ALWAYS_REACHABLE,
  invariants: [
    'every facade explains the same authority decision',
    'facades display base/premium family, Evaluation/purchase/recovery action, and safe masked status only',
    'facades cannot select grants, prices, feature activation, or runtime policy',
    'facades cannot turn dormant or premium fields on or off',
    'presenters project the authority decision; they never re-decide it',
    'exact-origin, session, and redirect rules remain Spec 152E-owned',
    'no raw keys, tokens, or customer PII in presenter output',
  ],
});

if (typeof globalThis !== 'undefined' && globalThis.customElements &&
    !globalThis.customElements.get('focusa-facade-policy-status')) {
  // Optional accessible status region for branded pages that include the
  // policy presenter. Presentation only; it never reads envelope internals
  // and carries no commercial decision.
  globalThis.customElements.define('focusa-facade-policy-status', class extends HTMLElement {
    connectedCallback() {
      const note = document.createElement('p');
      note.id = 'facade-policy-status';
      note.role = 'status';
      note.setAttribute('aria-live', 'polite');
      note.textContent =
        'This branded page presents the authority decision only. It cannot grant, price, or activate entitlement.';
      this.replaceChildren(note);
    }
  });
}
