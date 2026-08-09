import {
  allowedActionsFor,
  presenterNextAction,
  presenterStateForEntitlementStatus,
  type PresenterState,
} from './activationPresenter.ts';

export type EntitlementVisualState =
  | 'active'
  | 'offline_grace'
  | 'expired'
  | 'revoked'
  | 'invalid'
  | 'unactivated';

export interface LicenseStatusPayload {
  status?: string;
  masked_identity?: string | null;
  expires_at?: string | null;
  authority?: {
    state?: string;
    expires_at?: string | null;
    offline_grace_until?: string | null;
    recovery_reason?: string | null;
    limits?: Record<string, number>;
  } | null;
  capabilities?: Array<{
    capability?: string;
    outcome?: string;
    reason?: string | null;
  }>;
}

export interface EntitlementPosture {
  state: EntitlementVisualState;
  /** Shared Spec 152E presenter state (same vocabulary as TUI/REST/lifecycle
   * receipts for the same entitlement — never a duplicate business decision). */
  presenter_state: PresenterState;
  next_action: string;
  allowed_actions: string[];
  masked_identity?: string;
  expires_at?: string;
  offline_grace_until?: string;
  limits: Array<{ name: string; remaining: number }>;
  locked_capabilities: Array<{ name: string; reason: string }>;
  action: 'evaluate' | 'refresh' | 'manage' | 'purchase';
  recovery_policy: string;
  marketing_preference: 'managed_separately';
}

export function projectEntitlementPosture(
  payload: LicenseStatusPayload,
): EntitlementPosture {
  const state = visualState(payload);
  const presenter_state = presenterStateForEntitlementStatus(
    String(payload.status ?? ''),
  );
  const limits = Object.entries(payload.authority?.limits ?? {})
    .filter(([, value]) => Number.isSafeInteger(value) && value >= 0)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([name, remaining]) => ({ name, remaining }));
  const locked_capabilities = (payload.capabilities ?? [])
    .filter(
      (capability) =>
        capability.outcome === 'denied' && Boolean(capability.capability),
    )
    .map((capability) => ({
      name: capability.capability!,
      reason: boundedReason(capability.reason),
    }))
    .sort((left, right) => left.name.localeCompare(right.name));
  return {
    state,
    presenter_state,
    next_action: presenterNextAction(presenter_state),
    allowed_actions: allowedActionsFor(presenter_state),
    masked_identity: safeMaskedIdentity(payload.masked_identity),
    expires_at: safeTime(payload.authority?.expires_at ?? payload.expires_at),
    offline_grace_until: safeTime(payload.authority?.offline_grace_until),
    limits,
    locked_capabilities,
    action: actionFor(state, locked_capabilities.length),
    recovery_policy:
      'Recovery, export, repair, and uninstall remain available when execution is locked.',
    marketing_preference: 'managed_separately',
  };
}

function visualState(payload: LicenseStatusPayload): EntitlementVisualState {
  const status = String(payload.status ?? '').toLowerCase();
  if (status === 'active') return 'active';
  if (status === 'offline_grace') return 'offline_grace';
  if (status === 'expired') return 'expired';
  const reason = String(payload.authority?.recovery_reason ?? '').toLowerCase();
  if (reason.includes('revok')) return 'revoked';
  if (status === 'recovery_only')
    return reason.includes('expir') ? 'expired' : 'invalid';
  return 'unactivated';
}

function actionFor(
  state: EntitlementVisualState,
  lockedCount: number,
): EntitlementPosture['action'] {
  if (state === 'unactivated') return 'evaluate';
  if (state === 'offline_grace' || state === 'invalid') return 'refresh';
  if (state === 'expired' || state === 'revoked') return 'purchase';
  return lockedCount > 0 ? 'manage' : 'manage';
}

function safeMaskedIdentity(value?: string | null): string | undefined {
  if (!value || !/^[^@]*\*+[^@]*@[^@]+$/.test(value)) return undefined;
  return value.slice(0, 160);
}

function safeTime(value?: string | null): string | undefined {
  return value && Number.isFinite(Date.parse(value)) ? value : undefined;
}

function boundedReason(value?: string | null): string {
  return (value || 'not_granted')
    .replace(/[^a-zA-Z0-9_.:-]/g, '_')
    .slice(0, 120);
}
