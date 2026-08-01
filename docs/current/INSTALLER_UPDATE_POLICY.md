# Installer and Update Policy

> **Spec 152 release boundary:** New evaluator/customer installation must use an authority-issued signed entitlement. The current Bash/PowerShell `--eval` implementation is legacy pre-Spec-152 behavior and is not an approved distribution path. Until replacement lands, only non-mutating preflight, recovery, repair, export, and uninstall guidance may be presented as approved.

Focusa installers and updates must be explicit, reversible, entitlement-aware, and guarded by Context Authority. Pairing failures, daemon health warnings, stale UI state, or missing license state must never silently become install/update tasks or a locally self-issued Evaluation.

## Install channels

- Source reference/development shell: public BSL checkout and local development build; source visibility is not product entitlement and protected components are absent.
- Official release asset: signed/checksummed CLI/daemon bundle plus authority-issued product entitlement.
- Menubar app bundle: signed Mac app release artifact with entitlement-first onboarding.
- Pi extension package: versioned extension package loaded by Pi; authentication or tool discovery does not create entitlement.
- Protected worker/capsule channel: private signed components delivered only for verified product/features/node posture under Spec 152A.

## Pre-install modes

| Mode | Mutation | Entitlement requirement | Result |
| --- | --- | --- | --- |
| non-mutating preflight | none | none | platform/dependency/path/release-trust plan only |
| recovery/repair | bounded | may run without active lease only to restore licensing/health/export/uninstall surfaces | recovery receipt, never product-ready claim |
| Evaluation install | yes | verified account/email plus authority-issued signed Evaluation lease | bounded evaluation-ready receipt |
| paid/developer install | yes | authority-issued signed product lease | paid/development-ready receipt |
| uninstall | yes | none | managed artifacts removed, user data preserved |
| purge | destructive | none, but separate explicit destructive confirmation | itemized purge receipt |

A dry run must not download/activate runnable product components, create a local tier, extend Evaluation, or mark the node licensed.

## Required entitlement-first install sequence

```text
preflight
→ verify official release metadata/signatures
→ resolve existing signed lease or start device-code activation
→ verify account/email and terms at authority origin
→ issue/register license and node
→ verify lease signature/product/sequence/time/features/limits
→ acquire entitled public/protected artifacts
→ stage and verify
→ atomic activation
→ start daemon in entitled or recovery posture
→ reconcile canonical status
→ optional UIAI grant/child token
→ pairing
→ optional first project/Workpoint
→ lifecycle acceptance receipt
```

The installer must never treat a pairing token, local API token, environment variable, editable JSON/TOML, source checkout, or loopback caller as a license.

## OTA installability and Linux portability gate

A release is OTA-eligible only when the official released CLI resolves all signed trust metadata and returns `apply_allowed=true` for that same immutable tag **and** the canonical entitlement grants the applicable update feature. Required assets include signed manifest, provenance, trusted-key registry, checksums, deploy proof, and—where applicable—protected worker/capsule manifests.

Linux release/deploy artifacts use `x86_64-unknown-linux-musl`. Production AlmaLinux 8 provides GLIBC 2.28; an `x86_64-unknown-linux-gnu` artifact built on `ubuntu-latest` may require a newer GLIBC and is not deployable evidence. The release workflow builds musl, dispatches musl, and the deploy workflow must:

1. install and verify the exact musl daemon;
2. publish signed `deploy-success.json` only after live health/version proof;
3. run the released musl CLI `update plan` against the same tag;
4. require checksum, signature, manifest, provenance, deploy proof, entitlement eligibility, and zero-blocker truth;
5. verify compatible public shell, protected worker/capsule, Pi, TUI, and schema versions;
6. upload `ota-installability-proof-<tag>` before release closure.

`focusa update apply --json` exposes `installed`, `latest`, `applied`, `surfaces`, `rollback`, `next_action`, `blockers`, and `error` at top level. It must also expose a redacted entitlement decision/feature result. `blocked_read_only` or `blocked_entitlement` is a safe refusal, not installation success.

Release waits are observable rather than quiet. Status-query errors and timeouts are explicit failures. Pi agents should use non-blocking release dispatch plus bounded status polls when the harness cannot stream subprocess output.

## Customer lifecycle contract

| Transition | Required behavior | Required proof |
| --- | --- | --- |
| inspect | non-mutating platform, release-trust, existing-install, and redacted entitlement posture | bounded install plan; no license created |
| activate/evaluate | device-code/account verification and authority-issued signed lease | lease signature/product/node/sequence/time proof |
| install | trusted release assets, entitlement gate, atomic activation, daemon/Pi integration | health + coherent versions + entitlement receipt + optional first Workpoint |
| repair/rerun | idempotent declared intent; recovery repair cannot silently grant execution | prior-state backup + repaired health/entitlement posture |
| OTA/update | trusted metadata, anti-rollback, signed update feature, atomic replacement, extension/worker reload and rollback | artifact and entitlement proof + activated versions + rollback receipt |
| uninstall | public uninstall removes managed software and preserves user data by default | managed artifacts absent + data-preservation evidence |
| purge | destructive removal requires explicit separate confirmation | itemized operator approval + purge evidence |

## Approved current examples

Until the authority-issued Evaluation flow ships, active guides may show only non-mutating inspection and data-preserving removal:

