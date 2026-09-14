import assert from 'node:assert/strict';
import test from 'node:test';
import { randomUUID } from 'node:crypto';
import { loadRegisteredTool, loadPureTypeScript } from './helpers/registered-tool-harness.mjs';
const { silentPreflightResult } = loadPureTypeScript(new URL('../src/silent-preflight.ts', import.meta.url));

function load(response, requests) {
  return loadRegisteredTool('focusa_silent_sessions', {
    silentPreflightResult,
    focusaFetchDetailed: async (path, options) => { requests.push({path, ...options}); return response; },
  });
}

test('silent preflight wraps the config and forwards only caller-supplied idempotency', async () => {
  const requests = [];
  const tool = load({ok:true,status:200,body:{ok:true,status:'preflight_ok',canonical:true,data:{redacted_config_hash:'fixture'}}}, requests);
  const config = {schema:'focusa.silent_session_config.v1'};
  await tool.execute('fixture', {action:'preflight',config,idempotency_key:'fixture-replay'});
  assert.equal(requests[0].path, '/silent-sessions/preflight');
  assert.deepEqual(JSON.parse(requests[0].body), {config});
  assert.equal(requests[0].headers?.['Idempotency-Key'], 'fixture-replay');
  assert.equal(requests[0].headers?.['x-focusa-approval'], undefined);
});

for (const [http, body] of [
  [403, {ok:false,status:'authorization_denied',failure_class:'scope_mismatch'}],
  [409, {ok:false,status:'idempotency_required',retry:{idempotency_key_required:true}}],
  [200, {ok:false,status:'blocked',canonical:true}],
  [200, null],
]) {
  test(`silent preflight rejects ${http}/${body?.status ?? 'missing body'}`, async () => {
    const result = await load({ok:http===200,status:http,body}, []).execute('fixture', {action:'preflight',config:{}});
    assert.equal(result.isError,true);
    assert.equal(result.details.canonical,false);
    assert.equal(result.details.http_status,http);
    if (body?.retry) assert.equal(result.details.retry.idempotency_key_required,true);
  });
}

test('preflight omits private configs, request metadata and echoed values', () => {
  const marker = randomUUID();
  const result = silentPreflightResult({ok:false,status:422,body:{
    ok:false,status:'invalid_config',message:`invalid ${marker}`,
    field_errors:[{field:'auth',message:marker,value:marker}],
    request_overrides:{Authorization:marker},config:{credential:marker},
    receipt_refs:['receipt:fixture'],side_effects:[{effect:'principal_registered',status:'created',target_ref:'actor:fixture'}],
  }}, {credential:marker});
  assert.equal(JSON.stringify(result).includes(marker),false);
  assert.equal('request_overrides' in result.details,false);
  assert.equal('config' in result.details,false);
  assert.equal(result.details.field_errors[0].message,'[redacted]');
  assert.equal(result.details.side_effects[0].effect,'principal_registered');
  assert.deepEqual(result.details.receipt_refs,['receipt:fixture']);
});

test('preflight success exposes the producer hash without returning effective config', () => {
  const marker = randomUUID();
  const result = silentPreflightResult({ok:true,status:200,body:{ok:true,status:'preflight_ok',canonical:true,
    data:{redacted_config_hash:'hash:fixture',resolved_effective_config:{credential:marker}}}}, {credential:marker});
  assert.equal(result.isError,false);
  assert.equal(result.details.data.redacted_config_hash,'hash:fixture');
  assert.equal(JSON.stringify(result).includes(marker),false);
});

test('redaction handles cyclic or oversized inputs without exposing messages', () => {
  const config = {}; config.self = config;
  const result = silentPreflightResult({ok:false,status:422,body:{message:'omitted upstream detail'}}, config);
  assert.equal(result.details.message,'');
  assert.equal(result.isError,true);
});

test('silent preflight preserves actionable validation failures without false canonical success', async () => {
  const requests = [];
  const tool = load({ok:false,status:422,body:{ok:false,status:'invalid_config',canonical:false,failure_class:'config_validation',recovery_hint:'Correct the configuration and rerun preflight.'}},requests);
  const result = await tool.execute('fixture',{action:'preflight',config:{}});
  assert.match(result.content[0].text,/invalid_config/);
  assert.match(result.content[0].text,/Correct the configuration/);
  assert.equal(result.details.canonical,false);
  assert.equal(requests[0].headers?.['Idempotency-Key'],undefined);
});
