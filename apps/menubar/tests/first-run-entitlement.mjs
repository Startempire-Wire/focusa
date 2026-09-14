import assert from 'node:assert/strict';
import {
  MANUAL_AUTHORITY_FALLBACK,
  advanceFirstRun,
  entitlementReady,
  initialFirstRunState,
  parseAuthorityDeepLink,
  restoreFirstRunState,
  serializeFirstRunState,
} from '../src/lib/firstRunEntitlement.ts';

const active = {
  state: 'active',
  product: 'focusa',
  sequence: 7,
  signature_verified: true,
  channel_granted: true,
  terms_accepted: true,
  privacy_accepted: true,
};

let state = initialFirstRunState('2026-08-05T00:00:00Z');
state = advanceFirstRun(state, { type: 'authority_observed', authority: {
  ...active,
  state: 'unactivated',
  sequence: undefined,
  signature_verified: false,
  channel_granted: false,
  terms_accepted: false,
  privacy_accepted: false,
}});
assert.equal(state.stage, 'choice');
state = advanceFirstRun(state, { type: 'choice_selected', choice: 'evaluate' });
assert.equal(state.stage, 'device_code');

const challenge = {
  verification_uri: 'https://authority.example.test/device',
  user_code: 'FOCUS-1234',
  expires_at: '2026-08-05T00:10:00Z',
};
state = advanceFirstRun(state, { type: 'device_challenge', challenge });
const restored = restoreFirstRunState(serializeFirstRunState(state));
assert.equal(restored.stage, 'device_code');
assert.deepEqual(restored.challenge, challenge);
assert.doesNotMatch(serializeFirstRunState(restored), /email|access_token|refresh_token|credential/i);

const deepLink = parseAuthorityDeepLink(
  'focusa://authority?verification_uri=https%3A%2F%2Fauthority.example.test%2Fdevice&user_code=FOCUS-5678&expires_at=2026-08-05T00%3A10%3A00Z',
);
assert.equal(deepLink?.user_code, 'FOCUS-5678');
assert.equal(parseAuthorityDeepLink('https://evil.example.test/?user_code=FOCUS-9999'), null);
assert.match(MANUAL_AUTHORITY_FALLBACK, /focusa install --eval/);

let blocked = advanceFirstRun(initialFirstRunState(), { type: 'choice_selected', choice: 'evaluate' });
blocked = advanceFirstRun(blocked, { type: 'pairing_saved' });
assert.notEqual(blocked.stage, 'project', 'pairing created entitlement authority');

let ready = advanceFirstRun(initialFirstRunState(), { type: 'authority_observed', authority: active });
assert.equal(ready.stage, 'optional_uiai');
assert.equal(entitlementReady(ready.authority), true);
ready = advanceFirstRun(ready, { type: 'skip_optional_uiai' });
ready = advanceFirstRun(ready, { type: 'pairing_saved' });
ready = advanceFirstRun(ready, { type: 'project_verified' });
ready = advanceFirstRun(ready, { type: 'first_workpoint_accepted' });
assert.equal(ready.stage, 'complete');

const contaminated = JSON.stringify({
  schema: 'focusa.first_run_entitlement.v1',
  stage: 'device_code',
  email: 'operator@example.test',
  updated_at: '2026-08-05T00:00:00Z',
});
assert.equal(restoreFirstRunState(contaminated).stage, 'trust_recovery');

console.log('menubar first-run entitlement state tests passed');
