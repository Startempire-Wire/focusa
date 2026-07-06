# Spec 111 — Agent Context Bootstrap and Delivery

Purpose: lock all core Focusa bootstrap surfaces so a new agent receives a bounded, scope-verified, evidence-aware, advisory-only bootstrap packet before changing files.

## Schemas
- `focusa.preload.v1` (envelope schema)
- `AgentBootstrapPacket` (typed packet)
- `AgentBootstrapProfile` (id, label, description, includes_dynamic_context, includes_acceptance_prompt, max_dynamic_items)
- `AgentBootstrapReceipt` (preview + commit receipt, kind=`bootstrap_delivery`)
- `FOCUSA_PRELOAD_FAIL` (failure code)

## Profile ids
- `rules_only`
- `rules_and_context`
- `budget_light`
- `budget_deep`

Default profile: `rules_and_context`.

## Surfaces
- API: `/v1/preload/{profiles,build,render,verify,doctor,receipt-preview,receipt-commit,write}`
- CLI: `focusa preload {profiles|build|render|verify|doctor|write|receipt-preview}`
- Pi/tool contracts: `focusa_preload_profiles`, `focusa_preload_build`, `focusa_preload_write`, `focusa_preload_receipt_preview`
- Reference docs: `docs/focusa-tools/tools/focusa_preload_*.md`

## Safe write rules
- Target paths must use allowlisted prefixes: `/tmp/focusa-preload/`, `/var/cache/focusa/preload/`.
- `idempotency_key` is required for every write.
- Existing targets require `overwrite=true` to replace.
- Unknown profiles return `FOCUSA_PRELOAD_FAIL`.

## Slices
- Slice 1: schema + static contracts
- Slice 2: core packet types and renderers
- Slice 3: read-mostly API routes dispatch
- Slice 4: safe write route
- Slice 5: receipt preview integration
- Slice 6: CLI subcommands
- Slice 7: Pi/tool contracts
- Slice 8: docs + snapshots + acceptance
