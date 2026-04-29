# Daemon Resilience and In-Session Kickstart

Focusa is configured for two layers of daemon resilience:

1. **systemd hardening** keeps the production daemon restarting outside any agent session.
2. **Pi extension holdover/kickstart** preserves the active Pi session if the daemon is briefly unavailable.

## Live systemd hardening

Production override:

```text
/etc/systemd/system/focusa-daemon.service.d/reliability.conf
```

Current policy:

```ini
[Unit]
StartLimitIntervalSec=0

[Service]
Restart=always
RestartSec=1
TimeoutStartSec=20
TimeoutStopSec=20
```

Verify:

```bash
systemctl show focusa-daemon -p Restart -p RestartUSec -p StartLimitIntervalUSec --no-pager
systemctl is-active focusa-daemon
curl -fsS http://127.0.0.1:8787/v1/health | jq .
```

## Pi session holdover

When health checks fail during a Pi session, the Focusa extension now:

- enters `Focusa holdover` status,
- keeps tools available instead of disabling them,
- uses local shadow/holdover state until daemon returns,
- runs the configured kickstart command,
- probes health rapidly,
- reconnects SSE and reconciles local state after recovery,
- does not require restarting the Pi session.

Default kickstart command:

```bash
systemctl start focusa-daemon || systemctl restart focusa-daemon
```

Relevant config keys:

```json
{
  "daemonAutoRestart": true,
  "daemonRestartCommand": "systemctl start focusa-daemon || systemctl restart focusa-daemon",
  "daemonRestartCooldownMs": 5000,
  "daemonRestartMaxPerHour": 20,
  "daemonRecoveryProbeMs": 750
}
```

## Recovery commands

```bash
systemctl status focusa-daemon --no-pager
journalctl -u focusa-daemon -n 80 --no-pager
systemctl restart focusa-daemon
curl -fsS http://127.0.0.1:8787/v1/health | jq .
```


## Live proof

See [`docs/evidence/DAEMON_RESILIENCE_LIVE_PROOF_2026-04-28.md`](../evidence/DAEMON_RESILIENCE_LIVE_PROOF_2026-04-28.md).
