# 181 — Focusa × UIAI Web Runtime Integration Spec (mirror of UIAI-ENGINE-010)

**Status:** F1 code-complete (engine W2 live; bridge route + client widget landed) — awaiting daemon release ≥ bridge-route build to go live. F2–F4 pending per gating below.
**Engine counterpart:** `WPUIAI/uiai-engine/docs/010-uiai-engine-web-runtime-leap-spec.md` (claim IDs C-010-*)
**Owner surfaces:** workforce extension (startpage/wall/sidepanel), focusa-api bridge routes, silent sessions

## F1 — Browser Fleet widget family (engine C-010-26 metrics + health)
Focusa daemon gains a bounded read-only bridge: `GET /v1/browser-fleet/status` proxying the paired engine's `/api/health/browser` (token held server-side; never exposed to clients). Widget `focusa.browser.fleet` renders pools, queue p95, budget pauses; wall-safe (`mutation:none`).
**Accept:** widget renders live fixture + live engine; unauthorized engine → truthful `unauthorized` state.

## F2 — Event citizenship ingestion (C-010-02)
Engine emits `focusa.stream_event.v1`; daemon SSE fans them into the existing stream grammar. Workforce notification center + audit center ingest `budget.paused`, `auth.required`, `challenge.*`, `egress.degraded` with shared redaction grammar (raw payloads never persisted).
**Accept:** fixture stream → notification appears; audit center shows entry with source=engine.

## F3 — Governed browsing beads (C-010-01/03/22)
Silent sessions spawn **browser tasks**: intent-verb call chains wrapped as work items with budget ids, writer-lease per `(page,scope)`, outcomes attached as evidence citations (`artifact_ref` from C-010-09).
**Accept:** two concurrent tasks on one page → second waits on lease; completed task closes with citation.

## F4 — Continuity + personas (C-010-07/14/23/25)
Web-state checkpoints referenced inside Workpoint packets (compaction-safe); device pairing binds persona ids; walls display acting persona read-only.
**Accept:** resume-after-compaction restores browsing context hash; unbound persona denied.

## Status (2026-08-25)
- **F1:** code-complete — engine W2 claims live (verbs/budgets/warm/artifact-ref verified on OVH workers); bridge route + client widget landed; **goes live on next daemon release** (focusa#341).
- **F2:** engine emitter LIVE (focusa.stream_event.v1 via /api/focusa-events, deployed); daemon passthrough `/v1/browser-fleet/stream` + client ingestion landed in extension.

## Sequencing
F1/F2 after engine W3 stream emitter lands · F3 after W2 verbs (✅ available now) · F4 after engine W4.

## Non-goals
Direct browser→daemon authority; client-held engine tokens; wall mutation.
