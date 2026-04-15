# Legacy Bead Reconciliation — 2026-04-13

Purpose:
- prevent old coarse decomposition beads from being mistaken for sufficient decomposition
- map legacy beads onto the newer cutoff-driven hierarchy

## Legacy coarse beads found

- `focusa-3clf` — Decompose newer ontology/trajectory docs 51-78 into executable beads
- `focusa-hd8j` — Decompose Pi contract trajectory docs 51-57 into executable beads
- `focusa-w784` — Decompose visual/UI trajectory docs 58-65 into executable beads
- `focusa-rucx` — Decompose affordance/execution environment + scope trajectory docs 66-69 into executable beads
- `focusa-q1wh` — Decompose shared ontology/governance trajectory docs 70-77 into executable beads
- `focusa-l8e4` — Normalize existing newer-trajectory beads against docs 51-78

## Reconciliation judgment

These legacy beads were directionally useful, but too coarse.
They should now be treated as:
- umbrella / transition beads
- not proof that decomposition is complete
- pointers into the newer cutoff-driven hierarchy

## Replacement mapping

### `focusa-3clf`
Superseded by:
- `focusa-zerg`
- all artifacts in `docs/IMPLEMENTATION_*`, `docs/POST_CUTOFF_*`, `docs/*REVIEW*`, `docs/*MAP*`

Role now:
- historical umbrella only

### `focusa-hd8j`
Superseded by:
- first-frontier branch `focusa-7u1f`
- Pi contract/action/eval branch `focusa-e3id`
- doc-specific beads for 54, 55 impl, 57

Role now:
- coarse predecessor to the real docs 51-57 hierarchy

### `focusa-w784`
Superseded by:
- `focusa-s2z6`
- `focusa-6pd1`
- `focusa-ost9`
- doc-specific visual beads 58/59/60/62/63/64/65

Role now:
- coarse predecessor to the visual hierarchy

### `focusa-rucx`
Superseded by:
- first-frontier routing branch for docs 67-69
- `focusa-wmw7` for doc 66

Role now:
- mixed-scope predecessor that should not hide doc-66 vs docs-67-69 differences

### `focusa-q1wh`
Superseded by:
- `focusa-jz89`
- `focusa-3zav`
- `focusa-2m6e`
- `focusa-eczn`
- `focusa-v2n5`
- `focusa-ru3s`
- doc-specific 71-77 branches

Role now:
- coarse predecessor to the shared-substrate hierarchy

### `focusa-l8e4`
Still useful as:
- reconciliation / normalization meta-bead

But now should point to:
- this reconciliation document
- doc-to-bead map
- overlap/gap review

## Rule going forward

If a legacy coarse bead appears complete while its replacement hierarchy is still active:
- trust the replacement hierarchy
- treat the legacy bead as historical umbrella only
- do not use the legacy bead as evidence that decomposition is done
