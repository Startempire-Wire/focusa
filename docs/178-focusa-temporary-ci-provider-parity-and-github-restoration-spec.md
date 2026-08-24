# 178 — Focusa Temporary CI Provider Parity and GitHub Restoration Spec

**Status:** active temporary release routing
**Effective:** 2026-08-24
**Supersedes:** no release authority; extends 177 and `docs/current/RELEASE_RULES_2026-08-19.md`

## 1. Purpose

GitHub-hosted macOS capacity is billing-locked. Cirrus is defunct. This
spec records the complete temporary release build route so any agent can
execute and audit a release without guessing, treating a green GitHub
`macos-latest` result as available, or silently substituting a provider.

This is a temporary provider delegation, not a new release channel or a
second release authority. Tags, manifest freshness, release evidence, and
the canonical release scripts remain the authority.

## 2. Active provider map

| Required surface | Temporary provider | Entry point | Required proof | Current limitation |
|---|---|---|---|---|
| Linux daemon, CLI, API, specs | GitHub Actions self-hosted `host-focusa-deploy` | `.github/workflows/ci.yml`, `release.yml` | Rust, Spec Gates, release automation, meaningful commits green | Shared production host; Rust exit-241 flake is tracked separately. |
| Windows CLI verification/artifact | AppVeyor public-project lane | `.appveyor.yml` | MSVC build plus `focusa-license` and `focusa-core` tests; tagged CLI artifact | Does not replace Windows menubar packaging. |
| macOS menubar package proof | Codemagic cloud `mac_mini_m2` | `codemagic.yaml`, workflow `menubar-macos-package-proof` | npm ci, typecheck, web build, Rust/Tauri `.app`, plist lint, ad-hoc codesign and verification green | Proof is ad-hoc signed; it is not notarized customer distribution. |
| GitHub hosted macOS jobs | temporarily non-authoritative | `ci.yml`, `spec132-terminal-matrix.yml`, `release.yml` | Informational only while billing-locked | Must not be silently deleted or individually re-enabled. |

## 3. Canonical temporary release procedure

1. Run the normal canonical preflight and create the requested dev or stable
tag through the existing release scripts. Never hand-build, hand-copy, or
hand-publish an artifact.
2. Require the Linux/self-hosted GitHub evidence and AppVeyor Windows evidence
that apply to the requested release surface.
3. Start Codemagic workflow `menubar-macos-package-proof` against the exact
tag commit using the Codemagic API credential supplied through the approved
credential authority. The repository contains no token or secret path.
4. Retain the successful Codemagic build ID/receipt beside the release proof.
A GitHub hosted-macOS failure is expected during this temporary route and is
not a substitute for, or a failure of, the Codemagic proof.
5. Treat an absent, failed, wrong-commit, or unsigned Codemagic proof as a
release blocker for the menubar surface.
6. Publish only after all required proof surfaces for that release are green.

## 4. Spending and trigger boundary

- Codemagic uses the personal-account 500 free macOS M2 minutes/month budget.
- The workflow is release-tag (`v*`) scoped; ordinary push and PR work must
remain on the free self-hosted Linux and AppVeyor Windows lanes.
- No agent may enable paid hosted macOS capacity, create a paid plan, or widen
Codemagic triggers without direct operator authorization.

## 5. All-at-once GitHub restoration

The return to GitHub is a single operator-triggered transition, never a
partial provider flip. The operator asks to restore GitHub macOS, then one
change set performs all of the following together:

1. Confirm GitHub-hosted macOS billing/capacity is available with a manual
proof build.
2. Re-enable and prove every existing GitHub macOS consumer together:
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
