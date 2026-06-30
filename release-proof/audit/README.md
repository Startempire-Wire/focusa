# Focusa release/deploy audit trail

This directory is the **single source of truth** for additions, failures,
guard relationships, and remediation captured during Focusa release/deploy
automation.

Files:

- `audit.jsonl` — append-only events: `addition`, `failure`, `self_heal`
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

Auto-heal:

- Run `python3 scripts/auto-heal-audit.py` after any CI, release, or deploy run.
- It scans the latest entries and synthesizes one `self_heal` row per `failure` that lacks one.
- Idempotent: re-running produces no duplicates.
- Triggered automatically by:
  - `Release` workflow (`Run auto-heal-audit` step after version surface verification)
  - `Deploy Live Daemon` workflow (`Auto-heal audit trail` step after temp cleanup)
  - `CI` workflow (`Auto-heal audit trail` step before static proof)

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
