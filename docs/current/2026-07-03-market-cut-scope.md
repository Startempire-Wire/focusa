# 24h Market Cut — File-by-File Scope

**Date:** 2026-07-03
**Goal:** ship-ready for market by 2026-07-04
**Approach:** scope all files on all surfaces that need alignment, then execute

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
