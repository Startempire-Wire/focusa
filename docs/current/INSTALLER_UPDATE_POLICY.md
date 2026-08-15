# Installer and Update Policy

Focusa installers and updates must be explicit, reversible, and guarded by Context Authority. Pairing failures, daemon health warnings, or stale UI state must never silently become install/update tasks.

## Install channels

- Source build: developer/operator checkout, local build, manual service restart.
- Release asset: signed/checksummed CLI/daemon bundle for non-live build hosts.
- Menubar app bundle: signed Mac app release artifact.
- Pi extension package: versioned extension package loaded by Pi.

## OTA installability and Linux portability gate

A release is OTA-eligible only when the official released CLI resolves all signed trust metadata and returns `apply_allowed=true` for that same immutable tag. Required assets include signed manifest, provenance, trusted-key registry, checksums, and `deploy-success.json` proof.

Linux release/deploy artifacts use `x86_64-unknown-linux-musl`. Production AlmaLinux 8 provides GLIBC 2.28; an `x86_64-unknown-linux-gnu` artifact built on `ubuntu-latest` may require a newer GLIBC and is not deployable evidence. The release workflow builds musl, dispatches musl, and the deploy workflow must:

1. install and verify the exact musl daemon;
2. publish signed `deploy-success.json` only after live health/version proof;
3. run the released musl CLI `update plan` against the same tag;
4. require checksum, signature, manifest, provenance, deploy-proof, and zero-blocker truth;
5. upload `ota-installability-proof-<tag>` before release closure.

`focusa update apply --json` exposes `installed`, `latest`, `applied`, `surfaces`, `rollback`, `next_action`, `blockers`, and `error` at top level. `blocked_read_only` is a safe trust refusal, not an installation success; agents must report its blockers and must never bypass trust.

Release waits are observable rather than quiet: the canonical tag script reports discovery, run URL, elapsed heartbeat, per-job state, failed job/step names, a bounded error/assertion excerpt, and the exact full-log recovery command. Status-query errors and timeouts are explicit failures. Pi agents should use non-blocking release dispatch plus bounded status polls when the harness cannot stream subprocess output.

## Customer lifecycle contract

| Transition | Required behavior | Required proof |
| --- | --- | --- |
| inspect | `scripts/install-focusa.sh --dry-run --eval` performs no mutation | bounded install plan |
| install | signed/checksummed release assets, atomic activation, daemon/Pi integration | health + version + first Workpoint |
| repair/rerun | rerunning the same channel is idempotent; `--force` is explicit for downgrade/overwrite | prior state backup + repaired health |
| OTA/update | trusted release metadata, anti-rollback, atomic replacement, extension reload/rollback | artifact checksum/signature + activated version + rollback receipt |
| uninstall | public `--uninstall` removes managed software and preserves user data by default | managed artifacts absent + data-preservation evidence |
| purge | destructive data removal requires `--uninstall --purge-data` | explicit operator approval + purge evidence |

Public examples:

```bash
bash scripts/install-focusa.sh --dry-run --eval
curl -fsS https://install.focusa.dev/focusa | bash -s -- --eval
curl -fsS https://install.focusa.dev/focusa | bash -s -- --uninstall
# Explicit destructive removal only:
curl -fsS https://install.focusa.dev/focusa | bash -s -- --uninstall --purge-data
```

After install, repair, or update, verify daemon health/version, all-Pi-tool discovery, Mission Canvas, and canonical Workpoint resume. Uninstall must remain idempotent when binaries are already absent.

## Installer terminal UX policy

Spec 132 makes `focusa install` the owner of terminal presentation. Animated UI is an event consumer only: it renders sanitized phase/download/verification/service/Pi/PATH/cancel/rollback events to stderr and never owns install truth, rollback decisions, release selection, license validation, or file mutation.

Renderer selection:

