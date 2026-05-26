# Focusa Security Review — 2026-05-26

Scope: whole-repo security review after disk cleanup and daemon live-proof. Reviewed five tracks: dependency/supply-chain, secrets/config exposure, API auth/input validation, filesystem/process privilege boundaries, and persistence/state privacy.

## Environment and disk precheck

Before review, root filesystem was critically full:

- Before cleanup: `/` 319G/350G used, 96%, 14G free.
- After cleanup: `/` 278G/350G used, 84%, 55G free.
- Cleanup removed only build/temp/trash/cache residue: `/tmp/focusa-*` build targets, `~wirebot/.trash`, `~/focusa/target`, and dnf cache.

## Review 1 — Dependencies and supply chain

Commands/evidence:

- `npm audit --audit-level=moderate --json` in `apps/pi-extension` → `/tmp/focusa-npm-audit-pi-extension.json`.
- `npm audit --audit-level=moderate --json` in `apps/menubar` → `/tmp/focusa-npm-audit-menubar.json`.
- `Cargo.lock` contains 326 Rust packages.
- `cargo-audit` / `cargo-deny` were not installed, so Rust advisory review is incomplete.

Findings:

1. **High — Pi extension dependency tree has npm vulnerabilities.**
   - Summary: 10 total: 1 critical, 3 high, 6 moderate.
   - Critical: `protobufjs` arbitrary/code-generation related advisories (`<7.5.8` per npm audit range summary).
   - High: `basic-ftp`, `fast-uri`, `fast-xml-builder`.
   - Likely path: transitive dev dependency tree from Pi coding agent/provider SDK dependencies, but must be confirmed before release.

2. **High — Menubar dependency tree has npm vulnerabilities.**
   - Summary: 8 total: 4 high, 3 moderate, 1 low.
   - High surfaces include `@sveltejs/kit`, `vite`, `rollup`, `picomatch`.
   - Menubar is a Tauri/Svelte app; dev-server advisories are lower risk if not exposed, but packaged/runtime dependency impact must be separated.

3. **Medium — Rust dependency advisory coverage missing.**
   - `cargo-audit`/`cargo-deny` absent; no RustSec advisory proof was produced.

Recommendations:

- Add a committed supply-chain gate using `cargo audit`/`cargo deny` and npm audit summaries.
- Upgrade npm lockfiles or isolate dev-only vulnerabilities from runtime release risk.
- Track Pi extension SDK transitive dependency exposure explicitly.

## Review 2 — Secrets and configuration exposure

Command/evidence:

- Redacted regex scanner: `/tmp/focusa-secret-scan-redacted.json`.

Findings:

1. **No obvious raw private key or AWS access key committed in scanned source.**
2. **21 redacted hits require triage; most appear to be env-var reads, docs, generated build output, Beads text, or false positives.**
   - Examples: `MINIMAX_API_KEY` env reads, `FOCUSA_TOKEN` config mapping, docs mentioning gateway secret, generated Svelte build chunks.
3. **Potential false positive:** `crates/focusa-core/src/adapters/openai.rs` matched a GitHub-token pattern in a test/function string, not necessarily a secret.
4. **Build/generated artifacts are in review scope.**
   - `apps/menubar/build` and `.svelte-kit/output` were scanned and included false-positive/generated output.

Recommendations:

- Add a proper secret scanner allowlist/baseline (for env-var names and docs examples) while keeping raw token/private-key detection fail-closed.
- Ensure generated build directories are ignored or intentionally audited, but not committed if unnecessary.
- Keep `FOCUSA_AUTH_TOKEN` and provider keys only in env/secret store, never config committed to repo.

## Review 3 — API auth and input validation

Commands/evidence:

- Static grep for auth/bind/token surfaces in `crates/focusa-api`, CLI, and Pi extension.
- Static grep for route unwraps/panics/input-sensitive functions in API routes.

Findings:

1. **High — API auth is optional and disabled when `FOCUSA_AUTH_TOKEN` is unset.**
   - `crates/focusa-api/src/middleware/auth.rs` explicitly allows all requests if no token is configured.
   - This is acceptable only for local-only bind and trusted local users.

2. **Medium — deployment safety depends on bind address and host firewall.**
   - Service currently uses `FOCUSA_BIND=127.0.0.1:8787`, which mitigates remote exposure.
   - If bound to `0.0.0.0` without `FOCUSA_AUTH_TOKEN`, all mutation routes become exposed.

