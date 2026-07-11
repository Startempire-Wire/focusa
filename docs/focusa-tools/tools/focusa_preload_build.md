# focusa_preload_build

Build an AgentBootstrapPacket for a given profile id without writing to disk.

## CLI

```
focusa preload build --profile rules_and_context
```

## API

```
POST /v1/preload/build
```

## Arguments

[
  "profile"
]

## Output

packet JSON with rendered text and bounded dynamic context

## Evidence

- Spec 111 §9 (CLI surfaces)
- Spec 111 §11 (tool contracts)
- Spec 111 §19.4 (tool contract static test)

## Notes

- All routes are read-only by default; only the `write` subcommand persists data.
- `idempotency_key` is required for any write action and must be non-empty.
- Target paths must use allowlisted prefixes (`/tmp/focusa-preload/`, `/var/cache/focusa/preload/`).
- All failures return `FOCUSA_PRELOAD_FAIL` error code.
