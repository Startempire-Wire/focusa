# SPEC80 Claim Validation Protocol — 2026-04-21

Purpose: prevent false implemented-now assertions by defining validation steps for decomposition and closure evidence.

## Validation steps

1. Determine label (`implemented-now`, `documented-authority`, `planned-extension`).
2. If `implemented-now`:
   - verify route/command exists in code,
   - verify corresponding test or reproducible endpoint behavior,
   - record code citation (`file:line`) in bead closure.
3. If `documented-authority`:
   - cite authoritative doc section,
   - link to decomposition target proving requirement.
4. If `planned-extension`:
   - cite SPEC80 clause and target API/CLI contract row,
   - ensure no implementation claim in status text.

## Closure minimums

- `code: file:line`
- `spec section §...`
- `Evidence citations: ...`
- label-consistent language (no implemented wording for planned/documented-only tasks)

## Escalation rule

If label and claim conflict, task is reopened and relabeled before further progress.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_LABEL_TAXONOMY_ENFORCEMENT_2026-04-21.md
- scripts/enforce_bd_closure_evidence.sh
