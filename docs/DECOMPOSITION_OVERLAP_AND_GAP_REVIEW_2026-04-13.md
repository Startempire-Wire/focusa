# Decomposition Overlap and Gap Review — 2026-04-13

Purpose:
- identify where current decomposition is still thin, overlapping, or likely to drift into fake coverage
- guide additional passes without pretending the hierarchy is finished

## Confirmed strengths so far

- every post-cutoff doc 51-78 now has explicit bead coverage
- first implementation frontier is ordered before downstream tracks
- multiple branches now reach child/grandchild level
- exact code loci are identified for the Pi hot path and several API/test surfaces
- sparse later-doc branches are explicitly marked as sparse rather than treated as implemented

## Remaining thin areas

### Thin doc coverage
These docs still have comparatively thin decomposition and should receive more passes:
- **54** visible-output boundary
- **61** domain-general cognition core
- **71** governing priors
- **72** identity/role/self-model
- **73** intention/commitment/self-regulation
- **74** identity/reference resolution
- **75** projection/view semantics
- **76** retention/decay
- **78** bounded secondary cognition remainder

### Thin code-locus certainty
These branches still need deeper code mapping:
- visual/UI docs 58-65
- affordance doc 66
- shared-substrate docs 72-76 outside the already-identified test/route hints

## Overlap risks

### Intentional overlap
Some overlap is correct because one doc feeds another. Examples:
- docs 51, 54a, 54b, 67, 68, 69 all converge in the first-frontier Pi routing branch
- docs 55, 56, 57 partially overlap in route metadata, checkpoints, and eval surfaces
- docs 70-77 overlap through the shared-substrate branch

This overlap is acceptable when each bead has a distinct role:
- routing logic
- contract metadata
- trace proof
- eval governance
- migration compatibility

### Risky overlap
Potentially confusing overlap remains in:
- projection vs bounded world projection tests
- retention vs generic note-decay wording
- identity/reference docs vs existing generic handles/reference routes
- doc 78 autonomy work vs older autonomy/governance beads

These need more precise acceptance conditions in future passes.

## Hidden-gap risks

1. a doc mapped to one bead may still be under-specified
2. test/eval beads may prove infrastructure only, not behavior
3. route metadata may look complete while runtime remains drifted
4. TUI/view code may be mistaken for visual ontology implementation
5. older autonomy/proposal surfaces may be mistaken for doc-78 completion

## Required future passes

1. split thin docs into narrower grandchildren with exact file/consumer/proof targets
2. add acceptance-condition notes to ambiguous beads
3. mark which beads are prerequisite-only vs directly executable implementation work
4. reconcile doc-78 branch against all pre-existing autonomy/governance beads
5. reconcile visual/UI branch against any actual screenshot/evidence tooling before claiming implementation frontier depth

## Current judgment

The decomposition is now materially non-lazy and repo-grounded, but it is **not final**.
More refinement passes are still warranted, especially for sparse later-doc branches and shared-substrate semantics.
