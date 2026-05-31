# Path Traversal Security Tests

Status: current CWE-22 path traversal coverage and remaining route inventory.

## Implemented coverage

| Surface | Test/gate | Coverage |
| --- | --- | --- |
| `focusa cleanup --safe` `/tmp` glob expansion | `crates/focusa-cli/src/commands/cleanup.rs` unit tests | Simple `/tmp` prefix/suffix glob matcher rejects non-`/tmp` patterns, suffix mismatches, and `../`-style names. |
| recoverable cleanup target mapping | `safe_target_keeps_absolute_paths_inside_trash_root` | Absolute cleanup paths are re-rooted below the generated trash root instead of used as destination paths. |
| command boundary regression | `tests/security_shell_unwrap_static_test.sh` | Cleanup glob expansion must remain shell-free and bounded to Rust `/tmp` matching. |

## Path-sensitive route inventory

| Surface | Path inputs | Risk | Required next coverage |
| --- | --- | --- | --- |
| Project identity/verify | `cwd`, `project_root` | Broad root or cross-project confusion | Existing project-root safety guards; add canonicalization tests for symlink/`..` variants. |
| Focus/Workpoint/Trajectory | `project_root`, target refs | Cross-project state bleed | Route-level malicious `../`/encoded traversal payloads are covered by the dynamic API smoke and mutation JSON path guard. |
| Attachments | attachment refs/paths | arbitrary file linking or traversal | Add size/type/path canonicalization tests before remote exposure. |
| ECS/reference/artifact stores | evidence refs, artifact handles | off-root artifact deref | Add handle-only policy and canonical artifact-root checks. |
| Events/log paths | data_dir-derived log paths | reading/writing outside data dir if config abused | Add config/data-dir canonicalization tests. |
| Work-loop silent sessions | `cwd` | command execution in unexpected path | Enforce safe project-root allowlist before execution controls. |

## Current conclusion

Focusa now has concrete path traversal regression tests for cleanup plus dynamic API smoke coverage for malicious `../`/encoded traversal payloads in path-like JSON mutation fields. Remaining path work is narrower: attachments, ECS/artifact handles, data-dir canonicalization, and work-loop cwd allowlists before any broad network exposure.
