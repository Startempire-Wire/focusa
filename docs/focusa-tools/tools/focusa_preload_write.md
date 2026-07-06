# focusa_preload_write

Build the packet and write to an allowlisted target path with idempotency_key.

## CLI

```
focusa preload write --profile rules_and_context --target /tmp/focusa-preload/packet.md --idempotency-key abc
```

## API

```
GET /v1/preload/write
```

## Arguments

[
  "profile",
  "target",
  "idempotency_key",
  "overwrite?"
]

## Output

write receipt with idempotency_key, target_path, ok=true

## Evidence

- Spec 111 §9 (CLI surfaces)
- Spec 111 §11 (tool contracts)
- Spec 111 §19.4 (tool contract static test)

## Notes

- All routes are read-only by default; only the `write` subcommand persists data.
- `idempotency_key` is required for any write action and must be non-empty.
- Target paths must use allowlisted prefixes (`/tmp/focusa-preload/`, `/var/cache/focusa/preload/`).
- All failures return `FOCUSA_PRELOAD_FAIL` error code.
