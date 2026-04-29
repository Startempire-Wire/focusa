# Daemon Resilience Live Proof — 2026-04-28

## Scope

Proof for daemon restart hardening and Pi in-session holdover/kickstart support.

## Live systemd override

Path:

```text
/etc/systemd/system/focusa-daemon.service.d/reliability.conf
```

Effective properties after `systemctl daemon-reload`:

```text
Restart=always
RestartUSec=1s
TimeoutStartUSec=20s
TimeoutStopUSec=20s
StartLimitIntervalUSec=0
```

## Restart proof

Command:

```bash
systemctl kill -s SIGTERM focusa-daemon
```

Observed recovery:

```text
before_pid=2424538
probe=1 active=active pid=2424538 health=
probe=2 active=activating pid=0 health=
probe=3 active=active pid=2470049 health=
probe=4 active=active pid=2470049 health=
probe=5 active=active pid=2470049 health=true
```

Result: daemon restarted under systemd with a new PID and `/v1/health` returned `ok=true` without requiring a Pi session restart.

## Code support

Pi extension support added:

- daemon auto-restart config in `apps/pi-extension/src/config.ts`
- `kickstartFocusaDaemon()` in `apps/pi-extension/src/state.ts`
- holdover/kickstart/reconcile health loop in `apps/pi-extension/src/session.ts`

## Safety

- Pi tools are not disabled during daemon holdover.
- Local shadow state remains available while daemon is recovering.
- Restart attempts are rate-limited by cooldown and max-per-hour config.
- Reconnect path reconciles state and restores SSE.
