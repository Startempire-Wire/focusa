# Focusa — Claude Code recent-turns adapter

Spec 101 §5.12.11 — Adapter contract implementation for Claude Code.

This adapter is part of the cross-agent Focusa feature set: every major
coding agent sees the same recent-turns slice, recall-intent trigger, and
provider prompt cache split, sourced from a single canonical daemon.

## v0.9.142 surface

The adapter follows the current cross-harness contract: verify `project_root + continuity_id`, orient with Agent Card/Trajectory/Workpoint, keep mutations scope-bound, capture evidence after proof, and record/evaluate predictions around uncertainty. Context Cognition, Project Card/Genesis, preload, session transfer, Temporal Authority, Tool Discovery, and release-proof routes are available through the same typed API; operator steering remains authoritative.

## What it does

1. **Capture**: on every Claude Code turn boundary (Stop / PostToolUse /
   PreCompact), the adapter reads the latest assistant+user text from the
   session transcript and POSTs a structured turn slice to
   `POST /v1/turns/recent` on the Focusa daemon.
2. **Inject**: on `UserPromptSubmit` and `SessionStart`, the adapter
   fetches `GET /v1/turns/recent?n=4&continuity_id=...` and emits a
   compact `## Recent turns` section to stdout. Claude Code appends
   stdout output to the model context.
3. **Recall-intent trigger**: when the user's prompt matches the
   §5.12.10 recall-intent word set (`recall`, `remember`, `earlier`,
   `what did we`, etc.), the adapter force-emits the slice and POSTs
   `POST /v1/events/recall-trigger` for telemetry.
4. **Fail soft**: any daemon error → exit 0, no stdout, no block.

## Install

### 1. Copy the hook script somewhere stable

```bash
mkdir -p ~/.claude/hooks
cp /home/wirebot/focusa/adapters/claude-code/bin/focusa-claude-code-hook.sh \
   ~/.claude/hooks/focusa-recent-turns.sh
chmod +x ~/.claude/hooks/focusa-recent-turns.sh
```

### 2. Wire into Claude Code settings.json

Edit `~/.claude/settings.json` (or `.claude/settings.json` for
project-scoped) and add the hooks block:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/home/<user>/.claude/hooks/focusa-recent-turns.sh",
            "timeout": 5
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/home/<user>/.claude/hooks/focusa-recent-turns.sh",
            "timeout": 5
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/home/<user>/.claude/hooks/focusa-recent-turns.sh",
            "timeout": 10
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/home/<user>/.claude/hooks/focusa-recent-turns.sh",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

The hook receives the event JSON on stdin; no further wiring required.

### 3. Configure the daemon URL (optional)

Defaults to `http://127.0.0.1:8787`. To override:

```bash
export FOCUSA_DAEMON_URL=http://100.69.132.82:8787
export FOCUSA_CONTINUITY_ID="claude-code:myproject"
```

Add to `~/.bashrc` or wrap the script with these env vars.

## Verification

```bash
# 1. Inject path manually (SessionStart simulation)
echo '{"session_id":"test-123","cwd":"/tmp","hook_event_name":"UserPromptSubmit","prompt":"what did we do earlier"}' \
  | bash /home/wirebot/focusa/adapters/claude-code/bin/focusa-claude-code-hook.sh
# Expected: stdout contains "## Recent turns ..." (empty if daemon has no entries)
# stderr: "[focusa-hook] ..." lines on failure

# 2. Direct daemon probe
curl -sS 'http://127.0.0.1:8787/v1/turns/recent?n=4&continuity_id=claude-code:test-123' | jq

# 3. Post a turn
curl -sS -X POST -H 'Content-Type: application/json' \
  -d '{"turn_id":"t1","continuity_id":"claude-code:test-123","mission_at_turn":"audit gaps","outcome":"tooled","evidence_refs":[],"tool_call_count":3,"emitted_at":1783361221}' \
  'http://127.0.0.1:8787/v1/turns/recent' | jq
```

## Rollout

1. Stage the script in `~/.claude/hooks/` (per user, per machine).
2. Edit `~/.claude/settings.json` (no project-level coupling needed unless
   you want to share with teammates via `.claude/settings.json`).
3. Test with the verification block above.
4. After one session of real use, check `proof/claude-code-adapter-*.txt`
   for telemetry hits (the daemon logs every recall-trigger and append).

## Notes

- The hook script is intentionally bash + curl + jq (no node, no python)
  so it has zero install footprint beyond standard CLI tools.
- All HTTP calls are best-effort with a 3s timeout. Daemon down = silent
  noop (no error shown to operator).
- The adapter is read/write against the canonical ring buffer in the
  Focusa daemon. Other agents (Pi, Aider, Cursor, Cline, Gemini, Codex)
  share the same ring via the same routes.
- For Windows hosts, the `.ps1` equivalent is TODO (the script is
  POSIX-bash only today).
