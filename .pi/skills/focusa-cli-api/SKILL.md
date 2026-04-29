---
name: focusa-cli-api
description: "Use when operating Focusa through direct CLI or HTTP API: health checks, release proof, endpoint troubleshooting, parity checks, or daemon verification."
---

# Focusa CLI/API Operations Playbook

Use when operating Focusa through direct CLI or HTTP API: health checks, release proof, endpoint troubleshooting, parity checks, or daemon verification.

## Start here

1. Load the main Focusa skill if you need the whole system model: `/skill:focusa`.
2. Read the focused tool doc: `docs/focusa-tools/README.md`.
3. Prefer canonical Focusa state over transcript memory.
4. Preserve proof as evidence refs, not pasted logs.

## Primary docs

- Focused tools: `docs/focusa-tools/README.md`
- Tool index: `docs/focusa-tools/README.md`
- Operator guide: `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md`
- Live release proof: `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`

## Safety rules

- Treat `canonical=false`, `degraded=true`, `pending`, or `blocked` as recovery states, not success.
- Use Workpoint resume/checkpoint around compaction, context overflow, model switch, fork, or risky release work.
- Use writer-status/preflight before mutating work-loop state.
- Do not describe Focusa as complete or frozen; use current snapshot/version language.
