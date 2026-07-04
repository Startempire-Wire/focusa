# focusa_dxux_report

Spec105 DX/UX tool surface.

- API: see docs/current/focusa-tool-contracts.json
- CLI: see docs/current/focusa-tool-contracts.json
- Side effects: none unless running top-level `focusa preflight`

- API: `GET /v1/dxux/report`
- CLI: `focusa dxux report`
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
