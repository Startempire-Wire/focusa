import assert from 'node:assert/strict';
import { Type } from '@sinclair/typebox';
import { loadPureSourceModule, loadSourceSection } from './helpers/source-section-harness.mjs';
const base = process.argv[2];
assert.ok(base?.startsWith('http://127.0.0.1:'));
const request = JSON.parse(process.env.FOCUSA_PRELOAD_TEST_REQUEST);
const runtime = { focusaAvailable: true, sessionCwd: request.project_root, continuityId: request.continuity_id,
  cfg: {focusaApiBaseUrl:`${base}/v1`,focusaToken:process.env.FOCUSA_AUTH_TOKEN,focusaApiTimeoutMs:2500} };
const key = () => ({workstream:{root_scope:{root_path:request.project_root,scope_kind:'project',scope_id:'synthetic-preload'},continuity_id:request.continuity_id},session_id:'synthetic-preload-consumer'});
const detailed = loadSourceSection('tools.ts','  async function focusaFetchDetailed(','  function formatWorkLoopBudgetRemaining',{
  getAttachmentRuntime:()=>runtime,currentAttachmentKey:key,isProjectRootAuthoritySafe:r=>r===request.project_root,
  currentProjectBindingDecision:()=>({state:'BOUND'}),projectBindingAllowsDurableWrites:()=>true,
  focusaApiV1Base:()=>`${base}/v1`,timeoutBudgetForRoute:()=>2500,getActiveWorkpointPacket:()=>null,
  resolveCanonicalMarkerProjectRoot:()=>request.project_root,normalizeProjectRoot:r=>r||'',
  getLastProjectIdentity:()=>({project_root:request.project_root}),getLastProjectVerify:()=>({project_root:request.project_root}),
  getContinuityId:()=>request.continuity_id,getSessionFrameKey:()=>key().session_id,publishScopedStateChange(){},
  focusaRouteTier:()=>0,timeoutFailureClassForRoute:()=> 'timeout',
},'focusaFetchDetailed');
// Exercise the same embedded registry that Tool Doctor compares, over the real
// authenticated API, against the actual compiled runtime contract owner (#682).
const registryResponse = await loadSourceSection('tools.ts',
  '      const liveContracts = await focusaFetchDetailed("/ontology/tool-contracts",',
  '      const uiaiBrowser =', {focusaFetchDetailed:detailed}, 'liveContracts', true);
assert.equal(registryResponse.ok, true,
  `actual embedded registry must be readable: ${JSON.stringify(registryResponse)}`);
const contracts = loadPureSourceModule('tool-contracts.ts').FOCUSA_TOOL_CONTRACTS;
const liveContracts = registryResponse.body?.contracts;
assert.equal(liveContracts?.length, contracts.length);
for (const contract of contracts) {
  assert.deepEqual(liveContracts.find(item => item.name === contract.name), contract,
    `actual Tool Doctor registry projection must match ${contract.name}`);
}
console.log(`Actual authenticated registry consumer: ${contracts.length} runtime contracts match.`);
const transport = loadSourceSection('tools.ts','  async function callSpec80Tool(','  const SPEC81_ID_PATTERN',{
  preferredWriterId:async()=>undefined,currentWorkLoopLease:async()=>null,writerLeaseHeaders:()=>({}),
  getLastProjectIdentity:()=>({project_root:request.project_root}),normalizeProjectRoot:r=>r||'',
  resolveCanonicalMarkerProjectRoot:()=>request.project_root,getSessionCwd:()=>request.project_root,
  getContinuityId:()=>request.continuity_id,isProjectRootAuthoritySafe:r=>r===request.project_root,
  focusaFetchDetailed:detailed,
},'callSpec80Tool');
const tools = new Map();
let legacy = false;
loadSourceSection('tools.ts','  const preloadProfile =','  pi.registerTool({\n    name: "focusa_predict_stats"',{
  Type, strictObject:o=>Type.Object(o,{additionalProperties:false}), pi:{registerTool:tool=>tools.set(tool.name,tool)},
  resolveFocusaToolProjectRoot:async r=>r,getContinuityId:()=>request.continuity_id,ensureContinuityId:()=>request.continuity_id,
  callSpec80Tool:async(_name,path,body,options)=>{
    if(legacy)return{ok:true,status:200,body:{schema:'focusa.preload.v1',status:'completed',context_kind:'profile_template'}};
    return transport(_name,path,body,options);
  },
  spec80Result:(_name,_path,_request,result)=>result,
},'undefined');
const build = await tools.get('focusa_preload_build').execute('synthetic',request);
assert.equal(build.ok,true,JSON.stringify(build));
const artifact=build.body.artifact;
const exact={...request,artifact,artifact_ref:artifact.artifact_ref,expected_digest:artifact.content_sha256};
for(const name of ['focusa_preload_render','focusa_preload_verify','focusa_preload_doctor','focusa_preload_receipt_preview']) {
  const result=await tools.get(name).execute('synthetic',exact);
  assert.equal(result.ok,true,JSON.stringify(result));
  assert.equal(result.body.artifact.artifact_ref,artifact.artifact_ref);
  assert.equal(result.body.current_process_refresh_verified,false);
}
const changed={...exact,current_ask:'Different later instruction'};
assert.equal((await tools.get('focusa_preload_verify').execute('synthetic',changed)).ok,false);
legacy=true;
for(const name of ['focusa_preload_build','focusa_preload_verify','focusa_preload_receipt_preview','focusa_preload_receipt_commit','focusa_preload_write']){
  const result=await tools.get(name).execute('synthetic',{...exact,target:'/tmp/focusa-preload/not-written',idempotency_key:'synthetic'});
  assert.equal(result.ok,false,`${name} must reject legacy success as scoped proof`);
}
console.log('Actual API ↔ production registered preload tools/transport: exact artifact, changed request and legacy refusal passed.');
