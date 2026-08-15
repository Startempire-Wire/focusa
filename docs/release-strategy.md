# Focusa Release Strategy & Versioning

> Canonical release/versioning policy. Applies to `Startempire-Wire/focusa` and
> supersedes ad-hoc tagging. Mechanics live in
> [`docs/canonical-live-release-pipeline.md`](canonical-live-release-pipeline.md).

## 1. Versioning model

We use Semantic Versioning (`vMAJOR.MINOR.PATCH`) with the standard pre-1.0
convention:

| Version slot | Meaning |
|---|---|
| `MAJOR` | Breaking changes (only once we are at `>= 1.0`) |
| `MINOR` | New features; **at `0.x` this is the breaking slot** |
| `PATCH` | Backward-compatible fixes and security patches |

Until `1.0`, breaking changes land in a MINOR bump (`0.9.x -> 0.10.0`), never in
a PATCH bump. After `1.0`, breaking changes land in a MAJOR bump and require a
migration note.

## 2. Release lanes

We run three lanes. This is the hybrid model: small fast releases for security,
batched cadence releases for everything else.

| Lane | Version shape | Contents | Trigger | Cadence |
|---|---|---|---|---|
| **Patch** | `0.9.x` (current train) | Security fixes, critical regressions, hotfixes | Any time, immediately | No calendar gate |
| **Minor** | `0.10.0`, `0.11.0`, ... | Features, enhancements, non-critical fixes, refactors | Batched when the lane is cut | On cadence (target: weekly, or per planned milestone) |
| **Major** | `1.0.0`, `2.0.0`, ... | Breaking changes + migration notes | Planned, documented | Rare; requires a written breaking-change plan |

### 2.1 What qualifies for the Patch lane (ship now)

An issue ships in a patch release if it is **security/critical**:

- Credential, authentication, authorization, entitlement, or secret-handling defects
- Data loss or corruption
- Crash loops or hard failures on any shipped surface
- P0 regressions of released behavior
- License/signing/update-integrity defects

Patch releases stay small and low-surface. If a change is more than a few
commits, it is usually a minor-lane candidate, not a patch.

### 2.2 What waits for the Minor lane

- New features and enhancements
- Non-critical bug fixes
- Refactors, performance work, UX polish
- Documentation-driven surface changes

### 2.3 What waits for the Major lane

- Breaking CLI/API/contract changes (post-1.0: MAJOR bump + migration notes)
- Pre-1.0, breaking changes go into the next MINOR (`0.10.0`), never into a
  `0.9.x` patch.

## 3. Commit-driven version bumps

Commits are Conventional Commits (enforced by
`scripts/validate-commit-messages.sh` in CI and hooks). The version policy reads
commits since the last tag:

| Commit signal | Required bump |
|---|---|
| `BREAKING CHANGE:` trailer or `!` after type/scope | MINOR at `0.x`, MAJOR at `>= 1.0` |
| `feat` | MINOR (feature material rides the minor lane) |
| `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`, `revert`, `proof`, `merge` | PATCH |

`scripts/next-version.py` classifies a commit range and prints the required
bump, next version, and any policy violations. The canonical selector
`scripts/select-release-version.py` picks the next monotonic tag (including
dev/preview channel tags such as `v0.9.153-dev`); `next-version.py` validates
the selection locally and in CI — it never replaces the selector.

```bash
# What should the next tag be from current main?
python3 scripts/next-version.py

# Check a specific tag against the policy (used by CI):
python3 scripts/next-version.py --tag v0.9.153 --json
```

**Adoption note:** the hard gate enforces shape, monotonicity, and breaking
changes now. `feat`-in-patch is reported as a warning until feature work moves
onto the `0.10.0` minor lane; at that point the warning becomes a hard gate.

## 4. Enforcement

Enforcement is layered:

1. **Commits** — `scripts/validate-commit-messages.sh` rejects non-conventional
   and generic subjects (CI job `commit-messages` in `ci.yml`, plus local hooks).
2. **Labels** — issues carry a lane label (`lane:patch`, `lane:minor`,
   `lane:major`) and `security` where applicable. Security issues get patch-lane
   priority.
3. **Tag gate** — `release.yml` blocks any tag while a `release-gate:*` issue is
   open.
4. **Version policy** — the `version-policy` job in `release.yml` runs on
   every `v*` tag and blocks release creation on: malformed tag, non-monotonic
   tag, or a breaking change whose bump is too small. Channel-suffixed dev tags
   (`v0.9.153-dev`) are valid patch-lane tags subject to the same policy.
   `scripts/create-dev-release-tag.sh` runs the same check as a fail-fast
   preflight before stamping/pushing (honoring `--force-release` and dry-run);
   the CI job is the authoritative gate.
5. **Human lane** — `--force-release` in `scripts/create-dev-release-tag.sh`
   requires a plain-language reason and is for real emergencies only.

## 5. Cutting a release

Both lanes use the canonical pipeline only:
`scripts/create-dev-release-tag.sh --base <MAJOR.MINOR> --push`
(chain: `CI` -> `Release` -> `Deploy Live Daemon` -> audit/self-heal/watchdog).

### Patch cut checklist (`--base 0.9`)

- [ ] Only security/critical fixes in the range (verify with `next-version.py`)
- [ ] No open `release-gate:*` issues
- [ ] Changelog regenerated (`python3 scripts/changelog-gen.py`) if required
- [ ] CI green; release + deploy verified before handoff

### Minor cut checklist (`--base 0.10`)

- [ ] Features batched and reviewed; non-critical fixes included
- [ ] All open `release-gate:*` issues closed
- [ ] Breaking changes (if any) documented with migration notes
- [ ] Changelog regenerated; release notes written from commit subjects
- [ ] CI green; release + deploy verified before handoff

## 6. Triage rule of thumb

Every issue lands in exactly one lane. If it is security/critical -> `security`
+ `lane:patch`. If it is a feature or non-critical -> `lane:minor`. Breaking
design work -> `lane:major` and scheduled. When in doubt, default to
`lane:minor` — patch lane is the exception, not the default.
