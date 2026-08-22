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
