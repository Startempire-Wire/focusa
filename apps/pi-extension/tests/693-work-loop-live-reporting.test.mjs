// Actual native SDK + canonical daemon; read-only unless a separately authorized single pause is explicitly enabled.
// Never starts/resumes a loop or worker, acquires ownership, or renews a budget.
import assert from 'node:assert/strict';
import test from 'node:test';
import fs, { readFileSync, writeFileSync } from 'node:fs';
import { syncBuiltinESMExports } from 'node:module';
import { userInfo } from 'node:os';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const enabled = process.env.FOCUSA_LIVE_LOOP_REPORTING === '1';
test('actual native loop reporting distinguishes unverified lease and route preview', { skip: !enabled }, async () => {
  const root = process.env.FOCUSA_LIVE_PROJECT_ROOT;
  const continuity = process.env.FOCUSA_LIVE_CONTINUITY_ID;
  const sdk = process.env.FOCUSA_NATIVE_PI_SDK_ROOT + '/dist/core';
  const expected = JSON.parse(readFileSync(process.env.FOCUSA_LIVE_LOOP_EXPECTED_SCOPE, 'utf8'));
  const evidence = process.env.FOCUSA_LIVE_LOOP_EVIDENCE;
  assert.ok(root && continuity && evidence && expected.no_execution_grant === true);
  const requests = [];
  const checkpointWrites = [];
  let checkpointProbeHook = null;
  const pauseEnabled = process.env.FOCUSA_LIVE_PAUSE_RECOVERY === '1';
  let allowPause = false, pauseAttempts = 0, pauseRequests = 0, pauseHttpStatus = null;
  let lastLoopBody = null;
  let pauseEvidence = null;
  let verifiedLoopReads = 0;
  const relay = `import sys,json,urllib.request,urllib.error
x=json.load(sys.stdin)
r=urllib.request.Request(x['url'],data=x['body'].encode()if x.get('body')else None,headers=x['headers'],method=x['method'])
try:
 with urllib.request.urlopen(r,timeout=6)as p:out={'status':p.status,'headers':dict(p.headers),'body':p.read().decode()}
except urllib.error.HTTPError as e:out={'status':e.code,'headers':dict(e.headers),'body':e.read().decode()}
print(json.dumps(out))`;
  const allowed = new Map([
    ['/v1/health', ['GET']], ['/v1/agent/prompt', ['GET']], ['/v1/project/identity', ['GET','POST']],
    ['/v1/project/verify', ['GET','POST']], ['/v1/workpoint/resume', ['POST']],
    ['/v1/trajectory/view', ['GET']], ['/v1/work-loop/status', ['GET']],
    ['/v1/project/north-star-gate', ['GET']],
  ]);
  globalThis.fetch = async (url, init = {}) => {
    const u = new URL(String(url));
    const method = String(init.method || 'GET').toUpperCase();
    if (u.pathname === '/v1/workpoint/checkpoint' && method === 'POST') {
      const body = JSON.parse(String(init.body || '{}'));
      checkpointWrites.push(body);
      writeFileSync(evidence + '.checkpoint-intercept.json', JSON.stringify({
        checkpoint_write_attempts: checkpointWrites.length, checkpoint_network_writes: 0,
        intercepted_before_transport: true,
        footer_masked_empty_next_action: String(body.next_slice || '').includes('DO_NOT_DRIFT:') &&
          !String(body.next_slice || '').split('\n').filter(line => !line.startsWith('DO_NOT_DRIFT:')).join('').trim(),
        lifecycle_stage_present: Object.hasOwn(body.action_intent || {}, 'lifecycle_stage'),
      }, null, 2));
      checkpointProbeHook?.();
      throw new Error('Checkpoint POST intercepted before transport; no live record write');
    }
    assert.ok(['127.0.0.1','localhost'].includes(u.hostname) && u.port === '8787');
    assert.ok(allowed.get(u.pathname)?.includes(method) ||
      (u.pathname === '/v1/work-loop/pause' && pauseEnabled && allowPause && method === 'POST'),
      `Refuse unexpected operation: ${method} ${u.pathname}`);
    const headers = Object.fromEntries(new Headers(init.headers || {}));
    assert.ok(!headers.authorization && !headers.cookie, 'No credential-bearing diagnostic transport');
    if (u.pathname === '/v1/work-loop/pause') {
      assert.ok(pauseEnabled && allowPause && method === 'POST', 'Only the expressly authorized idle-scope pause is allowed');
      assert.equal(++pauseAttempts, 1, 'Never replay a control request');
      assert.equal(headers['x-scope-project-root'], root);
      assert.equal(headers['x-scope-continuity-id'], continuity);
      assert.equal(headers['x-focusa-writer-id'], `pi-${process.pid}`, 'Actual caller identity only');
      assert.equal(headers['x-focusa-fencing-token'], undefined, 'Never manufacture a fence');
      const body = JSON.parse(String(init.body || '{}'));
      assert.notEqual(body.renew_budget, true);
      for (const key of ['max_turns','max_wall_clock_ms','max_retries','force','approved']) assert.equal(body[key], undefined);
      pauseRequests++;
    }
    requests.push({ method, path: u.pathname });
    const raw = execFileSync('ssh', ['-F', process.env.FOCUSA_LIVE_SDK_SSH_CONFIG,
      '-o','BatchMode=yes','-o','ConnectTimeout=5','-o','StrictHostKeyChecking=yes',
      process.env.FOCUSA_LIVE_SDK_SSH_ALIAS, "python3 -c '" + relay.replaceAll("'", "'\\''") + "'"],
      { input: JSON.stringify({ url: u.href, method, headers, body: init.body ? String(init.body) : '' }),
        encoding: 'utf8', timeout: 12000, maxBuffer: 2 * 1024 * 1024 });
    const response = JSON.parse(raw);
    if (u.pathname === '/v1/work-loop/status' && response.status === 200) {
      const body = JSON.parse(response.body);
      assert.deepEqual(body.execution_partition?.project_root_key, expected.project_root_key, 'Actual project selection must match independently read scope');
      assert.deepEqual(body.execution_partition?.workstream_key, expected.workstream_key, 'Actual workstream selection must match independently read scope');
      assert.equal(body.active_writer, null, 'Live test is limited to the already-observed unclaimed loop');
      lastLoopBody = body;
      verifiedLoopReads++;
    }
    if (u.pathname === '/v1/work-loop/pause') pauseHttpStatus = response.status;
    return new Response(response.body, { status: response.status, headers: response.headers });
  };
  const { default: factory } = await import(new URL('../src/index.ts', import.meta.url));
  const { createExtensionRuntime, loadExtensionFromFactory } = await import(pathToFileURL(sdk + '/extensions/loader.js').href);
  const { createEventBus } = await import(pathToFileURL(sdk + '/event-bus.js').href);
  const { ExtensionRunner } = await import(pathToFileURL(sdk + '/extensions/runner.js').href);
  const { SessionManager } = await import(pathToFileURL(sdk + '/session-manager.js').href);
  const session = SessionManager.inMemory(root), runtime = createExtensionRuntime();
  const extension = await loadExtensionFromFactory(factory, root, createEventBus(), runtime, 'actual-loop-reporting-consumer');
  const runner = new ExtensionRunner([extension], runtime, root, session, undefined);
  const ctx = runner.createContext();
  const execute = (name, params) => extension.tools.get(name).definition.execute(name, params, undefined, undefined, ctx);
  const unverifiedSetup = await execute('focusa_north_star_gate', { trigger: 'foundation_before_normal_scope_recovery' });
  assert.equal(unverifiedSetup.details.canonical, false);
  assert.deepEqual(unverifiedSetup.details.next_tools, ['focusa_agent_prompt', 'focusa_project_identity', 'focusa_project_bootstrap', 'focusa_trajectory_view', 'focusa_project_genesis', 'focusa_workpoint_resume']);
  assert.match(unverifiedSetup.content[0].text, /inspect bootstrap\/Genesis status/);
  await execute('focusa_agent_prompt', {});
  // Normal existing-scope selection; no new project, trajectory, grant or writer.
  process.env.FOCUSA_PROJECT_ROOT = root;
  process.env.FOCUSA_CONTINUITY_ID = continuity;
  const resumePacket = await execute('focusa_workpoint_resume', { project_root: root, continuity_id: continuity, mode: 'compact_prompt' });
  assert.equal(resumePacket.details?.canonical, true, 'Normal canonical read recovery must succeed');
  await execute('focusa_project_identity', { cwd: root, project_root: root });
  const writer = await execute('focusa_work_loop_writer_status', {});
  assert.equal(writer.details.writer_observation_verified, true);
  assert.equal(writer.details.active_writer, 'none');
  const preview = await execute('focusa_work_loop_control', { action: 'pause', preflight: true });
  assert.match(preview.content[0].text, /route preview only/);
  assert.equal(preview.details.request_sent, false);
  assert.equal(preview.details.canonical, false);
  assert.equal(preview.details.execution_admission_checked, false);
  const resume = await execute('focusa_work_loop_control', { action: 'resume', renew_budget: false });
  assert.equal(resume.details.ok, false);
  assert.equal(resume.details.status, 'degraded');
  assert.equal(resume.details.failure_class, 'writer_lease_unverified');
  assert.equal(resume.details.request_sent, false);
  assert.equal(resume.details.canonical, false);
  assert.match(resume.content[0].text, /resume request not sent/);
  assert.doesNotMatch(resume.content[0].text, /blocked|owned by another writer/);
  assert.ok(verifiedLoopReads >= 2);
  assert.ok(requests.filter(x => x.path.startsWith('/v1/work-loop/')).every(x => x.method === 'GET'));
  const ready = await execute('focusa_north_star_gate', { trigger: 'foundation_recovery_existing_scope' });
  assert.equal(ready.details.ok, true);
  assert.deepEqual(ready.details.next_tools, ['focusa_workpoint_resume'], 'Valid setup must not be restarted');
  if (pauseEnabled) {
    const before = structuredClone(lastLoopBody);
    assert.equal(before.status, 'idle', 'Refuse to disturb running or already-paused work');
    assert.equal(before.active_writer, null);
    assert.equal(before.budget_remaining.remaining_wall_clock_ms, 0);
    allowPause = true;
    const pause = await execute('focusa_work_loop_control', { action: 'pause', renew_budget: false,
      reason: 'Operator-requested single live pause verification; no worker, ownership change or budget renewal.' });
    allowPause = false;
    await execute('focusa_work_loop_status', {});
    const after = structuredClone(lastLoopBody);
    const remainingBudget = (observation) => {
      const { elapsed_wall_clock_ms: _elapsedObservation, ...remaining } = observation;
      return remaining;
    };
    pauseEvidence = { http_status: pauseHttpStatus, control_requests: pauseRequests,
      api_ok: pause.details?.ok, response: pause.details?.response,
      before_status: before.status, after_status: after.status,
      remaining_budget_unchanged: JSON.stringify(remainingBudget(before.budget_remaining)) === JSON.stringify(remainingBudget(after.budget_remaining)),
      budget_observation_before: before.budget_remaining,
      budget_observation_after: after.budget_remaining,
      bounds_unchanged: JSON.stringify(before.bounds) === JSON.stringify(after.bounds),
      writer_unchanged: before.active_writer === after.active_writer,
      durability_verified: false, installed_adapter_fixed: false };
    writeFileSync(evidence + '.pause.json', JSON.stringify(pauseEvidence, null, 2));
    assert.equal(pauseRequests, 1);
    assert.equal(pauseEvidence.remaining_budget_unchanged, true);
    assert.equal(pauseEvidence.bounds_unchanged, true);
    assert.equal(pauseEvidence.writer_unchanged, true);
    assert.equal(pause.details.ok, true, `Installed daemon refused pause: HTTP ${pauseHttpStatus}`);
    assert.equal(after.status, 'paused', 'A transport acknowledgement alone is not effect proof');
  }
  const checkpointBase = { project_root: root, continuity_id: continuity,
    mission: 'Preserve the approved existing Focusa work', current_action: 'inspect_existing_artifact',
    next_action: 'Inspect the existing artifact and record the verification result',
    lifecycle_stage: 'implement', do_not_drift: ['Preserve the existing approved scope'] };
  const checkpointInvalid = [
    ['next_action', { next_action: '' }],
    ['next_action', { next_action: ' \t\n' }],
    ['mission', { mission: ' \n' }],
    ['lifecycle_stage', { lifecycle_stage: undefined }],
    ['lifecycle_stage', { lifecycle_stage: 'unknown' }],
    ['current_action', { current_action: ' \t' }],
  ];
  for (const [field, changed] of checkpointInvalid) {
    const attemptsBefore = checkpointWrites.length;
    const rejected = await execute('focusa_workpoint_checkpoint', { ...checkpointBase, ...changed });
    assert.equal(rejected.details.request_sent, false, `${field}: invalid checkpoint must be unsent`);
    assert.equal(rejected.details.canonical, false);
    assert.equal(rejected.details.failure_class, 'validation_rejected');
    assert.ok(rejected.details.missing_fields.includes(field));
    assert.match(rejected.content[0].text, /request not sent/i);
    assert.equal(checkpointWrites.length, attemptsBefore, `${field}: invalid input reached POST`);
  }
  await execute('focusa_workpoint_checkpoint', checkpointBase);
  assert.equal(checkpointWrites.length, 1, 'Complete canonical input must still reach the existing transport');
  const complete = checkpointWrites[0];
  assert.equal(complete.mission, checkpointBase.mission);
  assert.equal(complete.action_intent.action_type, checkpointBase.current_action);
  assert.equal(complete.action_intent.lifecycle_stage, 'implement');
  assert.equal(complete.next_slice, `${checkpointBase.next_action}\nDO_NOT_DRIFT: Preserve the existing approved scope`);
  assert.equal(complete.canonical, true);
  assert.equal(complete.promote, true);
  const baselineTransportRefusal = await execute('focusa_workpoint_checkpoint', { ...checkpointBase, canonical: false,
    lifecycle_stage: undefined, current_action: undefined });
  assert.equal(checkpointWrites.length, 2, 'Noncanonical stage omission retains its existing compatibility');
  assert.equal(checkpointWrites[1].canonical, false);
  assert.equal(checkpointWrites[1].promote, false);
  assert.equal(checkpointWrites[1].action_intent.action_type, 'checkpoint_workpoint');
  assert.equal(checkpointWrites[1].action_intent.lifecycle_stage, undefined);
  // A process-local, one-shot observation failure; no actual permissions or
  // caller identity changes, and every checkpoint remains intercepted.
  const originalStat = fs.statSync;
  let probeArmed = false, probeFailureObserved = false;
  checkpointProbeHook = () => { probeArmed = true; };
  fs.statSync = (path, ...args) => {
    if (probeArmed && String(path) === root) {
      probeArmed = false;
      probeFailureObserved = true;
      throw Object.assign(new Error('simulated EACCES in advisory ownership observation'), { code: 'EACCES' });
    }
    return originalStat(path, ...args);
  };
  syncBuiltinESMExports();
  try {
    const refusal = await execute('focusa_workpoint_checkpoint', { ...checkpointBase, canonical: false });
    assert.equal(probeFailureObserved, true);
    assert.equal(refusal.details.ok, false);
    const posture = refusal.details.project_root_permission_posture;
    assert.equal(posture.permission_observation_verified, false);
    assert.equal(posture.posture, 'unverified');
    assert.equal(posture.error_code, 'EACCES');
    assert.deepEqual(refusal.details.response, baselineTransportRefusal.details.response,
      'A failed diagnostic probe must preserve the existing structured operation error');
    assert.equal(refusal.details.response.failure_class, 'daemon_unavailable');
    assert.equal(refusal.details.response.endpoint, '/v1/workpoint/checkpoint');
  } finally {
    fs.statSync = originalStat;
    syncBuiltinESMExports();
    checkpointProbeHook = null;
  }
  assert.equal(checkpointWrites.length, 3);
  // Simulate only observed stat metadata after interception. The actual OS user,
  // request identity, directory ownership and permissions are never changed.
  const observer = userInfo();
  const observedUid = observer.uid === 0 ? 1 : 0;
  const homeAccount = root.match(/^\/home\/([^/]+)(?:\/|$)/)?.[1] || null;
  let ownerProbeArmed = false, ownerProbeObserved = false;
  checkpointProbeHook = () => { ownerProbeArmed = true; };
  fs.statSync = (path, ...args) => {
    const metadata = originalStat(path, ...args);
    if (ownerProbeArmed && String(path) === root) {
      ownerProbeArmed = false;
      ownerProbeObserved = true;
      return Object.assign(Object.create(Object.getPrototypeOf(metadata)), metadata, { uid: observedUid });
    }
    return metadata;
  };
  syncBuiltinESMExports();
  try {
    const result = await execute('focusa_workpoint_checkpoint', { ...checkpointBase, canonical: false });
    const posture = result.details.project_root_permission_posture;
    writeFileSync(evidence + '.owner-observation.json', JSON.stringify({
      owner_probe_observed: ownerProbeObserved,
      observed_uid_preserved: posture.root_owner?.uid === observedUid,
      owner_label_matches_observed_uid: posture.root_owner?.user === `uid:${observedUid}`,
      owner_label_inferred_from_home_account: homeAccount !== null && posture.root_owner?.user === homeAccount,
      caller_identity_unchanged: userInfo().uid === observer.uid && userInfo().username === observer.username,
      checkpoint_network_writes: 0,
    }, null, 2));
    assert.equal(ownerProbeObserved, true);
    assert.equal(posture.root_owner.uid, observedUid);
    assert.equal(posture.root_owner.user, `uid:${observedUid}`, 'A path account is not proof of the observed UID owner name');
    assert.equal(posture.root_owner.user_name_verified, false);
    assert.equal(posture.home_account, homeAccount);
    assert.equal(posture.root_owned_by_current_user, false);
    assert.equal(posture.posture, observer.username === homeAccount ? 'home_ownership_mismatch' : 'cross_user_home_use_as_owner');
    assert.deepEqual(result.details.response, baselineTransportRefusal.details.response);
    assert.equal(userInfo().uid, observer.uid);
    assert.equal(userInfo().username, observer.username);
  } finally {
    fs.statSync = originalStat;
    syncBuiltinESMExports();
    checkpointProbeHook = null;
  }
  assert.equal(checkpointWrites.length, 4);
  const diagnosticReads = requests.filter(x => x.path === '/v1/project/north-star-gate').length;
  const invalidSelectors = await execute('focusa_north_star_gate', { project_root: '/root', continuity_id: continuity, trigger: 'unsupported_selector_negative' });
  assert.equal(invalidSelectors.details.canonical, false);
  assert.equal(invalidSelectors.details.request_sent, false);
  assert.equal(invalidSelectors.details.failure_class, 'validation_rejected');
  assert.deepEqual(invalidSelectors.details.unsupported_parameters, ['project_root', 'continuity_id']);
  assert.equal(requests.filter(x => x.path === '/v1/project/north-star-gate').length, diagnosticReads);
  assert.match(invalidSelectors.content[0].text, /not selector inputs/);
  // Supplemental source-boundary check, not a claim of live incomplete-project or model acceptance.
  // No new logical scope is created to manufacture the negative case.
  const { northStarRecoveryTools, northStarRecoveryNotice } = await import(new URL('../src/north-star.ts', import.meta.url));
  assert.deepEqual(northStarRecoveryTools(), ['focusa_agent_prompt', 'focusa_project_identity', 'focusa_project_bootstrap', 'focusa_trajectory_view', 'focusa_project_genesis', 'focusa_workpoint_resume']);
  assert.match(northStarRecoveryNotice(), /Read the relevant guidance/);
  assert.match(northStarRecoveryNotice(), /inspect bootstrap\/Genesis status/);
  assert.match(northStarRecoveryNotice(), /existing operator-confirmed goal hierarchy/);
  writeFileSync(evidence, JSON.stringify({ status: 'actual_native_loop_reporting_verified',
    scope_verified: true, verified_loop_reads: verifiedLoopReads, requests,
    writer_text: writer.content[0].text, preview_text: preview.content[0].text,
    resume_text: resume.content[0].text, control_request_sent: pauseHttpStatus !== null,
    control_request_submitted: pauseRequests === 1, control_delivery_unverified: pauseRequests === 1 && pauseHttpStatus === null,
    pause_evidence: pauseEvidence,
    budget_renewed: false, model_or_worker_dispatch: false, model_behavior_measured: false,
    foundation_recovery_source_boundary_checked: true, live_unverified_client_setup_measured: true,
    live_incomplete_project_measured: false,
    valid_existing_setup_reused: true, unsupported_selector_request_rejected: true,
    invalid_checkpoint_inputs_unsent: checkpointInvalid.length,
    checkpoint_writes_intercepted: checkpointWrites.length, checkpoint_network_writes: 0,
    canonical_checkpoint_mapping_checked: true, noncanonical_stage_omission_compatible: true,
    permission_probe_failure_preserved: probeFailureObserved,
    owner_observation_label_verified: ownerProbeObserved, caller_identity_unchanged: true,
    installed: false, issue693_closed: false }, null, 2));
});
