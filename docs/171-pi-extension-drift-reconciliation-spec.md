# Pi Extension Drift Reconciliation Spec (#171)

**Bead:** focusa-wphg4 (P1) · **Date:** 2026-08-22 · **Scope:** apps/pi-extension only

## Findings (verified 2026-08-22)

| Surface | State |
| --- | --- |
| Installed extension (`~/.pi/agent/extensions/focusa`) | focusa-pi-bridge **0.9.177**; contains ONLY three unique deltas vs origin/main (the reception patches listed below) |
| Repo working tree (`/home/wirebot/focusa`, branch `local/work-loop-completion` @ b4ecc17ee) | apps/pi-extension at 0.9.121-dev — STALE branch missing 0.9.122→0.9.181 history |
| `origin/main` | 0.9.181 — canonical source; AHEAD of installed (newer `/v1/` API paths, EXTENSION_BUILD stamp 0.9.181) |

Diff origin/main vs installed src: 7 entries = 4 main-ahead files
(auto-compaction [build stamp only], commands/session/tools [`/v1/` path migration])
plus today's reception patches:

1. `index.ts` — status shortcut rebind ctrl+shift+f → ctrl+alt+f (built-in tui.altScreen.search conflict)
2. `north-star.ts` — progressive-disclosure renderNorthStarCard rewrite
3. `model-plan-advisory.ts` (+ index.ts wiring) — plan-gated Codex model graceful fallback to gpt-5.6-luna

## Plan

1. `git worktree add <tmp>/focusa-piext-sync -b fix/pi-ext-port-reception-patches origin/main`
   (no checkout switch of the live repo; other agents unaffected).
2. Apply the three patches onto the worktree source (port from installed copy).
3. Stamp version 0.9.182-dev (verify-version-surfaces rules apply).
4. Typecheck/lint as available; bun parse minimum.
5. Commit(s): conventional commits per patch cluster.
6. Deploy: dev-lane release via canonical script + OTA activation — REQUIRES operator
   approval (touches remote).

## Rollback

Worktree is disposable (`git worktree remove`); no changes to live repo, installed
extension untouched until an approved release activates.

## Acceptance

- origin/main contains the three reception patches with green gates
- Installed extension updated via canonical OTA path (no hand-copied artifacts)
- Duplicate-load warning stays absent; shortcut works; north-star card renders new style
