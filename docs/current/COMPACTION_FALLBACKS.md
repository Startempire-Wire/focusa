# Compaction Fallbacks

Pi extension compaction replaces default Pi compaction for Focusa sessions, so blank `none` fields are not acceptable.

## Policy

Compaction summaries must use intelligent related fallbacks, not random filler and not bare `none`.

Fallback order:

1. Focus State slot value.
2. Canonical WorkpointResumePacket fields:
   - `mission`
   - `next_slice`
   - `project_root`
   - `session_id`
   - `active_object_refs`
   - `verification_records`
   - `blockers`
3. Pi bridge local shadow:
   - recent decisions
   - constraints
   - failures
4. Current ask / active frame goal/title.
5. Session metadata such as current project root.
6. Only if no related canonical source exists: explicit explanatory sentence such as `No open questions recorded by Focusa or Workpoint.`

## Non-goals

- Do not hallucinate decisions, constraints, artifacts, or test results.
- Do not fill slots with unrelated repo facts.
- Do not emit bare `none` for cognitive summary fields.

## Guard

```bash
node scripts/validate-compaction-fallbacks.mjs
```

This static guard fails if legacy bare `none` summary fallbacks return or if Workpoint/current-ask/session fallback hooks disappear.
