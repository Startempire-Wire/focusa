# Focusa UIAI Packet Capture-Status Rendering

**Status:** implemented for `focusa-877z.8.10`.

Renderer: `scripts/focusa-uiai-packet-render`

## First-line format

```text
UIAI packet mode=<mode> evidence=<n> scope=<scope_status> scope_source=<scope_source> capture=<capture_status> tool=<preferred_tool> next=<headless_next_action>
```

## Capture status rendering

- `proposal_only` → `capture=proposal_only`
- `capture_pending` → `capture=pending_focusa_tool`
- `captured` → `capture=focusa_captured`
- `workpoint_linked` → `capture=workpoint_linked`
- `scope_mismatch` → `capture=rejected`
- `degraded_unknown` → `capture=degraded_unknown`

## Authority rule

Only `captured` or `workpoint_linked` with `scope_source=focusa_verified` can render `canonical=true`; all other packet renders are advisory/proposal-only and must point at `focusa_browser_diagnostics_intake`, `focusa_evidence_capture`, or `focusa_workpoint_link_evidence`.
