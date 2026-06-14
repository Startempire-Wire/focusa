# Installer and Update Policy

Focusa installers and updates must be explicit, reversible, and guarded by Context Authority. Pairing failures, daemon health warnings, or stale UI state must never silently become install/update tasks.

## Install channels

- Source build: developer/operator checkout, local build, manual service restart.
- Release asset: signed/checksummed CLI/daemon bundle for non-live build hosts.
- Menubar app bundle: signed Mac app release artifact.
- Pi extension package: versioned extension package loaded by Pi.

## Required preflight

Before replacing binaries, restarting production services, or installing release assets, run:

```bash
focusa action preflight \
  --current-ask "$CURRENT_ASK" \
  --kind binary_replace \
  --target /usr/local/bin/focusa \
  --source github_release_asset \
  --install-role live_build_host \
  --project-root "$PWD" \
  --json
```

If preflight returns `block` or `ask_operator`, stop mutation and report the recovery path.

## Live build host policy

On a live build host, prefer local repo build/restart over release asset replacement. A live daemon host must keep a rollback binary/service unit path before mutation.

## Update checklist

1. Verify project identity and current Workpoint.
2. Check git status/diff and release version consistency.
3. Run tests relevant to changed artifacts.
4. Verify checksum/signature for downloaded release assets.
5. Snapshot current binary/config/service state.
6. Stop/restart service only when authorized.
7. Run daemon health and `focusa release prove --tag <tag>`.
8. Capture evidence and rollback if health/proof fails.

## Rollback checklist

- Restore previous binary or app bundle.
- Restore previous service unit/config when changed.
- Restart daemon only after operator-safe preflight.
- Run health, version, and smoke checks.
- Link rollback evidence to Workpoint.

## Forbidden substitutions

- Pairing troubleshooting must not trigger installer/update work by default.
- Stale menubar UI must first try refresh/reconnect before reinstall.
- Release asset replacement on a live build host requires Context Authority approval.
- No auto-update path may publish or exfiltrate private Workpoint/Evidence data.

## Proof

- Static guard: `tests/installer_update_policy_static_test.sh`
- Related: `COMMERCIAL_PACKAGING.md`, `FIRST_RUN_FLOW.md`, `SECURITY_MODEL.md`
