import assert from 'node:assert/strict';
import { test } from 'node:test';
import { pollPairing, startPairing } from '../src/lib/pairing.mjs';
import { listConnections } from '../src/lib/storage.mjs';

function response(payload, ok=true, status=200) { return { ok, status, json: async()=>structuredClone(payload) }; }
function chromeMock(permission=true) {
  let state={}; return { permissions:{request:async()=>permission}, storage:{local:{get:async(k)=>({[k]:state[k]}),set:async(v)=>{state={...state,...structuredClone(v)};}}} };
}
const pending={state:'awaiting_approval',base_url:'https://focusa.example',label:'KH',code:'ABCD1234',device_id:'device-1',scopes:['read','write'],expires_at:'2026-08-24T13:00:00Z',operator_command:'focusa device pair-complete ABCD1234'};

test('pair start requests exact origin then read/write device code', async()=>{
  const chrome=chromeMock(); const calls=[];
  const result=await startPairing({base_url:'https://focusa.example',label:'KH'}, {chromeApi:chrome,fetchImpl:async(url,init)=>{calls.push({url,init,body:JSON.parse(init.body)});return response({code:'ABCD1234',device_id:'device-1',expires_at:pending.expires_at,scopes:['read','write'],operator_handoff:{command:'complete'}});}});
  assert.equal(result.state,'awaiting_approval'); assert.equal(calls[0].url,'https://focusa.example/v1/device/pair/start');
  assert.deepEqual(calls[0].body.scopes,['read','write']); assert.equal(calls[0].body.platform,'focusa-workforce-chrome');
});

test('permission denial performs no network request', async()=>{
  let fetched=false; const result=await startPairing({base_url:'https://focusa.example'}, {chromeApi:chromeMock(false),fetchImpl:async()=>{fetched=true;}});
  assert.equal(result.state,'permission_denied'); assert.equal(fetched,false);
});

test('one-shot token is durably stored before paired success', async()=>{
  const chrome=chromeMock();
  const result=await pollPairing(pending,{chromeApi:chrome,now:()=>new Date('2026-08-24T12:00:00Z'),fetchImpl:async()=>response({status:'completed',token:'secret',token_present:true,scopes:['read','write']})});
  assert.equal(result.state,'paired'); assert.equal((await listConnections(chrome))[0].token,'secret');
});

test('pending expired consumed and malformed states are explicit', async()=>{
  for (const [payload,state] of [[{status:'pending'},'awaiting_approval'],[{status:'expired'},'expired'],[{status:'consumed',token:null,token_present:true},'token_consumed_repair_required']]) {
    const result=await pollPairing(pending,{chromeApi:chromeMock(),fetchImpl:async()=>response(payload)}); assert.equal(result.state,state);
  }
  await assert.rejects(()=>pollPairing(pending,{chromeApi:chromeMock(),fetchImpl:async()=>response({status:'mystery'})}),/unsupported/);
});
