# 24h Market Cut — File-by-File Scope

**Date:** 2026-07-03
**Goal:** ship-ready for market by 2026-07-04
**Approach:** scope all files on all surfaces that need alignment, then execute

---

## Guiding principle (operator rule, not negotiable)

**NO COMPROMISE ON SCOPE ENFORCEMENT.**

Every change in this scope must preserve — and where possible strengthen —
the existing scope guards. Specifically:

- `unsafe_project_root_reason` continues to reject `/`, `/root`, `/home`,
  `/tmp`, `/var`, `/usr`, `/opt`, and any empty/whitespace string.
- `isProjectRootAuthoritySafe` continues to require a focused, non-trivial
  directory.
- Workpoint resume continues to require `project_root` + `continuity_id`
  match; `action_authority_for_current_ask=false` is the *correct*
  behavior when scopes mismatch (not a bug to fix).
- Trajectory packet resume continues to require continuity_id alignment
  across the scope switch ledger.
- Action preflight continues to Block release-binary replaces on
  live_build_host; this guard is the same one preventing the Cursor
  transcript's "generate dummy files" failure.
- `pushDelta` cwd-change detection does NOT lower the bar — it
  clears a stale frame cache so the *next* write enforces fresh
  authority, rather than bypassing authority.
- Workpoint `current` scope auto-detection does NOT bypass the daemon's
  scope guard — it only discovers a candidate scope from `.focusa-project.json`
  and forwards it to the existing safe-path checks.

If a change weakens any of the above, it is OUT of scope for the 24h cut
and must be filed as a separate spec with operator review.

### Scope enforcement invariants (test in CI, do not regress)

```
1. POST /v1/workpoint/checkpoint with project_root="/" → 400 scope_mismatch
2. POST /v1/workpoint/checkpoint with project_root="/root" → 400 scope_mismatch
3. POST /v1/workpoint/checkpoint with project_root="" → 400 missing_project_root
4. POST /v1/workpoint/checkpoint with project_root="/home/user/.focusa" → ok
5. GET /v1/project/identity?project_root="/root" → confidence=low
6. focusa action preflight with kind=binary_replace, install_role=live_build_host → Block
7. focusa action preflight with kind=binary_replace, install_role=unknown → AskOperator
8. focusa install with --project-root="/" → error
9. focusa install on macOS without codesign → continues (codesign verify is optional P0)
10. focusa install with --target=darwin on linux → error (cross-arch blocked)
```

These invariants are protected by tests. Any new test that violates one
must be removed or the underlying guard reverted.

---

## Surfaces (1-12)

| # | Surface | Path | Owner |
|---|---|---|---|
| 1 | CLI commands | `crates/focusa-cli/src/commands/*.rs` | Rust |
| 2 | CLI dispatch | `crates/focusa-cli/src/main.rs` | Rust |
| 3 | CLI module | `crates/focusa-cli/src/commands/mod.rs` | Rust |
| 4 | Daemon routes | `crates/focusa-api/src/routes/*.rs` | Rust |
| 5 | Daemon core | `crates/focusa-core/src/*.rs` | Rust |
| 6 | Pi extension | `apps/pi-extension/src/*.ts` | TypeScript |
| 7 | TUI | `crates/focusa-tui/src/*.rs` | Rust |
| 8 | Menubar | `apps/menubar/src/**` | TS + Rust |
| 9 | Installer scripts | `scripts/install-focusa.{sh,ps1}` | bash + PS1 |
| 10 | GitHub workflows | `.github/workflows/*.yml` | YAML |
| 11 | Docs | `docs/**/*.md` | Markdown |
| 12 | Tests + beads | `tests/spec_*.sh`, `.beads/issues.jsonl` | bash + JSONL |

---

## Beads → file impact (24h cut scope)

### A. Install body sub-beads (Rust CLI, surface 1-3)

| Bead | Files touched |
|---|---|
| `focusa-112-codesign-verify` | `crates/focusa-cli/src/commands/install.rs` (add `phase_codesign_verify()`); `tests/spec_install_codesign_verify_static_test.sh` (NEW) |
| `focusa-112-path-walkthrough-test` | `tests/spec_install_path_walkthrough_static_test.sh` (NEW) |
| `focusa-112-install-static-test` | `tests/spec_install_rust_static_test.sh` (NEW) |
| `focusa-112-foyr-close` | (verification only — no code) |
| `focusa-112-3cok-close` | (verification only — no code) |

### B. Build matrix (CI, surface 10)

| Bead | Files touched |
|---|---|
| `focusa-112-windows-arm64-asset` | `.github/workflows/release.yml` (add `aarch64-pc-windows-msvc` target) |
| `focusa-112-musl-asset` | `.github/workflows/release.yml` (add `x86_64-unknown-linux-musl` target) |

### C. Transcript gaps (Rust CLI, surface 1-3 + 4)