| Condition | Required behavior |
| --- | --- |
| `--json` | silent presenter; one stdout JSON document |
| `--quiet` | silent except durable errors |
| `--no-animation` / `FOCUSA_INSTALL_UI=plain` | plain presenter |
| CI, non-TTY stderr, `TERM=dumb`, or terminal smaller than 70×22 | plain presenter |
| `NO_COLOR` or `CLICOLOR=0` on suitable TTY | monochrome animated presenter |
| `FOCUSA_REDUCE_MOTION=1` on suitable TTY | reduced-motion presenter |
| truecolor/256-color capable suitable TTY | animated color presenter |

Supported controls: `FOCUSA_INSTALL_UI=auto|full|mono|reduced|plain`, `FOCUSA_INSTALL_SEED=<u64>`, and `FOCUSA_REDUCE_MOTION=0|1`. Invalid values fail preflight before mutation. Terminal failures restore the cursor/alternate screen, warn once, and continue in plain mode. Dynamic strings are sanitized and redact license keys, authorization headers, sensitive query parameters, and emails before animated display.

Pi integration is Rust-owned: Pi absent is skipped; verified installation is success; archive/dependency/setup failure is a warning and must not falsely fail core Focusa install.

## Required preflight

Before replacing binaries, restarting production services, or installing release assets, run:

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

If preflight returns `block` or `ask_operator`, stop mutation and report the recovery path.

## Live build host policy

On a live build host, prefer local repo build/restart over release asset replacement. A live daemon host must keep a rollback binary/service unit path before mutation.

## Update checklist

1. Verify project identity and current Workpoint.
2. Check git status/diff and release version consistency.
3. Run tests relevant to changed artifacts.
4. Verify checksum/signature for downloaded release assets.
5. Snapshot current binary/config/service state.
6. Stop/restart service only when authorized.
7. Run daemon health and `focusa release prove --tag <tag>`.
8. Capture evidence and rollback if health/proof fails.

## Rollback checklist

- Restore previous binary or app bundle.
- Restore previous service unit/config when changed.
- Restart daemon only after operator-safe preflight.
- Run health, version, and smoke checks.
- Link rollback evidence to Workpoint.

## Forbidden substitutions

- Pairing troubleshooting must not trigger installer/update work by default.
- Stale menubar UI must first try refresh/reconnect before reinstall.
- Release asset replacement on a live build host requires Context Authority approval.
- No auto-update path may publish or exfiltrate private Workpoint/Evidence data.

## Pi extension package transaction policy

- The installer stages and verifies the archive and dependencies; activation
  flows through the shared transaction in
  `crates/focusa-cli/src/commands/pi_package.rs`.
- **One canonical Focusa Pi package** (`focusa`, identity `focusa-pi-bridge`)
  is loadable per discovery root. Backups, stages, legacy, rollback, disabled,
  and quarantine copies are preserved under the sibling non-discovery root
  `~/.pi/agent/retired-extensions/` — never under the active discovery root.
- Retirement moves only verified Focusa-owned entries: package identity
  `focusa-pi-bridge` plus a known legacy/backup/old entry-name pattern.
  Unrelated extensions are never moved. Compatibility symlinks may resolve
  only to the canonical target.
- Activation returns a **typed activation receipt** (destination, prior
  backup, retired entries). The OTA apply transaction retains the receipt in
  its state (`pi-extension-activation.json`), records the Pi entry in the
  rollback ledger, and:
  - rolls the Pi package back together with promoted binary parts after any
    downstream failure;
  - commits (removes) the prior-package backup only after the
    restart-required receipt and every package phase succeed.
- Fault injection: `FOCUSA_UPDATE_FAULT_AFTER_PI_ACTIVATION=1` fails the
  update immediately after Pi activation; tests must prove the exact prior
  package is restored.
- `-ne`/`--no-extensions` **never satisfies acceptance**: a fresh Pi process
  must start with zero extension-load, duplicate-tool, and duplicate-flag
  errors.

## Proof

- Static guard: `tests/installer_update_policy_static_test.sh`
- Related: `COMMERCIAL_PACKAGING.md`, `FIRST_RUN_FLOW.md`, `SECURITY_MODEL.md`