3. **Medium — route input validation is uneven by route family.**
   - Some newer Pi tools enforce strict TypeBox schemas/no-extra-keys, but Rust route bodies vary.
   - Static grep found many `unwrap/expect` in test modules; runtime unwrap risk appears lower but needs automated classification.

4. **Medium — proxy route shells to `wb` and accepts upstream/proxy auth context.**
   - `crates/focusa-api/src/routes/proxy.rs` uses `tokio::process::Command::new("wb")`.
   - Needs tight input boundary review and command argument review before exposing beyond localhost.

Recommendations:

- Add startup guard: if bind is non-loopback and no auth token exists, daemon should refuse to start or enter read-only degraded mode.
- Add route-level request-size limits and schema tests for mutation routes.
- Classify runtime `unwrap/expect` separately from tests in a static gate.

## Review 4 — Filesystem, process, and privilege boundaries

Commands/evidence:

- Static grep for `Command::new`, shell execution, `systemctl`, deletion, filesystem writes.
- Systemd service inspection after daemon restart.

Findings:

1. **High — Pi extension daemon kickstart executes shell-configured restart commands.**
   - `apps/pi-extension/src/config.ts` default includes `focusa-daemon` background start and `systemctl start/restart focusa-daemon`.
   - Safe for trusted local operator; risky if configuration is attacker-controlled.

2. **Medium — CLI cleanup/wrap command paths execute shell or external commands.**
   - `crates/focusa-cli/src/commands/cleanup.rs` uses `bash -lc` for cleanup command execution.
   - `wrap.rs` uses `script` and harness commands.

3. **Medium — API proxy route executes `wb`.**
   - External command execution should remain argument-vector only, no shell interpolation, and local-only/auth-gated.

4. **Low/positive — production systemd unit has hardening.**
   - `ProtectSystem=strict`, `ReadWritePaths=/home/wirebot/focusa/data`, `NoNewPrivileges=true`, loopback bind, memory caps.

Recommendations:

- Require explicit approval/tool boundary for service restart operations in Pi extension and docs.
- Add static tests that reject shell interpolation in API routes.
- Consider adding `PrivateDevices`, `ProtectHome` compatibility review, and narrower `ReadWritePaths` if feasible.

## Review 5 — Persistence, state integrity, and privacy

Commands/evidence:

- Static grep for SQLite/persistence/data-dir/token/sanitize/raw payload surfaces.
- Runtime service data dir: `FOCUSA_DATA_DIR=/home/wirebot/focusa/data/.focusa`.

Findings:

1. **Medium — Focusa stores cognitive/workflow state locally; privacy classification needed.**
   - Workpoints, metacog, predictions, telemetry, events, evidence refs, and possible prompt/task summaries may be sensitive.

2. **Medium — auth token optional increases risk of local state exfiltration if bind changes or host is multi-user.**
   - Read routes can expose project/task metadata even if mutation routes are not used.

3. **Medium — raw payload policy is partly documented but needs enforcement proof.**
   - Tool docs emphasize evidence refs/handles, not raw provider payloads.
   - Need tests proving predictions/metacog/evidence reject or bound raw payloads/secrets.

4. **Low/positive — persistence is local-first and service write path is constrained by systemd.**

Recommendations:

- Add data classification docs for each persisted store: Focus State, Workpoint, metacog, predictions, events, telemetry, ECS/evidence.
- Add redaction tests for secret-like strings in persisted user-facing summaries where appropriate.
- Add backup/retention policy and secure-delete expectations for local SQLite/event stores.

## Immediate remediation backlog

1. Upgrade/triage npm audit vulnerabilities in `apps/pi-extension` and `apps/menubar`.
2. Add RustSec audit tool/gate (`cargo audit` or `cargo deny`) to CI/release proof.
3. Add daemon startup guard for non-loopback bind without auth token.
4. Add static gate for API route shell execution and runtime unwrap/expect classification.
5. Add persisted-state privacy classification and raw-payload/redaction tests.

## Current posture

- Local daemon live proof is green after rebuild/restart.
- CLI smoke is green.
- Tool suite safe audit is green.
- Disk pressure is reduced enough to proceed.
- Security review found no confirmed committed secret, but did find dependency and deployment-hardening work before any broader exposure or release claim.
