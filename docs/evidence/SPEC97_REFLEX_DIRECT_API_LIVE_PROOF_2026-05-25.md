# SPEC97 Reflex Direct API Live Proof — 2026-05-25

## Scope

Proof for direct read-only Reflex Primitive API after daemon rebuild/restart.

## Runtime activation

- Built release daemon: `cargo build -p focusa-api --release`
- Installed binary: `/usr/local/bin/focusa-daemon`
- Restarted service: `systemctl restart focusa-daemon.service`
- Service: `focusa-daemon.service` active/running

## Live smoke

Command:

```bash
curl -sS --max-time 5 'http://127.0.0.1:8787/v1/reflex/primitives?family=recovery&limit=2' \
  | tee /tmp/focusa-reflex-live.json \
  | jq '{status, read_only, advisory_only, returned:(.items|length), first:.items[0].primitive_id, truncated:.bounds.truncated}'
```

Observed:

```json
{
  "status": "completed",
  "read_only": true,
  "advisory_only": true,
  "returned": 2,
  "first": "route_noncanonical_result",
  "truncated": true
}
```

## Guardrails preserved

- Read-only/advisory metadata only.
- Existing Focusa tools/reducers retain mutation authority.
- Bounded summaries by default.
- Full registry payload requires explicit `include_payload=true` cold opt-in.
- Pi tool reload required before `focusa_reflex_primitives` is available in existing Pi sessions.

## Verification gates

- `node scripts/validate-focusa-tool-contracts.mjs --json` PASS (`tools=59`, `contracts=59`)
- `node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures --json` PASS (`payload_equal=true`, `static_count=59`, `live_count=59`)
- `tests/spec97_reflex_direct_route_static_test.sh` PASS
- `tests/spec97_reflex_traverse_routing_static_test.sh` PASS
- `tests/spec97_reflex_envelope_metadata_static_test.sh` PASS
- `tests/spec97_reflex_primitive_registry_static_test.sh` PASS
- `tests/spec97_reflex_golden_scenarios_static_test.sh` PASS
- `tests/spec97_reflex_utility_card_static_test.sh` PASS
- `tests/spec82_low_resource_efficiency_static_test.sh` PASS
- `cargo test -p focusa-api routes::reflex::tests::reflex_primitives_route_is_bounded_and_read_only -- --nocapture` PASS
- `cargo check -p focusa-api` PASS
- `apps/pi-extension npx tsc --noEmit` PASS

## Result

`SPEC97_REFLEX_DIRECT_API_LIVE_PROOF=PASS`
