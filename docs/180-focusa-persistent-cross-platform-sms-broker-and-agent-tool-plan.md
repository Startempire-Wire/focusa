# Focusa persistent cross-platform SMS broker and agent tool plan

Status: operator-directed implementation plan
Priority: P0 release-critical GitHub SMS lane; P1 general customer SMS product
Authority: Sir V3 direction, Spec 156 credential authority, Spec 178 release recovery, `focusa-cross-platform-sms-api-wa38m`
Date: 2026-08-29

## 1. Outcome

One final customer-approved phone pairing creates restart-durable private SMS access. Focusa agents can complete an active `github.com` SMS challenge through one-time broker injection without receiving message history, cookies, pairing state, Google/Apple credentials, or the OTP value.

The same broker later exposes separately granted thread, bounded-read, send, and event tools. Android/Google Messages is the release-critical bootstrap. iPhone/iOS remains a parallel first-class connector using only Apple-supported, user-consented routes.

## 2. Failure being corrected

The first Google Messages pairing lived only in an in-memory Chromium `BrowserContext`. Roughly 3.35 MB of cookie/IndexedDB/cache/local-storage state existed, but no durable encrypted checkpoint was created. Browser/context loss returned the connector to `/pair`.

This design forbids that state shape. Enrollment cannot report `paired` until an encrypted checkpoint exists and a fresh runtime restored from that checkpoint proves `ready`.

## 3. Non-negotiable boundaries

1. `os.focusa.dev` remains a credential-free public-demo trust class. Its Chrome profile, webtop container, public stream, extension, and demo daemon never store SMS/provider auth.
2. The SMS appliance has no public ingress. Browser CDP and broker HTTP listen only on private loopback/Unix socket; remote administration requires the authorized tailnet plus local peer authorization.
3. Pairing/profile/cookie/IndexedDB/service-worker state is P4. It never enters Focusa SQLite, model context, evidence, logs, screenshots, receipts, shell arguments, git, or release assets.
4. Runtime browser state exists only in a private tmpfs directory. Durable state is a bounded set of encrypted, atomic, mode-0600 checkpoint generations plus value-free metadata; at least current and previous verified generations are retained for rollback.
5. GitHub MFA is SMS-first. Connector degradation triggers repair/restore, not silent GitHub Mobile/passkey/TOTP substitution. Alternative renewable methods require explicit Sir V3 direction. Recovery-code automation is permanently forbidden.
6. `read_otp`/`inject_otp` never implies inbox, thread, notification, or send access.
7. Immutable release tags `v0.9.185` and `v0.9.186` remain unchanged. The appliance is private operator infrastructure; it cannot alter/tag/repackage release payloads.

## 4. Architecture

```text
Android Messages / future iPhone connector
                |
       connector-specific session
                |
 private browser/connector process
 runtime profile: private tmpfs only
                |
 encrypted atomic checkpoint (P4)
                |
 focusa-sms-broker (loopback/Unix socket)
      | policy | challenge | audit |
      +--------+-----------+-------+
       CLI / HTTP / MCP / Pi / OpenClaw
           scoped capabilities only
```

### 4.1 Private runtime and checkpoint

- Runtime root: `${XDG_RUNTIME_DIR}/focusa-sms-broker/`; owner-only, mode 0700, fail on symlink or foreign owner.
- Chromium profile: `${XDG_RUNTIME_DIR}/focusa-sms-broker/google-messages-profile`; launched by a dedicated private process, never the public Veragensia Chrome.
- Durable ciphertext: `${XDG_STATE_HOME}/focusa/sms-broker/google-messages-profile.tar.zst.aesgcm`; mode 0600, owner-only parent.
- Value-free metadata: `${XDG_STATE_HOME}/focusa/sms-broker/connector-state.json`; schema/version, connector ID, status, checkpoint generation, created/checked timestamps, ciphertext digest, and no URL/query, account, phone, sender, message, cookie, registration, pairing, or OTP values.
- Unlock identity: machine-bound protected credential supplied to the service through the approved credential authority/system credential surface. It is never an environment value exposed to agents and never committed beside ciphertext.
- Start: authenticate owner and paths; create tmpfs runtime; decrypt checkpoint; reject traversal/symlinks/foreign ownership; unpack; launch private Chromium; prove connector readiness.
- Checkpoint: pause the source browser without disposing it; omit rebuildable caches; reject links/foreign ownership/special files; archive; encrypt to same-filesystem temporary file; fsync file and directory; authenticate the generation; atomic rename; retain encrypted rollback generations; resume the source.
- Checkpoint triggers: successful enrollment, connector state change, bounded periodic dirty-state checkpoint, graceful shutdown, and explicit `focusa sms checkpoint`.
- Enrollment handoff order is fixed: prove source readiness; pause source; checkpoint; restore into an isolated successor profile/process; require two semantic readiness probes; only then retire the source. Any failure kills the successor, removes its plaintext runtime, resumes the original source, and reports `handoff_rolled_back`.
- Enrollment returns `paired_persisted` only after this source-preserving handoff succeeds. URL/title checks are insufficient: readiness requires the Messages conversations origin/route, no unable-to-connect state, and an operational bounded conversation-list probe.

