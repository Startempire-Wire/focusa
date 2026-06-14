# Public Stream Redaction Policy

Focusa public stream/card output is **deny-by-default**. A card can be displayed publicly only when it declares the required public-card fields and `publish_allowed=true` after redaction review. Current runtime cards default to `publish_allowed=false`.

## Required public card fields

Every public card declares:

- `schema`
- `project_identity_display_name`
- `redacted_scope_id`
- `canonical_status` (`canonical`, `advisory`, or `degraded`)
- `tool_family`
- `evidence_refs_public_safe`
- `redaction_status`
- `secret_scan_status`
- `publish_allowed`

## Redaction rules

Never publish by default:

- raw logs
- secrets
- tokens
- private file contents
- unredacted project paths if sensitive
- raw diffs unless explicitly allowed
- browser diagnostics with sensitive URLs
- environment contracts with host secrets

## Current implementation

`/v1/awareness/card` includes a `public_stream_policy` object and renders a `PUBLIC_CARD` block with required fields. The scope is represented by `redacted_scope_id`, not raw authority state. Evidence refs are empty unless a future caller marks them public-safe.

## Publish gates

A future public stream publisher must verify:

1. required fields present
2. `publish_allowed=true`
3. `redaction_status` is not `unredacted`
4. `secret_scan_status` is `passed` or explicitly `not_required_no_raw_payload`
5. no raw token/secret/path/diff/browser diagnostic payload is present
6. evidence refs are explicitly public-safe

## Proof

- Static guard: `tests/public_stream_redaction_policy_static_test.sh`
- Live-safe guard: `tests/public_stream_redaction_policy_live_safe_test.sh`
- API surface: `crates/focusa-api/src/routes/awareness.rs`
- Spec source: `docs/106-focusa-vision-tightening-spec.md`
