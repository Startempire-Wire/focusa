import assert from 'node:assert/strict';
import test from 'node:test';
import { loadSourceSection } from './helpers/source-section-harness.mjs';

const root = '/tmp/focusa-684-project';
const continuity = 'synthetic-684-continuity';
const accepted = 'Complete the operator-approved checkout repair while preserving customer data. '.repeat(20);

function history() {
  return { status: 'completed', project_root: root, continuity_id: continuity,
    latest_valid_for_continuity: accepted, entries_order: 'newest_first',
    entries: [{ event_id: 'synthetic-accepted-event', project_root: root,
      continuity_id: continuity, new_hlt: accepted, source: 'operator' },
    { event_id: 'synthetic-older-event', project_root: root,
      continuity_id: continuity, new_hlt: 'Superseded checkout objective', source: 'operator' }] };
}

function harness({ choice = 'D)', response = history(), onSelect, onHistory, onIdentity, onPost } = {}) {
  const runtime = { cfg: { vitalInfoPromptMode: 'prompt' }, vitalInfoPrompted: {},
    sessionFrameKey: 'synthetic-684-session', currentAsk: { text: 'Repair checkout without losing the accepted purpose.' } };
  const notices = [], calls = [], refreshes = [];
  const lifecycle = { current: true };
  const options = loadSourceSection('session.ts', 'function trajectoryDraftOptions(', 'function workpointDraftOptions(', {
    projectNameFromRoot: () => 'fixture-project', currentAskForProject: () => runtime.currentAsk.text,
    scopedWorkpointSeed: () => '',
  }, 'trajectoryDraftOptions');
  const prompt = loadSourceSection('session.ts', 'async function promptForTrajectoryIfNeeded(', 'function seedCurrentAskFromPersistedState(', {
    getAttachmentRuntime: () => runtime, getContinuityId: () => continuity,
    vitalPromptSurfaceEnabled: () => true, isProjectRootAuthoritySafe: (value) => value === root,
    trajectoryClarityForProject: () => ({ status: 'missing', recommended_action: 'define_goal' }),
    persistState: () => {}, focusaPost: () => {}, trajectoryDraftOptions: options,
    parseTrajectoryEditor: () => assert.fail('restoration must not invent or edit a generic draft'),
    ensureContinuityId: () => continuity,
    buildFocusaSessionIdentity: async () => { await onIdentity?.(runtime); return { fixture_only: true }; },
    focusaFetch: async (path, opts) => {
      calls.push({ path, opts });
      if (path.startsWith('/hlt/history?')) { await onHistory?.(runtime); return response; }
      assert.equal(path, '/trajectory/define-goal');
      await onPost?.(runtime);
      return { persisted: true };
    },
    refreshTrajectoryClarityLifecycle: async () => { refreshes.push('trajectory'); },
    refreshOntologyContextLifecycle: async () => { refreshes.push('ontology'); },
  }, 'promptForTrajectoryIfNeeded');
  const ctx = { ui: {
    select: async (_title, labels) => { await onSelect?.(runtime); return choice === null ? null : labels.find((label) => label.startsWith(choice)); },
    notify: (message, level) => notices.push({ message, level }),
    editor: () => assert.fail('restore/skip cannot open a generic goal editor'),
  } };
  return { runtime, notices, calls, refreshes, options, lifecycle,
    run: () => prompt(ctx, root, 'fixture', () => lifecycle.current) };
}

const mutations = (value) => value.calls.filter((call) => call.opts?.method === 'POST');

test('actual restore selection reads scoped history and preserves the complete accepted goal', async () => {
  const value = harness();
  assert.equal(value.options(root).find((option) => option.label.startsWith('D)')).draft, null);
  await value.run();
  const query = new URL('http://fixture.invalid' + value.calls[0].path).searchParams;
  assert.equal(query.get('project_root'), root);
  assert.equal(query.get('continuity_id'), continuity);
  assert.equal(query.get('limit'), '50');
  assert.equal(mutations(value).length, 1);
  const body = JSON.parse(mutations(value)[0].opts.body);
  assert.equal(body.long_term_goal, accepted);
  assert.equal(body.desired_end_state, accepted, 'missing history fields must not become an invented end-state template');
  assert.equal(body.goal_source, 'previous_valid_fallback');
  assert.equal(body.operator_confirmed, true);
  assert.deepEqual(body.supersession_evidence_refs, ['hlt-history-event:synthetic-accepted-event']);
  assert.equal(body.current_ask, value.runtime.currentAsk.text);
  assert.equal(body.evidence_refs, undefined, 'only the canonical request field may carry provenance');
  assert.deepEqual(value.refreshes, ['trajectory', 'ontology']);
});

test('missing or unreadable history reports a nonmutation rather than inventing a goal', async () => {
  for (const response of [null, { status: 'blocked' }, { ...history(), entries: null }, { ...history(), entries: [] }]) {
    const value = harness({ response });
    await value.run();
    assert.equal(mutations(value).length, 0);
    assert.ok(value.notices.some((notice) => notice.message.includes('Nothing was changed')));
  }
});

test('cancel and skip do not fetch or mutate accepted goals', async () => {
  for (const choice of [null, 'F)']) {
    const value = harness({ choice });
    await value.run();
    assert.equal(value.calls.length, 0);
  }
});

test('foreign or superseded history cannot be adopted', async () => {
  for (const mutate of [
    (data) => { data.project_root = '/tmp/foreign-project'; },
    (data) => { data.entries[0].continuity_id = 'foreign-continuity'; },
    (data) => { data.latest_valid_for_continuity = 'A newer accepted goal'; },
    (data) => { data.entries[0].event_id = ''; },
    (data) => { delete data.entries_order; },
    (data) => { data.entries_order = 'oldest_first'; },
  ]) {
    const response = history();
    mutate(response);
    const value = harness({ response });
    await value.run();
    assert.equal(mutations(value).length, 0);
    assert.ok(value.notices.some((notice) => notice.message.includes('Nothing was changed')));
  }
});

test('fresh operator steering wins during selection, history and identity waits', async () => {
  for (const boundary of ['onSelect', 'onHistory', 'onIdentity']) {
    const value = harness({ [boundary]: (runtime) => { runtime.currentAsk = { text: 'Stop restoration and keep the new purpose.' }; } });
    await value.run();
    assert.equal(mutations(value).length, 0, boundary);
  }
});

test('correction beyond a shared500-character prefix fences an in-flight restore', async () => {
  const prefix = 'Keep the complete current instruction. '.repeat(24);
  const value = harness({ onHistory: (runtime) => { runtime.currentAsk.text = prefix + 'Keep the new purpose instead.'; } });
  value.runtime.currentAsk.text = prefix + 'Restore the earlier purpose.';
  await value.run();
  assert.equal(mutations(value).length, 0);
});

test('the existing lifecycle fence rejects an obsolete prompt even when its attachment object is retained', async () => {
  const value = harness({ onHistory: () => { value.lifecycle.current = false; } });
  await value.run();
  assert.equal(mutations(value).length, 0);
});

test('a superseded session cannot receive an old goal response refresh', async () => {
  const value = harness({ onPost: (runtime) => { runtime.sessionFrameKey = 'new-session'; } });
  await value.run();
  assert.equal(mutations(value).length, 1, 'already-dispatched effects must not be blindly replayed');
  assert.deepEqual(value.refreshes, []);
});
