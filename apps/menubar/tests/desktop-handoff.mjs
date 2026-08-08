import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = (name) => readFileSync(resolve(root, `src/${name}`), 'utf8');
const view = read('lib/components/MissionCanvasView.svelte');
const seam = read('lib/desktop-present.ts');

// The menubar owns only bounded status/handoff responsibilities.
assert.match(view, /aria-label="Focusa Mission Canvas"/);
assert.match(view, /Open in Desktop/);
assert.match(view, /DesktopPresent\.invoke/);
assert.match(view, /requestDesktopOpen/);
assert.match(view, /bindHandoffContext/);
assert.doesNotMatch(view, /MissionCanvasShell/);
assert.doesNotMatch(view, /DesktopMissionCanvasRuntime/);

// DesktopPresent.invoke is the exact-scope handoff seam.
assert.match(seam, /export const DesktopPresent = \{/);
assert.match(seam, /invoke\(context: unknown\)/);
assert.match(seam, /failure: 'missing_authority' \| 'foreign_scope' \| 'invalid_scope'/);
assert.match(seam, /scope_kind !== 'project'/);

const valid = {
  workstream: {
    scope: { scope_kind: 'project', scope_key: { scope_id: 'project:focusa', root_path: '/example/focusa' } },
    workstream_id: 'ws:mission-canvas'
  },
  continuity_id: 'continuity:mission-canvas',
  attachment: {
    workstream: { workstream_id: 'ws:mission-canvas' },
    attachment_id: 'attachment:pi'
  },
  work_surface_id: 'surface:pi'
};

const { DesktopPresent } = await import(`file://${resolve(root, 'src/lib/desktop-present.ts')}?t=${Date.now()}`);

const ok = DesktopPresent.invoke(valid);
assert.equal(ok.ok, true);
assert.equal(ok.ok && ok.intent.action, 'desktop_open');
assert.equal(ok.ok && ok.intent.workstream_id, 'ws:mission-canvas');
assert.equal(ok.ok && ok.intent.attachment_bound, true);

assert.deepEqual(DesktopPresent.invoke(undefined), { ok: false, failure: 'missing_authority' });
assert.deepEqual(DesktopPresent.invoke(null), { ok: false, failure: 'missing_authority' });
assert.deepEqual(DesktopPresent.invoke({}), { ok: false, failure: 'missing_authority' });

const foreign = structuredClone(valid);
foreign.attachment.workstream.workstream_id = 'ws:foreign';
assert.deepEqual(DesktopPresent.invoke(foreign), { ok: false, failure: 'foreign_scope' });

const notProject = structuredClone(valid);
notProject.workstream.scope.scope_kind = 'host';
assert.deepEqual(DesktopPresent.invoke(notProject), { ok: false, failure: 'invalid_scope' });

const missingRoot = structuredClone(valid);
delete missingRoot.workstream.scope.scope_key.root_path;
assert.deepEqual(DesktopPresent.invoke(missingRoot), { ok: false, failure: 'invalid_scope' });

console.log('Menubar Desktop handoff: PASS (bounded status/resume/pairing/lifecycle/desktop-open, exact-scope invoke, fail-closed foreign/missing/invalid)');
