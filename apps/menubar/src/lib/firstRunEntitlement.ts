export const FIRST_RUN_SCHEMA = 'focusa.first_run_entitlement.v1';
export const FIRST_RUN_STORAGE_KEY = 'focusa_first_run_entitlement_v1';

export type FirstRunStage =
  | 'trust_recovery'
  | 'choice'
  | 'device_code'
  | 'account_pending'
  | 'terms_consent'
  | 'lease_verification'
  | 'optional_uiai'
  | 'pairing'
  | 'project'
  | 'first_workpoint'
  | 'complete';

export type EntitlementChoice = 'evaluate' | 'activate' | 'manage';

export interface SafeDeviceChallenge {
  verification_uri: string;
  user_code: string;
  expires_at: string;
}

export interface AuthorityProjection {
  state: 'unactivated' | 'active' | 'offline_grace' | 'recovery_only';
  product: string;
  sequence?: number;
  signature_verified: boolean;
  channel_granted: boolean;
  terms_accepted: boolean;
  privacy_accepted: boolean;
}

export interface FirstRunEntitlementState {
  schema: typeof FIRST_RUN_SCHEMA;
  stage: FirstRunStage;
  choice?: EntitlementChoice;
  challenge?: SafeDeviceChallenge;
  authority?: AuthorityProjection;
  last_error?: string;
  updated_at: string;
}

export type FirstRunEvent =
  | { type: 'authority_observed'; authority: AuthorityProjection }
  | { type: 'choice_selected'; choice: EntitlementChoice }
  | { type: 'device_challenge'; challenge: SafeDeviceChallenge }
  | { type: 'account_pending' }
  | { type: 'terms_required' }
  | { type: 'verify_lease' }
  | { type: 'skip_optional_uiai' }
  | { type: 'pairing_saved' }
  | { type: 'project_verified' }
  | { type: 'first_workpoint_accepted' }
  | { type: 'failed'; code: string };

export function initialFirstRunState(now = new Date().toISOString()): FirstRunEntitlementState {
  return { schema: FIRST_RUN_SCHEMA, stage: 'trust_recovery', updated_at: now };
}

export function entitlementReady(authority?: AuthorityProjection): boolean {
  return Boolean(
    authority &&
      (authority.state === 'active' || authority.state === 'offline_grace') &&
      authority.product === 'focusa' &&
      authority.sequence &&
      authority.signature_verified &&
      authority.channel_granted &&
      authority.terms_accepted &&
      authority.privacy_accepted,
  );
}

export function advanceFirstRun(
  state: FirstRunEntitlementState,
  event: FirstRunEvent,
  now = new Date().toISOString(),
): FirstRunEntitlementState {
  const next = { ...state, updated_at: now, last_error: undefined };
  switch (event.type) {
    case 'authority_observed':
      next.authority = event.authority;
      next.stage = entitlementReady(event.authority)
        ? 'optional_uiai'
        : event.authority.state === 'recovery_only'
          ? 'trust_recovery'
          : 'choice';
      return next;
    case 'choice_selected':
      if (state.stage !== 'choice' && state.stage !== 'trust_recovery') return state;
      next.choice = event.choice;
      next.stage = event.choice === 'manage' && entitlementReady(state.authority)
        ? 'lease_verification'
        : 'device_code';
      return next;
    case 'device_challenge':
      if (state.stage !== 'device_code') return state;
      next.challenge = safeChallenge(event.challenge);
      return next;
    case 'account_pending':
      if (state.stage !== 'device_code') return state;
      next.stage = 'account_pending';
      return next;
    case 'terms_required':
      if (!['device_code', 'account_pending'].includes(state.stage)) return state;
      next.stage = 'terms_consent';
      return next;
    case 'verify_lease':
      if (!['device_code', 'account_pending', 'terms_consent'].includes(state.stage)) return state;
      next.stage = 'lease_verification';
      return next;
    case 'skip_optional_uiai':
      if (state.stage !== 'optional_uiai' || !entitlementReady(state.authority)) return state;
      next.stage = 'pairing';
      return next;
    case 'pairing_saved':
      if (state.stage !== 'pairing' || !entitlementReady(state.authority)) return state;
      next.stage = 'project';
      return next;
    case 'project_verified':
      if (state.stage !== 'project' || !entitlementReady(state.authority)) return state;
      next.stage = 'first_workpoint';
      return next;
    case 'first_workpoint_accepted':
      if (state.stage !== 'first_workpoint' || !entitlementReady(state.authority)) return state;
      next.stage = 'complete';
      return next;
    case 'failed':
      return { ...next, last_error: boundedCode(event.code) };
  }
}

