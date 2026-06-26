# AX Gap Verification Notes — 2026-06-25

## Status of Closed AX Gaps (12/12)

All 12 AX gaps were "closed" but **verification depends on reload layer**:

### Daemon (Rust) — VERIFIED LIVE
Gaps fixed in `crates/focusa-api/src/`: rebuilt and daemon restarted via systemctl.
- focusa-7zky (BAD-003) — drift field in /v1/doctor ✓
- focusa-ioyu (BAD-001) — mismatch_reason in /v1/project/identity ✓
- focusa-msxt (BAD-005) — validation_errors in /v1/workpoint/checkpoint ✓
- focusa-uzr7 (BAD-002) — trajectory_workpoint_reconciliation ✓
- focusa-0qag (BAD-007) — next_step_hint in /v1/trajectory/view ✓
- focusa-zsld — degraded_reasons in /v1/project/identity ✓

### PI Extension (TypeScript) — REQUIRES PI RELOAD
Gaps fixed in `apps/pi-extension/src/tools.ts`: NOT VERIFIED LIVE in this session.
- focusa-4vpj — schema_invalid recovery_hint
- focusa-n7mx — allowed keys in error
- focusa-wk4t — gap details in text
- focusa-cgso — mismatch_reason in wrapper text
- focusa-pdrt — tool_doctor workpoint remediation
- focusa-uvkm — workpoint_resume "do NOT retry unchanged"
- focusa-x9av — hot_path_timeout with timeout_ms
- focusa-gazg — workpoint_checkpoint recovery hint
- focusa-nzru — packet_age freshness marker
- focusa-jj4j — verified at daemon level only
- focusa-05zu — verified at daemon level only

## Why Live Verification Failed

The current Pi session was started before most TypeScript changes. The Pi extension loads `./src/index.ts` once at session start and runs in memory. Changes to the source files are **NOT picked up by the running session**.

The agent's tool calls in this session show:
- `focusa_project_identity` still outputs `status=mismatch confidence=low` with no `mismatch_reason` line
- `focusa_trajectory_view` still shows `current=missing` for SET trajectory
- `focusa_trajectory_assess` still shows `gaps=1 action=verify_first` without gap details
- `focusa_workpoint_resume` still shows stale `next=` referring to closed items

But:
- The TypeScript source DOES contain the fixes (verified with grep)
- The daemon returns correct data (verified via curl)

## Required Next Steps

1. **Exit current Pi session** to force extension reload
2. **New Pi session** will load `./src/index.ts` fresh
3. **Re-run all focusa_* tools** to verify wrapper text now includes:
   - `mismatch_reason=` line in project_identity output
   - `current=<value>` (not `missing`) in trajectory_view output
   - `gap[0]: ref severity → action` in trajectory_assess output
   - `packet_age=Xmin` in workpoint_resume output

## Risk: False Positive Closures

If the new Pi session shows the SAME old output, the closures for the PI extension fixes are **false positives**. The fix:
1. Reopen affected beads
2. Investigate why the wrapper doesn't pick up the fields
3. Re-implement with stronger guarantees (e.g., always include field if present in body)
