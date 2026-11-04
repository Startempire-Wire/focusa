import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import test from 'node:test';

const state = readFileSync(new URL('../src/state.ts', import.meta.url), 'utf8');
const tools = readFileSync(new URL('../src/tools.ts', import.meta.url), 'utf8');
const root = '/scope-fixture/project';
const key = continuity => ({workstream:{root_scope:{scope_kind:'project',root_path:root},continuity_id:continuity},session_id:'native-current'});
const start = state.indexOf('export function adoptVerifiedContinuityForCurrentSession(');
const end = state.indexOf('export function adoptPersistedContinuityForSession(', start);
assert.ok(start >= 0 && end > start);
const code = stripTypeScriptTypes(state.slice(start,end)).replace(/^export /gm,'');

test('read confirmation refreshes only the already-owned binding; bootstrap stays unbound', () => {
  let caller = key('owned');
  const runtime = {continuityId:'owned'};
  let synced = 0;
  const deps = {
    normalizeProjectRoot:value=>String(value||'').trim(),
    getLastProjectIdentity:()=>({project_root:root}),
    currentProjectBindingDecision:()=>({selected_project_root:root}),
    resolveCanonicalMarkerProjectRoot:()=>root,
    isProjectRootAuthoritySafe:value=>value===root,
    currentAttachmentKey:()=>caller,
    getAttachmentRuntime:()=>runtime,
    verifiedContinuityBySessionRoot:new Map(),
    verifiedContinuityKey:(session,project)=>`${session}:${project}`,
    getSessionFrameKey:()=>caller.session_id,
    syncRuntimeFieldsToScopeStore:()=>{synced++;},
  };
  const confirm = new Function(...Object.keys(deps),`${code};return adoptVerifiedContinuityForCurrentSession;`)(...Object.values(deps));
  assert.equal(confirm(root,'owned'),true);
  assert.equal(synced,1);
  assert.equal(confirm(root,'other-workstream'),false);
  assert.equal(runtime.continuityId,'owned');
  caller = {...caller,workstream:{root_scope:{scope_kind:'host',root_path:'/'},continuity_id:'extension-bootstrap'}};
  assert.equal(confirm(root,'historical-candidate'),false);
  assert.equal(confirm(root,'owned'),false);
  assert.equal(runtime.continuityId,'owned');
  assert.equal(synced,1,'a read cannot create a new native association');
});

test('identity, card and trajectory reads leave transition ownership separate', () => {
  assert.doesNotMatch(tools,/adoptVerifiedContinuityForCurrentSession\(/);
  const identity = tools.slice(tools.indexOf('name: "focusa_project_identity"'),tools.indexOf('name: "focusa_project_card"'));
  assert.doesNotMatch(identity,/confirmPiProjectRoot\(|ensureContinuityId\(|promoteCurrentSessionAttachment\(/);
  assert.match(tools,/const continuityRecoveryAdopted = false/);
  assert.match(tools,/current_attachment_matches_query: responseMatchesCaller/);
  assert.match(tools,/QUERIED SCOPE — advisory to the current native attachment/);
  assert.doesNotMatch(tools,/hlg=\$\{String\(body\.trajectory\?\.hlt \|\| priorLadder/);
});
