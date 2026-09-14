# focusa upgrade command

Status: exact-release, rollback-safe stable upgrade path implemented.

## Why

Evaluator gap: stale daemon versions confused Cursor/evaluator runs. Operators need one command that shows current vs latest version and upgrades through the same self-healing installer path.

## Commands

```bash
focusa upgrade --dry-run
focusa upgrade --dry-run --channel preview
focusa upgrade --dry-run --check-github
focusa upgrade --channel stable
bash focusa-installer-<exact-nightly-tag>.sh --channel=nightly --release-tag=<exact-nightly-tag> --system-install
focusa --json upgrade --dry-run
```

## Behavior

- `focusa upgrade --dry-run` resolves and validates the same exact immutable release tag used by a real upgrade, then prints the plan without swapping binaries.
- Stable upgrades use an explicit valid `FOCUSA_RELEASE_TAG` when supplied; otherwise they resolve GitHub's published, non-prerelease Latest release through the canonical Releases API. Lookup or identity ambiguity fails closed.
- `--check-github` remains accepted for CLI compatibility; stable resolution is always authoritative and no longer depends on a local `gh` executable.
- Preview/nightly keep their channel-qualified compiled tag unless an exact valid `FOCUSA_RELEASE_TAG` is supplied.
- Real `focusa upgrade` binds the resolved tag into every delegated installer download; asset, Pi-extension, and agent-context surfaces cannot drift to another tag.
- Every upgrade downloads, verifies, and atomically installs the four canonical binaries: CLI, daemon, TUI, and session runner.
- When the running CLI comes from the authoritative `/usr/local/bin` surface, upgrade transactionally promotes all four links there, verifies all four exact versions, restarts an active system daemon against the promoted bytes, and restores/restarts the prior system installation on failure.
- An older system CLI that predates exact-tag upgrade support is bridged only through the verified shell bootstrap's explicit Linux-only `--system-install` flag. The shell passes the flag to the downloaded, checksum-verified Rust installer; it never copies or relinks product binaries itself.
- The installer path owns atomic stash and rollback, checksum verification, service rendering, authoritative-path promotion, and license-preserved behavior.
- Failure output includes `failure_class=upgrade_failed`, `recovery_hint`, and next tools: `focusa recover --dry-run`, `focusa doctor --scope host`, and `focusa install --dry-run`.

## Acceptance proof

- Implementation: `crates/focusa-cli/src/commands/upgrade.rs`
- CLI wiring: `crates/focusa-cli/src/main.rs`, `crates/focusa-cli/src/commands/mod.rs`
- Static guard: `tests/spec_upgrade_cmd_static_test.sh`
- Regression tests: exact Latest parsing, channel/tag validation, immutable release binding, system-surface detection, four-binary promotion, session-runner version reporting, and rollback restoration.

## Safety boundary

Upgrade does not implement a second install path. It is a thin, auditable command over the existing self-healing installer so atomicity, license authority, and rollback logic stay DRY.
