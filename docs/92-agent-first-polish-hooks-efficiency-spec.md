# Spec92 — Agent-First Polish, Hook Coverage, Token Efficiency, Cache UX, and Predictive Power

## 1. Purpose

Make Focusa feel like high-quality, agent-first software by adding the missing polish layer that turns existing core/API/CLI/Pi/Mac/runtime foundations into a predictable, fast, recoverable, token-efficient cognitive workflow system that agents choose first.

Spec92 covers:

- missing Pi lifecycle/provider/tool hooks,
- agent command-center UX,
- token telemetry and budget control,
- cache metadata and prompt-slice reuse,
- polished error/recovery/empty states,
- Mac app mission-control polish,
- docs/CI drift prevention,
- predictive-power accumulation from Focusa's existing event/outcome framework.

## 2. Direct answer: can Focusa accumulate predictive power now?

Yes. Focusa already has enough framework pieces to accumulate predictive power, but it needs a formal prediction loop and evaluation surfaces to make that power measurable and compounding.

### 2.1 Existing framework pieces that support prediction

Current Focusa surfaces already useful for prediction:

- **Event log** — captures state/action/outcome sequences.
- **Workpoint records** — capture mission, action intent, next slice, blockers, evidence, and continuation outcomes.
- **Metacognition** — captures/retrieves/reflects/adjusts/evaluates learning signals.
- **CLT / lineage tree** — captures context ancestry and branch/fork paths.
- **Telemetry** — captures tool usage, activity, tokens/cost surfaces, traces.
- **Work-loop outcome tracking** — captures continuation outcomes, quality grades, replay/comparative evidence.
- **Ontology contracts** — provide typed action/object labels for tools, tasks, affordances, risks, and boundaries.
- **ECS/evidence handles** — allow large proof artifacts to be summarized and rehydrated.
- **Spec90/Spec91 tool contracts/proof** — provide stable tool/action identities and live runtime parity.

### 2.2 What predictive power means in Focusa

Predictive power is not mystical memory. It means Focusa gets better at forecasting:

- which next action is likely to succeed,
- which tool should be used next,
- which context is relevant,
- whether a Workpoint is stale,
- whether a release will fail,
- whether compaction/resume will drift,
- whether a task is blocked by governance/permissions/scope,
- whether an output is likely low quality,
- whether a user steering event changes the mission,
- which cached slice will be useful,
- which docs/specs are likely needed,
- when a Mac app/API/CLI contract is stale,
- when a tool call will be expensive or redundant.

### 2.3 How Focusa accumulates prediction

Prediction accumulates through a loop:

```text
observe event/context/action
→ predict next outcome or risk
→ act or recommend
→ capture actual outcome
→ compare predicted vs actual
→ store calibration signal
→ adjust future retrieval/ranking/policy
```

Focusa already has most of this loop. Spec92 formalizes it.

### 2.4 Boundaries and caveats

- Prediction must be **evidence-calibrated**, not vibes.
- Prediction must be **bounded and inspectable**.
- Prediction should output confidence and explanation.
- Prediction must support JSON and human forms.
- Prediction must avoid storing secrets or raw large prompts.
- Prediction must use explicit retention/decay to prevent stale lessons from dominating.
- Prediction should improve recommendations, not silently remove operator authority.

## 3. Non-goals

- Do not build opaque autonomous decision-making.
- Do not store raw provider payloads by default.
- Do not cache secrets, tokens, credentials, or sensitive env vars.
- Do not make cleanup destructive.
- Do not mutate Focus State/Workpoint/Work-loop in default doctor/proof paths.
- Do not require remote network access for core proof.
- Do not replace Spec90/Spec91; Spec92 builds on them.
- Do not implement all phases in one commit.

## 4. Current baseline

Current implemented/released surfaces include:

- Rust daemon/API/CLI/TUI workspace.
- Pi extension with 43 `focusa_*` tools.
- Spec90 canonical tool contract registry.
- Spec91 live tool contract proof harness.
- Mac menubar app updated to current health/state/tool-contract/workpoint/work-loop/event surfaces.
- Production daemon live at local port `8787`.
- GitHub CI and Release green for `v0.9.10-dev`.
- Production release commands documented in `docs/current/PRODUCTION_RELEASE_COMMANDS.md`.

## 5. Design principles

