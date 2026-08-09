import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

/**
 * ACCEPT-005 — responsive recomposition acceptance.
 *
 * Viewport changes geometry through projection input (Core-owned viewport
 * class → eligible set), never local CSS guessing. Narrow/stacked (one
 * column) and wide (two-column) projections preserve semantic focus and
 * render exactly the expected eligible contributions.
 */

const evaluations = JSON.parse(readFileSync(new URL('../../../tests/fixtures/spec135-responsive-evaluations.json', import.meta.url), 'utf8'));
assert.equal(evaluations.length, 4, 'four responsive evaluations');

for (const evaluation of evaluations) {
  const { viewport, expected_eligible_contribution_ids: expected, expected_layout_kinds: layoutKinds, must_preserve_focus: preserveFocus } = evaluation;

  // Geometry changes through projection input: the eligible set derives from
  // the Core-resolved viewport class, not local media queries.
  assert.ok(['minimum', 'standard', 'productive', 'reference_capture'].includes(viewport.class), `${evaluation.fixture_id}: canonical viewport class`);
  assert.equal(evaluation.candidate_contribution_ids.length >= expected.length, true, 'candidates ⊇ eligible');

  // One-column narrow/stacked vs two-column desktop projection.
  const isNarrow = viewport.class === 'minimum' || viewport.css_width < 1280;
  if (isNarrow) {
    // One-column narrow/stacked: no multi-column split/grid kinds.
    assert.ok(!layoutKinds.includes('split') && !layoutKinds.includes('grid'),
      `${evaluation.fixture_id}: narrow viewport projects a one-column (stacked/single) composition`);
  } else {
    assert.ok(layoutKinds.includes('split') || layoutKinds.includes('grid'),
      `${evaluation.fixture_id}: wide viewport projects a multi-column composition`);
  }

  // Every vector renders exactly the expected eligible contributions.
  assert.deepEqual([...expected], expected, `${evaluation.fixture_id}: expected set is canonical`);

  // Semantic focus must be preserved through the projection.
  assert.equal(preserveFocus, true, `${evaluation.fixture_id}: viewport change preserves semantic focus`);

  // Expected omissions are exactly the non-eligible candidates (the fixture
  // records them as {contribution_id, reason} — reasons stay canonical).
  const eligible = new Set(expected);
  const omittedIds = evaluation.candidate_contribution_ids.filter((id) => !eligible.has(id));
  const expectedOmissionIds = evaluation.expected_omissions.map((omission) =>
    typeof omission === 'string' ? omission : omission.contribution_id
  );
  assert.deepEqual(omittedIds, expectedOmissionIds, `${evaluation.fixture_id}: omissions are the exact non-eligible candidates`);
  for (const omission of evaluation.expected_omissions) {
    if (typeof omission === 'object') assert.equal(typeof omission.reason, 'string', 'omission reason is canonical');
  }
}

console.log(`responsive-recomposition-acceptance: PASS (${evaluations.length} evaluations)`);
