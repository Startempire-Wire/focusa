# Canonical Live Release Pipeline

## Non-negotiable rule

**Every full Focusa release and production daemon upgrade uses the live GitHub release pipeline.** Approved extension-only updates are separate: use the host's supported activation/rollback and verify the actual Pi terminal or OpenClaw conversation. They do not upgrade the daemon or publish a stable tag.

Future agents must not build release artifacts locally, must not deploy from
`target/release`, must not run only part of the workflow, and must not bypass
CI/Release/Deploy gates. Local toolchains are for source inspection/static
checks only, never for release artifact creation or live daemon deployment.

## The only supported build/deploy path

1. Prepare the exact approved release candidate with:
   ```bash
   scripts/create-dev-release-tag.sh --base 0.9 --push --no-wait-ci
   ```
2. Pending exit 78 means the final source commit was pushed but no tag was created. After exact main CI succeeds, resume with the same tag. Never use an ordinary source push as evidence of a release.
3. Let GitHub Actions run the full chain:
   - `CI` tests the final candidate once; Release verifies its receipt.
   - `Release`, which requires, checksums, signs, and publishes `distribution-manifest.json` as an immutable prerelease with `Latest=false`
   - `Deploy Live Daemon`, which delegates binaries, installed manifest, systemd, health, CallGraph proof, and rollback to the Rust full-release lifecycle; after installed parity and OTA acceptance it re-signs the settled release manifest and alone promotes an unsuffixed tag to stable `Latest`

   Audit and recovery tools record and diagnose failures. They do not promote
   a release or authorize an automatic production retry.
4. Trust only GitHub run conclusions + deploy health proof as release truth.

## Recovery policy

When a build/deploy failure happens, fix the **pipeline/system**, not the host
by hand:

- CI/Release failure → diagnose and repair the exact failed canonical gate.
- Deploy failure → require the Rust transaction's rollback/process/unit evidence
  before retrying the same immutable tag.
- Audit/Watchdog failure → record it and repair that evidence lane; do not let a
  generic retry hide the underlying release or host failure.

Manual intervention is limited to editing source/workflow code that improves
this system. Manual release building or live daemon installation is not an
allowed recovery path.

## Forbidden for release/deploy

Do not use these as release/deploy actions:

```bash
cargo build --release
cargo test --workspace && cargo build --release
cp target/release/focusa-daemon ...
bash scripts/install-daemon.sh --binary target/release/focusa-daemon
gh workflow run 'Deploy Live Daemon' ...   # partial pipeline bypass
scripts/deploy-smoke-check.sh              # proof helper only, not deploy
```

If you see a doc, script, or bead suggesting those as live deployment paths,
update it to this canonical policy before continuing.

## Allowed local actions

These are allowed because they do not create/release/deploy artifacts:

- Read code/docs.
- Edit source/workflow/docs/tests.
- Run static shell guards in `tests/*_static_test.sh`.
- Query GitHub Actions/audit logs with `gh run view`, `gh run list`, and
  `gh release view`.
- Push fixes to `main` so the live pipeline can build/deploy them.

## Acceptance proof

A deployment is complete only when:

- The tag exists on GitHub.
- `CI` completed successfully for that tag commit.
- `Release` completed successfully and published the full asset matrix, including signed `distribution-manifest.json`, for that tag.
- `Deploy Live Daemon` completed successfully for that exact immutable tag.
- `/usr/local/lib/focusa/distribution-manifest.json` is byte-identical to the signed release asset and reports the installed version.
- All four installed binary versions and SHA-256 values, active Pi package, agent skills/docs, health response, and CallGraph probe are captured in the parity receipt.
- Install inputs remain pinned to the immutable release tag; audit signing and settlement use the exact workflow revision (`github.workflow_sha`) in a separate sparse controller checkout, never a moving helper.
- After OTA acceptance, the existing deployment-proof owner validates the exact plan and signs `distribution-parity.json` and `ota-update-plan.json`; their receipts and detached signatures are attached to the same Release. Mandatory release evidence does not depend on Actions artifact storage quota. Existing payloads and native updater signatures remain unchanged.
- `/v1/health` proof is emitted by the deploy workflow.
- Report any untested rollback or consumer surface explicitly. Release assets and receipts cannot prove an extension was reloaded in an already-running session.
