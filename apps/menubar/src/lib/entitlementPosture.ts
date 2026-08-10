import {
  allowedActionsFor,
  presenterNextAction,
  presenterStateForEntitlementStatus,
  type PresenterState,
} from './activationPresenter';

export type EntitlementVisualState =
  | 'active'
  | 'offline_grace'
  | 'expired'
  | 'revoked'
  | 'invalid'
  | 'unactivated';

/** Always-reachable surface families (Spec 152F P6 / §11.5, §13):
 * navigation, status, account, read, export, recovery, repair, update, and
 * uninstall are never disabled by a denied entitlement decision. Frozen
 * fixture shared with the TUI presenter
 * (crates/focusa-tui/src/activation_presenter.rs `ALWAYS_REACHABLE`) and
 * the menubar action map
 * (docs/contracts/spec152f-menubar-action-map.v1.json
 * `accessibility_fixtures.always_reachable`). */
export const ALWAYS_REACHABLE_ACTIONS = [
  'navigation',
  'status',
  'account',
  'read',
  'export',
  'recovery',
  'repair',
  'update',
  'uninstall',
] as const;

export type AlwaysReachableAction = (typeof ALWAYS_REACHABLE_ACTIONS)[number];

export interface ActionGuide {
  /** Same decision vocabulary as the posture `action` field. */
  action: 'evaluate' | 'refresh' | 'manage' | 'purchase';
  label: string;
  explanation: string;
  recovery_actions: readonly AlwaysReachableAction[];
}

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
  action_guide: ActionGuide;
  always_reachable: readonly AlwaysReachableAction[];
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
    action_guide: actionGuideFor(state),
    always_reachable: ALWAYS_REACHABLE_ACTIONS,
    recovery_policy:
      'Recovery, export, repair, and uninstall remain available when execution is locked.',
    marketing_preference: 'managed_separately',
  };
}

/** Frozen next-action guide (Spec 152F §11 conversion principles, P6): a
 * denied or missing value action always carries an accessible explanation
 * and an Evaluation/purchase/recovery action, and no presenter disables
 * it. This is a rendering projection of the visual state only — it never
 * adds commercial decisions or re-decides the canonical entitlement
 * decision. */
function actionGuideFor(state: EntitlementVisualState): ActionGuide {
  switch (state) {
    case 'unactivated':
      return {
        action: 'evaluate',
        label: 'Evaluate Focusa',
        explanation:
          'No signed entitlement is active. A card-free Evaluation demonstrates the complete base Focusa value loop.',
        recovery_actions: ALWAYS_REACHABLE_ACTIONS,
      };
    case 'offline_grace':
    case 'invalid':
      return {
        action: 'refresh',
        label: 'Refresh entitlement',
        explanation:
          'The signed lease is in a refreshable state. Refresh through the authority account or focusa license.',
        recovery_actions: ALWAYS_REACHABLE_ACTIONS,
      };
    case 'expired':
    case 'revoked':
      return {
        action: 'purchase',
        label: 'Purchase or renew entitlement',
        explanation:
          'The lease is expired or revoked. Purchase/renewal continues the same customer, project, node, and data state.',
        recovery_actions: ALWAYS_REACHABLE_ACTIONS,
      };
    default:
      return {
        action: 'manage',
        label: 'Manage entitlement',
        explanation:
          'The entitlement is usable. Manage nodes, refresh the lease, or manage the account through the authority.',
        recovery_actions: ALWAYS_REACHABLE_ACTIONS,
      };
  }
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
  _lockedCount: number,
): EntitlementPosture['action'] {
  if (state === 'unactivated') return 'evaluate';
  if (state === 'offline_grace' || state === 'invalid') return 'refresh';
  if (state === 'expired' || state === 'revoked') return 'purchase';
  return 'manage';
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
