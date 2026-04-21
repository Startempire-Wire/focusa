# SPEC80 F3.1 — Bead Metadata Lint Policy Spec

Date: 2026-04-21
Bead: `focusa-yro7.6.3.1`
Label: `documented-authority`

Purpose: enforce decomposition metadata requirements from anti-false-weaving policy.

## Lint rules

1. Each bead must declare one primary label:
   - `implemented-now`
   - `documented-authority`
   - `planned-extension`
2. Closure reason must include:
   - `code: file:line`
   - `spec section`
   - `Evidence citations`
3. `implemented-now` closures require code citation to executable route/command.
4. Label/claim mismatch is lint failure.

## Lint output schema

```json
{
  "bead_id": "string",
  "pass": false,
  "violations": [{"code":"LABEL_MISSING","message":"..."}]
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§19.3, §20.4)
- docs/evidence/SPEC80_LABEL_TAXONOMY_ENFORCEMENT_2026-04-21.md
