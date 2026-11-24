import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
const read = p => readFileSync(new URL(`../${p}`, import.meta.url), 'utf8');
const base = 'docs/contracts/spec135/generated-contract-v1/';
const registry = JSON.parse(read(base + 'operation-registry.json'));
const openapi = JSON.parse(read(base + 'openapi-3.0.3.json'));

test('all curated discovery operations have canonical method, path and identity', () => {
  const source = read('crates/focusa-api/src/routes/agent_capabilities.rs');
  const curated = source.slice(source.indexOf('fn build_operations'), source.indexOf('fn build_families'));
  const operations = [...curated.matchAll(/\bop\(\s*"([^"]+)",\s*"[^"]+",\s*"[^"]+",\s*"([A-Z]+)",\s*"([^"]+)"/g)];
  assert.ok(operations.length > 0);
  const byId = new Map(registry.operations.map(row => [row.operation_id, row]));
  for (const [, id, method, path] of operations) {
    assert.deepEqual([byId.get(id)?.method, byId.get(id)?.path], [method, path], id);
    assert.equal(openapi.paths[path]?.[method.toLowerCase()]?.operationId, id);
  }
});

test('CallGraph validation is a standalone read, not a dispatch contract', () => {
  const path = '/v1/callgraphs/validate';
  const op = registry.operations.find(row => row.path === path);
  assert.ok(read('crates/focusa-api/src/routes/callgraph.rs').includes(`.route("${path}", post(validate))`));
  assert.equal(op.method, 'POST');
  assert.equal(op.control.mode, 'read');
  assert.deepEqual(op.control.permission_scopes, ['callgraph:read']);
  assert.equal(op.control.receipt_required, false);
  assert.equal(op.control.idempotency_required, false);
  assert.deepEqual(op.scope.required_keys, []);
  const request = openapi.paths[path].post.requestBody.content['application/json'].schema.$ref;
  assert.deepEqual(openapi.components.schemas[request.split('/').at(-1)].properties.schema.enum, ['focusa.callgraph.v1']);
  assert.ok(read('packages/generated/spec135/typescript/schema.d.ts').includes(`"${path}": {`));
});


test('project verification and awareness discovery match the registered read handlers', () => {
  for (const [path, method, file, binding] of [
    ['/v1/project/verify', 'post', 'project', 'verify'],
    ['/v1/awareness/packet', 'get', 'utility', 'awareness_packet'],
  ]) {
    assert.ok(read(`crates/focusa-api/src/routes/${file}.rs`).includes(`.route("${path}", ${method}(${binding}))`));
    const op = registry.operations.find(op => op.path === path);
    const advertised = openapi.paths[path];
    assert.deepEqual(Object.keys(advertised), [method]);
    assert.equal(advertised[method].operationId, op.operation_id);
    assert.equal(op.control.mode, 'read');
    assert.equal(op.control.receipt_required, false);
    assert.deepEqual(advertised[method].parameters, []);
    assert.equal(Boolean(advertised[method].requestBody), method === 'post');
  }
});
