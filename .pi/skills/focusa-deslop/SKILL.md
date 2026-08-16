---
name: focusa-deslop
description: "Use when cleaning AI-code slop, reviewing diffs for slop patterns, checking for similar code before writing, or running the deslop duplication analysis in the Focusa codebase."
---

# Focusa Deslop Playbook

Use to keep the Focusa codebase free of AI-code slop: structural
duplication, overcautious defensive code, comment noise, type
escapes, and style drift. Synthesized 2026-08-16 from the deslop
research background, the 7 Deadly Sins of Agentic Coding, the
three-lens review protocol, the ampcode checks model, AsyncReview,
the Cursor team kit, and the Focusa convergence rules.

## Progressive disclosure

Read `references/01-focusa-deslop-runbook.md` for the exact
slop taxonomy, the review protocol, and the close-the-loop procedure.

## Start here

1. BEFORE writing a new helper/envelope/test-setup: run the deslop
   analysis (`deslop .` / CI reports / MCP `find-similar`) and check
   for an existing implementation. Renamed copies are rejected.
2. Convergence first: intentional boilerplate flows through the
   canonical constructors — `focusa_core::error_envelope::*`,
   `focusa.tool_result_v1` envelopes, the store/ledger patterns —
   never re-typed.
3. Review every diff against the slop taxonomy (7 sins + the Focusa
   extensions below) before commit.
4. Remove slop with the reference diff-scrub recipe; report a 1-3
   sentence summary of what changed.
5. The duplication ceiling lives in `.deslop.toml`; the CI job is
   advisory until the baseline lands — never ship over the ceiling.
