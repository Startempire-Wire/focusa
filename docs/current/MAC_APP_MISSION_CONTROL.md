# Mac App Mission Control

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

The Mac menubar app includes a **Mission** tab backed by live Focusa APIs.

## Cards

- Daemon health — `/v1/health`
- Workpoint — `/v1/workpoint/current`
- Work-loop — `/v1/work-loop/status`
- Tool contracts — `/v1/ontology/tool-contracts`
- Token budget — `/v1/telemetry/token-budget/status?limit=5`
- Cache metadata — `/v1/telemetry/cache-metadata/status?limit=5`
- Release proof command — `focusa release prove --tag <tag>`
- Recovery command — `systemctl restart focusa-daemon`

## Validation

```bash
cd /home/wirebot/focusa/apps/menubar
bun install
bun run check
bun run build
```

## Empty/offline behavior

If the daemon is unavailable, the Mission tab shows recovery state from the runtime store and surfaces the restart command documented in [`DAEMON_RESILIENCE.md`](DAEMON_RESILIENCE.md).
