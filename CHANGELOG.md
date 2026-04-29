# Changelog

Focusa is under active development. Versions below are current snapshot tags, not finished-product declarations.

## Unreleased — GitHub release and Mac app refresh

- Fixed GitHub CI/release clippy and spec-gate blockers.
- Updated the Mac menubar app to version `0.9.9` across package, Tauri, lockfile, and UI version surfaces.
- Added Bun lockfile for the menubar app because the release workflow installs with Bun.
- Updated release-note examples to use the active release tag instead of stale `v0.2.10` paths.
- Restored pending-gated compaction auto-resume retry wiring required by strict spec gates.

## Unreleased — Spec91 live tool contract proof

- Added Spec91 for live runtime proof that the daemon ontology tool-contract projection matches the canonical registry.
- Added `scripts/prove-focusa-tool-contracts-live.mjs`.
- Added read-only `--safe-fixtures` live probes for representative Workpoint, Work-loop, tree/lineage, metacognition, and Focus State surfaces.
- Added live proof docs at `docs/current/LIVE_TOOL_CONTRACT_PROOF.md`.

## Unreleased — Spec90 tool contract foundation

- Added Spec90 for ontology-backed Focusa tool contracts and parity hardening.
- Added current machine-readable registry for all 43 `focusa_*` Pi tools.
- Added JSON registry projection and `GET /v1/ontology/tool-contracts` API projection.
- Added deterministic contract validation script.
- Upgraded `focusa_tool_doctor` to report contract coverage summary.
- Added current tool contract registry documentation.

## v0.9.2-dev — Focusa tool docs split

- Added one individual doc for each current `focusa_*` Pi tool under `docs/focusa-tools/tools/`.
- Added a root README table linking all 43 individual tool docs.
- Kept family docs as navigation pages only.
- Validation: root README links 43/43 current tools with no missing or extra links.

## v0.9.1-dev — Focused skills and tool-doc navigation

- Added companion Pi skills: `focusa-workpoint`, `focusa-metacognition`, `focusa-work-loop`, `focusa-cli-api`, `focusa-troubleshooting`, `focusa-docs-maintenance`.
- Added focused tool family docs and linked them from README.
- Validated Pi skill loader diagnostics for project, extension, and installed skill directories.

## v0.9.0-dev — Public runtime docs alignment

- Updated GitHub-facing README and high-risk public docs to describe the current Rust core/API/CLI/Pi runtime snapshot.
- Removed pre-implementation and finished/frozen wording from key public docs.
- Documented current Workpoint, evidence, metacognition, state hygiene, and tool-result envelope behavior.

## Earlier current-build milestones

- Spec88 Workpoint continuity: checkpoint/current/resume/drift/evidence-link APIs and Pi tools.
- Spec89 tool hardening: common tool result envelope, Workpoint-linked evidence, tool doctor, active-object resolver, work-loop UX, metacog quality gates, dedupe/hygiene surfaces, live release proof.
