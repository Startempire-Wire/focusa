import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const page = readFileSync(new URL('../src/routes/+page.svelte', import.meta.url), 'utf8');
const manifest = readFileSync(new URL('../src/lib/shell/workspace-manifest.ts', import.meta.url), 'utf8');
const health = readFileSync(new URL('../src/lib/shell/daemon-health.ts', import.meta.url), 'utf8');
const tauri = readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8');

assert.match(page, /Focusa Desktop/);
assert.match(page, /No Workstream attached/);
assert.match(page, /No canonical state duplicated/);
assert.match(page, /UIAI Engine browser proof/);
assert.match(page, /Switch between TUI and Mission Canvas/);
assert.match(manifest, /mission-deck/);
assert.match(manifest, /pi-work-surface/);
assert.match(manifest, /agent-runtime/);
assert.match(health, /method: 'GET'/);
assert.doesNotMatch(health, /method: '(POST|PUT|PATCH|DELETE)'/);
assert.equal(JSON.parse(tauri).identifier, 'com.focusa.desktop');

console.log('Focusa Desktop 5% shell contract: PASS');
