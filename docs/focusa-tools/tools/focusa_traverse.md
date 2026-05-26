# `focusa_traverse`

## Purpose

Read-only surgical traversal across large Focusa surfaces. Use it for bounded lineage, ontology, Focus Stack, Workpoint, evidence/reference, telemetry, and tool-registry slices instead of full payload defaults.

## When to use

Use when Trajectory or Workpoint identifies a missing narrow slice and a full tree/store/log would be too broad.

## Inputs

- `surface`: `lineage`, `ontology`, `focus_stack`, `workpoints`, `evidence`, `telemetry`, `tool_registry`, etc.
- `selector`: `window`, `head`, `path`, `children`, `neighborhood`, `summaries`, `search`, `recent`, or `tags_verify`.
- `anchor`, `query`, `cursor`, `limit`, `depth`, `radius` for bounded traversal.
- `fields` for projection and `tags` for verification.
- `include_full_payload=true` is an explicit cold opt-in; default reads stay bounded.

## Expected result

A successful call returns a bounded slice, traversal metadata, stable tags, and `details.tool_result_v1.status=completed`; unsupported surfaces return `failure_class=validation_rejected` without side effects.

## Surface adapters

Current adapters include `trajectory`, `lineage`, `ontology`, `focus_stack`, `workpoints`, `evidence`/`ecs`/`references`, `metacognition`, `predictions`, `telemetry`/`commands`/`turns`, `snapshots`, and `tool_registry`/`capabilities`. Defaults are bounded; full payloads require explicit cold opt-in.

Trajectory context: `trajectory` slices expose the ladder directly; `evidence`/`ecs`/`references` default projections include bounded handle-level `trajectory` context when present so proof handles remain HLT/STG-aligned without requesting full payloads.

## Output contract

Responses include `items`, `traversal` metadata, `tag_scheme`, item/range/window/surface `tags`, `verified_tags`, `stale_tags`, `canonical/degraded`, `failure_class`, `next_tools`, and `details.tool_result_v1`.

## Anchor tag semantics

- `item` tags bind one returned item anchor to a SHA-256 digest of the projected item.
- `range` tags bind the current cursor range to the returned item digests.
- `window` tags bind cursor plus limit to the returned item digests.
- `surface` tags bind the surface total plus window digest and may change after unrelated surface changes.
- `tags_verify` checks tags and returns `verified_tags` and `stale_tags` without returning full payloads.
- Collision policy: 24-hex SHA-256 digest plus anchor; on suspected collision request narrower fields or a future longer tag version.

## Examples

```text
focusa_traverse surface="lineage" selector="path" anchor="<clt-node-id>" limit=25 fields=["node_id","summary"]
focusa_traverse surface="ontology" selector="neighborhood" anchor="Component:checkout" radius=2 limit=20
focusa_traverse surface="workpoints" selector="recent" limit=10
focusa_traverse surface="lineage" selector="tags_verify" tags=["focusa://lineage/window/<node>#0"]
```

## Related

- [`focusa_trajectory_view`](./focusa_trajectory_view.md)
- [`focusa_workpoint_resume`](./focusa_workpoint_resume.md)
- [`focusa_lineage_tree`](./focusa_lineage_tree.md)

## Contract summary

- Family: Traversal.
- Side effects: `read_state`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `POST /v1/traverse`, `POST /v1/traverse/verify-tags`
- CLI commands: none.
- Parity: `domain`; exemptions: `api_domain_only`.
- Core surface: Spec96 surgical traversal facade.
- Live check: contract_static plus /v1/traverse lineage smoke test.
- Contract source: `docs/current/focusa-tool-contracts.json`.
