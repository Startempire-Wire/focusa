# SPEC80 Label Taxonomy Enforcement — 2026-04-21

Purpose: enforce §19.3 classification labels during BD decomposition so no planned work is misrepresented as implemented.

## Required labels

1. `implemented-now`
- Meaning: functionality exists in current code and is testable now.
- Mandatory citation: code path (`crates/...:line`) and test/route evidence.

2. `documented-authority`
- Meaning: required by authoritative docs, not yet confirmed implemented.
- Mandatory citation: authoritative spec/doc section.

3. `planned-extension`
- Meaning: architecture introduced/extended by SPEC80 and not yet implemented.
- Mandatory citation: SPEC80 clause + target endpoint/command.

## Enforcement policy for decomposition

- Every BD generated from SPEC80 must include exactly one primary label.
- Mixed-state tasks must split into separate BDs by label.
- Any closure reason claiming implementation must use `implemented-now` and include code citation.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/24-capabilities-cli.md
- docs/44-pi-focusa-integration-spec.md
