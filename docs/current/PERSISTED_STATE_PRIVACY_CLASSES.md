# Persisted State Privacy Classes

Status: current privacy classification for local Focusa stores. Focusa is local-first, but persisted cognitive/workflow data can still contain sensitive project, operator, or business context.

## Privacy classes

| Class | Meaning | Examples | Handling |
| --- | --- | --- | --- |
| P0 Public | Safe to publish | Public docs/spec names, release status without project secrets | May appear in docs/evidence. |
| P1 Operational | Local workflow metadata | tool names, route names, status, bounded counts, non-secret file paths | Store locally; OK in bounded reports. |
| P2 Project-sensitive | Private project context | Workpoint missions, trajectory gaps, bead IDs, evidence refs, changed-file names | Store locally; avoid public logs unless summarized. |
| P3 Confidential | Secrets-adjacent or internal content | provider prompts, raw tool output, private customer/site details, auth URLs, DB paths | Prefer handles/refs and redaction; never publish raw. |
| P4 Secret | Credentials/key material | API keys, bearer tokens, private keys, passwords, TOTP seeds | Never persist in Focusa stores; use env/secret vault only. |

## Store classification

| Store/surface | Default class | Notes |
| --- | --- | --- |
| Focus State | P2 | Operator-curated; constraints/failures/results may mention private project facts. |
| Scratchpad | P2/P3 | Working notes can be verbose; avoid secrets and raw provider payloads. |
| Workpoint | P2 | Canonical continuation; should use target refs/evidence handles, not raw logs. |
| Trajectory | P2 | Project goal/gap state; may be business-sensitive. |
| Predictions | P2 | Forecasts, context refs, trajectory/ontology context; no provider raw payloads. |
| Metacognition | P2/P3 | Reusable lessons can encode project/process details; evidence refs preferred. |
| Evidence refs | P1/P2 | Refs are handles; linked artifacts may be P3 and require separate protection. |
| Telemetry/events | P1/P2 | Counts/status low risk; event payloads may include summaries. |
| ECS/reference store | P2/P3 | Artifact offloading surface; use handles and retrieval budgets. |
| Peer sync tokens | P4 | `auth_token` fields are credentials; peer registration rejects token persistence until encrypted secret storage exists. |
| Communications connector checkpoint | P4 | Only authenticated ciphertext generations may persist in the owner-only system state root; plaintext browser/connector state is tmpfs-only and never enters Focusa SQLite/evidence/model context. |
| Communications grant usage/audit/idempotency | P1/P2 | Value-free handles, counts, digests, status, and attribution only; no message/OTP/credential/pairing payloads. |

## Persistence rules

1. Store handles/evidence refs instead of raw provider payloads or raw logs.
2. P4 secrets are not valid Focusa memory; reject, redact, or keep only in approved secret storage.
3. Public docs/evidence must not include raw tokens, private keys, or bearer credentials.
4. Bounded model-visible slices should prefer P1/P2 summaries and omit P3/P4 payloads.
5. Local SQLite/event stores are private local data; backup/retention should follow operator policy.

## Redaction expectations

Secret-like strings matching API key, bearer token, private key, password, or token assignment patterns must be redacted before public docs or model-visible summaries. Env-var names such as `FOCUSA_AUTH_TOKEN` and `MINIMAX_API_KEY` are allowed as configuration names, not values.

## Current proof

- Redacted source scan evidence: `/tmp/focusa-secret-scan-redacted.json`.
- Static privacy gate: `tests/security_persisted_state_privacy_static_test.sh`.
- Security review: `docs/current/FOCUSA_SECURITY_REVIEW_2026-05-26.md`.
