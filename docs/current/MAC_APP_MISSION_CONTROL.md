# Mac App Mission Control

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

The Mac menubar app includes a **Mission** tab backed by live Focusa APIs.

## Mission-centered main panel

The primary Cockpit/Mission view must show the operator what matters first:

- ProjectIdentity
- Continuity ID
- HLT
- MLG
- STG
- Current Workpoint
- Next action
- Evidence count
- Scope status
- Context Authority status
- Daemon/CLI version status
- Pairing status
- Warnings
- Resume/copy button

## Cards

- Daemon health — `/v1/health`
- Workpoint — `/v1/workpoint/current` and `/v1/workpoint/resume`
- Work-loop — `/v1/work-loop/status?summary_only=true`
- Tool contracts — `/v1/ontology/tool-contracts`
- Token budget — `/v1/telemetry/token-budget/status?limit=5`
- Cache metadata — `/v1/telemetry/cache-metadata/status?limit=5`
- Release proof command — `focusa release prove --tag <tag>`
- Recovery command — `systemctl restart focusa-daemon`

## Validation

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/menubar
bun install
bun run check
bun run build
```

## Empty/offline behavior

If the daemon is unavailable, the Mission tab shows recovery state from the runtime store and surfaces the restart command documented in [`DAEMON_RESILIENCE.md`](DAEMON_RESILIENCE.md).
