// Menubar first-run/entitlement presenter (Spec 152E §21 surface
// consolidation).
//
// Renders the SAME shared activation states, next actions, allowed actions,
// masked identity, checkout/verify links, terminal delivery, node management,
// denial/recovery, and resume handles as the TUI, the daemon REST license
// routes, and lifecycle receipts for one canonical registration. It never
// reimplements identity, product, payment, Evaluation, license, node, or
// lease decisions: the shared reducer in focusa-license remains the only
// decision authority and this module only projects daemon payloads onto the
// frozen presenter-state vocabulary
// (docs/contracts/spec152e-activation-internal.v1.json `presenter_states`).
// Unknown states, unmasked emails, and non-authority links fail closed.
//
// Deterministic and dependency-free so tests can import it directly.

/** Frozen Spec 152E presenter states (rendering only). */
export const PRESENTER_STATES = [
  'email_required',
  'email_verification_pending',
  'email_verified',
  'selection_required',
  'checkout_required',
  'payment_pending',
  'license_delivery_ready',
  'activated',
  'denied',
  'recovery_only',
] as const;

export type PresenterState = (typeof PRESENTER_STATES)[number];

/** Frozen terminal presenter states (activation polls end there). */
export const TERMINAL_STATES = new Set<string>([
  'activated',
  'denied',
  'recovery_only',
]);

/** Frozen next-action table shared by every presenter. */
export function presenterNextAction(state: PresenterState): string {
  switch (state) {
    case 'email_required':
      return 'provide_email';
    case 'email_verification_pending':
      return 'verify_email';
    case 'email_verified':
      return 'select_offer';
    case 'selection_required':
      return 'select_offer';
    case 'checkout_required':
      return 'open_checkout';
    case 'payment_pending':
      return 'poll_after_retry_after';
    case 'license_delivery_ready':
      return 'deliver_license';
    case 'activated':
      return 'activated';
    case 'denied':
      return 'activate_or_manage_entitlement';
    case 'recovery_only':
      return 'recovery';
  }
}

/** Equivalent allowed actions for a presenter state (shared projection).
 * Rendering guidance only — every action executes through the daemon REST
 * surface, never locally. */
export function allowedActionsFor(state: PresenterState): string[] {
  switch (state) {
    case 'email_required':
      return ['provide_email'];
    case 'email_verification_pending':
      return ['verify_email', 'resend_code'];
    case 'email_verified':
      return ['select_offer'];
    case 'selection_required':
      return [
        'select_purchase',
        'select_limited_access',
        'select_existing_key',
      ];
    case 'checkout_required':
      return ['open_checkout'];
    case 'payment_pending':
      return ['poll', 'open_checkout'];
    case 'license_delivery_ready':
      return ['deliver_license', 'activate'];
    case 'activated':
      return ['resume'];
    case 'denied':
      return ['activate_or_manage_entitlement', 'recovery'];
    case 'recovery_only':
      return ['recovery', 'repair', 'export', 'uninstall'];
  }
}

/** Fail-closed masked-identity check: `^[^@]*\*[^@]*@[^@]+$` (frozen). */
export function isMaskedIdentity(value: string): boolean {
  const at = value.indexOf('@');
  if (at <= 0 || at === value.length - 1) return false;
  const local = value.slice(0, at);
  const domain = value.slice(at + 1);
  if (!domain.includes('.')) return false;
  if (domain.includes('@') || domain.includes('*')) return false;
  const star = local.indexOf('*');
  if (star < 0) return false;
  const head = local.slice(0, star);
  const tail = local.slice(star + 1);
  return head.length > 0 && (tail.length > 0 || local.endsWith('*'));
}

/** Authority-owned safe link: https only and no userinfo credentials. */
export function safeAuthorityLink(value: string): string | null {
  try {
    const parsed = new URL(value);
    if (parsed.protocol !== 'https:') return null;
    if (parsed.username || parsed.password) return null;
    return parsed.toString();
  } catch {
    return null;
  }
}

