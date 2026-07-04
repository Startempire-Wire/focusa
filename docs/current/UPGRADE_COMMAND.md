# focusa upgrade command

Status: implemented MVP for `focusa-upgrade-cmd`.

## Why

Evaluator gap: stale daemon versions confused Cursor/evaluator runs. Operators need one command that shows current vs latest version and upgrades through the same self-healing installer path.

## Commands

```bash
focusa upgrade --dry-run
focusa upgrade --dry-run --channel preview
focusa upgrade --dry-run --check-github
focusa upgrade --channel stable
focusa --json upgrade --dry-run
```

## Behavior

- `focusa upgrade --dry-run` prints current vs latest version information and the planned atomic installer route without swapping binaries.
- Latest version source order: `FOCUSA_LATEST_VERSION`, optional `gh release view` via `--check-github`, then `unknown` with a recovery hint.
- Real `focusa upgrade` delegates to `focusa install --target=auto --channel=<channel>`.
- The installer path owns atomic stash and rollback, checksum verification, service rendering, and license preserved behavior.
- Failure output includes `failure_class=upgrade_failed`, `recovery_hint`, and next tools: `focusa recover --dry-run`, `focusa doctor --scope host`, and `focusa install --dry-run`.

## Acceptance proof

- Implementation: `crates/focusa-cli/src/commands/upgrade.rs`
- CLI wiring: `crates/focusa-cli/src/main.rs`, `crates/focusa-cli/src/commands/mod.rs`
- Static guard: `tests/spec_upgrade_cmd_static_test.sh`

## Safety boundary

Upgrade does not implement a second install path. It is a thin, auditable command over the existing self-healing installer so atomicity, license authority, and rollback logic stay DRY.
