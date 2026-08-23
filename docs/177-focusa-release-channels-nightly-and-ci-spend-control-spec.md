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
| **nightly** | `vX.Y.Z-nightly.YYYYMMDD` | **1 job, ubuntu-latest, Linux x86_64 only** | Rolling prerelease (previous deleted) | ~5 min/day |

- Nightly is a **dev convenience lane**, not a release. It ships ONLY the
  Linux x86_64 CLI + daemon that the operator's dev machine runs.
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
3. Skip rule: if no non-docs commit since the previous nightly tag,
   exit early ("nothing new") — zero cost on quiet days.
4. One job: ubuntu-latest, `timeout-minutes: 20`.
   Build `focusa` + `focusa-daemon` (release, x86_64-unknown-linux-gnu).
5. Package: `focusa-<tag>-x86_64-unknown-linux-gnu{,.sha256}` +
   `SHA256SUMS` — names identical to release packaging so the canonical
   bootstrap works unmodified.
6. Publish: delete previous `*-nightly*` prerelease + tag (rolling),
   create `v<stamp>-nightly.YYYYMMDD` prerelease with those assets.
7. Permissions: `contents: write` only.

## 3. CI spend control (existing workflows)

1. `ci.yml`: add `paths-ignore: ['**.md', 'docs/**']` to push+PR triggers;
   extend `cancel-in-progress` to push events (not just PRs).
2. No new crons anywhere. Existing daily cron (`billing-bypass-expiry`)
   stays — it is documented cheap.
3. Disabled burners stay disabled (`audit-recorder`, watchdog schedule).
4. Any new workflow MUST declare its expected monthly minutes in a header
   comment. Reviewer gate: more than ~60 min/month needs operator approval.

## 4. Dev machine policy

Operator's build machine runs the **nightly channel**: installed via the
canonical installer `scripts/install-focusa.sh --channel=nightly` from the
published rolling prerelease. Never hand-copied binaries (AGENTS.md
CANONICAL RELEASE ONLY still applies — nightly artifacts are produced by
the canonical GitHub pipeline, just a smaller matrix).

## 5. Acceptance

- AC1: nightly.yml exists with cron+dispatch only, single job, 20m timeout,
  concurrency cancel-in-progress.
- AC2: a dispatch run produces `v…-nightly.YYYYMMDD` prerelease with
  `focusa-<tag>-x86_64-unknown-linux-gnu` + sha256 + SHA256SUMS assets.
- AC3: second dispatch replaces (deletes) the prior nightly — no asset
  accumulation.
- AC4: docs-only day skips with zero build minutes.
- AC5: `install-focusa.sh --channel=nightly` installs that build on the
  dev machine; `focusa --version` reports the nightly tag.
- AC6: ci.yml paths-ignore + push cancellation merged.

## 6. AppVeyor emergency full-release provider

Status: approved by operator directive 2026-08-23. AppVeyor is an emergency
provider for a GitHub Actions provider outage; it is not a reduced release
channel. `Release` and `dev release` retain the meanings in
`docs/current/RELEASE_RULES_2026-08-19.md`: every supported surface, OS,
artifact, trust proof, and live proof is mandatory.

Activation is fail-closed. An AppVeyor release requires all of:

1. `FOCUSA_RELEASE_PROVIDER=appveyor`;
2. an operator directive naming a full release or dev release;
3. an incident reference explaining why GitHub Actions cannot run;
4. exact tag and commit equality across GitHub, AppVeyor, packaged artifacts,
   release manifest, and provenance;
5. all 14 logical gates in section 7 green.

A tag, AppVeyor build, draft release, or partial artifact set is not a
Release. The release is complete only when GitHub reports the stable tag as
Latest (`isLatest=true`, `isPrerelease=false`) with the canonical asset set,
or reports a requested dev tag as a prerelease, and live OTA/deploy proof is
green.

Current connected project authority:

- account/project: `verioussmith/focusa`;
- repository: `Startempire-Wire/focusa`;
- plan: Free / Active, one concurrent job (matrix jobs queue sequentially);
- configuration authority: repository `.appveyor.yml`;
- build ceiling: 60 minutes per job;
- current known code blocker: GitHub issue #324, Unix-only `bg --detach`
  process-group handling compiled on Windows.

## 7. Provider-neutral 14-gate release ledger

AppVeyor job count and scheduling may differ from GitHub Actions, but it MUST
emit one exact-SHA receipt for each canonical logical gate. A final assembly
job consumes all receipts and rejects missing, failed, mismatched-SHA, stale,
or duplicate receipts.

