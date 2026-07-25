---
name: focusa-troubleshooting
description: "Use when Focusa is offline, degraded, stale, writer-conflicted, non-canonical, or tools return pending/blocked results that need recovery."
---

# Focusa Troubleshooting Playbook

Use when Focusa is offline, degraded, stale, writer-conflicted, non-canonical, or tools return pending/blocked results that need recovery.

## Progressive disclosure

Read `references/01-focusa-troubleshooting-runbook.md` for scoped diagnosis, contract drift, resource pressure, browser diagnostics, and safe retry posture.

## Start here

1. Load the main Focusa skill if you need the whole system model: `/skill:focusa`.
2. Read the focused tool doc: `docs/focusa-tools/diagnostics-hygiene.md`.
3. Prefer canonical Focusa state over transcript memory.
4. Preserve proof as evidence refs, not pasted logs.

## Primary docs

- Focused tools: `docs/focusa-tools/diagnostics-hygiene.md`
- Tool index: `docs/focusa-tools/README.md`
- Operator guide: `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md`
- Live release proof: `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`

## Common recovery playbooks

### Trajectory cached-advisory timeout

Message pattern: `trajectory view preserved cached advisory ...; cause=timeout` or older `timeout_preserved; noncanonical cached clarity`.

Meaning: trajectory hot-path timed out and the Pi wrapper preserved advisory cached orientation. It is not a task failure and is not canonical truth.

Recovery:

1. Continue from canonical Workpoint/project identity if already verified.
2. Run `focusa_resource_mode action="status"` and `focusa_tool_doctor scope="trajectory"`.
3. Retry `focusa_trajectory_view mode="summary"` with explicit `project_root` and `continuity_id`.
4. Avoid full lineage, full ontology, deep telemetry, or cold/full payload routes while resource mode is emergency/lowmem.

### UIAI/browser diagnostics intake blocked

If `focusa_browser_diagnostics_intake` is blocked by scope or trajectory clarity, preserve the diagnostics file path in the response, run `focusa_workpoint_resume`, then retry with explicit `project_root`/`workpoint_id` or use `attach_to_workpoint=false` for dry intake. If UIAI returned `url_not_allowed`, treat it as hardened private-target policy evidence; Focusa intake can record it but cannot bypass UIAI URL safety.

### KH/OVH UIAI route failure

KH clients continue to use `http://localhost:7456`, but `uiai-engine.service` is now an SSH compatibility forward to OVH sticky RR `7460`; UIAI/Chromium compute must not run on KH. Check the KH unit, then OVH `uiai-engine-ovh`, `uiai-engine-ovh-w2`, and `uiai-ovh-rr`. Require Pi-native health plus open/read/snapshot/diagnostics proof because malformed compression or duplicate framing may pass curl while breaking Node fetch. Use `/root/dual-server-master-plan/runbooks/21-focusa-ovh-uiai-and-build-offload-runbook.md` for rollback.

### KH local build detected

Inside `/home/wirebot/focusa`, root Pi resolves Cargo/npm/npx through `/root/.pi/agent/bin/` shims to `/usr/local/bin/focusa-ovh-build`; compile/test execution belongs on OVH as `wirebot`. `cargo test` must use the isolated OVH test daemon, never the KH production daemon. Commands outside Focusa remain local. Release builds stay GitHub-hosted and the KH `focusa-deploy` runner remains artifact-only.

## Safety rules

- Treat `canonical=false`, `degraded=true`, `pending`, or `blocked` as recovery states, not success.
- Use Workpoint resume/checkpoint around compaction, context overflow, model switch, fork, or risky release work.
- Healthy Workpoint continuity makes generic `/fork`/`/new` context-pressure warnings redundant; treat them as recovery prompts only when Focusa is degraded.
- Same-project post-compaction `session_id` drift is normal continuity metadata; `project_root` and `continuity_id` mismatches are isolation errors.
- Multiple same-root sessions remain distinct through continuity_id; trajectory/goals are corroborating signals only.
- If a Focus State write rejects validation, record the full note in `focusa_scratch`, retry once with declarative boundary phrasing, then continue.
- Use writer-status/preflight before mutating work-loop state.
- Do not describe Focusa as complete or frozen; use current snapshot/version language.
