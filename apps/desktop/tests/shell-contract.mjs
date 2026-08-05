import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const page = readFileSync(new URL('../src/routes/+page.svelte', import.meta.url), 'utf8');
const manifest = readFileSync(new URL('../src/lib/shell/workspace-manifest.ts', import.meta.url), 'utf8');
const health = readFileSync(new URL('../src/lib/shell/daemon-health.ts', import.meta.url), 'utf8');
const sidebarPreferences = readFileSync(new URL('../src/lib/shell/sidebar-preferences.ts', import.meta.url), 'utf8');
const tokens = readFileSync(new URL('../static/styles/tokens.css', import.meta.url), 'utf8');
const designSystem = readFileSync(new URL('../docs/design-system.md', import.meta.url), 'utf8');
const icon = readFileSync(new URL('../src/lib/ui/Icon.svelte', import.meta.url), 'utf8');
const tauri = readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8');

assert.match(page, /Focusa Desktop/);
assert.match(page, /No Workstream attached/);
assert.match(page, /No canonical state duplicated/);
assert.match(page, /UIAI Engine browser proof/);
assert.match(page, /Switch between Mission Canvas and Agent TUI/);
assert.match(page, /Expand.*sidebar/s);
assert.match(page, /Resize sidebar/);
assert.match(sidebarPreferences, /focusa\.desktop\.sidebar_preferences\.v1/);
assert.match(sidebarPreferences, /Math\.min\(320, Math\.max\(208/);
assert.match(tokens, /--layout-section-gap: 16px/);
assert.match(tokens, /--motion-normal: 210ms/);
assert.match(icon, /export type IconName/);
assert.match(designSystem, /calm cognitive cockpit/);
assert.match(designSystem, /Visual acceptance is exclusively performed through UIAI Engine/);
assert.match(manifest, /mission-deck/);
assert.match(manifest, /pi-work-surface/);
assert.match(manifest, /agent-runtime/);
assert.match(health, /method: 'GET'/);
assert.doesNotMatch(health, /method: '(POST|PUT|PATCH|DELETE)'/);
assert.equal(JSON.parse(tauri).identifier, 'com.focusa.desktop');

console.log('Focusa Desktop 5% shell contract: PASS');
