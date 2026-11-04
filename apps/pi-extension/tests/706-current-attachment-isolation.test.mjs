import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import test from 'node:test';

const read = name => readFileSync(new URL(`../src/${name}`, import.meta.url), 'utf8');
const state = read('state.ts');
const tools = read('tools.ts');
const index = read('index.ts');
const js = s => stripTypeScriptTypes(s).replace(/^export /gm, '');
function section(source, from, to) {
  const start = source.indexOf(from);
  const end = source.indexOf(to, start);
  assert.ok(start >= 0 && end > start, `production section ${from}`);
  return source.slice(start, end);
}
const root = resolve('attachment-isolation-fixture');
const key = (session, continuity) => ({ workstream: { root_scope: { scope_kind: 'project', scope_id: 'fixture', fingerprint: 'fixture-fingerprint', root_path: root }, continuity_id: continuity }, instance_id: 'fixture-pi', session_id: session, attachment_id: session });

const runtimeCode = section(index, '    const runtimeFor = ', '    const prepareRuntime = ');
function runtimeRouter(bindings, fallback) {
  return new Function('attachmentRoutingHints', 'attachmentRuntimeRegistry', 'sessionBinding', 'makeSessionBootstrapAttachmentKey', 'getAttachmentRuntime', 'extensionKey', 'verifiedScopeRefForRoot', 'makeAttachmentKey', `${js(runtimeCode)};return runtimeFor;`)(x => ({sessionId:x?.session_id, projectRoot:x?.project_root, continuityId:x?.continuity_id}), {boundSessionAttachment:id=>bindings.get(id)}, {resolve:()=>fallback}, id=>({host:true,session_id:id}), ()=>({sessionFrameKey:'extension-host',continuityId:'foreign-bootstrap'}), {},()=>key('fixture','fixture').workstream.root_scope, x=>key(x.sessionId,x.continuityId));
}
const ctx = id => ({sessionManager:{getSessionId:()=>id},cwd:root});

test('two native sessions keep their bindings even when queries select the other scope', () => {
  const a = key('native-a','workstream-a'), b = key('native-b','workstream-b');
  const route = runtimeRouter(new Map([['native-a',a],['native-b',b]]), b);
  assert.equal(route(ctx('native-a'), {session_id:'native-b',project_root:root,continuity_id:'workstream-b'}), a);
  assert.equal(route(ctx('native-b'), {session_id:'native-a',project_root:root,continuity_id:'workstream-a'}), b);
  for (const event of ['compaction','model_switch','session_resume']) assert.equal(route(ctx('native-a'), {event,continuity_id:'workstream-b'}), a);
});

test('unbound/crashed sessions bootstrap privately instead of borrowing an extension binding', () => {
  const route = runtimeRouter(new Map(), key('native-b','workstream-b'));
  assert.deepEqual(route(ctx('native-a'), {project_root:root,continuity_id:'workstream-b'}), {host:true,session_id:'native-a'});
  assert.deepEqual(route(ctx('native-a'), {}), {host:true,session_id:'native-a'});
});

const classes = section(state, 'export class TypedScopeStore', 'export const scopeStoreRegistry');
const Registry = new Function(`${js(classes)};return ScopeStoreRegistry;`)();
const identity = (sessionId,continuityId='workstream-a',workingSubpathId='primary') => ({scopeKind:'project',scopeId:'fixture',fingerprint:'fixture-fingerprint',rootPath:root,continuityId,workingSubpathId,sessionId});

test('transient caches partition native sessions without changing canonical Workstream identity', () => {
  const registry = new Registry();
  const a = registry.getOrCreate(identity('native-a'));
  const b = registry.getOrCreate(identity('native-b'));
  a.lastTrajectoryClarity = {hlt:'mission-a'};
  assert.notEqual(a,b);
  assert.equal(b.lastTrajectoryClarity,null);
  assert.equal(a.identity.continuityId,b.identity.continuityId);
  assert.equal(registry.getOrCreate(identity('native-a')),a);
  assert.notEqual(registry.getOrCreate(identity('native-a','workstream-b')),a);
  assert.notEqual(registry.getOrCreate(identity('native-a','workstream-a','other-worktree')),a);
});

const ladderCode = section(tools, 'function firstNonEmptyText(', 'function formatToolTrajectoryLadderSummary(');
function ladderReader(caller, cached=null) {
  return new Function('currentAttachmentKey','getLastTrajectoryClarity','normalizeProjectRoot',`${js(ladderCode)};return cachedToolTrajectoryLadder;`)(()=>caller,()=>cached, s=>resolve(String(s)));
}
const ladder = (continuity_id, session_id, hlt) => ({project_root:root,continuity_id,session_id,hlt});

