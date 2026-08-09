import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const state = fs.readFileSync(path.join(root, 'src/state.ts'), 'utf8');
const tools = fs.readFileSync(path.join(root, 'src/tools.ts'), 'utf8');

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

console.log('Pi entitlement denial projection gate passed');
