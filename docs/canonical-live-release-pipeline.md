# Canonical Live Release Pipeline

## Non-negotiable rule

**All Focusa build and deploy work uses the full live GitHub release pipeline.**

Future agents must not build release artifacts locally, must not deploy from
`target/release`, must not run only part of the workflow, and must not bypass
CI/Release/Deploy gates. Local toolchains are for source inspection/static
checks only, never for release artifact creation or live daemon deployment.

## The only supported build/deploy path

1. Land source changes on `main`.
2. Create the next dev release tag with:
   ```bash
   scripts/create-dev-release-tag.sh --base 0.9 --push
   ```
3. Let GitHub Actions run the full chain:
   - `CI`
   - `Release`
   - `Deploy Live Daemon`
   - `Audit Recorder (self-heal trigger)`
   - `Auto Heal Release Pipeline`
   - `Release Pipeline Watchdog`
4. Trust only GitHub run conclusions + deploy health proof as release truth.

## Recovery policy

When a build/deploy failure happens, fix the **pipeline/system**, not the host
by hand:

- CI/Release failure → Auto Heal reruns failed jobs once; Watchdog scans and
  reruns missed failures continuously.
- Deploy failure → Auto Heal/Watchdog re-dispatches `Deploy Live Daemon` with
  the release tag and canonical musl asset.
- Auto Heal failure → Audit Recorder records it; Watchdog reruns failed Auto
  Heal jobs.
- Watchdog failure → Audit Recorder records it; Auto Heal reruns failed
  Watchdog jobs.

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
- `Release` completed successfully and published assets for that tag.
- `Deploy Live Daemon` completed successfully for that tag.
- `/v1/health` proof is emitted by the deploy workflow.
- Audit Recorder has no unresolved process-error row for the pipeline run.
