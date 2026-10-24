# Current Troubleshooting Guide

## Daemon health

```bash
curl -sS http://127.0.0.1:8787/v1/health | jq .
focusa status
systemctl status focusa-daemon --no-pager -l
```

Health `ok` proves liveness, not writable storage. Check `persistence_readiness` in health or doctor: `unknown` means no completed write was observed; `blocked` means the latest write failed; `ready` means the last observed write succeeded, not guaranteed future capacity. Cumulative failure counts remain after recovery. Restore verified quota/storage headroom or repair the actual write error, then verify the requested operation's durable acknowledgement. Health does not write a probe or restart anything.

SQLite event/snapshot writers reserve their write transaction before reading chain or cursor metadata; an event and its hash-chain link commit together. A lock timeout is not a successful write. Reconcile durable event IDs before retrying an operation with uncertain effects.

## Snapshot recovery (current source)

`rebuild-state` requires a snapshot with a matching durable cursor and event-chain prefix. It streams the complete suffix in sequence order, preserves indexed ownership and recorded budget identity/time, and atomically saves the rebuilt snapshot with its cursor through the existing persistence owner. Missing prefixes, unlinked events, gaps or a target that advanced before writing fail without changing the snapshot. `--since` is an optional legacy hint, not a timestamp filter.

This is source behavior, not proof of the installed command. Older timestamp-only implementations are not qualified for current cursor-based databases. Use verified backups and the approved recovery procedure; no production rebuild is implied by source tests.

## Skill loading problems

If Pi reports `description is required`, the skill is missing YAML frontmatter. Validate with:

```bash
node --input-type=module - <<'NODE'
import { loadSkills } from '<pi-install-dir>/dist/core/skills.js';
const r = loadSkills({ cwd: process.cwd(), agentDir: '${PI_AGENT_DIR:-$HOME/.pi/agent}', skillPaths: [], includeDefaults: true });
console.log(r.skills.map(s => [s.name, s.filePath]));
console.log(r.diagnostics);
NODE
```

## Workpoint stale or unexpected

```bash
focusa workpoint current
focusa workpoint resume
curl -sS http://127.0.0.1:8787/v1/workpoint/current | jq .
```

If a result is `pending`, retry current/resume before relying on it.

Saved context remains orientation until a fresh daemon response is validated. Responses from a replaced attachment or ended startup lifecycle cannot renew local action authority, even when visible identifiers match. A fresh valid response can restore that local authority without changing the underlying grants.

## Work-loop writer conflict

Writer conflicts are blocked states, not daemon failures.

```bash
curl -sS http://127.0.0.1:8787/v1/work-loop/status?summary_only=true | jq .
```

Use `focusa_work_loop_writer_status` in Pi before mutating work-loop state.

## Non-canonical fallback

Treat non-canonical Workpoint output as a recovery hint. Call `focusa_workpoint_resume` or direct `/v1/workpoint/current` before continuing important work.

## Reflex suggestions

If a Pi/API result includes `reflex_suggestions`, treat those ids as advisory Spec97 recovery affordances. Inspect the registry without mutating state:

```bash
curl -sS 'http://127.0.0.1:8787/v1/reflex/primitives?family=recovery&limit=5' | jq .
```

In Pi, use `focusa_reflex_primitives` only to clarify the smallest safe next step; `next_tools`, operator steering, and canonical Workpoint/Trajectory scope still decide the route.

## Installer terminal UI recovery

`focusa install` restores the alternate screen and cursor through a terminal guard. If a host terminal is left in a damaged state after an external kill or emulator crash, run:

```bash
reset || tput reset
printf '\033[?25h\033[0m'
```

Then rerun with a nonanimated mode:

```bash
FOCUSA_INSTALL_UI=plain focusa install --no-animation
```

Installer JSON troubleshooting should always use `focusa install --json ...`; stdout must contain one JSON document and no ANSI. `NO_COLOR=1` requests monochrome animation on a suitable TTY, while `--no-animation` requests plain output.

## Release Git administration permissions

`release_git_admin_not_writable` stops release preparation and local preflight before costly gates; the same read-only check runs again before each release commit. Inspect the reported paths with:

```bash
python3 scripts/release-git-admin-preflight.py --repo /path/to/checkout
```

Run as the checkout owner. Linked worktrees keep `COMMIT_EDITMSG` in the Git-resolved administration directory, not necessarily `<checkout>/.git/`. The check never repairs permissions, removes locks, changes staged files, or grants release readiness. Use the approved owner-permission recovery procedure only for verified reported paths; preserve active locks and staged state. A successful check is not a strict preflight, published release, or deployment receipt.

## Real release proof

Use `docs/current/VALIDATION_AND_RELEASE_PROOF.md` for current validation expectations.
