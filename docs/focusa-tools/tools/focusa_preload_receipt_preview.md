# focusa_preload_receipt_preview

Render the bootstrap_delivery Focusa Receipt preview for a given profile.

## CLI

```
focusa preload receipt-preview --profile rules_and_context
```

## API

```
POST /v1/preload/receipt-preview
```

## Arguments

[
  "profile?"
]

## Output

receipt_kind=bootstrap_delivery + rendered packet

## Evidence

- Spec 111 §9 (CLI surfaces)
- Spec 111 §11 (tool contracts)
- Spec 111 §19.4 (tool contract static test)

## Notes

- All routes are read-only by default; only the `write` subcommand persists data.
- `idempotency_key` is required for any write action and must be non-empty.
- Target paths must use allowlisted prefixes (`/tmp/focusa-preload/`, `/var/cache/focusa/preload/`).
- All failures return `FOCUSA_PRELOAD_FAIL` error code.
