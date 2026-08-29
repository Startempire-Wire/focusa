# 178 — Focusa Temporary CI Provider Parity and GitHub Restoration Spec

**Status:** active temporary release routing
**Effective:** 2026-08-24
**Supersedes:** no release authority; extends 177 and `docs/current/RELEASE_RULES_2026-08-19.md`

## 1. Purpose

GitHub-hosted runner admission is account-wide billing-locked. Cirrus is
defunct. This spec records the complete temporary release build route so any
agent can execute and audit a release without guessing, waiting on an
unavailable GitHub-hosted Linux/macOS/Windows row, or silently substituting a
provider.

This is a temporary provider delegation, not a new release channel or a
second release authority. Tags, manifest freshness, release evidence, and
the canonical release scripts remain the authority.

## 2. Active provider map

| Required surface | Temporary provider | Entry point | Required proof | Current limitation |
|---|---|---|---|---|
| Linux daemon, CLI, API, specs | GitHub Actions self-hosted `host-focusa-deploy` | `.github/workflows/ci.yml`, `release.yml` | Rust, Spec Gates, release automation, meaningful commits green | Shared production host; Rust exit-241 flake is tracked separately. |
| Windows binaries and menubar packages | AppVeyor public-project lane | `.appveyor.yml` | MSVC builds/tests; tagged CLI, daemon, TUI, NSIS, MSI, and updater-signature assets | One concurrent public-project job; both target architectures run serially. |
| macOS menubar package proof | Codemagic cloud `mac_mini_m2` | `codemagic.yaml`, workflow `menubar-macos-package-proof` | npm ci, typecheck, web build, Rust/Tauri `.app`, plist lint, ad-hoc codesign and verification green | Proof is ad-hoc signed; it is not notarized customer distribution. |
| GitHub-hosted Linux/macOS/Windows jobs | temporarily non-authoritative | `ci.yml`, `spec132-terminal-matrix.yml`, `release.yml` | Informational only while account admission is billing-locked | Must not be silently deleted or individually re-enabled. The monolithic Spec 132 receipt is substituted by exact-SHA self-hosted CI plus the downstream AppVeyor/Codemagic receipt gates. |

## 3. Canonical temporary release procedure

1. Run the normal canonical preflight and create the requested dev or stable
tag through the existing release scripts. Exact candidate CI remains mandatory;
the billing-locked Spec 132 workflow is recorded as substituted and its external
surfaces remain publication gates. Never hand-build, hand-copy, or hand-publish
an artifact.
2. The matching `v*` tag automatically triggers both Codemagic release
workflows and AppVeyor; release tags are never filtered by changed paths.
3. Require Linux/self-hosted GitHub evidence plus AppVeyor Windows and
Codemagic macOS evidence for the exact tagged commit.
4. External providers wait boundedly for the canonical GitHub workflow to
create its gated draft Release. They never create a Release, skip missing
credentials, or swallow asset-upload failures.
5. Menubar receipts include both install bundles and Tauri updater signatures
for macOS and Windows. The canonical checksums job generates `latest.json`
from those signatures only after every required provider receipt is present.
6. Retain successful provider build IDs/receipts beside the release proof. A
GitHub hosted-macOS failure is expected during this temporary route and is not
a substitute for, or a failure of, the Codemagic proof.
7. Treat an absent, failed, wrong-commit, unsigned, or undisclosed-mode
provider proof as a release blocker. `beta_ad_hoc` is permitted only with the
canonical pre-license consent markers; `production_notarized` requires Apple
authority and notarization evidence.
8. Publish only after all required proof surfaces for that release are green.
9. If a tag-triggered controller run was blocked before draft creation, resume
it through `Release` `workflow_dispatch` with both the immutable `release_tag`
and its full `release_sha`. The controller must verify that the tag, input SHA,
and checkout HEAD are identical before creating or modifying the Release. The
candidate-lock JSON is attached to the draft Release itself; recovery must not
depend on billing-locked GitHub Actions artifact storage.
10. GitHub-hosted release packaging rows remain visible but skipped unless the
repository variable `FOCUSA_GITHUB_HOSTED_RELEASE_MATRIX` is explicitly set to
`enabled` as part of the all-at-once restoration procedure.
11. AppVeyor recovery may load `config/appveyor-release-recovery.json` from a
controller commit, verify the immutable tag/SHA pair, detach-checkout that SHA,
and only then build/test/package/upload. Recovery assets use the immutable tag,
not the controller commit. Disable the recovery record immediately after the
release. Cargo/Tauri/test stderr must be redirected inside `cmd.exe`; successful
native commands must not become PowerShell `NativeCommandError` failures.
12. Every `softprops/action-gh-release` step declares
`tag_name: ${{ env.RELEASE_TAG }}`. Recovery dispatch runs from a controller
branch, so no upload may infer release identity from `github.ref`.
13. Bounded self-hosted producers run before long external receipt waiters.
`external-rust-binaries` depends on the full Linux `rust-release` matrix, and
`external-menubar-receipts` depends on `pi-extension-release`; a provider wait
must never occupy all OVH lanes while local release artifacts remain queued.
14. Every pre-final artifact producer preserves the gated Release as a draft.
`softprops/action-gh-release` uploads declare `draft: true`, and restored Tauri
uploads declare `releaseDraft: true`. Exactly one final publisher may set
`draft=false`, only after both external receipt gates, canonical asset
verification, signatures, checksums, updater metadata, and trust metadata pass.
15. External receipt timeouts cover the provider topology, not one nominal job.
While AppVeyor has one-job concurrency, its x64 and ARM64 rows run serially;
receipt jobs therefore have a 150-minute outer bound and 145-minute polling
bound. The poll remains fail-closed on exact asset names, but must not expire
before two valid serial provider rows can complete.
16. Codemagic recovery may load `config/codemagic-release-recovery.json` only
when an API-triggered controller-branch build explicitly sets
`FOCUSA_CODEMAGIC_RECOVERY=enabled`. Both YAML workflows verify the record,
fetch the immutable tag, require its commit to equal the recorded full SHA,
detach-checkout that SHA, and export the verified identity before dependency,
build, package, or upload steps. A normal tag build ignores the recovery record.
Disable the recovery record immediately after release closure.
17. Codemagic release scripts use strict shell mode. The existing Tauri private
key is transported as a secure base64-encoded file payload, decoded only inside
the private ephemeral builder to a mode-0600 file, validated without printing,
used through `TAURI_SIGNING_PRIVATE_KEY=<path>`, overwritten and removed on
exit. Both architecture-specific updater signatures must exist and be nonempty
before any menubar asset upload; a signer decode error is a hard provider
failure, never a green warning.
18. AppVeyor uses the same secure base64 key-file payload contract. Windows
release packaging decodes and validates the key only in the private build-user
temporary directory, points Tauri at that path, then overwrites and removes the
file in a `finally` block. The provider project must hold both the key payload
and password as secure variables; absent authority fails before package work.

