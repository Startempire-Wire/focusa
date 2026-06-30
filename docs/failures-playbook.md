# Failures playbook

This document indexes **every CI, runtime, release, or deploy failure** that has
been observed during the Focusa release/deploy automation work.

Source of truth ledger:

- `release-proof/audit/audit.jsonl` — append-only machine-readable events
- `release-proof/audit/categories.md` — failure category playbook with fix
  and guard links

How to use this playbook:

1. Capture the failure in `release-proof/audit/audit.jsonl` with:

   ```json
   {"id":"fail-YYYY-MM-DD-<short>","ts":"...","event":"failure","subsystem":"...","scope":"...","category":"...","symptom":"...","root_cause":"...","fix":"...","guard":"...","test":"...","linked_run":"..."}
   ```

2. Map the failure to a category in `categories.md`. If the category is new,
   add it with a fix/guard/test triplet so the next agent can resolve it
   without re-debugging.

3. Every `addition` event (new code in `scripts/install-daemon.sh`,
   `scripts/safe-disk-cleanup.sh`, `scripts/install-self-hosted-runner.sh`,
   `.github/workflows/deploy-live-daemon.yml`) is paired with the categories
   its guards mitigate.

## Lessons (do-not-repeat)

- **Static / regex brittleness**: never use `rg -q '^pattern$'` against
  workflow files; use `grep -Fq 'literal'`. Replaced all such checks in
  `tests/release_deploy_automation_static_test.sh`.
- **Version drift**: every tag push must run
  `stamp-menubar-version.py` then `verify-version-surfaces.py`. The release
  workflow enforces this and the static proof references it.
- **GH-hosted transport risk**: do not rely on inbound SSH from GH-hosted
  runners; deploys must run on a self-hosted runner registered to the
  VPS with label `focusa-deploy`.
- **Privileged scripts**: only the deploy/cleanup scripts get a NOPASSWD
  sudoers rule; the runner user is otherwise unprivileged.
- **Disk pressure**: the deploy preflight runs the safe cleanup with a
  threshold; failure causes the deploy to abort instead of silently running
  the daemon on a starved root filesystem.

## Self-healing hooks (planned)

- workflow step: parse recent `audit.jsonl`, fail closed if a guard failure
  is older than expected
- agent loop: on `category=missing_ci_gate_passing`, suggest
  `gh run rerun` for the matching CI run before retrying deploy
- agent loop: on `category=brittle_regex_match`, refuse to add new `rg -q`
  checks without `grep -Fq` evidence

## Redaction

- do not write credential values into `audit.jsonl`
- do not include bearer tokens, SSH key contents, or release-signed URLs
- redact home paths, SSH config, or hostnames to opaque forms

## Operating principle

> Every failure must produce one new audit row, one new category fix,
> and one regression guard. No silent fixes.
