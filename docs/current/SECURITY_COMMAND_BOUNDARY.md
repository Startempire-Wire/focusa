# Security Command Boundary

Status: current static boundary for shell/external command execution and runtime panic APIs.

## Runtime unwrap/expect policy

Runtime slices under `crates/focusa-api/src`, `crates/focusa-core/src`, and `crates/focusa-cli/src` should not use panic APIs (`.unwrap()`, `.expect(...)`) before test modules. Static gate: `tests/security_shell_unwrap_static_test.sh`.

Test modules may use panic APIs. Runtime code should use explicit error envelopes, fallbacks, or `unwrap_or*`/`map_or*` non-panicking defaults.

## Reviewed shell execution hotspots

These shell-command hotspots existed at the time of the 2026-05-26 security review and are intentionally tracked by static allowlist. New shell execution should be rejected until reviewed.

| File | Hotspot | Risk | Required posture |
| --- | --- | --- | --- |
| `apps/pi-extension/src/state.ts` | fixed argv `systemctl restart focusa-daemon` daemon kickstart | Service restart from Pi extension | No shell interpolation; custom shell restart commands are refused by the kickstart path. |
| `apps/pi-extension/src/config.ts` | default `systemctl restart focusa-daemon` | Service restart from Pi extension | Local-only recovery path; fixed command string used as an enum marker. |
| `crates/focusa-cli/src/commands/cleanup.rs` | removed `bash -lc` cleanup glob expansion | Former shell glob expansion could widen CWE-78 surface | Uses bounded Rust `/tmp` `read_dir` plus simple prefix/suffix matching; static gate rejects regression. |
| `crates/focusa-cli/src/commands/release.rs` | `bash -lc` release proof command | Release command injection if command source is untrusted | Use fixed release command list; do not pass untrusted user strings. |
| `crates/focusa-core/src/runtime/daemon.rs` | `bash -c` with `wb wiki create` | Shell quoting risk from generated title/path/temp path | Replace with arg-vector/stdi­n implementation when feasible. |

## External command hotspots without shell

These commands use direct argv-style execution and are lower risk but still part of the command boundary:

- `bd` in work-loop/runtime code.
- `git` in work-loop status helpers.
- `wb` in proxy/runtime paths.
- `guardian`, `mesh`, `script`, `setsid`, `pkill`, `curl` in CLI/support commands.

## Remediation direction

1. Convert runtime `bash -c` Wiki write to `Command::new("wb").args([...]).stdin(...)` if `wb` supports stdin content.
2. Keep Pi daemon restart on the fixed argv `systemctl restart focusa-daemon` path unless a typed command enum is added.
3. Keep release shell execution restricted to curated commands and preserve-path guards.
4. Keep static gate updated when a hotspot is removed or deliberately reviewed.
