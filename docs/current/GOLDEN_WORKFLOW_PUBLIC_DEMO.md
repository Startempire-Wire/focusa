# Golden Workflow Public Demo

This demo explains Focusa through one safe story: an agent resumes a project, verifies scope, chooses the next Workpoint action, gates risky mutation, captures evidence, and reports proof without leaking private data.

## Demo path

1. **Project identity** — verify `project_root` and canonical project name.
2. **Trajectory** — read HLT/MLG/STG/Waypoints and active gap.
3. **Workpoint** — resume canonical Workpoint or checkpoint a new one.
4. **Context Authority** — preflight risky mutation before code/service changes.
5. **Context Cognition** — render compact advisory context, not authority.
6. **Call Stack** — design/verify the implementation path before broad feature work.
7. **Evidence** — capture/link proof handles after checks.
8. **Prediction + Metacognition** — forecast outcome, evaluate, and capture reusable lesson.
9. **Public card** — publish only redacted, required public-card fields when allowed.

## Demo commands

```bash
focusa project identity --json
focusa trajectory view --project-root <project-root> --json
focusa workpoint resume --project-root <project-root> --continuity-id <continuity> --json
focusa action preflight --action deploy --target focusa-daemon --json
focusa context-cognition render --project-root <project-root> --continuity-id <continuity>
focusa call-stack verify --project-root <project-root> --continuity-id <continuity> --entry-name /v1/call-stack/verify
```

## Public-safe story beats

- Operator steering wins.
- `project_root + continuity_id` is authority boundary.
- Workpoint is immediate continuation authority.
- Trajectory is north-star route context.
- Context Cognition is advisory.
- Evidence/proof handles replace raw logs.
- Public stream uses `PUBLIC_STREAM_REDACTION_POLICY.md` and `publish_allowed=false` by default.

## Non-demo data

Never include raw logs, tokens, private file contents, raw diffs, sensitive browser diagnostics, or environment contracts with host secrets in the public demo.

## Proof

- Static guard: `tests/golden_workflow_public_demo_static_test.sh`
- Canonical workflow: `docs/current/GOLDEN_WORKFLOW.md`
- Authority model: `docs/current/AUTHORITY_MODEL.md`
