import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { loadPureTypeScript } from './helpers/registered-tool-harness.mjs';

const binding = loadPureTypeScript(new URL('../src/workpoint-request-binding.ts', import.meta.url));
const state = readFileSync(new URL('../src/state.ts', import.meta.url), 'utf8');

test('generic missions fail closed', () => {
  for (const mission of [undefined, null, '', '   ', 'test', 'Test', '  TEST  ', 'untitled', 'unknown', 'unspecified mission']) {
    assert.equal(binding.isGenericWorkpointMission(mission), true, `must reject ${JSON.stringify(mission)}`);
  }
});

test('specific missions are never rejected by the guard', () => {
  for (const mission of [
    'Give current tally of open issues vs closed on Focusa repo',
    'test the login flow',
    'testing strategy for rollout',
    'a',
  ]) {
    assert.equal(binding.isGenericWorkpointMission(mission), false, `must accept ${JSON.stringify(mission)}`);
  }
});

test('scope check wires the generic-mission guard', () => {
  assert.match(state, /import \{ isGenericWorkpointMission \} from "\.\/workpoint-request-binding\.js"/);
  assert.match(
    state,
    /isWorkpointPacketScopedToCurrentSession[\s\S]*?isGenericWorkpointMission\(packet\.mission\)/,
    'isWorkpointPacketScopedToCurrentSession must reject generic missions before returning true',
  );
});

console.log('Issue #624 generic-mission resume guard passed');
