import assert from 'node:assert/strict';
import test from 'node:test';
import { loadRegisteredTool } from './helpers/registered-tool-harness.mjs';

const helperSections = [
  ['export function toolResult(', 'import type { ExtensionAPI }'],
  ['function safeErrorText(', 'import { registerAgentRuntimeTools'],
];

function validator(fetch) {
  return loadRegisteredTool('focusa_callgraph_validate', {
    getAttachmentRuntime: () => ({ cfg: {} }),
    focusaApiV1Base: () => 'http://daemon.invalid/v1',
    timeoutBudgetForRoute: () => 1000,
    fetch,
  }, helperSections);
}

const rejected = {
  code: 'validation_error',
  correlation_id: 'test-correlation-704',
  details: {
    tool_result_v1: {
      failure_class: 'validation_rejected',
      misuse_hint: 'Correct the side_effect_class enum before retrying.',
      next_tools: ['focusa_callgraph_validate'],
    },
  },
};

function response(body, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

test('422 keeps actual daemon rejection details instead of reporting an unavailable validator', async () => {
  const tool = validator(async (url, options) => {
    assert.equal(url, 'http://daemon.invalid/v1/callgraphs/validate');
    assert.equal(JSON.parse(options.body).side_effect_class, 'remote');
    return response(rejected, 422);
  });
  const result = await tool.execute('reject-422', { graph: { side_effect_class: 'remote' } });
  assert.match(result.content[0].text, /request rejected.*validation_error.*422/);
  assert.match(result.content[0].text, /Correct the side_effect_class enum/);
  assert.doesNotMatch(result.content[0].text, /unavailable/i);
  assert.equal(result.details.ok, false);
  assert.equal(result.details.status, 'validation_rejected');
  assert.equal(result.details.canonical, true);
  assert.equal(result.details.failure_class, 'validation_rejected');
  assert.equal(result.details.code, 'validation_error');
  assert.equal(result.details.correlation_id, rejected.correlation_id);
  assert.equal(result.details.misuse_hint, rejected.details.tool_result_v1.misuse_hint);
  assert.deepEqual(result.details.next_tools, ['focusa_callgraph_validate']);
  assert.deepEqual(result.details.data, rejected);
});

test('an owner-classified denial is not relabelled as malformed input', async () => {
  const body = {
    code: 'scope_mismatch', status: 'blocked',
    details: { tool_result_v1: {
      failure_class: 'scope_mismatch', misuse_hint: 'Verify the requested target scope.',
      next_tools: ['focusa_project_verify'],
    } },
  };
  const result = await validator(async () => response(body, 409)).execute('deny', { graph: {} });
  assert.equal(result.details.failure_class, 'scope_mismatch');
  assert.equal(result.details.status, 'blocked');
  assert.deepEqual(result.details.next_tools, ['focusa_project_verify']);
  assert.doesNotMatch(result.content[0].text, /unavailable/i);
});

test('a transport throw remains an unavailable validator, not a request rejection', async () => {
  const result = await validator(async () => { throw new Error('network unavailable'); })
    .execute('network', { graph: {} });
  assert.match(result.content[0].text, /validator unavailable/);
  assert.equal(result.details.status, 'transport_failed');
  assert.equal(result.details.data.failure_class, 'callgraph_validation_transport_failed');
});

test('successful graph validation remains successful', async () => {
  const body = { canonical: true, valid: true, status: 'valid', issues: [] };
  const result = await validator(async () => response(body)).execute('valid', { graph: {} });
  assert.equal(result.details.ok, true);
  assert.equal(result.details.data.valid, true);
  assert.match(result.content[0].text, /definition validates/);
});

test('an invalid graph returned by a working validator remains distinct from HTTP rejection', async () => {
  const body = {
    canonical: true, valid: false, status: 'invalid',
    issues: [{ path: 'entries', message: 'an entry is required' }],
  };
  const result = await validator(async () => response(body)).execute('invalid', { graph: {} });
  assert.equal(result.details.ok, true);
  assert.equal(result.details.data.valid, false);
  assert.match(result.content[0].text, /CallGraph invalid: entries: an entry is required/);
  assert.doesNotMatch(result.content[0].text, /unavailable|request rejected/i);
});
