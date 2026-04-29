# Spec93 Non-Pi Awareness Rollout Proof — 2026-04-29

## Scope

Spec93 makes Focusa awareness available beyond Pi, with explicit OpenClaw/oprnclaw Wirebot support.

## Implemented surfaces

- API: `GET /v1/awareness/card`
- CLI: `focusa awareness card`
- OpenClaw plugin: `apps/focusa-awareness`
- Production OpenClaw config: `/data/wirebot/users/verious/openclaw.json`
- Live proof scripts:
  - `scripts/prove-non-pi-agent-awareness-live.mjs`
  - `scripts/prove-openclaw-focusa-injection-live.mjs`
  - `scripts/validate-docs-runtime-parity.mjs`

## Gateway activation

OpenClaw gateway restarted successfully after correcting the systemd tmpfs env-file boot trap.

Systemd fix:

```text
EnvironmentFile=-/run/wirebot/gateway.env
```

Reason: `/run` is tmpfs and may be empty before `ExecStartPre=/data/wirebot/bin/inject-gateway-secrets.sh` recreates `/run/wirebot/gateway.env`.

Gateway state:

```text
openclaw-gateway: active
focusa-awareness: active url=http://127.0.0.1:8787 workspace=wirebot
```

## Live injection proof

A local OpenClaw agent run triggered the gateway/plugin lifecycle.

Observed log evidence:

```text
[hooks] running before_agent_start (6 handlers, sequential)
focusa-awareness: injected card session=agent:verious:cron:8b4b7bf2-5478-4da7-b932-826cb8aa9d45
```

Proof command:

```bash
node scripts/prove-openclaw-focusa-injection-live.mjs
```

Result:

```text
OpenClaw Focusa injection proof: passed
injections=1
latest=focusa-awareness: injected card session=agent:verious:cron:8b4b7bf2-5478-4da7-b932-826cb8aa9d45
```

## Awareness API proof

```bash
node scripts/prove-non-pi-agent-awareness-live.mjs
```

Result:

```text
Spec93 live awareness proof: passed
surface=focusa_awareness_card adapter=openclaw workspace=wirebot workpoint_canonical=false
```

## Config/plugin proof

```bash
node scripts/validate-openclaw-focusa-awareness-plugin.mjs
node scripts/validate-openclaw-focusa-awareness-config.mjs
```

Results:

```text
OpenClaw Focusa awareness plugin validation: passed
OpenClaw Focusa awareness config validation: passed
```

## Docs/runtime parity proof

```bash
node scripts/validate-docs-runtime-parity.mjs
```

Result:

```text
Docs/runtime parity validation: passed
claims=Spec92/Spec93 awareness, CLI/API refs, OpenClaw plugin, proof scripts
```

## Result

Spec93 is implemented and active: non-Pi agents have a Focusa awareness API/CLI, and OpenClaw/Wirebot now injects a Focusa Utility Card during `before_agent_start`.
