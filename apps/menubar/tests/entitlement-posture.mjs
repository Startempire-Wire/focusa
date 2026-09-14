import assert from 'node:assert/strict';
import { projectEntitlementPosture } from '../src/lib/entitlementPosture.ts';

const cases = [
  ['active', { status: 'active', masked_identity: 'o***@example.test', authority: { state: 'active', expires_at: '2026-08-12T00:00:00Z', limits: { workflow_runs: 8 } } }, 'manage'],
  ['offline_grace', { status: 'offline_grace', authority: { state: 'offline_grace', offline_grace_until: '2026-08-06T00:00:00Z' } }, 'refresh'],
  ['expired', { status: 'expired', authority: { recovery_reason: 'lease_expired' } }, 'purchase'],
  ['revoked', { status: 'recovery_only', authority: { recovery_reason: 'lease_revoked' } }, 'purchase'],
  ['invalid', { status: 'recovery_only', authority: { recovery_reason: 'signature_invalid' } }, 'refresh'],
  ['unactivated', { status: 'unactivated' }, 'evaluate'],
];

for (const [state, payload, action] of cases) {
  const posture = projectEntitlementPosture(payload);
  assert.equal(posture.state, state);
  assert.equal(posture.action, action);
  assert.match(posture.recovery_policy, /Recovery, export, repair, and uninstall/);
  assert.equal(posture.marketing_preference, 'managed_separately');
}

const active = projectEntitlementPosture({
  status: 'active',
  masked_identity: 'o***@example.test',
  authority: { limits: { workflow_runs: 8, projects: 2 } },
  capabilities: [
    { capability: 'hosted_mode', outcome: 'denied', reason: 'feature_not_granted' },
    { capability: 'local_use', outcome: 'permitted' },
  ],
});
assert.equal(active.masked_identity, 'o***@example.test');
assert.deepEqual(active.limits, [
  { name: 'projects', remaining: 2 },
  { name: 'workflow_runs', remaining: 8 },
]);
assert.deepEqual(active.locked_capabilities, [
  { name: 'hosted_mode', reason: 'feature_not_granted' },
]);
assert.equal(projectEntitlementPosture({ status: 'active', masked_identity: 'raw@example.test' }).masked_identity, undefined);
assert.doesNotMatch(JSON.stringify(active), /access_token|refresh_token|manage_token/i);

console.log('menubar entitlement posture semantic tests passed');