export function serializeFirstRunState(state: FirstRunEntitlementState): string {
  return JSON.stringify({
    schema: FIRST_RUN_SCHEMA,
    stage: state.stage,
    choice: state.choice,
    challenge: state.challenge ? safeChallenge(state.challenge) : undefined,
    authority: state.authority,
    last_error: state.last_error ? boundedCode(state.last_error) : undefined,
    updated_at: state.updated_at,
  });
}

export function restoreFirstRunState(raw: string | null): FirstRunEntitlementState {
  if (!raw) return initialFirstRunState();
  try {
    const value = JSON.parse(raw) as Partial<FirstRunEntitlementState> & Record<string, unknown>;
    if (value.schema !== FIRST_RUN_SCHEMA || !isStage(value.stage)) return initialFirstRunState();
    const serialized = JSON.stringify(value).toLowerCase();
    if (/"(email|access_token|refresh_token|device_code|credential)"\s*:/.test(serialized)) {
      return initialFirstRunState();
    }
    return {
      schema: FIRST_RUN_SCHEMA,
      stage: value.stage,
      choice: value.choice,
      challenge: value.challenge ? safeChallenge(value.challenge) : undefined,
      authority: value.authority,
      last_error: value.last_error ? boundedCode(value.last_error) : undefined,
      updated_at: typeof value.updated_at === 'string' ? value.updated_at : new Date().toISOString(),
    };
  } catch {
    return initialFirstRunState();
  }
}

export function parseAuthorityDeepLink(url: string): SafeDeviceChallenge | null {
  try {
    const value = new URL(url);
    if (value.protocol !== 'focusa:' || value.hostname !== 'authority') return null;
    const verification_uri = value.searchParams.get('verification_uri') || '';
    const user_code = value.searchParams.get('user_code') || '';
    const expires_at = value.searchParams.get('expires_at') || '';
    return safeChallenge({ verification_uri, user_code, expires_at });
  } catch {
    return null;
  }
}

export const MANUAL_AUTHORITY_FALLBACK =
  'Run `focusa install --eval` in a trusted terminal, then return here to verify the signed lease.';

function safeChallenge(challenge: SafeDeviceChallenge): SafeDeviceChallenge {
  const uri = new URL(challenge.verification_uri);
  if (uri.protocol !== 'https:') throw new Error('verification_uri_must_be_https');
  if (!/^[A-Z0-9-]{4,24}$/.test(challenge.user_code)) throw new Error('invalid_user_code');
  if (!Number.isFinite(Date.parse(challenge.expires_at))) throw new Error('invalid_expiry');
  return {
    verification_uri: uri.toString(),
    user_code: challenge.user_code,
    expires_at: challenge.expires_at,
  };
}

function boundedCode(value: string): string {
  return value.replace(/[^a-zA-Z0-9_.:-]/g, '_').slice(0, 120);
}

function isStage(value: unknown): value is FirstRunStage {
  return typeof value === 'string' && [
    'trust_recovery', 'choice', 'device_code', 'account_pending', 'terms_consent',
    'lease_verification', 'optional_uiai', 'pairing', 'project', 'first_workpoint', 'complete',
  ].includes(value);
}
