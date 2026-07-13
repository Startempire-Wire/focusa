# Current Troubleshooting Guide

## Daemon health

```bash
curl -sS http://127.0.0.1:8787/v1/health | jq .
focusa status
systemctl status focusa-daemon --no-pager -l
```

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

## Real release proof

Use `docs/current/VALIDATION_AND_RELEASE_PROOF.md` for current validation expectations.
