# Non-Pi Agent Focusa Usage

Focusa awareness must include OpenClaw/Wirebot, Claude Code, OpenCode, Letta, and any other agent harness that can read prompts, call HTTP APIs, or use CLI wrappers.

## Scope

This document covers agents that do not run through the Pi extension and therefore do not automatically receive the Pi Focusa Utility Card.

Included explicitly:

- **OpenClaw / oprnclaw gateway** — Wirebot's primary chat/agent runtime.
- **Wirebot agent** — sovereign/business partner agent surfaces backed by OpenClaw, memory, wiki, scoreboard, and workspace context.
- Claude Code.
- OpenCode.
- Letta.
- Other CLI/HTTP-compatible harnesses.

## Requirement

Every non-Pi agent entrypoint must receive a compact Focusa Utility Card or equivalent startup instruction before reasoning when Focusa is available.

Minimum card content:

```text
Focusa is the local cognitive continuity/governance runtime.
Use it for Workpoints, Focus State, evidence refs, predictions, metacognition, lineage, work-loop state, and recovery.
If uncertain/degraded: run focusa doctor or call /v1/doctor.
Before compaction/model switch/fork/risky continuation: checkpoint a Workpoint.
After resume/reload: fetch Workpoint resume packet and follow it unless operator steers otherwise.
After proof/tests/API/file result: capture/link evidence.
Before risky/uncertain next action: record prediction; after outcome: evaluate it.
Operator steering wins.
```

## Awareness card endpoint

```bash
curl -sS "http://127.0.0.1:8787/v1/awareness/card?adapter_id=openclaw&workspace_id=wirebot&agent_id=wirebot&operator_id=verious.smith" | jq -r .rendered_card
focusa awareness card --adapter-id openclaw --workspace-id wirebot --agent-id wirebot --operator-id verious.smith
```

## OpenClaw / Wirebot integration modes

### Mode A — Proxy-level injection

Route OpenClaw model calls through Focusa proxy and inject the Focusa Utility Card server-side.

Desired flow:

```text
Wirebot/OpenClaw turn
  -> Focusa session/context lookup
  -> Focusa Utility Card + current Workpoint/Focus slice
  -> model call
  -> result/event/evidence hooks
```

Candidate existing target from organism specs:

```text
OpenClaw -> Focusa :8787/proxy/v1/chat/completions -> model backend
```

### Mode B — Gateway plugin injection

If proxy routing is not active, the OpenClaw gateway/plugin layer should prepend the Focusa Utility Card and current Workpoint summary to the system/developer prompt.

Required plugin behavior:

1. Health check Focusa.
2. Fetch `/v1/workpoint/resume` with OpenClaw/Wirebot session/workspace scope.
3. Fetch `/v1/doctor` or `/v1/focus/frame/current` when needed.
4. Insert compact utility card and canonical Workpoint packet.
5. On proof/test/release events, call evidence endpoints.
6. On risky decisions, record/evaluate predictions.

### Mode C — CLI wrapper fallback

For agents that cannot call HTTP directly, wrap with CLI commands:

```bash
focusa doctor --json
focusa workpoint resume --json
focusa predict record ... --json
focusa predict evaluate ... --json
```

## Wirebot-specific requirements

Wirebot must treat Focusa as the cognition/governance layer, not as the hidden durable truth source.

- Durable personal/business knowledge still promotes to Wiki/Mem0/Letta/workspace where appropriate.
- Focusa stores current bounded focus, Workpoints, evidence refs, predictions, and recovery state.
- OpenClaw/Wirebot fallback when Focusa is down must clearly mark `cognition_degraded=true` and continue only with direct model + Wiki/Mem0/Letta context.
- Every Wirebot turn should be attributable to a Focusa session/workspace id when Focusa is available.

Suggested scope identifiers:

```json
{
  "adapter_id": "openclaw",
  "workspace_id": "wirebot",
  "agent_id": "wirebot",
  "operator_id": "verious.smith"
}
```

## Success criteria

- OpenClaw/Wirebot startup/reload shows or injects a Focusa Utility Card equivalent.
- Wirebot turns can fetch a scoped Workpoint resume packet.
- Cross-session/project packets are rejected or demoted, not blindly injected.
- Evidence from Wirebot actions can be linked to the active Workpoint.
- Predictions can be recorded/evaluated for risky Wirebot choices.
- If Focusa is unavailable, Wirebot reports degraded cognition rather than silently pretending continuity is intact.

## Validation target

Future guard:

```bash
node scripts/validate-non-pi-agent-awareness.mjs
```

The guard should verify that public docs and integration snippets mention OpenClaw/Wirebot, Workpoint resume/checkpoint, doctor, evidence, prediction, degraded fallback, and operator steering.


## OpenClaw plugin implementation

Focusa ships a local OpenClaw plugin skeleton for gateway injection:

```text
apps/focusa-awareness/
  openclaw.plugin.json
  index.ts
```

The plugin listens on `before_agent_start`, fetches `/v1/awareness/card`, and returns `{ prependContext: rendered_card }`. If Focusa is unavailable, it injects a degraded fallback card with `cognition_degraded=true`.

Validation:

```bash
node scripts/validate-openclaw-focusa-awareness-plugin.mjs
node scripts/validate-openclaw-focusa-awareness-config.mjs
node scripts/prove-non-pi-agent-awareness-live.mjs
```

Configured production path:

```text
/data/wirebot/users/verious/openclaw.json
plugins.allow += focusa-awareness
plugins.load.paths += /home/wirebot/focusa/apps/focusa-awareness
plugins.entries.focusa-awareness.enabled = true
```

Activation requires an OpenClaw gateway restart/reload. Because that restarts the live Wirebot gateway, perform it in an operator-approved window:

```bash
systemctl restart openclaw-gateway
```


## Systemd startup note

The production `openclaw-gateway.service` must not require `/run/wirebot/gateway.env` before `ExecStartPre=/data/wirebot/bin/inject-gateway-secrets.sh` runs, because `/run` is tmpfs and may be empty after reboot. The unit should use:

```text
EnvironmentFile=-/run/wirebot/gateway.env
```

The injector then writes the real runtime env file before `ExecStart` launches OpenClaw.