| # | Receipt | Mandatory capability |
|---|---|---|
| 1 | `release-contract` | project identity, clean scope, exact tag/SHA, version policy, issue/PR gates, candidate lock |
| 2 | `source-ci` | meaningful commits, workspace build/tests, Clippy `-D warnings`, release automation static gates |
| 3 | `strict-spec-gates` | Pi runtime tests, final release gap, Spec104, daemon API probe, `run-spec-gates.sh` |
| 4 | `terminal-windows` | x64 ConPTY runtime proof and Windows dependency preflight |
| 5 | `terminal-linux` | GNU terminal proof plus musl artifact/runtime proof |
| 6 | `terminal-macos` | native macOS terminal and bundle proof |
| 7 | `rust-macos` | CLI, daemon, and TUI for ARM64 and x64 |
| 8 | `rust-linux` | CLI, daemon, and TUI for GNU x64/ARM64 and musl x64 |
| 9 | `rust-windows` | CLI, daemon, and TUI for MSVC x64/ARM64 |
| 10 | `desktop-macos` | Tauri ARM64/x64 app, DMG, updater archive/signature, ad-hoc or production signing policy |
| 11 | `desktop-windows` | Tauri MSVC x64/ARM64 setup executable, MSI, updater artifacts |
| 12 | `portable-surfaces` | Pi extension, shell/PowerShell installers, generated clients, agent context, skills |
| 13 | `trust-and-publication` | canonical asset verification, SBOMs, SHA256SUMS, Ed25519 signatures, Fulcio/Rekor Cosign proof, manifest, provenance, intelligence, GitHub publication |
| 14 | `ota-deploy-live` | clean install/update/rollback/reapply, Pi activation, daemon canary/health/version, production deployment receipt |

Each receipt is JSON with this minimum output shape:

```json
{
  "schema": "focusa.release_gate_receipt.v1",
  "gate": "rust-windows",
  "provider": "appveyor",
  "repository": "Startempire-Wire/focusa",
  "tag": "vX.Y.Z",
  "commit": "40-hex-sha",
  "build_id": "appveyor-build-id",
  "job_id": "appveyor-job-id",
  "status": "passed",
  "artifact_sha256": "64-hex-or-null",
  "evidence_url": "https://ci.appveyor.com/...",
  "completed_at": "RFC3339"
}
```

Receipts are artifacts and release provenance inputs, never mutable UI-only
claims. The final ledger is `focusa.release_gate_ledger.v1`; it contains
exactly gates 1-14, one receipt each, and `all_green=true` only after strict
validation.

## 8. Exact build and artifact matrix

`.appveyor.yml` MUST cover, without host substitution:

- native macOS ARM64 and x64 Rust builds;
- Linux GNU x64, cross-built GNU ARM64, and static musl x64;
- native/cross MSVC x64 and ARM64 builds;
- `focusa`, `focusa-daemon`, and `focusa-tui` on every Rust target;
- Tauri menubar ARM64/x64 on macOS and Windows;
- macOS `.app.zip`, `.dmg`, updater archive, and updater signature;
- Windows setup `.exe`, `.msi`, and updater artifacts;
- Pi extension, generated clients, agent context, skills, and both installers;
- `SHA256SUMS.txt`, every Ed25519 detached signature, SBOMs, release manifest,
  provenance, release intelligence JSON/Markdown, trusted key metadata, Cosign
  signature, and Fulcio certificate.

`scripts/verify-canonical-release-assets.py --dist <dir> --tag <tag>` remains
the single asset-set authority. Provider-specific allowlists are banned.
Release artifacts keep existing names so installers and old consumers remain
compatible.

## 9. Trust, identity, secrets, and provenance

No trust downgrade is allowed. Existing stable installers require a real
Sigstore certificate; AppVeyor therefore MUST NOT replace keyless signing with
a locally self-signed certificate or checksum-only publication.

AppVeyor signing uses a dedicated Google Cloud service-account identity and
the official Sigstore automated-environment flow:

1. a dedicated least-privilege release signer mints a short-lived Google OIDC
   identity token with audience `sigstore` and email included;
2. Cosign receives it through `--identity-token` or `SIGSTORE_ID_TOKEN`;
3. Fulcio issues the ephemeral certificate and Rekor records the signing event;
4. the publication gate verifies certificate issuer
   `https://accounts.google.com`, exact signer identity, signature, Rekor
   inclusion, tag, SHA, and ledger digest before upload;
5. existing `SHA256SUMS.txt.cosign.sig` and
   `SHA256SUMS.txt.cosign.pem` filenames remain unchanged, preserving old
   installer compatibility.

The signer credential, GitHub publication token, Tauri updater key, Focusa
Ed25519 release key, and optional Apple production credentials are AppVeyor
secure variables/files. They are never committed, logged, embedded in build
artifacts, or made available to pull-request builds. Tag publication jobs alone
receive release secrets. The dedicated Google identity has no source-write,
release-write, deployment, or unrelated cloud permissions.

