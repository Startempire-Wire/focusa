# `focusa_epistemic_operation`

Invoke one of the 27 canonical Spec 138/138A operations from the generated operation contract.

- **Authority:** the daemon's durable, typed `ScopedAuthorityEvent` ledger; the Pi/CLI/UI client never decides or settles authority locally.
- **Scope:** exact `project_root + continuity_id`.
- **Reads:** send typed scope query fields to the descriptor's exact `GET` path.
- **Mutations:** send the exact `operation_id`, typed scope, and a `ScopedAuthorityEvent` to the descriptor's exact `POST` path. The daemon rejects mismatched operation IDs, scopes, and event kinds.
- **Path IDs:** provide `id` when the generated path contains `{id}`.

The machine-readable source is `docs/contracts/spec138-generated-operation-contracts.v1.json`. Regenerate all client tables with:

```bash
python3 scripts/generate-spec138-operation-clients.py --write
```

CLI equivalent:

```bash
focusa predict operation \
  --operation prediction.question.create \
  --continuity-id <continuity> \
  --event-json '<ScopedAuthorityEvent JSON>' \
  --json
```