1. **Agent-first:** every surface must help an agent decide the next safe action.
2. **Predictable:** every command/tool should use consistent result shapes.
3. **Token-aware:** Focusa must reduce context waste and surface costs.
4. **Cache-aware:** repeated state/doc/contract/prompt slices must be reusable or summarized.
5. **Evidence-first:** proof refs and handles beat raw transcript blobs.
6. **Recoverable:** every failure gives exact recovery commands.
7. **Read-only by default:** doctors/proofs/status commands do not mutate unless explicitly named and approved.
8. **Operator-governed:** predictions guide; they do not override operator steering.
9. **Current-build truth:** docs and UI must not imply unfinished behavior is live.
10. **No secret leakage:** hook telemetry and cache metadata must be redacted/bounded.

## 6. Phase A — Pi hook coverage

### 6.1 Required missing hooks

Add handlers for these Pi extension events:

- `resources_discover`
- `agent_start`
- `message_start`
- `message_end`
- `before_provider_request`
- `after_provider_response`
- `tool_execution_start`
- `tool_execution_update`
- `tool_execution_end`
- `session_tree`

### 6.2 Existing hooks to preserve

Existing Focusa handlers must keep working:

- `before_agent_start`
- `context`
- `input`
- `message_update`
- `model_select`
- `session_start`
- `session_shutdown`
- `session_before_switch`
- `session_switch`
- `session_before_fork`
- `session_fork`
- `session_before_tree`
- `session_before_compact`
- `session_compact`
- `turn_start`
- `turn_end`
- `agent_end`
- `tool_call`
- `tool_result`

### 6.3 Hook responsibilities

#### `resources_discover`

Purpose: auto-register Focusa skills/prompts/themes where possible.

Must:

- contribute project/extension skill paths,
- avoid duplicate paths,
- expose diagnostics if paths missing,
- never fail Pi startup because a path is absent.

#### `agent_start`

Purpose: start a bounded agent-run record.

Capture:

- session id,
- run id,
- model/provider if available,
- current ask summary/label,
- active Workpoint id if available,
- timestamp.

#### `message_start` / `message_end`

Purpose: track assistant message lifecycle.

Capture:

- message id,
- approximate character/token count,
- whether message had tool calls,
- summary handle if large,
- outcome status.

Do not store raw full message by default.

#### `before_provider_request`

Purpose: token/caching critical hook.

Must compute:

- provider payload hash,
- approximate input token count,
- repeated prefix hash,
- Focusa injected slice size,
- tool schema size estimate,
- message count,
- largest message estimate,
- cache eligibility,
- budget warnings.

May modify payload only if explicit policy allows safe pruning/minimal-slice enforcement.

#### `after_provider_response`

Purpose: capture provider result metadata.

Capture when available:

- response status,
- provider request id,
- rate-limit headers,
- usage/cost fields,
- latency,
- retry-after hints.

Do not assume all providers expose headers.

#### `tool_execution_start/update/end`

Purpose: better tool observability than `tool_call`/`tool_result` alone.

Capture:

- tool name,
- call id,
- start/end times,
- duration,
- args size estimate,
- result size estimate,
- partial-output size,
- blocked/error status,
- whether output was summarized/handled.

#### `session_tree`

Purpose: capture post-tree navigation state.

Must:

- update lineage/branch context,
- mark possible Workpoint drift,
- recommend `focusa_workpoint_resume` when needed.

### 6.4 Acceptance criteria

- All listed hooks are registered.
- Hook outputs are bounded.
- Hook telemetry is inspectable through API/CLI/Pi doctor.
- Hook failures are non-fatal.
- Guardian scan passes changed hook code/docs.

## 7. Phase B — token telemetry and budget control

### 7.1 Token telemetry object

Add a bounded token telemetry record:

```json
{
  "record_id": "...",
  "ts": "...",
  "session_id": "...",
  "turn_id": "...",
  "provider": "...",
  "model": "...",
  "payload_hash": "sha256:...",
  "prefix_hash": "sha256:...",
  "message_count": 0,
  "input_token_estimate": 0,
  "focusa_slice_token_estimate": 0,
  "tool_schema_token_estimate": 0,
  "largest_message_token_estimate": 0,
  "cache_eligible": true,
  "budget_class": "ok|watch|high|critical",
  "warnings": []
}
```

### 7.2 Token doctor command

Add:

