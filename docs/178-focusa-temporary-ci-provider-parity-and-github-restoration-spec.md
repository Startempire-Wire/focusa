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
| Windows binaries and menubar packages | AppVeyor public-project lane plus GitHub self-hosted intake | `.appveyor.yml`, `scripts/intake-appveyor-release-artifacts.py`, `release.yml` | Exact-tag MSVC builds/tests; retained CLI, daemon, session-runner, TUI, NSIS, MSI, updater signatures, and a typed digest receipt | One concurrent public-project job; six target/surface rows run serially. |
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
4. Codemagic waits boundedly for the canonical GitHub workflow to create its
gated draft Release and uploads with its approved scoped credential. AppVeyor
never receives GitHub write authority: it retains public read-only artifacts,
and the canonical self-hosted intake job pulls and verifies them before using
the workflow-scoped `GITHUB_TOKEN` to attach them to the existing draft.
Neither provider creates a Release or swallows build, intake, or upload
failures.
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
13. Bounded self-hosted producers run before the consolidated external intake.
`external-provider-receipts` depends on the full Linux `rust-release` matrix
and `pi-extension-release`, then performs one AppVeyor pull plus one complete
external-asset check. Duplicate long waiters are forbidden; provider waiting
must never occupy all OVH lanes while local release artifacts remain queued.
14. Every pre-final artifact producer preserves the gated Release as a draft.
`softprops/action-gh-release` uploads declare `draft: true`, and restored Tauri
uploads declare `releaseDraft: true`. Exactly one final publisher may set
`draft=false`, only after the external provider intake, canonical asset
verification, signatures, checksums, updater metadata, and trust metadata pass.
15. External receipt timeouts cover the provider topology, not one nominal job.
AppVeyor has one-job concurrency and six x64/ARM64 build/test/package rows, each
with a 60-minute provider ceiling. The consolidated intake therefore has a
400-minute outer bound, 385-minute AppVeyor polling bound, and a final bounded
five-minute completeness check after settlement. It remains fail-closed on
exact identity and asset names, but does not expire before a valid serial
matrix can complete.
16. Codemagic recovery may load `config/codemagic-release-recovery.json` only
when an API-triggered controller-branch build explicitly sets
`FOCUSA_CODEMAGIC_RECOVERY=enabled`. Both YAML workflows verify the record,
fetch the immutable tag, require its commit to equal the recorded full SHA,
detach-checkout that SHA, and export the verified identity before dependency,
build, package, or upload steps. A normal tag build ignores the recovery record.
For the exact `v0.9.187` / `01aae7ea9ab886627d49b68e7aed2349d9ceafc0`
Rust recovery only, Cargo may normalize the four workspace-local lock versions
named in Issue #473 from `0.9.186` to `0.9.187`. The adapter must snapshot and
parse both lockfiles, prove identical package multisets after normalizing only
those four local version fields, reject every source/checksum/dependency/count
or external-package difference, return to `cargo build --locked`, and restore
the candidate lockfile after packaging. This exception is unavailable to normal
tag builds or any other tag/SHA. Disable the recovery record immediately after
release closure.
17. Codemagic release scripts use strict shell mode. The existing two-line
Minisign private-key file is transported as one secure outer-base64 payload.
The private ephemeral builder validates both the outer payload and inner key
line without printing or persisting decoded content. Before conversion, it
installs libsodium only when absent and proves that Python can discover the
resulting ephemeral runtime. It then normalizes the authenticated legacy
envelope in memory through `scripts/ci/convert-legacy-tauri-signing-key.py` and
passes the resulting current outer-base64 value through
`TAURI_SIGNING_PRIVATE_KEY`. The conversion preserves
the signing identity and never logs or persists private material. Both
architecture-specific updater signatures must exist and be nonempty before any
menubar asset upload; a signer decode or conversion error is a hard provider
failure, never a green warning. Tauri's initial ad-hoc executable signature is
not sufficient release proof: Codemagic must seal the completed `.app` resource
tree, require a nonempty `_CodeSignature/CodeResources`, and pass strict deep
verification. The `.app.zip`, DMG, and updater archive must then be regenerated
from those exact sealed app bytes, and the regenerated updater archive must be
signed through the environment-bound Tauri signer. Validating one byte set
while uploading a different pre-seal bundle is forbidden.
18. AppVeyor uses the same secure outer-base64 payload contract and validates it
in memory before Tauri packaging. No decoded signing-key file is written on
either provider. The AppVeyor project must hold both the key payload and
password as secure variables; absent authority fails before package work. One
exact reviewed branch performs immutable-tag recovery. The duplicate PR webhook
and every unrelated branch must stop before dependencies, so one recovery
request cannot fan out into two six-job serial matrices.
19. AppVeyor artifact intake discovers a normal release only from a public
project-history record whose repository, tag, full commit SHA, and terminal
success match the immutable candidate. An immutable-tag recovery additionally
requires an explicit provider build number, full reviewed controller SHA, exact
controller branch, and the exact candidate checkout marker in every job log;
failed or partial recovery builds remain inadmissible. Either route requires
exactly six successful jobs: binaries, tests, and menubar for both MSVC
architectures. Recovery logs must contain the exact branch-route candidate
marker; another route suffix is inadmissible. The two test jobs must retain no
artifacts. The two binary jobs must retain exactly CLI, daemon, session-runner,
and TUI executables; the two menubar jobs must retain exactly NSIS/MSI bundles
and both updater
signatures for the tagged version. Downloads must match provider sizes, use
safe basenames, and receive local SHA-256 digests in
`focusa.appveyor_release_artifact_receipt.v1`. Existing draft assets are either
byte-identical and retained or produce a hard collision failure. Missing,
extra, duplicate, failed, stale, wrong-project, wrong-tag, or wrong-SHA evidence
blocks checksums and publication. Rollback reverts the intake change set and
restores the previous provider-push adapter only after equivalent renewable,
scoped secret authority is proven; it never permits hand-copied artifacts or a
partial release.

## 4. Spending and trigger boundary

- Codemagic uses the personal-account 500 free macOS M2 minutes/month budget.
- Both Codemagic workflows are release-tag (`v*`) scoped; ordinary push and PR
work remains on the free self-hosted Linux and AppVeyor Windows lanes. The only
branch-build exception is a bounded API-triggered immutable-tag recovery with
the explicit recovery variable and enabled recovery record described above.
- Signing variables live only in provider secret groups; the repository stores
names and fail-closed checks, never secret values. GitHub upload authority stays
inside the canonical GitHub workflow and is not delegated to AppVeyor.
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
