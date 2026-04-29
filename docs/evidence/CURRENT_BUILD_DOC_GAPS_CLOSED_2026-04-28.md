# Current Build Documentation Gaps Closed — 2026-04-28

## Scope

Operator asked to close documentation gaps and surgically update README, adding only what is current in the present build.

## Added current-build docs

- `CHANGELOG.md`
- `docs/current/CURRENT_RUNTIME_STATUS.md`
- `docs/current/API_REFERENCE_CURRENT.md`
- `docs/current/CLI_REFERENCE_CURRENT.md`
- `docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md`
- `docs/current/WORKPOINT_LIFECYCLE_GUIDE.md`
- `docs/current/TOOL_RESULT_ENVELOPE_V1.md`
- `docs/current/TROUBLESHOOTING_CURRENT.md`
- `docs/current/VALIDATION_AND_RELEASE_PROOF.md`

## README/doc index updates

- Root `README.md` now links the current-build reference docs.
- `docs/README.md` now links the current-build reference docs.

## Current-build basis

- Current repo head before doc generation: `093b7f8` / tag `v0.9.2-dev`.
- API route inventory generated from `crates/focusa-api/src/routes/*.rs` route registrations.
- CLI reference generated from `cargo run --quiet --bin focusa -- --help`.
- Tool and skills references use current `apps/pi-extension/src/tools.ts` and current skill directories.

## Validation

Commands used:

```bash
cargo run --quiet --bin focusa -- --help
python3 route inventory extraction from crates/focusa-api/src/routes/*.rs
```

Docs intentionally avoid roadmap-only additions and describe current present-build surfaces only.