```bash
focusa tokens doctor
focusa tokens doctor --json
focusa tokens status --agent
```

Human output shape:

```text
Status: watch
Input estimate: 82k / 200k
Focusa slice: 1.2k
Tool schemas: 18k
Largest message: 24k
Cache eligible: yes
Next: compact tool-result history
Command: focusa tokens compact-plan
Recovery: use ECS handles for large outputs
```

### 7.3 Budget classes

- `ok` — no action needed.
- `watch` — bloat detected but below critical threshold.
- `high` — agent should prefer compact/summary/ECS handles soon.
- `critical` — block or strongly steer before provider request if policy allows.

### 7.4 Acceptance criteria

- `before_provider_request` records token telemetry.
- `focusa tokens doctor` summarizes recent token health.
- Telemetry omits secrets/raw provider payloads.
- Mac app can display token-budget card.

## 8. Phase C — cache metadata and prompt-slice reuse

### 8.1 Cache surfaces

Add metadata caches for:

- prompt slices,
- provider payload hashes,
- repeated prefix hashes,
- tool-contract registry,
- API route inventory,
- CLI command inventory,
- skill inventory,
- doc command inventory,
- evidence handles.

### 8.2 Cache record shape

```json
{
  "cache_key": "...",
  "cache_type": "prompt_slice|payload_prefix|tool_contracts|api_routes|cli_inventory|skill_inventory|evidence_handle",
  "state_version": 0,
  "contract_version": "spec90.tool_contracts.v1",
  "created_at": "...",
  "last_used_at": "...",
  "hit_count": 0,
  "size_estimate_tokens": 0,
  "invalidates_on": ["state_version", "contract_version", "skill_paths", "docs_hash"]
}
```

### 8.3 Cache doctor

Add:

```bash
focusa cache doctor
focusa cache status --agent
focusa cache explain <cache_key>
```

### 8.4 Acceptance criteria

- Cache metadata is bounded.
- Cache invalidation is explicit.
- Cache status is visible in CLI/API/Mac app.
- No raw secrets or full provider payloads are cached by default.

## 9. Phase D — agent command center

### 9.1 Commands

Add or polish:

```bash
focusa doctor
focusa doctor --json
focusa continue
focusa continue --json
focusa status --agent
focusa explain --why-next
focusa release prove --tag <tag>
focusa cleanup --safe
focusa compatibility
focusa docs status
```

### 9.2 Standard human output shape

Every command should use:

```text
Status: <completed|watch|degraded|blocked>
Summary: <one sentence>
Next action: <exact next action>
Why: <short explanation>
Command: <copyable command>
Recovery: <copyable fallback>
Evidence: <refs/handles>
Docs: <paths>
```

### 9.3 Standard JSON output shape

```json
{
  "status": "completed|watch|degraded|blocked",
  "summary": "...",
  "next_action": "...",
  "why": "...",
  "commands": [],
  "recovery": [],
  "evidence_refs": [],
  "docs": [],
  "warnings": [],
  "details": {}
}
```

### 9.4 `focusa doctor`

Must check:

- daemon health,
- daemon exe path,
- CLI reachable,
- API route inventory,
- Spec90 contracts,
- Spec91 live proof,
- Pi skill paths,
- Pi tools count,
- Workpoint canonicality,
- Work-loop writer state,
- token telemetry status,
- cache status,
- Mac app version/status if local build exists,
- GitHub latest release state if `gh` is available.

### 9.5 `focusa continue`

Must return:

- current mission,
- current action,
- next exact action,
- active object refs,
- blockers,
- do-not-drift list,
- suggested tools/commands,
- Workpoint id and canonical flag.

### 9.6 `focusa release prove`

Must run or orchestrate:

- Spec90 static contracts,
- Spec91 live proof,
- cargo tests/clippy,
- strict spec gates,
- Mac app check/build,
- daemon release build/restart proof,
- GitHub run/release asset checks,
- Guardian scans,
- cleanup status.

### 9.7 Acceptance criteria

- Commands exist or docs explicitly mark not yet implemented.
- JSON and human modes exist for implemented commands.
- Commands are safe by default.
- Output includes exact next command and recovery.

## 10. Phase E — polished errors, empty states, and recovery

### 10.1 Error envelope

Every CLI/API/Pi doctor-style failure should contain:

