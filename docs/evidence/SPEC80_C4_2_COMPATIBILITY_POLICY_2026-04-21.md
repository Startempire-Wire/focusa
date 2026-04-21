# SPEC80 C4.2 — Compatibility Policy

Date: 2026-04-21
Bead: `focusa-yro7.3.4.2`
Label: `documented-authority`

Purpose: define backward-compatible JSON contract evolution policy for CLI/API parity surfaces.

## Versioning rules

1. Additive fields → minor-compatible within same major schema.
2. Field removal/rename/type change → major schema bump required.
3. Deprecated fields require one full minor cycle before removal.

## Behavioral rules

1. `--json` output is canonical machine contract surface.
2. Human-readable mode may change copy; JSON contract may not.
3. Error envelope contract (B4) is stable across domains.

## Migration procedure

1. Publish schema diff in `docs/schemas/cli-api/CHANGELOG.md`.
2. Update tool consumers and test fixtures.
3. Run contract regression suite before merge.

## Exit criteria

- All C2/C3/C1 JSON surfaces governed by explicit version policy.
- No unversioned breaking change allowed in contract-consuming commands.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §15)
- docs/evidence/SPEC80_B4_1_TYPED_ERROR_ENVELOPE_MAPPING_2026-04-21.md