test('foreign ladders, fallbacks, and candidates never become the caller mission', () => {
  const current = ladderReader(key('native-a','workstream-a'));
  assert.equal(current({trajectory_ladder:ladder('workstream-b','native-a','foreign')}),null);
  assert.equal(current({trajectory_ladder:ladder('workstream-a','native-b','foreign')}),null);
  assert.equal(current({trajectory_candidate:ladder('workstream-a','native-a','proposed')}),null);
  assert.equal(current({trajectory_ladder:{...ladder('workstream-a','native-a','old'),fallback_prior_project_trajectory:true}}),null);
  assert.equal(current({trajectory_ladder:ladder('workstream-a','native-a','actual')}).hlt,'actual');
});

const persistedCode = section(state, 'export function adoptPersistedContinuityForSession(', '// ── Build compact instructions');
test('old adapter restamps cannot launder another native session through crash recovery', () => {
  const runtime = {continuityId:'workstream-a',activeFrameGoal:'foreign cached goal',activeFrameTitle:'foreign title'}; let active = 'untouched';
  const adopt = new Function('getAttachmentRuntime','setActiveWorkpointPacket','setActiveWorkpointSummary','setLastTrajectoryClarity','normalizeProjectRoot','isProjectRootAuthoritySafe','stampWorkpointPacketForCurrentPiSession','revokeWorkpointRequestAuthority',`${js(persistedCode)};return adoptPersistedContinuityForSession;`)(()=>runtime,x=>active=x,()=>{},()=>{},s=>s,()=>true,x=>x,x=>x);
  adopt({sessionId:'native-a',continuityId:'workstream-b',activeWorkpointPacket:{project_root:root,continuity_id:'workstream-b',session_id:'native-b',pi_session_frame_key:'native-a',canonical:true,status:'active'}},'native-a',root);
  assert.equal(runtime.continuityId,'workstream-a');
  assert.equal(runtime.activeFrameGoal,null);
  assert.equal(runtime.activeFrameTitle,null);
  assert.equal(active,null);
});

const adoptCode = section(state, 'export function adoptWorkpointScopeForFrameRecovery(', 'function scopedWorkpointFrameRecoveryCwd(');
function adopter(caller, runtime) {
  let active=null;
  const fn=new Function('currentAttachmentKey','getAttachmentRuntime','normalizeProjectRoot','isProjectRootAuthoritySafe','setActiveWorkpointPacket','stampWorkpointPacketForCurrentPiSession',`${js(adoptCode)};return adoptWorkpointScopeForFrameRecovery;`)(()=>caller,()=>runtime,s=>resolve(String(s)),()=>true,p=>active=p,p=>p);
  return {fn,active:()=>active};
}

test('read selectors cannot authorize a different Workstream even with the legacy boolean', () => {
  const runtime={sessionFrameKey:'native-a',continuityId:'workstream-a'};
  const a=adopter(key('native-a','workstream-a'),runtime);
  const foreign={project_root:root,continuity_id:'workstream-b',session_id:'native-b',canonical:true,status:'active'};
  assert.equal(a.fn(foreign,'workpoint_resume_tool',{projectRoot:root,continuityId:'workstream-b',allowSessionTransfer:true}),null);
  assert.equal(a.fn(foreign,'session_transfer_tool',{projectRoot:root,continuityId:'workstream-b',allowSessionTransfer:true}),null);
  assert.equal(a.active(),null);
  assert.equal(runtime.continuityId,'workstream-a');
});

test('legitimate same-Workstream continuation retains authority across native processes', () => {
  const runtime={sessionFrameKey:'native-a',continuityId:'workstream-a'};
  const a=adopter(key('native-a','workstream-a'),runtime);
  assert.equal(a.fn({project_root:root,continuity_id:'workstream-a',session_id:'native-old',canonical:true,status:'active'},'workpoint_resume_tool'),root);
  assert.equal(a.active().continuity_id,'workstream-a');
});

test('authorized transfer control works only after its owner re-keys the target attachment', () => {
  const runtime={sessionFrameKey:'native-a',continuityId:'workstream-b'};
  const a=adopter(key('native-a','workstream-b'),runtime);
  assert.equal(a.fn({project_root:root,continuity_id:'workstream-b',session_id:'native-old',canonical:true,status:'active'},'session_transfer_tool',{projectRoot:root,continuityId:'workstream-b',allowSessionTransfer:true}),root);
});

test('compaction instructions explicitly preserve correction over unverified carryover', () => {
  assert.match(state,/a prior summary, rejected Workpoint, or record canonical only in another attachment cannot define this session's mission/);
});
