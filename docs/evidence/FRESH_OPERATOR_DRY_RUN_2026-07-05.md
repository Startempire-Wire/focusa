# Fresh-Operator Dry Run — 2026-07-05

> Honest first-impression transcript from a literal fresh-operator walkthrough of Focusa on this AlmaLinux VPS. Run via tmux session `focusa-fresh-op` (200x50, real interactive shell). Goal: capture what an evaluator hits before they read any docs, find the gaps that block MVP Cohort launch, and turn them into fixups.

## Environment

- Host: AlmaLinux 8.10 (production VPS)
- Working dir at start: `/tmp/focusa-fresh-op`
- Daemon reachable: `http://127.0.0.1:8787/v1/health` → `{"ok":true,"status":"ok","uptime_ms":13292763,"version":"0.9.64-dev"}`
- Local binary on host: `/usr/local/bin/focusa` (87 MB, dated Jun 28 — pre-dates recent commits)
- No `focusa-tui` on host PATH (separate binary not installed)
- tmux: `tmux 2.7` (no update required)

## Operator Design Constraints (operator directive, 2026-07-05)

- Prefer **interactive** prompts (select/radio-style with arrow keys) over flat `--scope project|host` menus.
- Add **ASCII intros** at first run (Focusa wordmark + ambient banner).
- "Beautiful" — colors, alignment, separators, calm cadence.
- Honor these in any new onboarding/quickstart surfaces; do not replace CLI flags, but make them discoverable.

## Transcript (annotated)

```
$ tmux new-session -d -s focusa-fresh-op -x 200 -y 50 'bash -lc ...'
$ tmux send-keys -t focusa-fresh-op 'curl .../v1/health' C-m
{"ok":true,"status":"ok","uptime_ms":13264761,"version":"0.9.64-dev"}

# Step 1: WHAT IS FOCUSA?
$ curl -fsS http://127.0.0.1:8787/v1/health       # OK
$ curl -fsS http://127.0.0.1:8787/v1/version      # 404 / no endpoint
$ curl -fsS http://127.0.0.1:8787/v1/about        # 404
→ gap: no `/v1/about` or `/v1/version` orientation endpoint.

# Step 2: WHAT CAN I DO?
$ which focusa                                    # NOT FOUND on PATH
$ ls /usr/local/bin/focusa                        # present (older binary, Jun 28)
→ gap: README assumes installed binary but install path is implicit.

# Step 3: README
README is comprehensive (~470 lines) but starts with technical authority model;
no quickstart near the top. Evaluator must scroll to "Run from repo root" section.
→ gap: README lacks a 60-second quickstart block.

# Step 4: FOCUSA --HELP
$ focusa --help
Focusa cognitive governance CLI
Commands:
  start              Start the Focusa daemon
  stop               Stop the Focusa daemon
  install-service    Install and enable the Focusa daemon service
  codesign           macOS code signing + notarization inspection helper
  status             Show daemon status
  onboard            Run first-run Operator Preview onboarding
  pair               Open a Mac Pairing Room...
  pairing            Pairing umbrella (focusa-ui0y v0.9.39-dev)
  ...
→ observation: long flat list. No ASCII intro, no grouping, no color.
→ observation: `tui` not listed (only the new headless self-test surface added today).

# Step 5: FOCUSA ABOUT
$ focusa about | head
... verbose daemon stats. No orientation, no "what next".

# Step 6: FOCUSA DOCTOR
$ focusa doctor
Next action: Run the recovery command for the first blocked check, then re-run focusa doctor
Command: focusa doctor
→ ok but verbose; evaluator doesn't know what to *do*.

# Step 7: FOCUSA ONBOARD
$ focusa onboard --scope project --no-demo-workpoint
error: unexpected argument '--scope' found
→ THE LOCAL BINARY IS STALE (Jun 28). New `--scope` flag and `tui` subcommand
   exist in source but were never built into the installed binary. Fresh evaluator
   reading README would believe `--scope` works; it does not on this install.

# Step 8: FOCUSA TUI
$ focusa tui --headless-self-test
error: unrecognized subcommand 'tui'
→ confirms binary staleness.

# Step 9: FOCUSA ONBOARD --REMOTE QUICKSTART
$ focusa onboard --scope project --remote <git-url> --project-root /tmp/freshop-proj
bash: cd: /tmp/freshop-proj: No such file or directory
→ gap: `--project-root` is not auto-created; evaluator expected `mkdir -p`.

# Step 10: API IDENTITY PROBE
$ curl ".../v1/project/identity?project_root=/home/wirebot/focusa"
{"canonical":true,"degraded":false, ..., "next_tools":["focusa_project_identity",
 "focusa_project_verify","focusa_trajectory_view","focusa_workpoint_resume"]}
→ ok; returns canonical=true. Authority model is healthy.

# Step 11: AUDIT TAIL
$ tail -n 1 .../audit.jsonl | python3 -m json.tool
... self-heal synthesis rows continue accumulating, 388 rows total.
→ ok; telemetry is alive.

# Step 12: TELEMETRY SNAPSHOT
$ curl .../v1/telemetry/snapshot
... ok
```

## Gaps identified (5 ranked by ROI for MVP Cohort launch)

1. **README quickstart missing.** No 60-second "3 commands to health ok" block.
   Fix: add a `## Quickstart (60 seconds)` block at the top of `README.md`.
   ~15 minutes.

2. **No `/v1/about` or `/v1/version` orientation endpoint.** Evaluator hitting `/v1/health`
   gets no pointer to next steps.
   Fix: add `GET /v1/about` returning `{project, version, quickstart, next_tools, owner}`.
   ~30 minutes.

3. **Installed binary drift.** Source has `--scope`/`tui`/quickstart; binary at
   `/usr/local/bin/focusa` predates them. Evaluator fails on documented commands.
   Fix: rebuild + reinstall, document `scripts/install-daemon.sh` rotation.
   ~20 minutes.

4. **`focusa onboard --project-root <path>` requires pre-created dir.**
   Fix: `ensure_dir_all` before writing marker.
   ~5 minutes.

5. **`focusa --help` is a flat list, no ASCII intro, no color, no interactive grouping.**
   Operator directive explicitly asks for ASCII intros + interactive prompts.
   Fix: introduce `cliclack` or `dialoguer` interactive prompts in `onboard`,
   ASCII wordmark for `focusa --help`, banner in `focusa about`.
   ~2 hours.

## Tracker cross-refs

- `focusa-7wgk` (this bead) — Tier2-2 fresh-operator dry-run captured.
- `focusa-cme3` — Tauri release artifacts in `release.yml` (closes gap #3 partially).
- `focusa-ui0y` — Mac pairing E2E (still blocked on interactive Mac GUI session).

## Evidence

- tmux session log slices: `/tmp/freshop-help.txt`, `/tmp/freshop-onboard.txt`,
  `/tmp/freshop-final.txt`, `/tmp/freshop-full-all.txt` (360 lines).
- Local daemon health: `{"ok":true,"status":"ok","version":"0.9.64-dev"}`.
- Identity probe response captured above (canonical=true).

## Next steps

1. Implement README quickstart (gap #1) → push, CI.
2. Implement `/v1/about` endpoint (gap #2) → static guard + push.
3. Add `ensure_dir_all` to onboard `--project-root` (gap #4).
4. Re-run install + smoke `focusa onboard --scope project` against fresh binary.
5. Land ASCII intro + interactive prompts (gap #5) as a follow-up bead.