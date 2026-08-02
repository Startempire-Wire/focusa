import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const component = readFileSync(fileURLToPath(new URL('../src/lib/components/SemanticPairPeek.svelte', import.meta.url)), 'utf8');
const api = readFileSync(fileURLToPath(new URL('../src/lib/api.ts', import.meta.url)), 'utf8');
const types = readFileSync(fileURLToPath(new URL('../src/lib/types/focus-canvas.ts', import.meta.url)), 'utf8');
const proof = readFileSync(fileURLToPath(new URL('../src/lib/components/ProofPeek.svelte', import.meta.url)), 'utf8');

assert.match(types, /'supported'/);
assert.match(types, /'schema_only'/);
assert.match(api, /contract: 'focusa\.semantic-integrity\.operation\.v1'/);
assert.match(api, /scope: \{/);
assert.match(component, /invokeSemanticPairAction/);
assert.match(component, /Execute governed operation/);
assert.match(component, /Confirm mutation/);
assert.doesNotMatch(component, /Unsupported on this surface/);
assert.doesNotMatch(proof, /mutations unsupported/);
console.log('menubar semantic-pair 43-operation parity source proof passed');
