---
name: focusa-deslop
description: "Use when cleaning AI-code slop, reviewing diffs, checking for similar code before writing, or running the deslop duplication analysis."
---

# Focusa Deslop Playbook

Slop-free codebase playbook. Detail lives in
`references/01-focusa-deslop-runbook.md`.

## Start here

1. Before writing: `deslop .` / MCP `find-similar` — never write a
   renamed copy of an existing helper.
2. Boilerplate converges through canonical constructors
   (error_envelope, tool_result_v1, store/ledger patterns).
3. Review diffs against the runbook taxonomy; scrub with the recipe.
4. Report a 1-3 sentence summary of what changed.
5. `.deslop.toml` owns the ceiling (CI advisory → gating after baseline).
