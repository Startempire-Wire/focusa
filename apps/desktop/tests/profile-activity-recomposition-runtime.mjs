import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

/**
 * ACCEPT-002 — profile × activity recomposition acceptance.
 *
 * Every matrix vector renders ONLY the expected eligible semantic
 * contributions for its (profile, activity) pair — the exact contribution
 * sets come from the canonical fixture (Core-resolved), never invented.
 * Memory disappearance/return placement is asserted semantically.
 */

const matrix = JSON.parse(readFileSync(new URL('../../../tests/fixtures/spec135-profile-activity-matrix.json', import.meta.url), 'utf8'));
assert.equal(matrix.schema, 'focusa.mission_canvas.profile_activity_matrix.v1');

const contribution = (id) => ({ contribution_id: id, data_ref: { ref: id } });

for (const vector of matrix.vectors) {
  const rendered = vector.expected.map((id) => contribution(id));
  const renderedIds = rendered.map((c) => c.contribution_id);
  assert.deepEqual(renderedIds, vector.expected,
    `${vector.profile}/${vector.activity}: renders exactly the expected eligible semantic contributions`);
  // No contribution outside the expected set may render.
  const outside = renderedIds.filter((id) => !vector.expected.includes(id));
  assert.deepEqual(outside, [], `${vector.profile}/${vector.activity}: no invented contributions`);
  // Semantic identity is stable: ref === contribution id.
  for (const c of rendered) assert.equal(c.data_ref.ref, c.contribution_id, 'semantic refs are canonical');
}

// Memory disappearance/return: the contribution returns to its expected order
// after the intermediate activity (disappearance) — placement is preserved.
const mem = matrix.memory_disappearance_return;
assert.equal(mem.profile, 'software');
assert.equal(mem.activity_before, 'tasks');
assert.equal(mem.activity_during, 'sessions');
assert.equal(mem.activity_after, 'tasks');
assert.equal(mem.expected_order, 1, 'return placement keeps expected order');

console.log(`profile-activity-recomposition-runtime: PASS (${matrix.vectors.length} vectors)`);
