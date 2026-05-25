# `focusa_reflex_primitives`

## Purpose

Read bounded Spec97 Reflex Primitive summaries from the read-only registry. This is advisory routing metadata only; existing Focusa tools and reducers retain mutation authority.

## When to use

Use when a blocked/degraded result includes `reflex_suggestions`, or when an agent needs the smallest safe next-step affordance for a recurring risk, family, object, or action.

## Inputs

- `family`: optional primitive family filter, e.g. `recovery`, `evidence`, `resource`.
- `query`: optional risk/object/action search text.
- `limit`: bounded result limit, max 50.
- `include_payload`: explicit cold opt-in for full primitive payloads; default is bounded summaries.

## Expected result

A successful call returns `status=completed`, `read_only=true`, `advisory_only=true`, bounded `items`, `bounds`, and `details.tool_result_v1`.

## Output contract

Default items include primitive id, family, trigger, recommended tool, authority boundary, escalation boundary, hot-path budget, failure envelope, and source marker. Full registry payloads require `include_payload=true`.

## Related

- [`focusa_traverse`](./focusa_traverse.md)
- [`focusa_tool_doctor`](./focusa_tool_doctor.md)
- `docs/97-focusa-reflex-primitives-spec.md`
- `docs/evidence/SPEC97_REFLEX_DIRECT_API_LIVE_PROOF_2026-05-25.md`

## Contract summary

- Family: Traversal.
- Side effects: `read_state`.
- Result envelope: `tool_result_v1`.
- API route: `GET /v1/reflex/primitives`.
- CLI commands: none.
- Parity: `domain`; exemptions: `api_domain_only`.
- Core surface: Spec97 Reflex Primitive registry and bounded direct API.
- Live check: contract_static plus `/v1/reflex/primitives?family=recovery&limit=2` smoke test.
- Contract source: `docs/current/focusa-tool-contracts.json`.
