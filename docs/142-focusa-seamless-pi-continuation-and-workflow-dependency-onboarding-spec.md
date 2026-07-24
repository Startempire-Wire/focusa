# Spec 142 — Focusa Seamless Pi Continuation and Workflow Dependency Onboarding

**Status:** operator-required pre-release blocker  
**Project:** Focusa  
**Authority:** operator steering, 2026-07-24  
**Supersedes:** any behavior that blocks a Pi prompt at high context pressure or treats workflow dependencies as optional undocumented setup

## 1. Goal

A customer installs Focusa once and reaches a verified, usable agent workflow without hidden prerequisites. Pi prompts must keep flowing through compaction without manual command/resend work.

## 2. Release blockers

1. Focusa must not return `handled` for an ordinary high-pressure Pi prompt.
2. Pi native threshold/overflow compaction must retain text, images, and steering and retry automatically.
3. Pi prompt-critical hooks (`input`, `before_agent_start`, `context`) must not await Focusa daemon/network work.
4. Numeric pressure values and file/log artifacts must not become project aliases or scope conflicts.
5. Onboarding must inventory, install with consent, and verify Node.js, npm, Pi, the bundled Focusa Pi extension, and UIAI Engine connectivity.
6. “Full functionality ready” is false until all required checks pass or an unsupported local UIAI platform has a verified remote endpoint.

## 3. Prompt-flow contract

### 3.1 High-pressure input

`apps/pi-extension/src/auto-compaction.ts` records pressure telemetry and returns `continue`. Pi remains the prompt owner and performs native threshold/overflow compaction with `willRetry=true`.

The normal path must not display or require:

```text
/focusa-rollover execute
resend it in the replacement session
```

`/focusa-rollover` remains an explicit session-maintenance command, not a prerequisite for ordinary prompt delivery.

### 3.2 Hot-path latency

The following Pi hooks use only in-memory/cached Focusa projections:

- `input`
- `before_agent_start`
- `context`

Daemon refresh, frame rescope, writer context, telemetry, and rich ontology refresh are background/best-effort. Daemon failure may degrade Focusa enrichment but must not delay or reject the operator prompt.

### 3.3 Scope parsing

Project alias extraction rejects:

- numeric versions/percentages such as `129.5`;
- artifact filenames ending in `.log`, `.txt`, `.md`, `.json`, `.jsonl`, `.yaml`, `.yml`, `.toml`, `.rs`, `.ts`, `.js`, `.mjs`, `.sh`, or `.py`;
- absolute paths quoted inside errors, stack traces, logs, or tool output unless the operator also uses explicit project-switch language.

Current-ask alignment and operator steering outrank stale invalid ledger observations. Scope uncertainty gates durable project writes, never conversation or read-only diagnosis.

## 4. Workflow dependency contract

### 4.1 Required inventory

| Dependency | Detection | Required result |
|---|---|---|
| Node.js | `node --version` | major version 20 or newer |
| npm | `npm --version` | exits 0 |
| Pi | `pi --version` | supported `@earendil-works/pi-coding-agent` installation |
| Focusa Pi extension | managed extension `package.json` plus dependency install/check | bundled release version activated atomically |
| UIAI Engine | `/v1/health` on configured endpoint | healthy endpoint before full-ready status |

Bootstrap utilities (`curl`, `python3`, checksums, `tar`) remain required.

### 4.2 Consent and reruns

- Interactive public bootstrap offers the full workflow dependency install with an explicit yes/no prompt.
- Unattended installation requires `--install-dependencies --assume-yes`.
- Dependency commands are ordered: Node.js → npm → Pi → Focusa Pi extension.
- Reruns detect already-present dependencies and do not reinstall them.
- Existing Pi extensions are staged, checked, and atomically replaced by the existing Focusa integration path.
- A declined or unsupported dependency produces an exact command, expected result, and recovery path; it may not be reported as full-ready.

### 4.3 UIAI platform matrix

| Platform | Onboarding mode |
|---|---|
| Linux amd64 | verified local binary is supported from `WPUIAI/uiai-engine` release `engine-vw20-multipool-20260705-2119`; verify the published `.sha256` before activation |
| Other platforms | configure a remote/private UIAI endpoint and prove `/v1/health`; local install remains unsupported until a checksummed upstream artifact exists |

UIAI source/release authority: `https://github.com/WPUIAI/uiai-engine`. The release asset is `uiai-engine-linux-amd64`; the published GitHub digest is `sha256:963883a19eec91c81ee88bc70c23e8db77f0cc12c673be872f6ee3bda3bba5b5`.

No installer may silently use an unverified “latest” binary.

### 4.4 Ownership safety and automatic repair

- Release preflight detects root-owned files when the checkout itself belongs to a non-root project user.
- On the managed KH release host, a root-run gate uses the approved `fix-user-perms <project-user>` helper and rechecks before build/test work.
- A non-root gate fails fast with the exact repair command instead of continuing with mixed ownership.
- Public installers create and atomically replace only Focusa-managed paths under the invoking/target user. They must not recursively chown a customer home directory.
- Release evidence includes the ownership preflight result so root-run agents cannot silently leave an uneditable checkout.

## 5. Exact implementation surfaces

1. `apps/pi-extension/src/auto-compaction.ts`
2. `apps/pi-extension/src/turns.ts`
3. `apps/pi-extension/src/state.ts`
4. `apps/pi-extension/tests/auto-compaction-resilience.test.mjs`
5. `crates/focusa-cli/src/commands/install.rs`
6. `scripts/install-focusa.sh`
7. release/static gates under `tests/`
8. onboarding/customer docs and release notes that describe the verified flow

## 6. Acceptance checks

1. A 95%+ simulated Pi input returns `continue`, never `handled`.
2. The stale manual rollover/resend error string is absent from the extension.
3. Prompt-critical hooks contain no awaited daemon/network operation.
4. `129.5` and `focusa-final-ci-spec-pirpc.log` cannot create a project-scope conflict.
5. Pi extension typecheck, lint, formatting, and resilience tests pass.
6. Installer JSON preflight lists Node.js, npm, Pi, Focusa Pi extension, and UIAI readiness.
7. Dependency installation tests prove ordered/idempotent install plans and consent behavior.
8. Bundled Pi extension installation is checksum-verified and activated only after npm check passes.
9. UIAI local activation verifies the pinned checksum, or remote mode verifies `/v1/health`.
10. A fresh onboarding proof reaches full-ready status without undocumented manual setup.
11. Release preflight finds no root-owned source files under a non-root-owned checkout and auto-repairs on the managed root-run release host.
12. Final CI and release gates are green only after all checks above pass.

## 7. Rollback

- Pi extension prompt-flow changes: restore prior package version through Focusa OTA rollback; Pi native compaction remains available.
- Dependency installation: do not remove customer-owned Node/Pi/UIAI installations automatically. Restore only Focusa-managed extension backups and Focusa-managed UIAI service/config files.
- UIAI activation: stop only the Focusa-managed process/service, restore its previous managed binary/config, and preserve browser/user data.
- Failed installer reruns use the existing transaction journal/stash and must not corrupt an existing Focusa or Pi installation.

## 8. Evidence

Required proof artifacts:

- Pi prompt-flow resilience test log;
- fresh Pi RPC/TUI over-limit continuation transcript with one delivery of operator steering;
- dependency preflight/install JSON for clean and rerun fixtures;
- bundled Pi extension activation proof;
- UIAI checksum and health proof (local or remote mode);
- final strict CI run and release artifact verification.
