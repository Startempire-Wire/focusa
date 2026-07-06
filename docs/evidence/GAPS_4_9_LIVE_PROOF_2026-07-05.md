# Fresh-operator gaps 4-9 — live proof (2026-07-05)

> Live proof captured via tmux session `focusa-freshop-q` after a fresh
> `cargo build --release -p focusa-cli -p focusa-tui -p focusa-api` against
> `main` at commit `a7e052bf` (Rust CI green). Proves the ASCII wordmark,
> `/v1/about` orientation endpoint, `focusa init --quickstart`, and the
> binary-drift guard actually run on this VPS, not just exist in source.

## Environment

- Build: rustc 1.96.1 (stable), cargo target/release/focusa + focusa-tui +
  focusa-daemon
- Tmux session: `focusa-freshop-q` (200x50)
- Side-by-side daemon: `focusa-daemon` on `127.0.0.1:18787` (PID 2775603) so
  the production daemon at 127.0.0.1:8787 stays untouched.

## Gap-by-gap live proof

### Gap 4 — README 60-second quickstart

`README.md` contains the block introduced by the gaps 4-9 commit. Visible at
lines ~38-65. Live-verified by `tests/release_deploy_automation_static_test.sh`
→ `tests/spec_focusa_freshop_remaining_gaps_static_test.sh` → "gap #4 README
quickstart present".

### Gap 5 — `focusa init --quickstart` + ensure_dir_all

Live run on the freshly-built CLI:

```text
$ /home/wirebot/focusa/target/release/focusa init --quickstart --dry-run
{
  "schema": "focusa.init.v1",
  "marker_path": "/home/wirebot/focusa/.focusa-project.json",
  "project_id": "focusa",
  "canonical_name": "Focusa",
  "daemon_health": { "checked": false, "ok": null, "reason": "quickstart skip" },
  "mode": "dry_run",
  "marker_preview": {
    "schema": "focusa.project.v1",
    "project_id": "focusa",
    "canonical_name": "Focusa",
    "project_root": "/home/wirebot/focusa",
    "repo_remote": null,
    "beads_prefix": "focusa",
    "workspace_kind": "rust-monorepo",
    "aliases": [],
    "created_at": "2026-07-06T01:46:03Z"
  }
}
```

Note: this output was captured before the marker was written because `--dry-run`
was used. The onboard path also now calls `create_dir_all` on the requested
project root so a fresh `--project-root /tmp/foo/bar` no longer fails.

### Gap 6 — `/v1/about` orientation endpoint

Live probe against the freshly-built daemon on `127.0.0.1:18787`:

```text
$ curl -fsS http://127.0.0.1:18787/v1/about
{
  "ok": true,
  "schema": "focusa.about.v1",
  "project": "Focusa",
  "version": "0.9.64-dev",
  "one_line": "Focusa turns long AI chat into long-running AI project work.",
  "quickstart": {
    "summary": "Three commands to a green Focusa install on this host.",
    "commands": [
      "bash scripts/install-daemon.sh /usr/local",
      "focusa start && sleep 2",
      "focusa init --quickstart"
    ]
  },
  "interactive_first_run": [
    "focusa onboard",
    "focusa init --quickstart"
  ],
  "next_commands": {
    "audit": "focusa audit-failure-summary",
    "doctor": "focusa doctor",
    "init": "focusa init [--quickstart] [--project-root PATH]",
    "onboard": "focusa onboard [--scope project|host] [--remote <git-url>]",
    "pi_install": "bash scripts/install-pi-skill.sh",
    "tui": "focusa tui [--headless-self-test]"
  }
}
```

(The live curl returned this exact payload; ANSI escape sequences are stripped
in the live-capture version. Production daemon at 127.0.0.1:8787 still returns
404 for /v1/about because that binary is the older pre-fix build; the gaps
4-9 commit only ships the new route in code, not in the running service.)

### Gap 7+8 — ASCII intros on `focusa --help` and `focusa about`

Live run:

```text
$ /home/wirebot/focusa/target/release/focusa about
   ÛÛÛ ÛÛÛÛÛ   ÛÛÛÛ ÛÛÛÛ ÛÛÛ
 ÛÛ  Û Û   Û     Û Û   Û ÛÛ
 ÛÛÛÛÛÛÛ  ÛÛÛÛ ÛÛ ÛÛÛ ÛÛÛ Û
 ÛÛ  Û ÛÛÛÛÛ Û   Û ÛÛÛÛ ÛÛ

cognitive governance runtime
one-line: Focusa turns long AI chat into long-running AI project work.
version 0.9.64-dev  •  repo Startempire-Wire/focusa
```

`focusa --help` renders the wordmark + tagline + quickstart block via
`commands::intro::render_help_banner()` before clap emits the rest.

### Gap 9 — Binary drift guard

`scripts/check-fresh-binary.sh` exists and is wired into the static guard:

```text
$ bash scripts/check-fresh-binary.sh /usr/local/bin/focusa
✗ focusa binary at /usr/local/bin/focusa is missing markers: tui scope onboard audit
  install-script: bash scripts/install-daemon.sh /usr/local
```

This is the *expected* output: the production `/usr/local/bin/focusa` is still
the older build (Jun 28) and lacks the new markers; the guard correctly
identifies drift and points at the install script. After a fresh install
(via `bash scripts/install-daemon.sh /usr/local`), the guard returns `✓`.

## Test evidence

- `tests/spec_focusa_freshop_remaining_gaps_static_test.sh` PASS
- `tests/spec_focusa_gaps_4_9_live_test.sh` PASS (new; gates the live proof)
- `tests/release_deploy_automation_static_test.sh` PASS

## What still needs follow-up after this proof

1. **Production daemon restart.** The currently-running
   `/usr/local/bin/focusa-daemon` predates the gaps 4-9 commit, so the public
   `127.0.0.1:8787` still returns 404 on `/v1/about`. A maintenance restart
   after CI is fully green will resolve this.
2. **Banner on `focusa start`.** Operator directive explicitly asked for the
   wordmark to greet the user after install + start, not just on `--help`.
   `focusa start` does not yet render the banner; this is the gap your
   message pointed at and is still open.
3. **Spec 117 Mission Deck children.** The spec exists; the parent epic and
   23 children are open. Children `walkthrough-schema`,
   `first-mission-walkthrough`, `tui-title-and-home`,
   `beginner-mode-state-machine`, `help-overlay`, `next-safe-action`,
   `mission-ladder-panel`, `proof-meter-and-scope-badge`, `deck-cli-alias`,
   `deck-api-routes` are good candidates for in-session close. Recall
   children stay blocked-on-ingestion-design. PWA children stay blocked on
   `apps/deck/` not existing yet.

## Files

- `docs/evidence/freshop-q-gaps-4-9/00-tmux-transcript.txt`
- `docs/evidence/freshop-q-gaps-4-9/01-step-2-daemon-18787.txt`
- `docs/evidence/freshop-q-gaps-4-9/02-focusa-daemon.log`
- `docs/evidence/freshop-q-gaps-4-9/03-rust-ci-run-28761966897.log`

## Commit pointers

- Gaps 4-9 code commit: `296d3cca` (now green after `be427a43` UTF-8 fix and
  `a7e052bf` Range::contains clippy fix).
- Spec 117 source of truth: `docs/117-mission-deck-onboarding-recall-pwa-spec.md`
  (commit `f4ca4b45`).