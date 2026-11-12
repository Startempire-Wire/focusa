import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const catalog = read('crates/focusa-api/src/routes/agent_capabilities.rs');
const router = read('crates/focusa-api/src/routes/workpoint.rs');
const registry = JSON.parse(read('docs/contracts/spec135/generated-contract-v1/operation-registry.json'));
const openapi = JSON.parse(read('docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json'));

for (const [operation, handler] of [
  ['focusa.workpoint.resume', 'resume'],
  ['focusa.workpoint.link_evidence', 'link_evidence'],
]) {
  test(`${operation} advertises the method and path of its registered handler`, () => {
    const entry = catalog.match(new RegExp(`op\\(\\s*"${operation.replaceAll('.', '\\.')}"\\s*,\\s*"[^"]+"\\s*,\\s*"[^"]+"\\s*,\\s*"([A-Z]+)"\\s*,\\s*"([^"]+)"`));
    assert.ok(entry, `Missing operation ${operation}`);
    const registered = router.match(new RegExp(`\\.route\\(\\s*"([^"]+)"\\s*,\\s*(get|post)\\(${handler}\\)\\s*\\)`));
    assert.ok(registered, `Missing registered handler ${handler}`);
    assert.equal(entry[1], registered[2].toUpperCase());
    assert.equal(entry[2], registered[1]);
    const descriptor = registry.operations.find(row => row.operation_id === operation);
    assert.ok(descriptor, `Missing canonical operation ${operation}`);
    assert.equal(descriptor.method, entry[1]);
    assert.equal(descriptor.path, entry[2]);
    assert.deepEqual(Object.keys(openapi.paths[entry[2]]), ['post']);
    const exported = openapi.paths[entry[2]].post;
    assert.equal(exported.operationId, operation);
    assert.ok(exported.requestBody?.content['application/json']?.schema);
    assert.equal(openapi.paths['/v1/workpoint/link-evidence'], undefined);
    if (handler === 'resume') {
      assert.equal(descriptor.control.mode, 'read');
      assert.equal(descriptor.control.receipt_required, false);
      assert.deepEqual(descriptor.control.permission_scopes, ['workpoint:read']);
      assert.equal(exported['x-focusa-mode'], 'read');
    }
  });
}
