import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const api = fs.readFileSync(path.join(root, 'src/lib/api.ts'), 'utf8');
assert.match(api, /values\.idempotency_key/);
assert.match(api, /mergedHeaders\['Idempotency-Key'\] = key\.trim\(\)/);

const block = api.slice(api.indexOf('if (!resp.ok)'), api.indexOf("diagnosticsStore.record", api.indexOf('if (!resp.ok)')));

assert.match(block, /errorBody\?\.code/);
assert.match(block, /code\.startsWith\('ENTITLEMENT_'\)/);
assert.match(block, /'entitlement_blocked'/);
assert.match(block, /required_feature/);
assert.match(block, /limit_bucket/);
assert.match(block, /recovery/);
assert.doesNotMatch(block, /localStorage|eval|dev_mode|license\.json/i);

console.log('menubar entitlement error projection gate passed');
