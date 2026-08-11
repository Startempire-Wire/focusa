import assert from 'node:assert/strict';
import { readFile, writeFile, unlink } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

let dependencyRequire = createRequire(new URL('../package.json', import.meta.url));
try {
  dependencyRequire.resolve('svelte/compiler');
} catch {
  const projectMarker = JSON.parse(
    await readFile(new URL('../../../.focusa-project.json', import.meta.url), 'utf8')
  );
  dependencyRequire = createRequire(
    join(projectMarker.project_root, 'apps/menubar/package.json')
  );
}
const compilerModule = await import(
  pathToFileURL(dependencyRequire.resolve('svelte/compiler')).href
);
const { compile } = compilerModule.default ?? compilerModule;
const { render } = await import(
  pathToFileURL(dependencyRequire.resolve('svelte/server')).href
);
const internalServerUrl = pathToFileURL(
  dependencyRequire.resolve('svelte/internal/server')
).href;

const sourcePath = new URL('../src/lib/components/TemporalAuthorityPeek.svelte', import.meta.url);
const generatedPath = new URL('./.temporal-authority-peek-test.mjs', import.meta.url);
const source = await readFile(sourcePath, 'utf8');
const testableSource = source
  .replace("  import { runtimeStore } from '$lib/stores/runtime.svelte';\n\n", '')
  .replace('  let snapshot = $derived(runtimeStore.snapshot);', '  let { snapshot = {} } = $props();');
const compiled = compile(testableSource, {
  filename: 'TemporalAuthorityPeek.svelte',
  generate: 'server',
});
const generatedCode = compiled.js.code
  .replaceAll("'svelte/internal/server'", JSON.stringify(internalServerUrl))
  .replaceAll('"svelte/internal/server"', JSON.stringify(internalServerUrl));
await writeFile(generatedPath, generatedCode);

try {
  const { default: TemporalAuthorityPeek } = await import(
    `${pathToFileURL(generatedPath.pathname).href}?v=${Date.now()}`
  );
  const proven = render(TemporalAuthorityPeek, {
    props: {
      snapshot: {
        temporal: {
          status: 'projected',
          deadline_status: 'approaching',
          slack_ms: 3000,
          critical_path_ms: 12000,
          observed_duration_count: 4,
          approaching_deadlines: [{ claim_id: 'deadline-1' }],
          deadline_conflict_state: 'overcommitted',
          human_calendar_context: { context_id: 'calendar-1' },
          authorized_forecast_range: {
            p50_ms: 8000,
            p95_ms: 16000,
            confidence: 'grounded',
          },
          urgency: { subject_ref: 'release-proof' },
          last_material_progress_at: '2026-08-11T20:00:00Z',
          no_progress_age_ms: 4500,
          lost_time_incident_count: 2,
          opportunity_posture: 'risk',
          cancellation_state: 'none',
          warnings: ['deadline conflict requires operator revision'],
          conformance: {
            full_conformance_status: 'blocked_live_proof_required',
            warnings: ['two exact-scope installed runs remain'],
          },
        },
      },
    },
  }).body;
  for (const expected of [
    'projected',
    'approaching',
    '3000 ms',
    '12000 ms',
    'overcommitted',
    '8000–16000 ms (grounded)',
    'release-proof',
    '2026-08-11T20:00:00Z',
    '4500 ms',
    '2',
    'risk',
    'blocked_live_proof_required',
    'deadline conflict requires operator revision',
    'two exact-scope installed runs remain',
  ]) {
    assert.match(proven, new RegExp(expected.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }

  const unavailable = render(TemporalAuthorityPeek, { props: { snapshot: {} } }).body;
  assert.match(unavailable, /unavailable/);
  assert.match(unavailable, /<dt[^>]*>Forecast<\/dt><dd[^>]*>none<\/dd>/);
  assert.match(unavailable, /<dt[^>]*>Urgency<\/dt><dd[^>]*>none<\/dd>/);
  assert.match(unavailable, /<dt[^>]*>Conformance<\/dt><dd[^>]*>unknown<\/dd>/);
  assert.match(unavailable, /No exact temporal authority/);
  assert.doesNotMatch(unavailable, /approaching|overcommitted|grounded/);
} finally {
  await unlink(generatedPath).catch(() => {});
}

console.log('Temporal authority menubar rendering: PASS (grounded projection + fail-closed unavailable state)');
