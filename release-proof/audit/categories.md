# Failure category playbook

This file indexes every recorded failure so future automation can detect, self-heal, or at minimum quote the known fix. Categories:

- `brittle_regex_match`
- `stale_version_surface`
- `missing_ci_gate_passing`
- `infrastructure_blocked`
- `disk_pressure`
- `permission_denied`
- `hostname_assumption`
- `policy_violation`

Each entry links back to `release-proof/audit/audit.jsonl` and the regression guard/test.

## brittle_regex_match

- **symptom**: static proof or smoke check fails in CI but passes locally
- **root cause**: regex anchors or whitespace-sensitive patterns differed between rg versions
- **fix**: use fixed-string `grep -Fq`
- **guard**: all workflow YAML assertions in `tests/release_deploy_automation_static_test.sh` are now fixed-string
- **test**: re-run `./tests/release_deploy_automation_static_test.sh` and observe PASS in CI logs

## stale_version_surface

- **symptom**: visible version in Settings, menubar, tauri.conf, root Cargo.toml drifts from latest tag
- **root cause**: manual tag pushes did not run stamp script
- **fix**: run `python3 scripts/stamp-menubar-version.py <tag>` then `python3 scripts/verify-version-surfaces.py <tag>` and commit
- **guard**: `verify-version-surfaces.py` is part of the release workflow
- **test**: release workflow post-stamp verification step

## missing_ci_gate_passing

- **symptom**: deploy gate errors `Deploy blocked: no successful CI push run on main for <sha>`
- **root cause**: target commit's CI run had not gone green
- **fix**: this is the intended behavior; do not bypass
- **guard**: deploy workflow CI gate step
- **test**: re-run after CI for that SHA is green

## infrastructure_blocked

- **symptom**: deploy transport (SSH) unreachable from GH-hosted runner
- **root cause**: VPS firewall blocks inbound SSH from external IPs
- **fix**: install self-hosted GitHub runner on VPS
- **guard**: `actions: read` permission + `runs-on: [self-hosted, linux, x64, focusa-deploy]`
- **test**: live deployment workflow dispatch proves path

## disk_pressure

- **symptom**: `guardian check disk` returns critical
- **root cause**: rebuildable cargo/target or unused /tmp artifacts
- **fix**: deploy preflight runs `scripts/safe-disk-cleanup.sh --apply`
- **guard**: deploy workflow preflight step
- **test**: static proof references `MIN_FREE_GB`

## permission_denied

- **symptom**: `as-user` runs cannot read root-owned toolchain
- **root cause**: Rust toolchain installed by root
- **fix**: deploy scripts invoked via narrow sudoers rule
- **guard**: `/etc/sudoers.d/focusa-github-runner`
- **test**: live deploy proof

## hostname_assumption

- **symptom**: runner registered with wrong name
- **root cause**: FQDN used for runner name
- **fix**: `hostname -s` short form
- **guard**: installer logs name before config
- **test**: dry-run output

## policy_violation

- **symptom**: `git push` blocked by `bd-evidence` hook
- **root cause**: closed bead lacked explicit evidence citations
- **fix**: reopen + close with `Evidence citations:` line
- **guard**: bead policy hook enforces citation form
- **test**: actual push attempt