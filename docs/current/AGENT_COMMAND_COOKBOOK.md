# Agent Command Cookbook

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

Copy/paste commands for agent-first Focusa operation.

## Starting work

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
focusa doctor
focusa status --agent
focusa status --operator
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


## Before risky mutation / install / daemon repair

```bash
focusa --json action classify-intent --prompt "$CURRENT_ASK"
focusa --json env contract show
focusa --json runtime inventory --owner ${FOCUSA_OWNER:-$USER}
focusa --json action preflight \
  --current-ask "$CURRENT_ASK" \
  --kind binary_replace \
  --target /usr/local/bin/focusa \
  --source github_release_asset \
  --install-role live_build_host \
  --project-root "$PWD"
```

If verdict is `block` or `ask_operator`, do not mutate. On a live build host, use local repo build/restart, never a release asset replacement.

## Before compaction

```text
focusa_workpoint_checkpoint checkpoint_reason="before_compact" mission="..." next_action="..."
```

## After compaction

```text
focusa_workpoint_resume mode="compact_prompt"
```

For non-Pi/manual agents:

```bash
focusa workpoint resume --copy-prompt
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

## Reflex suggestions / recurring recovery

```bash
curl -sS 'http://127.0.0.1:8787/v1/reflex/primitives?family=recovery&limit=5' | jq .
# In Pi: focusa_reflex_primitives family=recovery limit=5
```

Use this only when `tool_result_v1.reflex_suggestions` names a primitive or recurring risk; it is read-only advisory metadata.

## Mac app stale

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/menubar
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
