import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(new URL('..', import.meta.url).pathname, 'src');
const html = fs.readFileSync(path.join(root, 'sidepanel.html'), 'utf8');
const css = fs.readFileSync(path.join(root, 'styles.css'), 'utf8');

const requiredIds = [
  'connection-status', 'connection-select', 'pair-form', 'pair-check',
  'orientation-form', 'capture-tab', 'observation-summary', 'mission-preview',
  'creation-form', 'preflight', 'create-draft', 'start-session',
  'refresh-roster', 'roster', 'stream-status', 'audit',
];

test('sidepanel preserves behavior hooks and accessible landmarks', () => {
  assert.match(html, /<main>/);
  assert.match(html, /<header[^>]*class="app-header"/);
  for (const id of requiredIds) assert.match(html, new RegExp(`id="${id}"`), id);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /aria-labelledby="orientation-heading"/);
  assert.match(html, /<script type="module" src="sidepanel\.mjs"><\/script>/);
});

test('sidepanel visual system supports responsive, light, and reduced-motion users', () => {
  assert.match(css, /@media\(max-width:420px\)/);
  assert.match(css, /prefers-color-scheme:light/);
  assert.match(css, /prefers-reduced-motion:reduce/);
  assert.match(css, /:focus-visible/);
  assert.match(css, /backdrop-filter:blur/);
});
