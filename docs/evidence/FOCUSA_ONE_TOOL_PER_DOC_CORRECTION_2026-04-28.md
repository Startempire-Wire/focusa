# Focusa One-Tool-Per-Doc Correction — 2026-04-28

## Correction

Operator clarified that the prior family-doc implementation did not follow the instruction. The docs now use one individual Markdown page per current `focusa_*` Pi tool.

## Implemented

- Created `docs/focusa-tools/tools/` with 43 individual tool docs.
- Converted family docs into navigation/index pages only.
- Updated `docs/focusa-tools/README.md` to link every tool doc.
- Updated root `README.md` with an individual table row for every current `focusa_*` tool and its doc link.

## Validation

Validation command counted current tools from `apps/pi-extension/src/tools.ts`, checked root README table rows, checked files exist, and verified no missing/extra links.

Result:

```text
readme_tool_rows 43
src 43 linked 43 missing_files [] missing_links [] extra_links []
```

Pi extension TypeScript compile also passed:

```bash
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
```

## Current doc shape

- Root README: individual tool table with 43 linked rows.
- `docs/focusa-tools/README.md`: tool-doc index.
- `docs/focusa-tools/<family>.md`: family navigation only.
- `docs/focusa-tools/tools/<tool>.md`: one tool per file, with purpose, when to use, when not to use, example usage, expected result, recovery notes, related tools, and source.
