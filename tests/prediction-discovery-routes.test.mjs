import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const router = read('crates/focusa-api/src/routes/predictions.rs') +
  read('crates/focusa-api/src/routes/prediction_authority_canonical.rs');
const discovery = read('crates/focusa-api/src/routes/agent_capabilities.rs').split('#[cfg(test)]')[0];
const openapi = JSON.parse(read('docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json'));
const client = read('packages/generated/spec135/typescript/schema.d.ts');

for (const [method, path, id, handler] of [
  ['post', '/v1/predictions', 'focusa.prediction.record', 'record'],
  ['post', '/v1/evaluations/predictions', 'focusa.prediction.evaluate', 'prediction_evaluate'],
  ['get', '/v1/predictions/recent', 'focusa.prediction.list', 'predictions_recent'],
]) {
  test(`${id} contract and generated client match its registered route`, () => {
    assert.ok(router.includes(`.route("${path}", ${method}(${handler}))`));
    assert.equal(openapi.paths[path][method].operationId, id);
    assert.ok(client.includes(`"${path}": {`));
  });
}

test('legacy prediction metadata cannot duplicate evaluation or shadow listing', () => {
  for (const path of ['/v1/predictions/record', '/v1/predictions/evaluate']) {
    assert.equal(openapi.paths[path]?.post, undefined);
    assert.ok(!discovery.includes(`"${path}"`));
  }
  assert.ok(!discovery.includes('"focusa.prediction.recent"'));
});

test('prediction recording preserves the installed route’s reviewed metering', () => {
  const registry = JSON.parse(read('docs/contracts/spec135/generated-contract-v1/operation-registry.json'));
  const op = registry.operations.find(row => row.operation_id === 'focusa.prediction.record');
  const reviewed = read('crates/focusa-api/src/middleware/entitlement_routes.rs');
  const bucket = reviewed.match(/template: "\/v1\/predictions",\s+feature: "[^"]+",\s+limit_bucket: Some\("([^"]+)"\)/)?.[1];
  assert.ok(bucket);
  assert.equal(op.limit_bucket, bucket);
});
