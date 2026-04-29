# Spec93 — Non-Pi Agent Focusa Awareness

## Status

Implementation in progress. First runtime slice implemented: `/v1/awareness/card`, `focusa awareness card`, and live proof harness.

## Purpose

Pi now receives Focusa awareness through the runtime Utility Card. Non-Pi agents need equivalent awareness so Focusa is visible to OpenClaw/Wirebot, Claude Code, OpenCode, Letta, and other harnesses.

## Explicit inclusion

This spec explicitly includes the operator's **OpenClaw / oprnclaw Wirebot agent**.

Wirebot/OpenClaw is not an afterthought: it is the primary non-Pi target for Focusa awareness.

## Problem

Focusa is useful only when agents know to use it. Public docs and Pi startup cards are insufficient for non-Pi agents because they may never load Pi extension prompts or skills.

Failure modes:

- Wirebot/OpenClaw reasons without Focusa Workpoint context.
- Wirebot actions produce proof but do not link evidence.
- Risky choices are not predicted/evaluated.
- Focusa outage is invisible and continuity is silently degraded.
- Claude/OpenCode/Letta repeat work because Focusa awareness is not injected at their entrypoints.

## Goals

1. Every non-Pi agent gets a Focusa Utility Card equivalent.
2. OpenClaw/Wirebot can fetch scoped Workpoint resume state.
3. OpenClaw/Wirebot can checkpoint before risky boundaries.
4. OpenClaw/Wirebot can capture/link evidence after proof.
5. OpenClaw/Wirebot can record/evaluate predictions for risky choices.
6. Focusa unavailable state is explicitly reported as degraded cognition.
7. Operator steering remains supreme.

## Non-goals

- Do not make Focusa the durable personal/business knowledge store.
- Do not replace Wiki, Mem0, Letta, Scoreboard, or workspace files.
- Do not require every non-Pi harness to support Pi tools.
- Do not block direct fallback when Focusa is down; mark degradation instead.

## Architecture

### Awareness card contract

All non-Pi agents receive a compact card with:

- what Focusa is;
- current availability/degraded state;
- current mission/workspace/session scope when known;
- Workpoint resume/checkpoint rule;
- doctor/recovery rule;
- evidence capture/link rule;
- prediction record/evaluate rule;
- operator steering rule.

### OpenClaw/Wirebot primary path

Preferred:

```text
OpenClaw/Wirebot -> Focusa proxy/context injector -> model backend
```

Focusa injects:

1. Focusa Utility Card.
2. Scoped Workpoint resume packet.
3. Minimal Focus State slice.
4. Degraded-state warning when Focusa surfaces are unavailable.

### OpenClaw/Wirebot fallback path

If proxy injection is unavailable, an OpenClaw gateway plugin performs:

1. `GET /v1/health`.
2. `POST /v1/workpoint/resume` scoped with `adapter_id=openclaw`, `workspace_id=wirebot`, `agent_id=wirebot`.
3. Optional `GET /v1/doctor` for recovery state.
4. Prompt insertion.
5. Event hooks for evidence/prediction.

### Generic non-Pi path

Harnesses without plugin support use CLI wrapper snippets:

```bash
focusa doctor --json
focusa workpoint resume --json
focusa workpoint checkpoint --json
focusa predict record ... --json
focusa predict evaluate ... --json
```

## API needs

Existing surfaces are enough for first pass:

- `/v1/health`
- `/v1/doctor`
- `/v1/workpoint/resume`
- `/v1/workpoint/checkpoint`
- `/v1/workpoint/evidence/link`
- `/v1/evidence/capture` if exposed through current evidence route family
- `/v1/predictions`
- `/v1/predictions/{prediction_id}/evaluate`
- `/v1/predictions/stats`

Potential follow-up endpoint:

```text
GET /v1/awareness/card?adapter_id=openclaw&workspace_id=wirebot&agent_id=wirebot
```

This would serve a pre-rendered Focusa Utility Card for any harness.

## Wirebot-specific state model

Suggested scope envelope:

```json
{
  "adapter_id": "openclaw",
  "workspace_id": "wirebot",
  "agent_id": "wirebot",
  "operator_id": "verious.smith",
  "project_root": "/data/wirebot/users/verious",
  "session_id": "<openclaw-session-id>"
}
```

Rules:

- Scope-bound Workpoint packets must not cross Wirebot user/workspace/session boundaries.
- If Focusa state is stale or mismatched, demote packet and use latest operator turn plus Wiki/Mem0/Letta context.
- Durable discoveries promote to Wiki/Mem0/Letta/workspace; Focusa keeps bounded active cognition and proof links.

## Validation

Add:

```bash
node scripts/validate-non-pi-agent-awareness.mjs
```

The guard must verify:

- docs mention OpenClaw/Wirebot explicitly;
- awareness card content includes doctor, Workpoint, evidence, prediction, degraded fallback, operator steering;
- integration docs link to this spec;
- if an awareness-card endpoint is implemented, route inventory and live proof include it.

## Acceptance criteria

- OpenClaw/Wirebot has a documented Focusa awareness path.
- Non-Pi awareness docs are linked from README and docs index.
- A validation guard prevents removal of OpenClaw/Wirebot awareness language.
- Future implementation can add `/v1/awareness/card` without changing the contract.

## First implementation slice

1. Publish this spec.
2. Publish `docs/current/NON_PI_AGENT_FOCUSA_USAGE.md`.
3. Add docs links.
4. Add `scripts/validate-non-pi-agent-awareness.mjs`.
5. Implemented local OpenClaw plugin skeleton at `apps/focusa-awareness`; production config is enabled and activation requires an operator-approved gateway restart.
6. Later: live end-to-end OpenClaw turn proof after restart.