`focusa.release_provenance.v1` remains additive and gains provider evidence:
`builder="appveyor"`, AppVeyor account/project/build/job URLs, configuration
digest, release-gate-ledger digest, signer identity, and Rekor entry reference.
Old consumers may ignore the additive fields. Current consumers test both
GitHub Actions and AppVeyor provenance shapes.

## 10. Orchestration and publication

The provider workflow is deterministic:

1. validate exact tag/SHA and create or reuse a GitHub draft release;
2. run source/spec/platform jobs, sequentially when the Free plan queues them;
3. upload immutable job artifacts and receipts;
4. final assembly downloads artifacts by exact AppVeyor build and job IDs;
5. verify all 14 receipts and canonical assets;
6. generate notes only with `scripts/generate-release-notes.py`;
7. generate SBOM, checksums, Ed25519 metadata, Cosign/Fulcio/Rekor proof,
   manifest, provenance, and release intelligence;
8. upload with force-update disabled unless the existing asset hash matches;
9. execute OTA/install/rollback/reapply and live daemon deployment proofs;
10. publish the draft and set Latest only after gate 14 passes.

A failed job leaves the release draft/non-Latest and records the exact failure.
Retry resumes from immutable successful receipts for the same SHA; a SHA change
invalidates every prior receipt. No manual artifact copy, local release binary,
or hand-edited manifest is permitted.

## 11. Exact implementation scope

Files authorized for this fallback implementation:

- `.appveyor.yml` — matrix, dependency graph, protected tag publication;
- `config/focusa-release-topology.json` — retain primary
  `provider=github_actions`; add an emergency AppVeyor provider contract for
  new consumers;
- `crates/focusa-cli/src/commands/bg.rs` — platform-correct detached monitor
  behavior for issue #324;
- `scripts/release-trust-metadata.py` — additive AppVeyor provenance inputs;
- `scripts/verify-canonical-release-assets.py` — provider-neutral verification
  only if tests prove no asset-policy weakening;
- `scripts/install-focusa.sh`, `scripts/install-focusa.ps1`, and the Rust update
  verifier — exact Google signer identity verification while retaining GitHub
  certificate compatibility;
- `tests/177_appveyor_full_release_fallback_test.py` — static contract,
  14-receipt completeness, secret isolation, target/artifact matrix, and
  fail-closed publication tests;
- existing Windows, Spec132, OTA, release, and production-consistency tests for
  consumer and live proofs.

Before adding a helper, run the Focusa deslop similarity workflow and reuse
existing release-note, asset verification, trust metadata, and installer
functions. A second release engine or copied GitHub workflow in shell is
forbidden.

## 12. Production-consistency proofs

- **Versioned contract:** gate receipt and ledger schemas; additive AppVeyor
  provenance fields.
- **Producer tests:** AppVeyor receipt generation, target packaging, trust and
  publication fail-closed tests.
- **Consumer tests:** final assembler, installers, updater, daemon deployer, and
  Pi extension consume AppVeyor-produced assets and receipts.
- **Cross-version interop:** old installers accept unchanged Fulcio asset names
  and certificate shape; current consumers accept both provider provenance
  shapes.
- **Live e2e:** Windows x64/ARM64 OTA, macOS updater/bundle, Linux GNU/musl,
  Pi activation, and production daemon canary all run against the published
  candidate before Latest flips.

## 13. Rollback and unplug

- Any mismatch leaves the draft unpublished and current Latest untouched.
- Deployment rollback uses the previous signed release and proves daemon health,
  version, state retention, and Pi extension restoration.
- Clearing the GitHub Actions billing outage does not invalidate AppVeyor
  receipts, but new releases return to the primary provider by default.
- Unplug removes the emergency provider entry and `.appveyor.yml` only after a
  later GitHub Actions full release proves all 14 gates and every live surface.
  Signing identity and audit records are retired, not deleted.

## 14. Full-release acceptance

- AC7: AppVeyor emits 14/14 exact-SHA gate receipts and one validated ledger.
- AC8: every canonical target and surface is built; canonical asset verifier
  passes with 30+ assets.
- AC9: Windows native build passes issue #324 regression and ConPTY/OTA proofs.
- AC10: Fulcio certificate issuer, exact Google signer identity, and Rekor entry
  verify; stable installers accept the release without compatibility flags.
- AC11: AppVeyor secrets are absent from PR jobs, logs, artifacts, and receipts.
- AC12: GitHub release stays draft/non-Latest until live gate 14 passes.
- AC13: stable publication is `isLatest=true`, `isPrerelease=false`; dev
  publication is full-matrix and `isPrerelease=true`.
- AC14: clean install, update, rollback, reapply, Pi activation, desktop updater,
  and daemon canary proofs are attached and green.
- AC15: `config/focusa-release-topology.json`, release manifest, provenance,
  release rules, and actual provider receipts agree on tag, SHA, surfaces, and
  trust identity.
