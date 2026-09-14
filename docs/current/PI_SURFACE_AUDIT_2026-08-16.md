# Pi widget/status/advisory surface audit — 2026-08-16 (#264)

Read-only audit of the deployed extension's 124 uiCtx.notify/setStatus
surfaces. Classification per #264 progressive-disclosure policy.

## Classification

| Class | Count | Policy |
| --- | --- | --- |
| user-triggered command feedback | ~104 | keep (direct response to an operator action) |
| SSE-driven decision notifications | 8 | keep (cross-surface decisions; already scoped) |
| metacog banner (transient) | 6 | keep (auto-clears within 15s) |
| auto-compaction status | 6 | keep (bounded: one status key, cleared on completion) |

## Findings (no fixes required, verified 2026-08-16)

1. **No screen-trash loops**: every recurring surface uses a bounded
   status key (`focusa-auto-compaction`, `focusa-pressure`) or a
   self-clearing metacog banner — none re-notify without state change.
2. **No private-context leakage**: notification text carries only
   display names / posture classes; the #45/#124 regression tests still
   cover the scoped-refresh path.
3. **bg completion notifications** (new): single uiCtx.notify per job
   completion with the job name + status; no per-line chatter — the log
   path is in the envelope, not in the banner.
4. **HLT banner** (§93): bounded display length (80 chars) and clears
   via metacog banner timeout.

## Exclusions

- menubar/TUI surfaces are out of scope for this audit (separate
  surfaces, #296/#297).
- Commands without an attachment scope already fail closed with a
  typed error (verified in commands.ts).
