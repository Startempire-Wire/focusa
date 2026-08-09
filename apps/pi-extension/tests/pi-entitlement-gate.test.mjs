import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const state = fs.readFileSync(path.join(root, 'src/state.ts'), 'utf8');
const tools = fs.readFileSync(path.join(root, 'src/tools.ts'), 'utf8');
const adapter = fs.readFileSync(path.join(root, 'src/entitlement-policy-adapter.ts'), 'utf8');

const helper = state.slice(state.indexOf('export async function focusaFetch'), state.indexOf('// Fire-and-forget variant'));
assert.match(helper, /r\.status === 403/);
assert.match(helper, /body\.idempotency_key/);
assert.match(helper, /"Idempotency-Key": key\.trim\(\)/);
assert.match(helper, /code\.startsWith\("ENTITLEMENT_"\)/);
assert.match(helper, /failure_class: "entitlement_blocked"/);
assert.match(helper, /required_feature/);
assert.match(helper, /limit_bucket/);
assert.match(helper, /update_for_recovery/);
assert.match(helper, /\"uninstall\"/);
assert.match(helper, /safe_read/);
assert.match(helper, /status_path/);
assert.ok(helper.indexOf('r.status === 403') < helper.indexOf('return null;'), 'entitlement denial is discarded before projection');

assert.match(tools, /focusaFetch\(/, 'Pi tools do not use the governed daemon client');
assert.doesNotMatch(helper, /eval|dev_mode|license\.json/i, 'Pi client contains local entitlement bypass');

// Spec 152F §7: every Pi tool inherits its canonical operation policy through
// the policy adapter; denials are projected into stable machine JSON and
// recovery actions, and tool visibility never grants entitlement.
assert.match(tools, /entitlement-policy-adapter/, 'Pi tools project canonical decisions through the policy adapter');
assert.match(tools, /projectEntitlementDecision\(/, 'blocked Pi tools inherit the canonical entitlement decision');
assert.match(tools, /entitlement_blocked/, 'Pi tools carry the entitlement_blocked failure class');
assert.match(tools, /operator_required/, 'entitlement denials require operator action, never auto-approval');
assert.match(adapter, /ENTITLEMENT_DECISION_SCHEMA = "focusa\.entitlement_decision\.v1"/, 'adapter emits stable entitlement machine JSON');

assert.match(adapter, /licensing_grants_capability_only: true/, 'adapter: licensing grants capability only');
assert.match(adapter, /operator_authority_granted: false/, 'adapter: no operator authority grant');
assert.match(adapter, /cognitive_authority_granted: false/, 'adapter: no cognitive authority grant');
assert.match(adapter, /approval_inferred: false/, 'adapter: operator permission/confirmation preserved independently');
assert.match(adapter, /discovery_visibility_granted: false/, 'adapter: visibility never grants entitlement');
assert.match(adapter, /unknown_tool_has_no_operation_policy/, 'adapter: unknown tools fail closed');
assert.match(adapter, /preflightAuthority/, 'adapter: preflight resolves policy before side effects');
assert.match(adapter, /update_for_recovery/, 'adapter: recovery actions expose stable security updates');
assert.match(adapter, /"uninstall"/, 'adapter: recovery actions expose uninstall');
assert.match(adapter, /safe_read/, 'adapter: recovery actions expose safe read');
assert.doesNotMatch(
  adapter,
  /operator_authority_granted: true|cognitive_authority_granted: true|approval_inferred: true/i,
  'adapter must never emit authority grants'
);

console.log('Pi entitlement denial projection gate passed');
