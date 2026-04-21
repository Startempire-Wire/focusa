# SPEC80 C4.2 — CLI JSON Compatibility Policy

Date: 2026-04-21
Bead: `focusa-yro7.3.4.2`
Purpose: define backward-compatible JSON field evolution and deprecation policy for tool-consumed CLI surfaces under Spec80 Epic C.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §15, §20.1)
- docs/evidence/SPEC80_CLI_JSON_SCHEMA_REGISTRY_2026-04-21.md

## Versioning model

Schema ids follow `cli.<domain>.<shape>.v<major>`.

Compatibility classes:
- **Patch-compatible**: textual clarification or example updates only; no schema field changes.
- **Minor-compatible**: optional field additions only; no required-field changes.
- **Major-breaking**: required-field removal/rename/type change, or semantic contract inversion.

## Hard compatibility rules

1. Required fields listed in a `vN` schema are immutable for that major version.
2. New fields must be optional by default in existing major versions.
3. Field type changes require major bump (`vN -> vN+1`).
4. Enum narrowing (removing accepted values) requires major bump.
5. Any change to `status` envelope semantics requires explicit migration note and schema-major bump.

## Planned-extension envelope policy

For planned-extension command surfaces (e.g., metacognition domain):
- `status="not_implemented"` envelope is stable in current major.
- Required envelope fields remain:
  - `status`
  - `command`
  - `planned_api_path`
  - `reason`
  - `label`
- Transition from planned-extension envelope to implemented payload requires:
  1. introducing new schema id major,
  2. documenting migration path,
  3. keeping old envelope available for one deprecation window when feasible.

## Deprecation policy

1. Deprecation must be announced in schema registry notes before removal.
2. Minimum deprecation window: one release cycle for tool-consumed fields.
3. Deprecated fields may emit warnings in non-JSON mode, but JSON output must remain valid through window.
4. Removal can occur only in next major schema id.

## Validation gates

- FAIL if a contract test detects missing required field for a stable major schema.
- FAIL if a tool-consumed command changes required field type without schema-major bump.
- FAIL if planned-extension envelope field set changes without registry/version update.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_CLI_JSON_SCHEMA_REGISTRY_2026-04-21.md
- tests/spec80_cli_json_schema_registry_test.sh
- tests/spec80_cli_metacognition_contract_test.sh
