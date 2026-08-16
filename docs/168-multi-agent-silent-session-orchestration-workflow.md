# Multi-agent orchestration via Silent Sessions + Workloop — #292/#253 workflow

Status: canonical workflow (no new surface — this binds the EXISTING
silent-session runtime, the workloop, and the bg receipt ledger).

## Principle

Multi-agent work is NOT a new orchestrator. It is N silent sessions
(Spec133) launched in parallel, each bound to a workloop work item,
with the workloop as the single scheduler and the silent-session
completion stream + bg receipts as the single delivery path.

## Fan-out recipe (all existing routes)

1. Workloop owns the queue: `GET /v1/work-loop/status`, select work
   items via `focusa_work_loop_select_next` / the ready snapshot.
2. For each selected work item, create ONE silent session with a
   workloop-compatible config:
   - `identity.work_item_ref` = the work item id (the binding),
   - `identity.project_root` + `continuity_id` = the workstream scope,
   - `identity.agent_identity_ref` / `role_profile_ref` = the team role,
   - `supervision` budget fields aligned with the workloop policy
     (max turns, wall clock) — never unbounded.
3. `POST /v1/silent-sessions` (create) → `POST .../start` per session.
4. Each session's completion arrives on the EXISTING completion SSE
   (silent_session_completion, #311) and is also recorded as a bg job
   receipt when the execution leg is detached (`focusa bg run`).
5. Join: `focusa silent wait` / `GET /v1/silent-sessions/wait` per
   session id; the workloop checkpoint records the settlement
   (`focusa work-loop checkpoint`).

## Workloop compatibility invariants

- One work item → one silent session; the workloop's `current_task` +
  `decision_context` stay the authority for what runs next.
- Transport fields (`transport_session_id` etc.) bind the silent
  session to the workloop run — no orphan sessions.
- Budgets come from the workloop policy (`max_turns`,
  `max_wall_clock_ms`); a session exceeding them is interrupted via the
  existing `/interrupt` route and recorded as degraded fallback.
- Completion truth: the silent-session completion event + the bg
  receipt + the acceptance verdict (#276) — three agreeing surfaces,
  never three opinions.

## Parallel speed pattern

`focusa_bg_run_many` fans out INDEPENDENT pipeline commands (builds,
test shards). Multi-agent TASK work uses the fan-out recipe above —
sessions, not raw shells — so every agent action is ledger-backed and
adjudicable.
