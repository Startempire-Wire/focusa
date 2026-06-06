# Focusa Exact-Handle Evidence Write Semantics

**Status:** implemented for `focusa-877z.8.9`.

Evidence writes must return and link exact handles, never select artifacts by duplicate-prone labels.

## Rules

- Store routes pre-generate `handle_id` and poll for `h.id == handle_id`.
- Responses return both `id` and the full `handle` object.
- `tool_result_v1.evidence_refs` uses `focusa-handle:<uuid>` or the caller-supplied stable evidence ref.
- Project/workstream scope is carried on handle metadata where available: `project_root`, `continuity_id`, optional `workpoint_id`.
- Legacy handles lacking scope remain readable but are labeled `legacy_scope_missing` and `evidence_handle_only_not_object_truth`.
- Labels are display/search metadata only; labels do not decide which artifact was just created.

## Surfaces

- `crates/focusa-api/src/routes/ecs.rs`
- `crates/focusa-api/src/routes/visual_workflow.rs`
- `apps/pi-extension/src/tools.ts` (`focusa_evidence_capture`, `focusa_workpoint_link_evidence`, `focusa_browser_diagnostics_intake`)

## Acceptance

Duplicate labels cannot select the wrong artifact; exact created handle id is the evidence reference source of truth.