```bash
# Inspect the local source/bootstrap plan without claiming Evaluation or installing.
focusa install --preflight --json

# Inspect update posture without applying.
focusa update status --json
focusa update plan --json

# Remove managed software while preserving user data.
curl -fsS https://install.focusa.dev/focusa | bash -s -- --uninstall

# Explicit destructive removal only.
curl -fsS https://install.focusa.dev/focusa | bash -s -- --uninstall --purge-data
```

Target commands such as `focusa license start --product bundle` and device-code polling are normative in Spec 152 but must not be advertised as shipped until implemented and proven.

The legacy `scripts/install-focusa.sh --eval`, PowerShell Evaluation switch, and equivalent curl examples may remain in current code or historical evidence during migration. They are release blockers and must not be recommended for new evaluators.

After install, repair, or update, verify daemon health/version, canonical entitlement state, coherent components, all-Pi-tool discovery, optional UIAI independent entitlement, Mission Canvas, and canonical Workpoint resume. Uninstall remains idempotent when binaries are absent.

## Installer terminal UX policy

Spec 132 makes `focusa install` the owner of terminal presentation. Animated UI is an event consumer only: it renders sanitized preflight/release-trust/device-code-wait/lease-verification/download/service/Pi/UIAI/PATH/cancel/rollback events to stderr and never owns install truth, license issuance, consent, rollback, release selection, or mutation.

Renderer selection:

| Condition | Required behavior |
| --- | --- |
| `--json` | silent presenter; one stdout JSON document |
| `--quiet` | silent except durable errors |
| `--no-animation` / `FOCUSA_INSTALL_UI=plain` | plain presenter |
| CI, non-TTY stderr, `TERM=dumb`, or terminal smaller than 70×22 | plain presenter |
| `NO_COLOR` or `CLICOLOR=0` on suitable TTY | monochrome animated presenter |
| `FOCUSA_REDUCE_MOTION=1` on suitable TTY | reduced-motion presenter |
| suitable color TTY | animated color presenter |

Supported controls remain `FOCUSA_INSTALL_UI=auto|full|mono|reduced|plain`, `FOCUSA_INSTALL_SEED=<u64>`, and `FOCUSA_REDUCE_MOTION=0|1`. Invalid values fail before mutation. Terminal failures restore terminal state, warn once, and continue in plain mode.

Dynamic strings must redact:

- raw license keys;
- device-code secrets and polling credentials;
- bearer/refresh/child tokens;
- authorization headers;
- signed lease account/customer identity;
- unmasked emails;
- capsule content keys and envelopes;
- sensitive query parameters.

A short human-entered `user_code` may be displayed only for its intended activation step and must expire quickly.

Pi integration is Rust-owned: Pi absence may be skipped; archive/dependency/setup failure is truthful degraded state and must not falsely fail a licensed core install. Pi readiness never supplies entitlement.

## Required preflight

Before replacing binaries, restarting production services, or installing release assets, run the applicable Context Authority preflight and canonical entitlement preflight. Example action preflight:

```bash
focusa action preflight \
  --current-ask "$CURRENT_ASK" \
  --kind binary_replace \
  --target /usr/local/bin/focusa \
  --source github_release_asset \
  --install-role live_build_host \
  --project-root "$PWD" \
  --json
```

If either context or entitlement returns `block`, `operator_required`, `recovery_only`, or unsupported schema, stop mutation and report the exact recovery path.

## Live build host policy

On a live build host, prefer the canonical repository/release pipeline rather than release-asset replacement shortcuts. Keep rollback binary/service/config state. An authority-issued developer license is required for protected developer components and is never inferred from repository access.

## Update checklist

1. Verify project identity and current Workpoint when project-scoped.
2. Verify current lease state/sequence/product/update features without logging secrets.
3. Check git status/diff and release version consistency.
4. Run tests relevant to changed artifacts.
5. Verify manifest, checksum, signature, provenance, and protected-component compatibility.
6. Snapshot current binary/config/service/worker state, excluding raw secrets.
7. Stop/restart services only when authorized.
8. Run daemon/UIAI health, canonical status, and release proof.
9. Capture evidence and rollback if health, entitlement, or proof fails.

## Rollback checklist

- Restore previous trusted binary/app/worker/capsule set.
- Restore service unit/config when changed.
- Never roll back entitlement sequence, revocation state, or absolute Evaluation expiry.
- Restart only after context and entitlement-safe preflight.
- Run health, version, canonical license status, and smoke checks.
- Link rollback evidence to the lifecycle receipt/Workpoint without secret values.

## Forbidden substitutions

- Pairing troubleshooting must not trigger install/update work by default.
- Stale menubar UI must try refresh/reconnect before reinstall.
- Release replacement on a live build host requires Context Authority approval.
- No update path may publish or exfiltrate private Workpoint/Evidence data.
- Missing authority connectivity cannot create a fresh Evaluation; only a previously signed offline window may continue.
- An updater cannot infer commercial/developer status from environment variables or local files.
- A healthy public gateway cannot substitute for a missing protected worker or product grant.

## Combined proof

- Existing installer policy guard: `tests/installer_update_policy_static_test.sh`
- Documentation consistency: `tests/spec152_documentation_consistency_gate.py`
- Lifecycle integration: Spec 150A
- Entitlement authority: Spec 152
- Protected distribution: Spec 152A
- Related active docs: `COMMERCIAL_PACKAGING.md`, `FIRST_RUN_FLOW.md`, `FOCUSA_FRIENDLY_ONBOARDING.md`
