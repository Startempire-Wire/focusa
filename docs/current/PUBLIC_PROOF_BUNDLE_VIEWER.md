# Public Proof Bundle Viewer

The public proof bundle viewer is a redaction-gated presentation layer for Focusa proof artifacts. It must show enough evidence to demonstrate the Golden Workflow without exposing private project state.

## Viewer inputs

- release proof bundle from `focusa release prove --tag <tag>`
- Workpoint resume/checkpoint packet summary
- evidence refs marked public-safe
- tool contract summary
- current runtime status
- public card fields from `/v1/awareness/card`
- redaction review status

## Required viewer fields

Every public proof bundle view declares:

- schema
- project identity display name
- redacted scope id
- canonical/advisory/degraded status
- proof bundle version/tag
- tool family or workflow family
- evidence refs if public-safe
- redaction status
- secret scan status
- publish_allowed

## Viewer states

- `draft_private` — local/operator preview only.
- `redaction_pending` — required fields present but redaction/secret scan incomplete.
- `publish_blocked` — failed redaction, missing public-safe evidence, or `publish_allowed=false`.
- `publish_ready` — redaction passed and `publish_allowed=true`.
- `published_snapshot` — immutable/public artifact with source proof refs.

## Menubar/operator preview

`apps/menubar/src/lib/components/ProofPeek.svelte` is the local proof preview surface. It may display Workpoint evidence, status, and side effects for the operator, but it is not a public publisher and must not bypass `PUBLIC_STREAM_REDACTION_POLICY.md`.

## Public safety gates

- Deny by default.
- Never show raw logs, tokens, raw diffs, private file contents, or sensitive browser diagnostics.
- Prefer evidence refs and summaries over payload blobs.
- Public evidence must be explicitly marked public-safe.
- Publish only when `secret_scan_status=passed` or `not_required_no_raw_payload` and `publish_allowed=true`.

## Proof

- Static guard: `tests/public_proof_bundle_viewer_static_test.sh`
- Related: `PUBLIC_STREAM_REDACTION_POLICY.md`, `GOLDEN_WORKFLOW_PUBLIC_DEMO.md`, `VALIDATION_AND_RELEASE_PROOF.md`, `CURRENT_RUNTIME_STATUS.md`
