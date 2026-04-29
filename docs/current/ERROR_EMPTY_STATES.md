# Error and Empty-State Envelopes

**Spec:** [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](../92-agent-first-polish-hooks-efficiency-spec.md)

Focusa doctor-style CLI/API failures use recovery-first envelopes.

## Required fields

```json
{
  "status": "blocked",
  "code": "API_CONNECT_ERROR",
  "what_failed": "Could not connect to Focusa API",
  "likely_why": "daemon down or port unavailable",
  "safe_recovery": "focusa start || systemctl restart focusa-daemon",
  "command": "focusa start || systemctl restart focusa-daemon",
  "fallback": "focusa doctor",
  "docs": ["docs/current/ERROR_EMPTY_STATES.md"],
  "evidence_refs": [],
  "severity": "blocked",
  "details": {}
}
```

## CLI behavior

Use JSON mode to get machine-readable failure envelopes:

```bash
focusa --json doctor
focusa --json status --agent
focusa --json cleanup --safe --dry-run
```

If the daemon is down, recovery command:

```bash
focusa start || systemctl restart focusa-daemon
```

## API behavior

Non-JSON HTTP errors are wrapped by API middleware with:

- `what_failed`
- `likely_why`
- `safe_recovery`
- `command`
- `fallback`
- `docs`
- `evidence_refs`
- `severity`
- `correlation_id`

## Empty states

### No active Workpoint

```text
Status: watch
Summary: No active Workpoint is currently promoted.
Next action: Create a checkpoint before compaction or risky work.
Command: focusa_workpoint_checkpoint mission="..." next_action="..."
Recovery: focusa_workpoint_resume if a previous packet exists.
```

### Daemon holdover

```text
Status: degraded
Summary: Focusa daemon unavailable; Pi holdover active and kickstart in progress.
Next action: Continue bounded local work while daemon restarts.
Command: systemctl status focusa-daemon --no-pager
Recovery: systemctl restart focusa-daemon
```

See also [`DAEMON_RESILIENCE.md`](DAEMON_RESILIENCE.md).