| Bead | Files touched |
|---|---|
| `focusa-112-action-preflight-structured` | `crates/focusa-cli/src/commands/action.rs`; `crates/focusa-api/src/routes/action.rs`; `tests/spec_action_preflight_structured_static_test.sh` (NEW) |
| `focusa-112-doctor-scope-modes` | `crates/focusa-cli/src/commands/doctor.rs`; `tests/spec_doctor_scope_modes_static_test.sh` (NEW) |
| `focusa-112-mcp-jsonrpc` | `crates/focusa-api/src/routes/mcp.rs` (NEW); `crates/focusa-api/src/server.rs` (mount); `crates/focusa-api/src/routes/mod.rs` (pub mod); `tests/spec_mcp_jsonrpc_static_test.sh` (NEW) |
| `focusa-112-onboard-scoped` | `crates/focusa-cli/src/commands/onboard.rs`; `tests/spec_onboard_scoped_static_test.sh` (NEW) |

### D. GitHub issue fixes (deferred from earlier)

| Bead | Files touched |
|---|---|
| `focusa-gh-4-perpetua-scope` (GH #4) | `crates/focusa-api/src/routes/project.rs:201` (modify `identity_name_matches` to also consider `project_root`); `tests/spec_focusa_g4_perpetua_scope_static_test.sh` (extend) |

### E. New leverage beads (Rust CLI, surface 1-3)

For each new command, three file touches:

| Bead | Command file | Module | Dispatch |
|---|---|---|---|
| `focusa-workflow-cmd` | `crates/focusa-cli/src/commands/workflow.rs` (NEW) | `mod.rs` | `main.rs` |
| `focusa-audit-cmd` | `crates/focusa-cli/src/commands/audit.rs` (NEW) | `mod.rs` | `main.rs` |
| `focusa-status-enrich` | `crates/focusa-cli/src/commands/daemon.rs` (extend status) | (existing) | (existing) |
| `focusa-recover-cmd` | `crates/focusa-cli/src/commands/recover.rs` (NEW) | `mod.rs` | `main.rs` |
| `focusa-spec92-hooks` | `crates/focusa-cli/src/commands/action.rs` (extend) | (existing) | (existing) |
| `focusa-upgrade-cmd` | `crates/focusa-cli/src/commands/upgrade.rs` (NEW) | `mod.rs` | `main.rs` |
| `focusa-license-enforcement-audit` | `crates/focusa-cli/src/commands/license.rs` (extend) | (existing) | (existing) |

For each NEW command:
- `tests/spec_<name>_static_test.sh` (NEW)

### F. Documentation updates (surface 11)

| Bead | File |
|---|---|
| All of A-C | `docs/112-install-binary-architecture-spec.md` (acceptance checklist update per closure) |
| `focusa-112-about-command` | `docs/llms.txt` (cross-link to `focusa about`) |
| All beads | `.beads/issues.jsonl` (status, notes, closure) |

---

## Order of operations (24h)

### Phase 1 (next 2h) — Transcript gaps closest to done
- **1A** `focusa-112-action-preflight-structured` (CLI + daemon, ~45min)
- **1B** `focusa-112-doctor-scope-modes` (~30min)
- **1C** `focusa-112-onboard-scoped` (~30min)
- **1D** `focusa-112-install-static-test` (test only, ~15min)

### Phase 2 (next 2h) — Build matrix + install verify
- **2A** `focusa-112-windows-arm64-asset` (CI yaml, ~30min)
- **2B** `focusa-112-musl-asset` (CI yaml, ~30min)
- **2C** `focusa-112-codesign-verify` (~30min)
- **2D** `focusa-112-path-walkthrough-test` (~15min)

### Phase 3 (next 2h) — New leverage commands
- **3A** `focusa-workflow-cmd` (~45min)
- **3B** `focusa-audit-cmd` (~30min)
- **3C** `focusa-status-enrich` (~20min)
- **3D** `focusa-recover-cmd` (~30min)
- **3E** `focusa-upgrade-cmd` (~45min)
- **3F** `focusa-spec92-hooks` (~30min)
- **3G** `focusa-license-enforcement-audit` (~30min)

### Phase 4 (last 1-2h) — GH #4 fix + final closure
- **4A** `focusa-gh-4-perpetua-scope` (project.rs, ~1h)
- **4B** Final `bd close` pass on the EPIC + child closures

---

## Total scope (file count)

- **NEW files (~13)**: workflow.rs, audit.rs, recover.rs, upgrade.rs, mcp.rs, +7 spec_*.sh tests, +pi-extension update
- **MODIFIED files (~15)**: install.rs, action.rs, doctor.rs, onboard.rs, license.rs, daemon.rs, project.rs, workpoint.rs, main.rs, mod.rs, server.rs, release.yml, llms.txt, install-focusa.sh, install-focusa.ps1
- **BEADS file**: `.beads/issues.jsonl` (constant churn as beads close)

---

## Acceptance for "GTM-ready"

1. `bd ready --priority=0` returns empty (or only ops/maintenance beads)
2. All 6 GH issues closed
3. `cargo check --workspace` clean
4. `cargo test -p focusa-cli` clean
5. `tests/spec_*.sh` all green
6. CI green on main

---

## Execution notes

- This scope covers 24h, not 1 week. Stretch goals deferred:
  - MCP bridge (architectural, days not hours)
  - Spec 109/110/111 broader work
  - Spec 92 polish hooks (sub-bead of focusa-spec92-hooks is a small slice)
- Each phase has clear acceptance; complete + commit + push at end of each.
- Pre-existing GTM blockers (focusa-foyr, focusa-3cok, focusa-9im1) are now closed.

---

## Backward compatibility policy (24h cut)

**Rule: additive changes only.** Every change must preserve the wire/CLI
contract that existing clients (Pi extension, scripts, menubar, third-party
agents) depend on. Concretely:

| Change type | Allowed in 24h cut? | Required practice |
|---|---|---|
| Add new field to envelope | YES | Default `Option<T>`; never remove existing field |
| Add new enum variant | NO without a wire marker | Add at end of variant list; do NOT renumber |
| Add new command/subcommand | YES | Goes through `focusa <name>` — never renames an existing top-level command |
| Add new flag to existing command | YES | Default value preserves current behavior |
| Add new field to `--json` output | YES | Older clients ignore unknown fields (Spec 92 / envelope contract) |
| Change return type / field shape | **NO** | Would break older clients that destructure |
| Rename existing field | **NO** | Add new field, deprecate old in a follow-up |
| Remove existing flag | **NO** | Even if deprecated, keep the parser accepting it |
| Change error envelope code | **NO** without compat | Add new code, keep old as alias |
| Change HTTP route path | **NO** | Mount new route, keep old with deprecation header |
| Change default behavior of existing flag | **NO** | New behavior must be opt-in via a new flag |

### What we did to enforce this in the 24h cut

- `ActionPreflightEnvelope.checks` — **new optional field** (older clients ignore it). Closed in commit `06cab28b` (planned).
- `GET /v1/workpoint/current` response under `not_found` — **new optional fields** `detected_project_root` / `detected_continuity_id` / `recovery_hint`. Status code unchanged. Closed in commit `5310c2f6`.
- `pushDelta` cwd-change detection — **internal change**, no API surface shift. Closed in commit `20fcd7cc`.
- `focusa uninstall` — **new top-level command**. Existing `install` and `install-service` unchanged. Closed in commit `796aad81`.
- `focusa about` — **new top-level command**. Closed in commit `efc6cbbc`.
- `GET /llms.txt` — **new HTTP route**. Closed in commit `b16cf3b3`.
- `install-focusa.sh` / `install-focusa.ps1` — **rewritten** but functional contract unchanged (curl|bash / irm|iex still works, downloads `focusa`, exec's `focusa install`). Closed in commit `aa246287`.

### Compat tests to add (per envelope change)

For every envelope change, the corresponding static guard
(`tests/spec_<bead>_static_test.sh`) must include:

1. A check that existing fields still exist (added-not-removed)
2. A check that the new field is `Option<T>` (older clients can ignore)
3. A check that --json output produces the field list unchanged
4. A check that existing test vectors (e.g., `evaluate_preflight` on a
   binary_replace on live_build_host) still produces the same verdict

### Compatibility risk register

| Change | Risk | Mitigation |
|---|---|---|
| `ActionPreflightEnvelope.checks` | Low (additive) | Field is `Vec<...>`; older clients ignore |
| `GET /v1/workpoint/current` adds fields | Low (additive) | Status code unchanged; older clients ignore new fields |
| `pushDelta` cwd-clear | Medium (changes runtime behavior) | Stale-frame retry path was already there; we just trigger it earlier |
| `install-focusa.sh` rewrite | Low (download + exec contract same) | Functional contract preserved; static guard checks new shape |
| New `focusa uninstall` / `about` commands | None (additive) | Existing commands unchanged |
| `GET /llms.txt` | None (new route) | Other routes unchanged |
| Phase 1B: doctor scope modes | **NO existing flag changes** | New `--scope=host|project|repo` only |
| Phase 1C: onboard scoped | **NO** | New flag only |
| Phase 1D: install static test | **NO** (test only) | n/a |
| Phase 2A/B: build matrix | **NO** (CI yaml) | Existing targets unchanged |
| Phase 2C: codesign-verify | Low (new install phase) | Optional; only runs on macOS, on binary path |
| Phase 2D: path-walkthrough-test | None (test only) | n/a |
| Phase 3A-G: new leverage commands | None (additive) | All new top-level commands |
| Phase 4A: GH #4 fix | **HIGH** | `identity_name_matches` change is global; must preserve behavior for projects without overlap. Add per-identity signal: only match alias if `project_root` is consistent with marker. |