/** Map the daemon license-status `status` label onto the frozen presenter
 * vocabulary. Identical mapping is enforced in the TUI and daemon REST
 * surfaces and bound by the cross-surface tests. */
export function presenterStateForEntitlementStatus(
  status: string,
): PresenterState {
  switch (status) {
    case 'active':
    case 'offline_grace':
      return 'activated';
    case 'recovery_only':
      return 'recovery_only';
    case 'expired':
    case 'revoked':
      return 'denied';
    default:
      // Unactivated and legacy-migration-only postures re-enter the shared
      // activation flow; they never grant anything locally.
      return 'email_required';
  }
}

export interface MenubarActivationView {
  registration_id: string;
  state: PresenterState;
  terminal: boolean;
  next_action: string;
  actions: string[];
  masked_email?: string;
  safe_url?: string;
  retry_posture: string;
  resume_handle?: string;
}

/** Project one daemon `GET /v1/activation/status` payload into a typed
 * menubar view. Deterministic: the first valid registration snapshot wins.
 * Unknown states, unmasked emails, and non-authority links fail closed. */
export function projectActivationStatus(
  payload: unknown,
): MenubarActivationView | null {
  const registrations = Array.isArray(
    (payload as { registrations?: unknown })?.registrations,
  )
    ? (payload as { registrations: unknown[] }).registrations
    : [];
  for (const registration of registrations) {
    if (!registration || typeof registration !== 'object') continue;
    const record = registration as Record<string, unknown>;
    const state = String(record.state ?? '');
    if (!PRESENTER_STATES.includes(state as PresenterState)) continue;
    const registrationId = String(record.registration_id ?? '').trim();
    if (!registrationId) continue;
    const maskedEmail =
      typeof record.masked_email === 'string' &&
      isMaskedIdentity(record.masked_email)
        ? record.masked_email
        : undefined;
    const safeUrl =
      typeof record.safe_url === 'string'
        ? (safeAuthorityLink(record.safe_url) ?? undefined)
        : undefined;
    const typed = state as PresenterState;
    return {
      registration_id: registrationId,
      state: typed,
      terminal: TERMINAL_STATES.has(typed),
      next_action: presenterNextAction(typed),
      actions: allowedActionsFor(typed),
      retry_posture:
        typeof record.retry_posture === 'string'
          ? record.retry_posture
          : 'none',
      ...(maskedEmail ? { masked_email: maskedEmail } : {}),
      ...(safeUrl ? { safe_url: safeUrl } : {}),
      ...(!TERMINAL_STATES.has(typed) ? { resume_handle: registrationId } : {}),
    };
  }
  return null;
}

/** Project the daemon `GET /v1/license/status` payload into a typed menubar
 * posture; unmasked identities fail closed. */
export function projectLicenseStatus(
  payload: unknown,
): MenubarLicensePosture | null {
  const record = (payload ?? {}) as Record<string, unknown>;
  if (typeof record.status !== 'string') return null;
  const state = presenterStateForEntitlementStatus(record.status);
  const maskedIdentity =
    typeof record.masked_identity === 'string' &&
    isMaskedIdentity(record.masked_identity)
      ? record.masked_identity
      : undefined;
  return {
    presenter_state: state,
    next_action: presenterNextAction(state),
    actions: allowedActionsFor(state),
    ...(maskedIdentity ? { masked_identity: maskedIdentity } : {}),
  };
}

export interface MenubarLicensePosture {
  presenter_state: PresenterState;
  next_action: string;
  actions: string[];
  masked_identity?: string;
}

/** Denial/recovery rendering shared by every presenter: a denied or
 * recovery-only registration exposes recovery actions and never a grant. */
export function denialRecovery(view: MenubarActivationView): {
  denial: boolean;
  recovery_only: boolean;
  recovery_actions: string[];
} {
  return {
    denial: view.state === 'denied',
    recovery_only: view.state === 'recovery_only',
    recovery_actions:
      view.state === 'denied' || view.state === 'recovery_only'
        ? ['recovery', 'repair', 'export', 'uninstall']
        : [],
  };
}
