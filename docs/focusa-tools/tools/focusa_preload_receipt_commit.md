# focusa_preload_receipt_commit

Commit an idempotent bootstrap-delivery receipt for a rendered preload packet.

## CLI

```sh
focusa preload receipt-commit --profile rules_and_context --idempotency-key example-key
```

## API

`POST /v1/preload/receipt-commit`

## Arguments

- `profile` (optional)
- `idempotency_key` (required, non-empty)

## Output

A canonical `tool_result_v1` envelope containing the committed receipt or an idempotent replay.

## Safety

Writes only the per-user receipt ledger. Failures use `FOCUSA_PRELOAD_FAIL`.

## Evidence

Spec 111 §§9–11.