- `what_failed`,
- `likely_why`,
- `safe_recovery`,
- `command`,
- `fallback`,
- `docs`,
- `evidence_refs`,
- `severity`.

### 10.2 Empty state examples

No active Workpoint:

```text
Status: watch
Summary: No active Workpoint is currently promoted.
Next action: Create a checkpoint before compaction or risky work.
Command: focusa_workpoint_checkpoint mission="..." next_action="..."
Recovery: focusa_workpoint_resume if a previous packet exists.
```

Daemon down:

```text
Status: blocked
Summary: Focusa daemon is not reachable at 127.0.0.1:8787.
Next action: restart daemon.
Command: systemctl restart focusa-daemon && curl -sS http://127.0.0.1:8787/v1/health | jq .
Recovery: use local Workpoint fallback only as non-canonical guidance.
```

### 10.3 Acceptance criteria

- Top 20 known failure modes have polished recovery text.
- Empty states never display raw `null` without explanation.
- Mac app displays friendly error/empty copy.

## 11. Phase F — Mac app mission-control polish

### 11.1 Mission-control cards

Add cards for:

- daemon health,
- Workpoint current packet,
- next action,
- tool contract health,
- token budget,
- cache health,
- release status,
- compatibility matrix,
- recent evidence refs,
- docs/commands.

### 11.2 Buttons

Add copy/open buttons:

- Copy health command,
- Copy live proof command,
- Copy release proof command,
- Copy cleanup command,
- Open docs path,
- Refresh status.

### 11.3 Safety labels

Every read-only surface should say read-only.
Every mutating action must require confirmation.

### 11.4 Acceptance criteria

- No unmarked mock/demo data in production app routes.
- Mac app check/build passes.
- App displays current `spec90.tool_contracts.v1` and contract count.
- App displays Workpoint/Work-loop status.
- App handles daemon unavailable state gracefully.

## 12. Phase G — predictive power loop

### 12.1 Prediction record

Add a prediction record type:

```json
{
  "prediction_id": "...",
  "ts": "...",
  "prediction_type": "next_action_success|tool_choice|release_failure|stale_state|context_relevance|token_risk|cache_hit|drift_risk",
  "context_refs": [],
  "predicted_outcome": "...",
  "confidence": 0.0,
  "recommended_action": "...",
  "why": "...",
  "actual_outcome": null,
  "evaluated_at": null,
  "score": null,
  "learning_signal_ref": null
}
```

### 12.2 Prediction types

Initial prediction types:

- `next_action_success`
- `tool_choice`
- `release_failure`
- `stale_state`
- `context_relevance`
- `token_risk`
- `cache_hit`
- `drift_risk`
- `workpoint_resume_success`
- `compaction_resume_risk`

### 12.3 Prediction commands

Add:

```bash
focusa predict next-action
focusa predict release --tag <tag>
focusa predict token-risk
focusa predict drift
focusa predict evaluate --prediction-id <id> --actual <outcome>
focusa predict stats
```

### 12.4 Learning loop

```text
prediction emitted
→ action/proof occurs
→ actual outcome captured
→ prediction evaluated
→ metacog signal captured
→ strategy adjustment proposed
→ future prediction ranking improves
```

### 12.5 Scoring

Use simple transparent scoring first:

- exact success/failure match: 1.0,
- partially correct risk: 0.5,
- wrong direction: 0.0,
- unknown/not evaluated: excluded from accuracy.

Track:

- accuracy by prediction type,
- calibration buckets by confidence,
- false positives,
- false negatives,
- most useful predictors,
- stale predictors.

### 12.6 Acceptance criteria

- Prediction records are bounded and inspectable.
- Predictions can be evaluated against actual outcomes.
- Metacog captures prediction-quality signals.
- `focusa predict stats` reports accuracy/calibration.
- No prediction silently mutates state or overrides operator steering.

## 13. Phase H — docs, recipes, and command cookbook

### 13.1 Required docs

Add/update:

- `docs/current/AGENT_COMMAND_COOKBOOK.md`
- `docs/current/HOOK_COVERAGE.md`
- `docs/current/TOKEN_AND_CACHE_GUIDE.md`
- `docs/current/PREDICTIVE_POWER_GUIDE.md`
- `docs/current/MAC_APP_MISSION_CONTROL.md`
- `docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md`

### 13.2 Cookbook sections

