# Spec 177 — Release Channels (Stable / Preview / Nightly) + CI Spend Control

Status: approved direction (operator directive 2026-08-22). Operator is a
dev-channel user on the build machine ("always on nightly here"). Primary
goal: drastically reduce GitHub Actions spend; secondary: professional,
plain channel model. No convoluted processes.

## 1. Vocabulary (plain)

| Channel | Tag pattern | Matrix | Publishes | Cost |
|---|---|---|---|---|
| **stable** | `vX.Y.Z` | Full 14-job Release matrix | Latest, immutable | Expensive, rare — unchanged |
| **preview** | `vX.Y.Z-(dev\|rc)` | Full 14-job Release matrix | Prerelease | Expensive, rare — unchanged |
| **nightly** | `vX.Y.Z-nightly.YYYYMMDD` | **1 OVH self-hosted job, Linux x86_64 musl only** | Rolling prerelease (previous deleted) | zero GitHub-hosted runner minutes |

- Nightly is a **developer convenience lane**, not a stable/preview release. It
  ships the installable Linux x86_64 musl runtime surfaces required by the
  canonical installer: CLI, daemon, TUI, `focusa-session-runner`, Pi extension,
  installer, agent context, and SHA-256 manifest.
- Stable/Preview keep the full matrix per RELEASE_RULES ("no partial
  releases" applies to stable/preview, not to the nightly dev lane which
  the operator explicitly authorizes here).
- Installer already supports all three channels (`--channel=`); nightly
  resolves tags `^v\d+.\d+.\d+-nightly(.\..*)?$`.

## 2. Nightly workflow contract

`.github/workflows/nightly.yml`:

1. Triggers: `schedule` cron `30 14 * * *` (once daily) +
   `workflow_dispatch`. Nothing else. Never on push/PR.
2. `concurrency`: group `nightly`, cancel-in-progress true (a stuck run
   cannot stack).
3. Skip rule: fetch full tag history; if no non-docs commit exists since the
   previous nightly tag, exit early ("nothing new") — zero build cost on quiet days.
4. One job: the authorized OVH `[self-hosted, Linux, X64, ovh-build-2]`
   builder, `timeout-minutes: 20`; never a billing-locked hosted runner.
5. Before compiling, derive `v<stamp>-nightly.YYYYMMDD`, stamp all version
   surfaces, and verify them. Build with the pinned canonical release toolchain,
   `x86_64-unknown-linux-musl`, and `FOCUSA_AUTHORITY_ROOT_KEYS_JSON`.
6. Package the CLI, daemon, TUI, session runner, Pi extension, installer, and
   agent context with names identical to installer resolution. Cover every
   artifact in `SHA256SUMS.txt`, then use the existing protected release-signing
   secret to generate the canonical Ed25519 manifest/provenance set and keyless
   Cosign checksum proof without exposing key material.
7. Publish: delete the previous `*-nightly.*` prerelease + tag (rolling), then
   create the dated prerelease against the exact initiating source SHA.
8. Permissions: `contents: write` only.

## 3. CI spend control (existing workflows)

### Temporary macOS build path — Codemagic

GitHub-hosted macOS is billing-locked and AppVeyor's free public-project
plan is Windows-only. Until GitHub-hosted macOS capacity returns, the
menubar macOS package proof runs on Codemagic cloud M2 through
`codemagic.yaml` workflow `menubar-macos-package-proof`. The complete
provider map, receipt contract, and all-at-once GitHub restoration protocol
are canonical in `docs/178-focusa-temporary-ci-provider-parity-and-github-restoration-spec.md`.

- Scope: `apps/menubar/**` plus `codemagic.yaml`; no ordinary push or PR
  builds consume Mac minutes.
- Trigger: canonical dev/stable release tags (`v*`) only; the release
  controller starts the Codemagic workflow and retains its successful build
  receipt with the release proof.
- Proof: npm dependency install, typecheck, web build, Rust/Tauri `.app`
  package, plist validation, deterministic ad-hoc codesign, and strict
  codesign verification.
- Budget: Codemagic personal account's 500 free macOS M2 minutes/month;
  no paid hosted-Mac switch is authorized by this temporary path.
- Exit: remove `codemagic.yaml` and this subsection only after the GitHub
  hosted macOS lane is restored and proves the same package contract green.

1. `ci.yml`: add `paths-ignore: ['**.md', 'docs/**']` to push+PR triggers;
   extend `cancel-in-progress` to push events (not just PRs).
2. No new crons anywhere. The `billing-bypass-expiry` Azure cron was removed
   with the Azure bypass (not approved for use).
3. Disabled burners stay disabled (`audit-recorder`, watchdog schedule).
4. Any new workflow MUST declare its expected monthly minutes in a header
   comment. Reviewer gate: more than ~60 min/month needs operator approval.

## 4. Dev machine policy

The operator's KnownHost runtime runs the **nightly channel** while OVH owns
its build. Installation uses the canonical installer with an exact rolling tag:
`scripts/install-focusa.sh --channel=nightly --release-tag=<nightly-tag>`.
Never hand-copy binaries. The nightly artifacts are produced by the canonical
GitHub pipeline on OVH and promoted only after version, checksum, authority-root,
health, and rollback checks pass.

## 5. Acceptance

- AC1: nightly.yml exists with cron+dispatch only, one OVH self-hosted job,
  20m timeout, concurrency cancel-in-progress, and zero GitHub-hosted runner minutes.
- AC2: a dispatch run produces `v…-nightly.YYYYMMDD` with every current
  installer-required `x86_64-unknown-linux-musl` runtime/package asset and
  `SHA256SUMS.txt` coverage plus canonical release-manifest, provenance,
  detached Ed25519, and keyless Cosign trust metadata.
- AC3: second dispatch replaces (deletes) the prior nightly — no asset
  accumulation.
- AC4: docs-only day skips with zero build minutes.
- AC5: `install-focusa.sh --channel=nightly --release-tag=<nightly-tag>`
  installs that build on KnownHost; CLI, daemon, TUI, and session runner report
  the nightly version and the daemon proves the compiled authority root.
- AC6: ci.yml paths-ignore + push cancellation merged.
- AC7: while GitHub hosted macOS is unavailable, a release-tag Codemagic
  `menubar-macos-package-proof` build is green and retains its build receipt;
  remove this temporary proof only after the GitHub macOS replacement is green.