## 4. Spending and trigger boundary

- Codemagic uses the personal-account 500 free macOS M2 minutes/month budget.
- Both Codemagic workflows are release-tag (`v*`) scoped; ordinary push and PR
work remains on the free self-hosted Linux and AppVeyor Windows lanes. The only
branch-build exception is a bounded API-triggered immutable-tag recovery with
the explicit recovery variable and enabled recovery record described above.
- Secure upload/signing variables live only in provider secret groups; the
repository stores names and fail-closed checks, never secret values.
- No agent may enable paid hosted macOS capacity, create a paid plan, or widen
Codemagic triggers without direct operator authorization.

## 5. All-at-once GitHub restoration

The return to GitHub is a single operator-triggered transition, never a
partial provider flip. The operator asks to restore GitHub macOS, then one
change set performs all of the following together:

1. Confirm GitHub-hosted Linux, macOS, and Windows billing/capacity is available
with a manual proof build on every OS.
2. Set `FOCUSA_GITHUB_HOSTED_RELEASE_MATRIX=enabled` and re-enable/prove every
existing GitHub-hosted consumer together:
   `ci.yml` Menubar, `spec132-terminal-matrix.yml` macOS terminal rows, and
   `release.yml` macOS Tauri/artifact rows.
3. Require the GitHub macOS package contract to match Codemagic's current
proof: dependencies, typecheck, web build, Tauri `.app`, plist validation,
and codesign validation.
4. Run one release-tag candidate through the full GitHub matrix and retain its
successful receipts.
5. Remove `codemagic.yaml`, this spec, the temporary text in Spec 177 and
release rules, and the Codemagic credential only in that same approved
change set.
6. Remove the AppVeyor stop-gap only when GitHub Windows evidence/artifact
coverage is proven equivalent for the active canonical release contract.

No agent may remove one temporary provider while another required GitHub
replacement remains unproven.

## 6. Agent checklist

Before any release action, an agent must read:

1. `docs/current/RELEASE_RULES_2026-08-19.md`
2. `docs/177-focusa-release-channels-nightly-and-ci-spend-control-spec.md`
3. this document
4. `.appveyor.yml` and `codemagic.yaml`

Then report the exact provider receipts required for the requested release.
If a provider is unavailable, report the failed surface; do not claim full
canonical release completion from a partial matrix.