- Starting work.
- Before risky edit.
- Before compaction.
- After compaction.
- Daemon down.
- Release failed.
- Mac app stale.
- Token budget high.
- Cache stale.
- Need evidence.
- Need cleanup.
- Need prediction/evaluation.

### 13.3 Acceptance criteria

- Every recipe has copyable commands.
- README links all current docs.
- Guardian scan passes docs.
- Docs avoid stale version claims.

## 14. Phase I — CI and drift prevention

### 14.1 CI checks

Add checks for:

- tool contract registry/doc parity,
- live proof script syntax,
- docs links,
- no stale known version strings,
- no unmarked mock/demo Mac app surfaces,
- Guardian scan docs/scripts where feasible,
- release command doc exists,
- hook coverage doc matches implemented hook inventory.

### 14.2 Acceptance criteria

- CI fails if a current `focusa_*` tool lacks contract/doc.
- CI fails if Mac app reintroduces unmarked mock data.
- CI fails if command docs omit key release commands.
- CI fails if Spec92 hook inventory drifts from implementation without docs update.

## 15. Bead decomposition

Root epic:

```text
Spec92 agent-first polish hooks token cache predictive power
```

Subtasks:

1. Author Spec92 and acceptance checklist.
2. Implement Pi missing hook coverage.
3. Implement token telemetry records.
4. Implement token doctor/status commands.
5. Implement cache metadata records.
6. Implement cache doctor/status commands.
7. Implement agent command center: doctor/continue/status-agent.
8. Implement release prove and cleanup-safe commands.
9. Implement polished error/empty-state envelopes.
10. Implement Mac app mission-control cards.
11. Implement predictive record/evaluation loop.
12. Implement predictive stats and metacog feedback.
13. Add agent command cookbook and hook/token/cache docs.
14. Add CI drift checks.
15. Release proof, GitHub release, production restart, cleanup.

## 16. Milestones

### Milestone 1 — hook and telemetry foundation

- Missing hooks registered.
- Bounded hook telemetry stored.
- Token telemetry emitted from provider request hook.

### Milestone 2 — doctor/status UX

- `focusa doctor`
- `focusa continue`
- `focusa status --agent`
- polished error envelopes.

### Milestone 3 — token/cache UX

- `focusa tokens doctor`
- `focusa cache doctor`
- prompt-slice/cache metadata.

### Milestone 4 — predictive loop

- prediction records,
- evaluation,
- stats,
- metacog feedback.

### Milestone 5 — Mac/docs/CI/release polish

- mission-control Mac app cards,
- command cookbook,
- drift CI,
- release proof.

## 17. Release acceptance checklist

Spec92 is complete when:

- hooks implemented and documented,
- token telemetry visible,
- cache metadata visible,
- agent command center commands exist,
- predictive loop can record/evaluate predictions,
- Mac app shows mission-control cards,
- docs/cookbook updated,
- CI/drift checks pass,
- Guardian scan passes,
- production daemon rebuilt/restarted,
- GitHub CI and Release pass,
- residual junk cleaned recoverably.

## 18. Commands for implementation validation

```bash
cd /home/wirebot/focusa
cargo test --workspace
cargo clippy --workspace -- -D warnings
./scripts/ci/run-spec-gates.sh
node scripts/validate-focusa-tool-contracts.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
cd apps/menubar && bun install && bun run check && bun run build
```

## 19. Privacy/security requirements

- Redact secrets in hook telemetry.
- Do not store raw provider payloads by default.
- Hash prompt/prefix payloads before storage.
- Bound all telemetry arrays.
- Use ECS handles for large outputs.
- Run Guardian scan before release.

## 20. Open implementation questions

1. Should prediction records live under metacognition, telemetry, or a new prediction namespace?
2. Should `focusa doctor` be CLI-only first, or CLI/API/Pi simultaneously?
3. Should token estimates use heuristic chars/token first or model-specific tokenizer later?
4. Should cache metadata persist in existing state, SQLite, or ECS manifest?
5. How much Mac app write capability is acceptable, if any, before confirmation UX is implemented?

## 21. Recommended first implementation slice

Start with:

1. hook coverage inventory doc and new hook registrations,
2. token telemetry from `before_provider_request`,
3. `focusa tokens doctor`,
4. `focusa doctor` minimal version,
5. Mac app token/contract/Workpoint cards.

This gives immediate polish and creates the substrate for predictive power.
