# Spec 117 walkthrough/deck proof — 2026-07-05

Scope: focusa-117-arch.6 walkthrough-schema, focusa-117-arch.7 first-mission-walkthrough, focusa-117-arch.15 deck-cli-alias.

## Live CLI proof
```
== list ==
{"catalog":["first-mission"],"schema":"focusa.walkthrough.v1"}
== show ==
{"schema_version":"focusa.walkthrough.v1","id":"first-mission","title":"First Mission","audience":"beginner","step_count":5,"first_step":"start-daemon"}
== events ==
started walkthrough first-mission step=start-daemon
advance first-mission step=start-daemon
{"schema":"focusa.walkthrough.v1","walkthrough_id":"first-mission","progress":{"start-daemon":"advanced"}}
== deck web alias ==
```

## Static gates
```
=== release deploy automation static test ===
✓ PASS: onboard exposes low-risk --remote marker creation path
✓ PASS: remote marker onboarding avoids hardcoded project roots
✓ PASS: remote marker schema fields are statically present
GH5/remote marker static test: PASS
✓ PASS: focusa CLI exposes TUI subcommand and headless self-test
✓ PASS: focusa-tui binary supports --headless-self-test and snapshot JSON
✓ PASS: headless snapshot payload schema fields present
daemon health reachable: {"ok":true,"status":"ok","uptime_ms":4121850,"version":"0.9.64-dev"}
focusa-yixp TUI usage test: PASS
✓ PASS: Pi startup nag has marker check, suppress flag, commands, and telemetry
✓ PASS: Pi startup nag suppresses when marker present or already emitted
GH7/Pi unbound nag static test: PASS
✓ .beads owner=wirebot
✓ .beads/issues.jsonl owner=wirebot
✓ .git/beads-worktrees/beads-sync/.beads/issues.jsonl owner=wirebot
✓ bd daemon pid=1957576 owner=wirebot
BD sync ownership policy: PASS
Self-heal classifier fixtures: PASS (9 fixtures)
Release deploy automation static test: PASS
```

## Build/tests
- cargo build --release -p focusa-cli: PASS
- cargo test --release -p focusa-cli -- commands::walkthrough: PASS (3 passed)
