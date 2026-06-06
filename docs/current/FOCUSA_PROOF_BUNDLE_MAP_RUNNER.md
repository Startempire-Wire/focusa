# Focusa Proof Bundle Map Runner

**Status:** implemented for `focusa-877z.8.6`.

Runner: `scripts/focusa-proof-bundle`

## Purpose

Map a changed authority surface or worksheet item to required proof commands, then fail closed when no mapping exists.

## Sources

- `docs/worksheets/focusa-877z.8-authority-taxonomy.yaml` — item-level `proof_commands`.
- `docs/worksheets/focusa-877z.18-migration-side-effect-proof-plan.yaml` — cross-surface `proof_bundle_map`.
- `docs/current/FOCUSA_POLICY_PROFILE_REGISTRY.json` — extra registry proof commands for `policy_profiles.registry`.

## Examples

```bash
scripts/focusa-proof-bundle api_routes
scripts/focusa-proof-bundle policy_profiles.registry --json
scripts/focusa-proof-bundle --changed-path crates/focusa-api/src/routes/workpoint.rs
scripts/focusa-proof-bundle proof_suite --run
```

## Surface aliases

- `daemon` → `daemon_core`
- `api` → `api_routes`
- `pi` / `pi_plugin` → `pi_extension`
- `uiai` / `browser` → `uiai_external`
- `proof` → `proof_suite`

## Run mode

Default mode only prints commands. `--run` executes local-safe commands and skips external/manual commands such as `as-user wpuiai ...`.

## Failure rule

Unknown targets or targets without `proof_commands` return `status=missing_proof_mapping` and exit nonzero.
