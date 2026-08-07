import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';

class FakeElement {
  constructor(attributes = {}) {
    this.attributes = new Map(Object.entries(attributes));
    this.children = [];
    this.parentElement = null;
    this.ownerDocument = null;
    this.scrollLeft = 0;
    this.scrollTop = 0;
    this.selectionStart = null;
    this.selectionEnd = null;
    this.selectionDirection = null;
    this.focused = false;
  }
  append(child) {
    child.parentElement = this;
    child.ownerDocument = this.ownerDocument;
    this.children.push(child);
    return child;
  }
  getAttribute(name) { return this.attributes.get(name) ?? null; }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  contains(candidate) { return this === candidate || this.children.some((child) => child.contains(candidate)); }
  querySelector(selector) { return this.querySelectorAll(selector)[0] ?? null; }
  querySelectorAll(selector) {
    const selectors = selector.split(',').map((item) => item.trim());
    const descendants = [];
    const visit = (element) => {
      for (const child of element.children) { descendants.push(child); visit(child); }
    };
    visit(this);
    return descendants.filter((element) => selectors.some((item) => matches(element, item)));
  }
  focus() { this.ownerDocument.activeElement = this; this.focused = true; }
  setSelectionRange(start, end, direction = 'none') {
    this.selectionStart = start;
    this.selectionEnd = end;
    this.selectionDirection = direction;
  }
}

globalThis.HTMLElement = FakeElement;

function matches(element, selector) {
  const attributes = [...selector.matchAll(/\[([^=\]]+)(?:="([^"]*)")?\]/g)];
  return attributes.length > 0 && attributes.every(([, name, value]) => {
    const observed = element.getAttribute(name);
    return value === undefined ? observed !== null : observed === value;
  });
}

function tree() {
  const document = { activeElement: null };
  const root = new FakeElement({ 'data-presentation-root': 'true' });
  root.ownerDocument = document;
  const contribution = root.append(new FakeElement({ 'data-contribution-id': 'contribution:editor' }));
  const input = contribution.append(new FakeElement({ 'data-semantic-object-id': 'editor:input' }));
  input.selectionStart = 2;
  input.selectionEnd = 5;
  input.selectionDirection = 'forward';
  contribution.scrollTop = 120;
  contribution.scrollLeft = 7;
  const tab = root.append(new FakeElement({ role: 'tab', 'aria-selected': 'true', 'aria-controls': 'panel:editor' }));
  document.activeElement = input;
  return { document, root, contribution, input, tab };
}

const desktopRoot = fileURLToPath(new URL('../', import.meta.url));
const repositoryRoot = fileURLToPath(new URL('../../', import.meta.url));
process.chdir(desktopRoot);
const server = await createServer({
  configFile: fileURLToPath(new URL('../vite.config.ts', import.meta.url)),
  appType: 'custom',
  server: { middlewareMode: true, fs: { allow: [repositoryRoot] } },
  optimizeDeps: { disabled: true },
  oxc: false,
  esbuild: { tsconfigRaw: { compilerOptions: { target: 'ES2021', module: 'ESNext' } } },
  logLevel: 'error'
});

try {
  const { capturePresentationState, restoreIfStillPresent } =
    await server.ssrLoadModule('/src/lib/mission-canvas/presentation-state.ts');
  const fixture = JSON.parse(await readFile(new URL('./fixtures/mission-canvas/populated-projection.json', import.meta.url), 'utf8'));
  fixture.projection_revision = 11;

  const prior = tree();
  const snapshot = capturePresentationState(prior.root, fixture);
  assert.equal(snapshot.projectionRevision, 11);
  assert.equal(snapshot.scroll.length, 1);
  assert.equal(snapshot.selection.start, 2);

  const next = tree();
  next.contribution.scrollTop = 0;
  next.contribution.scrollLeft = 0;
  next.input.selectionStart = 0;
  next.input.selectionEnd = 0;
  next.document.activeElement = null;
  const refreshed = structuredClone(fixture);
  refreshed.projection_revision = 12;
  assert.equal(restoreIfStillPresent(next.root, snapshot, refreshed), true);
  assert.equal(next.contribution.scrollTop, 120);
  assert.equal(next.contribution.scrollLeft, 7);
  assert.equal(next.input.selectionStart, 2);
  assert.equal(next.input.selectionEnd, 5);
  assert.equal(next.input.focused, true);

  const foreign = structuredClone(refreshed);
  foreign.workstream.workstream_id = 'ws:foreign';
  next.contribution.scrollTop = 0;
  assert.equal(restoreIfStillPresent(next.root, snapshot, foreign), false);
  assert.equal(next.contribution.scrollTop, 0);

  const stale = structuredClone(fixture);
  stale.projection_revision = 10;
  assert.equal(restoreIfStillPresent(next.root, snapshot, stale), false);

  const missing = tree();
  missing.root.children = missing.root.children.filter((child) => child !== missing.contribution);
  missing.document.activeElement = null;
  assert.equal(restoreIfStillPresent(missing.root, snapshot, refreshed), true);
  assert.equal(missing.document.activeElement, missing.tab);

  console.log('Mission Canvas presentation state: PASS (focus, scroll, selection, active tab, scope and stale revision)');
} finally {
  await server.watcher.close();
  await server.ws.close();
  if (server.httpServer) await new Promise((resolve) => server.httpServer.close(resolve));
}
