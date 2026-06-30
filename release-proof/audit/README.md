# Focusa release/deploy audit trail

This directory is the **single source of truth** for additions, failures,
guard relationships, and remediation captured during Focusa release/deploy
automation.

Files:

- `audit.jsonl` — append-only events: `addition` and `failure`
- `categories.md` — failure category playbook with category-level fix and guard

How to record a new event:

```json
{
  "id":"add-YYYY-MM-DD-<short>",
  "ts":"YYYY-MM-DDTHH:MM:SSZ",
  "event":"addition" or "failure",
  "subsystem":"deploy|runner|ci|release|ops|git",
  "scope":"<path>",
  "category":"<see categories.md or new>",
  "symptom":"<observed error>",
  "root_cause":"<why it happened>",
  "fix":"<what was changed>",
  "guard":"<how we now catch/prevent it>",
  "test":"<regression guard>",
  "linked_run":"<GH run id or manual>"
}
```

Required:

- id format: `add-YYYY-MM-DD-<short>` or `fail-YYYY-MM-DD-<short>`
- always include category, root_cause, fix, guard, test, linked_run
- redact credentials, host keys, tokens, hostnames beyond what is necessary

Categories so far:

- brittle_regex_match
- stale_version_surface
- missing_ci_gate_passing
- infrastructure_blocked
- disk_pressure
- permission_denied
- hostname_assumption
- policy_violation

When a new failure doesn't fit, add a category to `categories.md` before
adding the audit row.