### 4.2 Broker authority

The daemon owns:

- connector lifecycle: `unconfigured`, `enrolling`, `checkpointing`, `restoring`, `ready`, `degraded`, `revoked`;
- grants: capability, provider, active challenge handle, enrolled phone handle, expected sender/message class, expiry, consumer identity, use count;
- one-time OTP match/injection with baseline-before-request and new-message-after-request semantics;
- replay rejection, duplicate suppression, expiry, rate limits, audit attribution, and revocation;
- connector adapters, never raw browser state.

Agent-visible audit records contain only request/grant/challenge handles, connector class, timestamps, status/failure class, consumer identity, and redacted evidence references.

### 4.3 Public contracts

Release-critical capability:

- `POST /v1/sms/otp/challenges` — register active provider challenge; returns redacted challenge handle.
- `POST /v1/sms/otp/inject` — find exactly one eligible post-baseline OTP and inject into the bound private browser target; returns `tool_result_v1` with `injected=true`, never the value.
- `GET /v1/sms/health` — value-free connector/checkpoint status.
- `POST /v1/sms/checkpoint` — owner-only durable checkpoint.
- `POST /v1/sms/revoke` — owner-only revoke, dispose runtime, and cryptographically erase protected unlock authority/ciphertext under explicit confirmation.

CLI:

```text
focusa sms health --json
focusa sms enroll --connector google-messages --json
focusa sms checkpoint --json
focusa sms otp challenge --provider github.com --target-handle <handle> --ttl 5m --json
focusa sms otp inject --challenge-handle <handle> --json
focusa sms revoke --connector <id> --json
```

Pi/MCP/OpenClaw tools:

```text
focusa_sms_health
focusa_sms_otp_challenge
focusa_sms_otp_inject
focusa_sms_checkpoint
focusa_sms_revoke
```

Separately granted tools: `focusa_sms_threads`, `focusa_sms_read_thread`, `focusa_sms_search`, `focusa_sms_send`, and `focusa_sms_events`. They are implemented but remain inaccessible to an OTP-only grant.

## 5. Five-hour release-critical lane

### Phase R0 — freeze and protect (0–15 minutes)

1. Preserve the immutable release tag and current draft.
2. Confirm public Veragensia health and zero viewers; do not use it for enrollment.
3. Create private owner-only runtime/state roots and the dedicated appliance lifecycle.
4. Prove paths, process ownership, no public listener, and ciphertext-only durable state.

### Phase R1 — one final pairing with durable proof (15–35 minutes)

1. Start the private enrollment runtime with a new persistent profile in tmpfs.
2. Sir V3 performs the one required phone pairing/consent action.
3. Broker observes `ready` without reading any message.
4. Pause—but do not dispose—the paired source and create/verify an encrypted atomic generation.
5. Restore that generation into a separate tmpfs profile/process while the source remains recoverable.
6. Require two semantic `ready` probes from the restored successor.
7. Only after both proofs, retire the source and report `paired_persisted`. If restore/probe fails, destroy only the successor, resume the source, and do not accept enrollment or start release authentication.

### Phase R2 — GitHub SMS and AppVeyor authority (35–55 minutes)

1. Open a fresh private GitHub/AppVeyor auth context.
2. Register the active `github.com` challenge before requesting SMS.
3. Request GitHub SMS.
4. Broker matches one new eligible OTP and injects it directly into that exact target.
5. Create renewable AppVeyor API authority and store it in Bitwarden.
6. Provision secure AppVeyor `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` variables without revealing values.
7. Cancel obsolete guaranteed-failure builds; trigger one exact-tag/SHA Windows recovery.
8. Checkpoint the still-ready SMS connector again and dispose provider auth context after mutation.

### Phase R3 — provider/release completion (55 minutes–4 hours 30 minutes)

1. Require terminal green Codemagic exact-tag/SHA macOS updater signatures.
2. Require AppVeyor x64 + ARM64 Rust, NSIS, MSI, updater signatures, exact tag/SHA, and a successful typed pull-intake receipt.
3. Dispatch canonical Spec 178 recovery through the consolidated 385-minute intake plus five-minute completeness gate.
4. Verify candidate lock, asset matrix, hashes, updater metadata, trust roots, installers, and all 14/14 release gates.
5. Publish only through the canonical finalizer; verify Latest and updater/installer consumption.
6. Disable both provider recovery records after closure.

### Phase R4 — release handoff (by 5 hours)

- Release `v0.9.186` public and Latest, or fail closed with the exact remaining provider/gate—not a partial release.
- SMS connector remains encrypted, restart-durable, private, healthy, and available to authorized agents.

## 6. Product implementation workbreakdown

Exact planned Focusa surfaces:

