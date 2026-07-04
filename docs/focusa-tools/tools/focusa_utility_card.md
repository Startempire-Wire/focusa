# `focusa_utility_card`

**Family:** `diagnostics_hygiene`  
**Label:** Focusa Utility Card

## Purpose

Read compact startup, bootstrap, post-compaction, recovery, and tool-brevity guidance from current core surfaces.

## When to use

- Session start or post-compaction resume.
- Scope conflict or stale transcript risk.
- Tool behavior/docs feel verbose or pre-current-core.

## Expected result

A `tool_result_v1` envelope with `focusa.utility_card.v1` raw payload containing:

- `authority_boundary`
- `usefulness_bar`
- `scope_gate`
- `bootstrap_card`
- `post_compaction_card`
- `exact_next_actions`
- `do_not_drift`
- `evidence_policy`
- `brevity_rules`
- `recovery_order`
- `proof_commands`
- `next_tools`

## Contract summary

- API: `GET /v1/utility/card`, `GET /v1/utility/bootstrap`, `GET /v1/utility/post-compaction`.
- CLI: `focusa utility card`, `focusa utility bootstrap`, `focusa utility post-compaction`.
- Side effects: read-only.
- Core: `focusa_core::utility_card::UtilityCard`.

- API: `GET /v1/utility/bootstrap`
- API: `GET /v1/utility/post-compaction`
- CLI: `focusa utility bootstrap`
- CLI: `focusa utility post-compaction`
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
