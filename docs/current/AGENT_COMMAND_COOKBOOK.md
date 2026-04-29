# Agent Command Cookbook

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

Copy/paste commands for agent-first Focusa operation.

## Starting work

```bash
cd /home/wirebot/focusa
focusa doctor
focusa status --agent
bd ready
```

## Before risky edit

```text
focusa_workpoint_checkpoint mission="..." next_action="..." checkpoint_reason="manual"
```

```bash
git status --short
git diff --stat
```

## Before compaction

```text
focusa_workpoint_checkpoint checkpoint_reason="before_compact" mission="..." next_action="..."
```

## After compaction

```text
focusa_workpoint_resume mode="compact_prompt"
```

If daemon was down:

```bash
systemctl status focusa-daemon --no-pager
curl -fsS http://127.0.0.1:8787/v1/health | jq .
```

## Daemon down / holdover

```bash
systemctl restart focusa-daemon
journalctl -u focusa-daemon -n 80 --no-pager
focusa doctor
```

## Continue work

```bash
focusa continue
focusa continue --parent-work-item-id focusa-bzwt
focusa continue --enable --parent-work-item-id focusa-bzwt
```

## Token budget high

```bash
focusa tokens doctor
focusa tokens compact-plan
focusa telemetry token-budget
```

## Cache stale

```bash
focusa cache doctor
focusa cache status
focusa cache policy
```

## Release failed

```bash
focusa release prove --tag <tag> --fast
focusa release prove --tag <tag> --github
journalctl -u focusa-daemon -n 80 --no-pager
```

## Mac app stale

```bash
cd /home/wirebot/focusa/apps/menubar
bun install
./node_modules/.bin/svelte-kit sync
bun run check
bun run build
```

## Prediction loop

```bash
focusa predict record --prediction-type next_action_success --predicted-outcome completed --confidence 0.7 --recommended-action "continue" --why "all gates green"
focusa predict recent
focusa predict evaluate <prediction_id> --actual-outcome completed --score 1.0
focusa predict stats
```

## Safe cleanup

```bash
focusa cleanup --safe --dry-run
focusa cleanup --safe
```

Never delete `.beads/`, `data/`, or `target/release/focusa-daemon` during production work.
