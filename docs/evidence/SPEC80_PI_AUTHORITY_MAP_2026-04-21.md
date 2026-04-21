# SPEC80 Pi Authority Map — 2026-04-21

Purpose: complete `focusa-yro7.1.1.1` by mapping Pi authoritative docs to concrete obligations in `docs/80-pi-tree-li-metacognition-tooling-spec.md`.

## Mapping matrix

| Pi source | Verified capability/constraint | SPEC80 binding |
|---|---|---|
| `README.md` | Pi command/session surface is extension + tool driven; architecture supports external cognitive control surfaces | §1 source alignment; §5 architecture decisions; §11 phased execution |
| `docs/extensions.md` | First-class `registerTool`, lifecycle hooks, command registration; extension is proper place to expose `/tree`-linked bridge tools | §6 tool contracts; Appendix A tool catalog; §13 planning next actions |
| `docs/sdk.md` | Programmatic session orchestration and tool execution surface supports deterministic integrations and automation harnesses | §5 API authority; Appendix B bindings; §10 acceptance gates |
| `docs/tui.md` | UI overlays and custom components allow explicit status/branch indicators without hidden mutation | §3 scope (operational widgets), §5 no hidden writes, §12 latency mitigations |
| `docs/skills.md` | Skill packaging supports progressive disclosure and explicit capability packs; suitable for metacognitive tool bundles | §5 compounding loop design; §6.2 metacognitive tools; §13 decomposition actions |
| `docs/keybindings.md` | `/tree` and tree navigation are first-class session operations, requiring branch-aware cognitive restore semantics | §2 problem statement (tree/fork gap), §6.1 tree tools, Appendix D replay pack |

## Derived implementation-planning obligations

1. Keep integration tool-first (not marker parsing) for metacognitive capture.
2. Treat `/tree` navigation as lineage-critical and branch-restore-sensitive.
3. Keep API as authority, CLI as parity/fallback, no hidden extension-side memory writes.
4. Require deterministic schemas and replay tests before rollout.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- /opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/README.md
- /opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/extensions.md
- /opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/sdk.md
- /opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/tui.md
- /opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/skills.md
- /opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/keybindings.md
