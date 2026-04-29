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
import { loadSkills } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';
const r = loadSkills({ cwd: process.cwd(), agentDir: '/root/.pi/agent', skillPaths: [], includeDefaults: true });
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
curl -sS http://127.0.0.1:8787/v1/work-loop/status | jq .
```

Use `focusa_work_loop_writer_status` in Pi before mutating work-loop state.

## Non-canonical fallback

Treat non-canonical Workpoint output as a recovery hint. Call `focusa_workpoint_resume` or direct `/v1/workpoint/current` before continuing important work.

## Real release proof

Use `docs/current/VALIDATION_AND_RELEASE_PROOF.md` for current validation expectations.
