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
Superseded by the authoritative docs 51-57 replacement hierarchy:
- `focusa-7u1f` — doc 51 substrate and first-frontier routing base used by downstream Pi-contract work
- `focusa-e3id` — docs 52-57 parent implementation track
- `focusa-jtrl` — doc 54 visible-output boundary frontier
- `focusa-93sn` — doc 55 implementation-priority frontier
- `focusa-n4fo` — doc 57 golden-task/eval governance frontier

Doc-to-bead mapping (replacement hierarchy):
- doc 51 → `focusa-7u1f`
- doc 52 → `focusa-e3id`
- doc 53 → `focusa-e3id` (via behavioral-alignment child/dependents)
- doc 54 → `focusa-jtrl` (under `focusa-e3id`)
- docs 55 / 55-impl → `focusa-93sn` (under `focusa-e3id` via `focusa-vhbq`)
- doc 56 → `focusa-e3id` (via trace/checkpoint child `focusa-qs4c`)
- doc 57 → `focusa-n4fo` (under `focusa-e3id` via `focusa-qs4c`)

Role now:
- coarse predecessor to the real docs 51-57 hierarchy; never sole completion evidence

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
Superseded by the authoritative docs 70-77 replacement hierarchy:
- `focusa-jz89` — parent shared-ontology substrate branch for docs 70-77
- `focusa-3zav` — doc 70 verifiable/actionable/scoped field discipline
- `focusa-ru3s` — grouped branch for docs 71-74 decomposition
- `focusa-2m6e` — doc 75 projection/view-profile surfaces
- `focusa-eczn` — doc 76 retention/decay semantics
- `focusa-v2n5` — doc 77 governance/versioning/migration checks
- doc-specific 71-74 branches under `focusa-ru3s`: `focusa-suwi` (71), `focusa-e8wn` (72), `focusa-16us` (73), `focusa-eg8i` (74)

Doc-to-bead mapping (replacement hierarchy):
- doc 70 → `focusa-3zav` (under `focusa-jz89`)
- doc 71 → `focusa-suwi` (under `focusa-ru3s`)
- doc 72 → `focusa-e8wn` (under `focusa-ru3s`)
- doc 73 → `focusa-16us` (under `focusa-ru3s`)
- doc 74 → `focusa-eg8i` (under `focusa-ru3s`)
- doc 75 → `focusa-2m6e` (under `focusa-jz89`)
- doc 76 → `focusa-eczn` (under `focusa-jz89`)
- doc 77 → `focusa-v2n5` (under `focusa-jz89`)

Role now:
- coarse predecessor to the shared-substrate hierarchy; never sole completion evidence

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