- `scripts/focusa-sms-appliance.py` — sole DRY authority for atomic authenticated generations, safe archive/restore, rollback, guarded handoff, readiness promotion, and explicit cryptographic revoke.
- `scripts/focusa-sms-supervisor.py` — managed enrollment/restore/browser/broker owner, source-preserving handoff, periodic/shutdown checkpoint, crash recovery, and watchdog.
- `scripts/focusa-sms-ready-probe.mjs` — value-free semantic Messages readiness probe.
- `scripts/focusa-google-messages-broker.py` — private Android/Google Messages adapter with read-only grant projections, separate usage ledger, challenge baseline, exact target injection, replay rejection, and supervisor control.
- `scripts/install-focusa-sms-appliance-service.sh` and `templates/systemd/focusa-sms-appliance.service` — encrypted-credential, restart-always, watchdog-managed private service.
- `tests/sms_appliance_checkpoint_test.py` — round-trip, permissions, redaction, generation rollback, symlink rejection, source-resume rollback, and explicit revoke.
- `tests/sms_broker_contract_test.py` and `tests/sms_pair_once_lifecycle_static_test.py` — capability isolation, replay ledger, consumer surfaces, lifecycle order, semantic gate, and managed service posture.
- `crates/focusa-core/src/sms.rs` — domain contracts, lifecycle reducer, grant policy, redacted audit types.
- `crates/focusa-core/src/lib.rs` — module export.
- `crates/focusa-api/src/routes/sms.rs` — loopback authenticated health/enroll/checkpoint/revoke/challenge/inject routes.
- `crates/focusa-api/src/server.rs` — route registration.
- `crates/focusa-cli/src/main.rs` — `focusa sms` command hierarchy.
- `apps/pi-extension/src/sms-tools.ts` — thin `tool_result_v1` tools.
- `apps/pi-extension/src/index.ts` — registration only.
- `docs/current/AGENT_ADAPTER_CONTRACT.md` and `docs/current/NON_PI_AGENT_ADAPTER_EXAMPLES.md` — connector-neutral CLI/HTTP/MCP use.
- `docs/current/PERSISTED_STATE_PRIVACY_CLASSES.md` — P4 encrypted connector checkpoint classification.
- `tests/sms_broker_static_test.sh` — no raw values/profile leakage and contract presence.
- `crates/focusa-core/tests/sms_broker_contract.rs` — lifecycle/grant/replay/redaction tests.
- `crates/focusa-api/tests/sms_routes_consumer.rs` — consumer-side auth/scope/envelope tests.
- `apps/pi-extension/tests/sms-tools.test.mjs` — thin adapter and no-plaintext tests.

Connector implementations remain behind a versioned trait/adapter boundary. Google Messages/Android-specific selectors, CDP, storage, and lifecycle details cannot enter shared request/response types. The iPhone implementation task starts before shared v1 contracts freeze.

## 7. Acceptance proofs

### Persistence

1. Pair once; checkpoint verifies.
2. Kill private Chromium; restore; `ready`.
3. Restart appliance container/process; restore; `ready`.
4. Restart host or equivalent controlled boot proof; restore; `ready`.
5. Durable disk scan finds ciphertext + value-free metadata only; runtime plaintext is absent after stop.
6. Corrupt/current-generation test rolls back to last verified encrypted generation and reports degraded truthfully.

### OTP safety

1. Pre-existing matching messages cannot satisfy a new challenge.
2. Wrong provider/sender/class/phone/target/expiry is rejected.
3. Exactly one post-baseline OTP is injected once; value is absent from model, CLI argv, logs, screenshots, events, receipts, evidence, Focusa stores, and shell history.
4. Replay, duplicate delivery, expired grant, revoked connector, and ambiguous multiple candidates fail closed.
5. OTP grant cannot list/read threads or send messages.

### Trust isolation

1. Public Veragensia contains zero Google/GitHub/AppVeyor cookies or private profile state and has zero credential-bearing mounts.
2. No SMS/CDP/broker listener is publicly routable.
3. Only expected owner/peer can call broker; cross-UID requests fail.
4. Revoke terminates connector/browser, invalidates grants, removes runtime plaintext, and destroys durable unlock/ciphertext under explicit authorization.

### Production consistency

- Versioned contracts.
- Producer tests.
- Consumer-side CLI/API/Pi/MCP/OpenClaw tests.
- Cross-version interop.
- Live Android and iPhone real-device evidence before cross-platform product completion.

## 8. Rollback

- Before enrollment: stop private appliance; remove empty runtime; retain no state.
- After checkpoint but before agent enablement: stop runtime and retain encrypted checkpoint while grants stay disabled.
- Connector defect: disable grants, stop browser, keep last verified encrypted generation, report `degraded`.
- Security concern: explicit revoke destroys grants, runtime, encrypted generations, and machine unlock authority; re-pair required.
- Release work never falls back to recovery codes or a public credential-bearing desktop.

## 9. Done condition

This plan is complete only when one pairing survives fresh-runtime, process/container, and controlled restart proofs; GitHub SMS can be injected through an authorized one-time tool without value exposure; public Veragensia remains credential-free; revocation is proven; and all client surfaces share the same capability contract. The general SMS product is complete only after real Android and iPhone proofs.
