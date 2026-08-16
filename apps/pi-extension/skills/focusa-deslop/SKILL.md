---
name: focusa-deslop
description: "Use when cleaning AI-code slop, reviewing diffs for slop patterns, checking for similar code before writing, or running the deslop duplication analysis in the Focusa codebase."
---

# Focusa Deslop Playbook

Use to keep the Focusa codebase free of AI-code slop. The full
taxonomy, diff-scrub recipe, review protocol, checks list, and
close-the-loop procedure live in
`references/01-focusa-deslop-runbook.md` — this file is the entry
point only.

## Start here

1. Before writing a helper/envelope/test-setup: run the deslop
   analysis and check for an existing implementation (renamed copies
   are rejected).
2. Converge intentional boilerplate through the canonical
   constructors (error_envelope, tool_result_v1, store/ledger
   patterns).
3. Review the diff against the runbook taxonomy before commit.
4. Scrub slop with the runbook recipe; report a 1-3 sentence summary
   of what changed.
5. The committed `.deslop.toml` owns the ceiling; CI reports against
   it (advisory until the baseline lands, then gating).
