# Spec88 Live Operational Evidence — 2026-04-28

## Operator Standard

Source-level implementation is insufficient. Spec88 counts as ready only when Focusa daemon, CLI, and Pi bridge behavior are deployed or discoverable and smoke-tested.

## Deployment

- Built release binaries from current tree:
  - `target/release/focusa-daemon`
  - `target/release/focusa`
- Restarted `focusa-daemon.service`.
- Verified service active on `127.0.0.1:8787`.

## Live API Smoke

Used `POST /v1/workpoint/checkpoint` with idempotency key `spec88-live-smoke-1777403610`.

Results:

- First checkpoint accepted:
  - `status=accepted`
  - `canonical=true`
  - `idempotent_replay=false`
  - `workpoint_id=019dd582-c3d3-7492-b51c-411706e41691`
- Immediate duplicate checkpoint returned existing record:
  - `status=completed`
  - `canonical=true`
  - `idempotent_replay=true`
  - same `workpoint_id=019dd582-c3d3-7492-b51c-411706e41691`
- `GET /v1/workpoint/current` returned active canonical Workpoint.
- `POST /v1/workpoint/resume` returned a canonical `resume_packet` and rendered summary.

## Live CLI Smoke

- `target/release/focusa workpoint --help` lists:
  - `checkpoint`
  - `current`
  - `resume`
  - `drift-check`
- `target/release/focusa --help` lists `workpoint` as a top-level command group.
- `target/release/focusa workpoint current` returned the active canonical Workpoint.
- `target/release/focusa workpoint resume` returned the active WorkpointResumePacket summary.

## Pi Extension Discoverability

Source and installed skill expose the first-class Pi tools:

- `apps/pi-extension/src/tools.ts`
  - `focusa_workpoint_checkpoint`
  - `focusa_workpoint_resume`
- `apps/pi-extension/skills/focusa/SKILL.md`
- `/root/.pi/skills/focusa/SKILL.md`

The Pi extension package loads TypeScript directly from `apps/pi-extension/src/index.ts`, so updated source is the runtime load target for new/reloaded Pi sessions.

## Additional Hardening From Live Failure

Live compaction #70 spam showed that source tests alone were not enough. The compaction auto-resume path now persists duplicate suppression across extension reload/session state:

- `S.lastCompactResumeKey`
- `S.lastCompactResumeAt`
- persisted through `focusa-state`
- restored on session start/switch
- duplicate same-cycle auto-resume messages suppressed with a UI notification

## Validation

- `./tests/spec88_workpoint_golden_eval_contract_test.sh` — 21/0
- `cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit`
- `cargo test -p focusa-api drift_classifier --target-dir /tmp/focusa-cargo-target`
- `cargo test -p focusa-api workpoint_packet --target-dir /tmp/focusa-cargo-target`
- `cargo check -p focusa-api -p focusa-cli -p focusa-core --target-dir /tmp/focusa-cargo-target`
- release build: `cargo build --release -p focusa-api -p focusa-cli --target-dir /home/wirebot/focusa/target`

## Caveat

A currently running Pi process may need a session/extension reload to pick up the changed TypeScript module. New/reloaded Pi sessions should load the hardened duplicate-suppression logic.
