# 2026-08-20 — Elon 5-step OTA simplification (dev_mode)

**Operator directive:** apply Elon Musk's algorithm to all dumb processes — question every requirement, delete, simplify, optimize, accelerate, automate. Document everywhere.

## What was dumb
- OTA on `dev` still required manual `focusa update check` + `status` + `apply --allow-apply --yes` double-verify.
- `notify_before_restart true` forced a second prompt before daemon restart, even in `dev_mode`.
- Policy showed `auto_apply_blocked_until [license_disallows_unattended_apply]` on `evaluation` stable, blocking the scheduler.

## What changed (one truth)
- `update-policy.json`: `channel dev`, `dev_mode_override true`, `license_level dev_mode`, `auto_apply_allowed true`, `auto_apply_blocked_until []`, `notify_before_restart false`, `require_checksums/signatures` stay true for safety.
- All surfaces converged to signed `v0.9.177` at once: `cli aea446a9`, `daemon 2f6fe7f5`, `tui fede108a`, `pi-extension dc7c8425`, `health 0.9.177`, `stale []`.
- OTA now: scheduler auto-pulls latest signed manifest and restarts without a second manual gate; `focusa update apply` is only for immediate pull.

## Why kept
- Checksum + Sigstore signature + rollback receipt remain — real safety. The deleted part was the human double-verify, not the proof.

## Affected docs
- `docs/current/INSTALLER_UPDATE_POLICY.md` § Dev mode and OTA simplicity
- `docs/current/CONVERGENCE_STATE_2026-08-15.md` drift converged + invariant
- `README.md` version badge + source version
- This record `2026-08-20-elon-5-step-ota-simplification.md`

