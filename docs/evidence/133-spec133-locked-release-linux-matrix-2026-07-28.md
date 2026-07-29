# 133 — Spec 133 Locked-Release Linux Matrix Evidence

**Status:** Linux/runtime slice passed; external macOS/Windows release evidence still required before final closure.

## Candidate

- Base: `origin/main@94d039bee3e8365a9664fbad19fa0cad30f333a4`
- Worktree: `/tmp/focusa-next-locked-release`
- Rust profile: dev, debug info disabled, two jobs, incremental disabled
- Host: AlmaLinux/glibc 2.28

## Passed proof

- 36 Spec 133 Python/static contract cases: `/tmp/lock-spec133-python-matrix.txt`
- 17 Spec 133 shell/runtime/operator cases: `/tmp/lock-spec133-shell-matrix.txt`
- Phase 4 supervision/recovery/resource/failure matrix
- Phase 5 isolation/writer/scheduler/workspace matrix
- Phase 6 authority/context/checkpoint/evidence/receipt/learning matrix
- Phase 7 daemon/API/CLI/Pi/menubar operator matrix: `/tmp/lock-spec133-phase7-proof.txt`
- Full Rust workspace tests with the documented daemon-first CI sequence: `/tmp/lock-cargo-test-workspace-ci.txt`

## Defects closed during proof

1. Repaired 37 missing Spec 133 sequential/gate dependency edges in Beads.
2. Added provider-valid `spec:133` grounding and acceptance criteria to every open final-gate item.
3. Updated stale static gates to canonical `silent_session_control_*` equivalent projections permitted by §11.
4. Replaced legacy tmux-only Spec 96 assertions with daemon-native control, parity, and typed failure checks.
5. Added `process_control_failed` recovery to public Silent Session tool documentation and regenerated capability descriptors.
6. Made menubar pnpm execution portable to glibc 2.28 through an explicit Rollup WASM override and esbuild build allowlist.
7. Replaced invalid Pi-extension pnpm test commands with the repository's locked npm/typecheck/runtime scripts.

## Remaining final-gate boundary

The Linux host cannot truthfully prove current macOS and Windows clean lifecycle execution. Those results must come from authorized CI/native runners and be attached before closing `focusa-a6yq6.10.5`, `.10.8`, and `.10.9`.
