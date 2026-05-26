# Tamper-Evident Event Chain

Status: initial STRIDE repudiation/CIS audit-log hardening.

## Implemented behavior

SQLite persistence now writes an `event_hash_chain` row for every appended event:

| Field | Purpose |
| --- | --- |
| `event_id` | Links hash checkpoint to canonical `events.event_id`. |
| `chain_index` | Monotonic append order for the hash chain. |
| `previous_hash` | Prior `event_hash`, or `GENESIS` for the first event. |
| `payload_sha256` | SHA-256 of the serialized persisted event entry. |
| `event_hash` | SHA-256 over `previous_hash`, `event_id`, timestamp, and payload hash. |
| `created_at` | Event timestamp used for checkpoint creation. |

This does not prevent local database tampering by a privileged user, but it makes ordinary row edits/deletions detectable by comparing chain continuity and payload hashes.

## Test proof

- `sqlite_event_hash_chain_links_appended_events` verifies the first row starts at `GENESIS`, the second row points at the first hash, and appended events produce distinct hashes.
- `tests/security_tamper_evident_event_chain_static_test.sh` verifies the schema, hash helpers, append path, and test markers remain present.

## Remaining hardening

1. Add a verification route/CLI command that recomputes all payload hashes and chain links.
2. Periodically export or sign the latest event hash checkpoint outside the SQLite DB.
3. Include hash-chain status in `focusa doctor security`.
4. Add migration/backfill for legacy event rows created before this table existed.
