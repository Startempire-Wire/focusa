# Security Model

Focusa is local-first cognitive runtime software. The default threat posture is local operator control, explicit authority boundaries, and no public publication without redaction review.

## Core controls

- Local-first daemon and data paths by default.
- API auth token support for exposed/non-local deployments.
- Exact Scope/Project, Workstream, continuity and applicable attachment through installed bindings; `project_root + continuity_id` alone are not the complete authority boundary.
- Context Authority preflight before risky mutation.
- Append-only ledgers for auditable records.
- `tool_result_v1` failure envelopes with recovery hints.
- Public stream deny-by-default redaction policy.

## Sensitive data classes

- API tokens and pairing tokens
- private project paths and file contents
- raw logs and raw diffs
- browser diagnostics with sensitive URLs
- environment contracts with host secrets
- Workpoint/evidence refs that point to private artifacts

## Operator responsibilities

- Bind daemon locally unless deliberately exposing it.
- Configure API auth before public/network exposure.
- Revoke lost paired devices.
- Use release proof and security docs before publishing builds.

## Contextual recovery and delivery

Use [the cohesive project journey](../agent/02-focusa-cohesive-project-flow.md). Discovery/health does not grant execution, and a failed reporting or input step does not automatically revoke independently verified authority. Required safety/consent/entitlement/binding dependencies pause their affected actions; supported recovery must be verified before resumption. Active development uses its approved reload/deploy/test path. Git push is project-specific; production release/install and public redaction controls remain.

## Related docs

- `TOKEN_AND_SECRET_HANDLING.md`
- `DEVICE_PAIRING_THREAT_MODEL.md`
- `PUBLIC_STREAM_REDACTION_POLICY.md`
- `LOCAL_FIRST_DATA_MODEL.md`
- `MULTI_AGENT_SCOPE_MODEL.md`
- `CONTEXT_AUTHORITY_CURRENT.md`
