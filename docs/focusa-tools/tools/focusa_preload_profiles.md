# focusa_preload_profiles

List available agent bootstrap profiles (rules_only, rules_and_context, budget_light, budget_deep).

## CLI

```
focusa preload profiles
```

## API

```
GET /v1/preload/profiles
```

## Arguments

[]

## Output

profile metadata array + default_profile

## Evidence

- Spec 111 §9 (CLI surfaces)
- Spec 111 §11 (tool contracts)
- Spec 111 §19.4 (tool contract static test)

## Notes

- All routes are read-only by default; only the `write` subcommand persists data.
- `idempotency_key` is required for any write action and must be non-empty.
- Target paths must use allowlisted prefixes (`/tmp/focusa-preload/`, `/var/cache/focusa/preload/`).
- All failures expose `failure_class` and return the `FOCUSA_PRELOAD_FAIL` error code.
