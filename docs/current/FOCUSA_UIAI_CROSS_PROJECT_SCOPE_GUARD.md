# Focusa UIAI Cross-Project Scope Guard

**Status:** implemented for `focusa-877z.8.11`.

Guard: `scripts/focusa-uiai-scope-guard`

## Contract

UIAI packets are proposal material until their embedded `project_root`, `continuity_id`, and `scope_source` match Focusa-verified authority.

## Guard behavior

- Expected authority is `project_root + continuity_id`.
- Broad or missing roots are rejected.
- Cross-project packets return `failure_class=scope_mismatch` and `capture_status=scope_mismatch`.
- Cross-workstream packets return `failure_class=scope_mismatch` and `capture_status=scope_mismatch`.
- `scope_source` must be `focusa_verified` before the packet can proceed to capture/intake.
- Passing packets remain advisory; durable evidence still requires `focusa_browser_diagnostics_intake`, `focusa_evidence_capture`, or `focusa_workpoint_link_evidence`.

## Acceptance

A UIAI packet for another project or continuity cannot be rendered as captured Focusa evidence for the active Workpoint.
