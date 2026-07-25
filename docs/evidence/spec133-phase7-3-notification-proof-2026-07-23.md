# Spec 133 Phase 7.3 — deduplicated notifications and waiting-input UX

Date: 2026-07-23
Bead: `focusa-a6yq6.8.3`
Scope: Spec 133 §22.6

## Complete trigger coverage

`evaluate_notifications` covers every required trigger:

1. waiting for operator input;
2. blocker requiring judgment;
3. model mismatch;
4. authentication or entitlement failure;
5. repeated provider failure;
6. resource pressure;
7. checkpoint failure;
8. process failure;
9. orphaned run;
10. completion blocked by missing evidence;
11. verified completion.

Active triggers require exact evidence refs. Waiting-input also requires the durable prompt and emits an exact `focusa silent send <session> --run <run> --text <response>` action.

## Channel-neutral delivery

`NotificationPolicy` fans each active condition across the configured set of `menubar`, `desktop`, `webhook`, and/or `email` channels. Empty channel sets, zero cooldown, zero provider-failure threshold, or invalid session/run scope fail closed.

Each delivery carries trigger, session/run/generation, title, exact why, exact action, evidence ref, and stable condition-specific dedupe key.

## Dedupe and flood prevention

Dedupe keys hash session, run, generation, trigger, and evidence condition. Existing delivery history is read from the normal event chain. An unresolved identical condition is suppressed during the configured cooldown; changed evidence, resolution, or cooldown expiry permits a new notification.

There is no notification ledger. Every request requires:

- `persist_delivery_via_existing_event_chain = true`;
- `persistent_dashboard_visible = true`.

A channel adapter therefore cannot create invisible background activity or bypass the persistent dashboard/event audit path.

## Local non-building proof

Per operator policy, no local Cargo, CI, compilation, or tests were run.

```bash
rustfmt --edition 2024 --check \
  crates/focusa-core/src/lib.rs \
  crates/focusa-core/src/silent_session_notifications.rs
git diff --check
```

Result: passed.

## Required server proof

Run only on the build server:

```bash
cargo test -p focusa-core silent_session_notifications -- --nocapture
cargo test -p focusa-core
cargo clippy -p focusa-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server tests must prove all eleven triggers, multi-channel fan-out, exact waiting-input action, missing-evidence rejection, same-condition suppression, cooldown/resolution behavior, changed-condition re-notification, and mandatory dashboard/event visibility.

## Gate disposition

Implementation and local static review are complete. Build/test closure remains server-owned and must pass before this bead is marked fully proven.
