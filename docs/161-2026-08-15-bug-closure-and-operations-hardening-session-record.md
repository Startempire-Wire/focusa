# 2026-08-15 Session Record — Bug Closure, Crash Fixes, DB Retention, and Completion Notifications

**Scope:** full Pi-extension crash closure, Pi package activation transaction
(#309), event-ledger retention + live DB reclamation, silent-session
completion notification (#311), bug-squash cluster (#45, #250, #251, #266,
#282, #302-#306), and operational hardening of the anchor server.

## 1. Crash fix: Mission Canvas stale session context (#124, #301)

- **Root cause:** the Mission Canvas work rail cached the session ctx in a
  module global and re-read it from a `queueMicrotask` dispatch after
  session replacement/reload invalidated it; Pi's `assertActive` threw on
  the `ctx.hasUI` read, escaping handler protection as an uncaught exception.
- **Deployed hotfix** (extension 0.9.152, both hardlinked/symlinked trees):
  try/catch degradation in `refreshMissionCanvasWidget`, per-listener guard
  in the scoped-refresh dispatch loop, `.catch` on both polling calls,
  `ctrl+shift+f` → `ctrl+shift+e` rebind (built-in `tui.altScreen.search`
  collision). Backups: `/root/.pi/focusa-ext-fix-backup-20260815/`.
  Verified: tsc clean, surface-refresh suite 10/10, new stale-ctx regression
  test 4/4, live restart clean (session ledger zero errors).
- **Repo-side fix (PR #310):** the other session's
  `fix/issue-301-stale-mission-context` branch adds a
  `LifecycleGenerationGuard` (generation-fenced timers/listeners/in-flight
  polls) + `mission-canvas-context-lifecycle.test.mjs`. I completed the PR by
  adding the missing `lifecycle-guard.ts` (commit `36436353` on the PR
  branch). Verification: lifecycle test 1/1 green; full mission-canvas suite
  run logged at `/tmp/focusa-pr310-suite.log`.
- Docs: `docs/157-pi-extension-mission-canvas-stale-ctx-crash-resolution.md`
  (pushed).

## 2. Pi package activation transaction + OTA rollback (#309)

- Shared transaction module `crates/focusa-cli/src/commands/pi_package.rs`:
  identity-gated retirement to `~/.pi/agent/retired-extensions/`, backups
  outside discovery, typed `PiActivationReceipt` with
  commit/rollback boundaries; OTA retains the receipt, rolls Pi back with
  binaries on downstream failure, commits only after settlement;
  `FOCUSA_UPDATE_FAULT_AFTER_PI_ACTIVATION` fault injection.
- Enforcement: static policy gate in pre-push + CI; AGENTS.md
  one-canonical-Pi-package rule; INSTALLER_UPDATE_POLICY section.
- Tests: `cargo test -p focusa-cli pi_package` 8/8; `update` 12/12; clippy
  `--all-targets -D warnings` clean. Commit `32b6440f`.
- Docs: `docs/160-pi-package-activation-transaction-and-ota-rollback.md`.

## 3. Event-ledger retention + live DB reclamation

- **Diagnosis:** daemon SQLite 11.1GB = `events` 6.26M rows +
  `event_hash_chain` 6.26M rows; ~3.26M were epoch-0 placeholder events from
  the retired temporal fallback (writer no longer exists in source).
- **Engine** `crates/focusa-core/src/runtime/event_retention.rs`: batched
  junk pruning, hot-window cold export (JSONL), hash-chain anchoring (meta
  checkpoint), bounded incremental vacuum; daemon daily sweep +
  `POST /v1/events/prune` + `focusa events prune`.
- **Live reclamation** (executed): index-drop + rowid-range bulk deletes
  (junk remaining = 0), index rebuild, then `VACUUM INTO` via Node's bundled
  SQLite (system sqlite3 3.26 lacks `VACUUM INTO`) —
  11.1GB → ~1.2GB compacted file, swapped in with the daemon service.
- **Operations discovered:** the daemon is managed by
  `focusa-daemon.service` + a 30s `focusa-daemon-healthcheck.timer` watchdog;
  both must be stopped for any exclusive maintenance window. The OVH build
  host had a 12-hour-stale `focusa-daemon --version` process holding the
  test lock file, blocking all remote test runs (killed). wirebot user quota
  (30GB) was exhausted; rebuildable caches and age-bounded pre-migration DB
  backups were removed with operator authorization; the nightly rustup
  toolchain was uninstalled (rebuildable; local builds go via the OVH host).
- Docs: `docs/158-event-ledger-retention-and-db-size-architecture.md`
  (incl. the runbook).

## 4. Silent-session completion notification (#311)

- Durable deduped completion-event ledger
  (`silent_session_completion_events`, UNIQUE(session,run,status)); SSE
  broadcast `silent_session_completed`; 30s daemon sweeper;
  `GET /v1/silent-sessions/wait` long-poll + `completions` backfill +
  `sweep-completions`; `focusa silent wait` CLI; Pi extension
  `uiCtx.notify` handler in both extension trees.
- Herdr research attached to the issue: herdr.dev is an agent-aware terminal
  runtime (socket API, pane state machine, plugin marketplace); no Focusa
  integration exists; optional adapter later.
- TBQ rule codified in AGENTS.md: terminal-blocking queries must run
  asynchronously; background terminals auto-report completion into the
  originating terminal.
- Gates: ALL-GATES-GREEN chain (retention 4/4, completion-events 2/2,
  cargo check api+cli, clippy -D warnings ×3 crates). Commit `f090faf7`.
- Docs: `docs/159-silent-session-completion-notification.md`,
  `docs/current/BACKGROUND_EXECUTION_AND_COMPLETION_NOTIFICATION.md`.

## 5. Bug-squash cluster (commits `9e3dff4b`, `7568883f`, `85bb3b9a`)

| Issue | Root cause | Fix |
| --- | --- | --- |
| #302 | demo script read flat identity fields; `curl -f` aborted on entitlement 4xx | nested `project_identity.*` jq; non-`-f` bounded rendering |
| #303 | deprecation alias pointed at nonexistent `focusa setup walkthrough` | registered `SetupCmd::Walkthrough` |
| #304 | text inference read descriptor prose as failure; default family `focus_state` | contract-registry-first inference (`findFocusaToolContract`); prose never read as outcome |
| #305 | audit/prove scripts conflated authority-gated 403 with daemon_unavailable | typed `authority_gated`/`validation_typed` classification; bounded scope fixtures |
| #306 | `tool_result_v1` schema rejected its own envelope fields | schema declares tool/family/endpoint/workpoint_id/reflex/ontology/error/raw + `not_found` |
| #250 | blocking reqwest client inside Tokio | superseded: async client already in current code (no patch needed) |
| #251 | Bonjour failure crash-loop; `FOCUSA_DISABLE_MDNS` ignored | env gate + non-fatal logging in `main.rs` |
| #266 | `[object Object]` from object `error` envelopes; bootstrap gate gone on branch | stringify object errors in `explainWorkLoopResult` |
| #45 | SSE envelope sends `event_type`; extension read `evt.type` | `evt.event_type || evt.type` in both trees |
| #282 | read-only planning rejected + `[object Object]` (v0.9.152) | superseded: current handler is typed; evidence logged |

## 6. Issue-ledger state at session end

Closed today: #45, #124, #250, #251, #266, #282, #301, #302, #303, #304,
#305, #306, #309, #311 (14). #275 checklist updated (4 additional ticks).
Remaining mandatory-candidate open: #52, #89, #101, #112, #119, #125, #128,
#138, #195, #243, #244, #307 (feature-scale; scoped on #275). P0 roadmap
programs #252-#299 remain explicitly out of the 0.9.x release scope.

## 7. Commit ledger (local/work-loop-completion)

```
c2a71b72 docs: 157 — stale-ctx crash resolution (#124, #301)
32b6440f feat(cli): crash-safe Pi package activation transaction (#309)
3b305670 docs: TBQs must run asynchronously (#311)
f090faf7 feat(daemon): event-ledger retention + silent-session completion notification (#311)
9e3dff4b fix: bug-squash cluster — demo identity, walkthrough, tool-result truth, audits, schemas
7568883f fix(daemon): honor FOCUSA_DISABLE_MDNS (#251)
85bb3b9a fix(pi): stringify object error envelopes in workpoint blocked text (#266)
8ec9e298 fix(pi): read daemon SSE event_type (#45)
```

## 8. Runtime/bun assessment (operator question)

Node stays the canonical runtime (Pi loads the extension in-process). Bun is
adopted only for one-shot script execution where startup speed helps and no
repo change is required; extension installs stay on npm/package-lock to
avoid dual-lockfile drift with CI.
