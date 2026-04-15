# Post-Cutoff Sparse Locus Notes — 2026-04-13

Purpose:
- record where later-doc code support is still sparse/scattered
- prevent over-claiming implementation just because a few references exist

## Projection / bounded world surfaces

Strongest visible current evidence:
- `tests/ontology_world_contract_test.sh`
  - explicitly verifies bounded runtime ontology projection beyond the Pi hot path
  - useful as proof that projection is not purely aspirational
  - still only a thin surface relative to full doc 75 projection/view semantics

Implication:
- doc 75 should be treated as **partial influence + thin test-backed surface**, not as a mature runtime projection subsystem

Relevant beads:
- `focusa-2m6e`
- `focusa-ue1o`

## Retention / decay surfaces

Visible current evidence:
- `crates/focusa-api/src/routes/commands.rs`
  - `memory.decay_tick`
- `apps/pi-extension/src/tools.ts`
  - bounded notes with oldest-decay-first wording
- `tests/golden_tasks_eval.sh`
  - metric mentions like mission retention

Implication:
- doc 76 has **real hooks**, but still lacks a broad explicit retention policy layer
- current code suggests local mechanisms, not a coherent cross-domain retention substrate

Relevant beads:
- `focusa-eczn`
- `focusa-ystw`
- `focusa-zkhg`

## Identity / reference / role surfaces

Visible current evidence is scattered across:
- reference/meta routes in `crates/focusa-api/src/routes/capabilities.rs`
- attachment role types in `crates/focusa-api/src/routes/attachments.rs`
- validation/self-reference rules in `crates/focusa-api/src/routes/focus.rs`
- focus stack/state code in `crates/focusa-core/src/focus/*`

Implication:
- docs 72-74 are **not zero-implementation**, but support is distributed and vocabulary-heavy
- these docs need decomposition around concrete consumers instead of assuming one unified domain already exists

Relevant beads:
- `focusa-e8wn`
- `focusa-k9pt`
- `focusa-eg8i`
- `focusa-6kn4`

## Visual / UI trajectory surfaces

Code search did not reveal a strong dedicated visual ontology implementation track in current runtime.
Most visible hits were:
- TUI view code (`crates/focusa-tui/src/views/*`)
- older docs/spec prose
- scattered generic UI words

Implication:
- docs 58-65 remain overwhelmingly **docs-first / sparse-code**
- TUI rendering should not be mistaken for a visual cognition ontology
- visual-track beads must stay conservative and object-first

Relevant beads:
- `focusa-s2z6`
- `focusa-6pd1`
- `focusa-ost9`
- `focusa-5bqc`
- `focusa-piwx`

## Affordance / execution environment surfaces

Current search evidence appears sparse and distributed, with stronger signals around:
- workspace/session metadata
- references and capability-like route surfaces
- ontology world tests

Implication:
- doc 66 remains largely unimplemented as an explicit ontology domain
- decomposition should anchor on the first truthful consumer path rather than broad ontology claims

Relevant beads:
- `focusa-wmw7`
- `focusa-2w17`
- `focusa-u7ck`

## Decomposition consequence

For docs 58-66 and 72-76:
- sparse code presence does **not** justify broad completion claims
- each implementation bead should name:
  - concrete file locus
  - first consumer
  - observable behavior
  - proof surface (test/trace/eval)
