# Spec 117 Deck API Routes proof — 2026-07-06

Scope: focusa-117-arch.16 /v1/deck/* read-first API routes.

## Routes added
- /v1/deck/home
- /v1/deck/walkthroughs
- /v1/deck/recall/schema
- /v1/deck/proof-meter
- /v1/deck/next-safe-action

## Tests/gates
- cargo test --release -p focusa-api -- routes::deck: PASS (1 test)
- cargo build --release -p focusa-api: PASS
- tests/spec_focusa_117_deck_api_routes_static_test.sh: PASS
- tests/release_deploy_automation_static_test.sh: PASS
