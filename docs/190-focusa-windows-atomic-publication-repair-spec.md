#190 — Remaining Windows atomic-publication callers

## Outcome and authority

Repair issue691 within the operator-selected seven-issue core release batch. The reported Windows consumer writes data and then falsely fails when a POSIX directory-open/sync primitive is used. Actual Windows acceptance remains required; Linux checks alone do not close this issue.

## Existing owner and bounded changes

Reuse `focusa-core::durable_fs`, whose Windows atomic replacement already uses the native write-through primitive. No dependency, signing authority, service, storage quota or provider setting is added or changed.

- Retain `atomic_replace`; add `atomic_publish_new` at the same owner, preserving no-clobber semantics. POSIX retains the existing hard-link publication; Windows uses the same write-through adapter without replace-existing permission. Callers retain cleanup ownership for temporary sources that the platform may retain or consume.
- Migrate remaining Genesis, marker and Bootstrap publication paths to this owner. Preserve source-file sync, metadata/ownership checks, idempotency, safe-path checks, error classification and explicit replace versus no-clobber intent. Close temporary file handles before publication.
- Use the existing directory-sync owner for reservation settlement; preserve the requirement that its durable matching receipt exists before reservation removal.

Exact source scope: `crates/focusa-core/src/durable_fs.rs`, API `project.rs`, `project_genesis_support.rs`, `project_bootstrap_fs.rs`, `project_bootstrap_pre_root.rs`, and scoped regression files at those owners.

## Proof and completion

1. Add cross-platform owner tests for replacement, successful new publication and refusal to overwrite; add API create/replace, no-clobber and nonregular/symlink-destination regressions.
2. Execute producer and actual API-writer consumers on the isolated OVH workspace, preserving terminal receipts, component hashes, formatting and strict lint. Recheck replay tests after changing the core owner.
3. Run actual Windows producer/consumer checks through the approved release/test route. The external Windows reproduction is evidence, not a fabricated Linux expected-failure result.
4. Preserve all19 Spec187 requirements, client state and the fixed seven priorities. Complete the full signed release and installed platform tests before acceptance-based closure.

Rollback is limited to the new scoped source commit through ordinary reviewed history; no client files, databases, locks or unrelated services are removed or reset. No paid builds or partial release is authorized.

## Verified source checks — 2026-10-22

- Three publication-owner, seven replay and78 API/project consumer tests passed on Linux; formatting passed.
- Strict core/API all-target Clippy with warnings denied and structural guards passed. Composite proof1740 matches all six source-file hashes across the functional checks and completed lint.
- The initial test-import compilation failure was corrected. A subsequent lint worker reached its own memory limit; its original receipt and kernel-backed terminal reconciliation were preserved. Only unfinished checks were resumed, with one compiler worker; completed tests were not replayed.
- The actual Windows publication owner typechecked for installed x86_64 and aarch64 MSVC targets with warnings denied. This is compile-only evidence, not native Windows behavior, signed installation or acceptance. Issue691 remains open pending those proofs.
