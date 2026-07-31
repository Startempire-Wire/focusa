# 141 — Focusa Archived Worktree and Ref Semantic-Diff Evidence

- Locked release head: `5d1b0f4c968ed66514500ec084aed2b79d55bc44`
- Status: `verified`
- Evidence digest: `sha256:564a822a161fa34de799f5eb65ef5b0fb2ef1e4da2fa984a81abd500dddacda0`
- Preserved bundle: none present under the operator-declared search roots; refs remain preserved in Git.

- Unsettled refs: `0`

| Ref | Classification | Unique | Equivalent | Conflicts | Security-sensitive | Settlement |
|---|---:|---:|---:|---:|---:|---:|
| `agents/canary-a42` | `conflicting_candidate` | 1 | 0 | 3 | no | evidence_superseded |
| `agents/focusa-bootstrap-w2` | `conflicting_candidate` | 5 | 0 | 23 | no | evidence_superseded |
| `agents/focusa-genesis-w2` | `conflicting_candidate` | 5 | 0 | 28 | no | evidence_superseded |
| `agents/focusa-temporal-w1` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/focusa-vbcqu-w1` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/spark-bloat-circuit` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/spark-bloat-contract` | `patch_equivalent` | 0 | 1 | 0 | yes | not_required |
| `agents/spark-bloat-profile` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/spark-bloat-profile-atomic` | `patch_equivalent` | 0 | 1 | 0 | yes | not_required |
| `agents/spark-e6-runtime` | `patch_equivalent` | 0 | 1 | 0 | no | not_required |
| `agents/spark-h1-portable-proof` | `patch_equivalent` | 0 | 1 | 0 | no | not_required |
| `agents/spark-h2-runtime-matrix` | `conflicting_candidate` | 1 | 0 | 2 | yes | evidence_superseded |
| `agents/spark-h3-gate` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/spark-intro-animation` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/spec130-pi-launch` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/v135-issue15` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/worker-env-inventory` | `conflicting_candidate` | 1 | 0 | 2 | no | evidence_superseded |
| `agents/worker-h2-integrate` | `obsolete_integrated_ancestor` | 0 | 0 | 0 | no | not_required |
| `agents/worker-launch-hardening` | `patch_equivalent` | 0 | 1 | 0 | no | not_required |
| `agents/worker-onboard-json` | `patch_equivalent` | 0 | 1 | 0 | no | not_required |
| `archive/focusa-api-scope` | `conflicting_candidate` | 2 | 0 | 13 | no | evidence_superseded |
| `archive/focusa-pi-scope` | `conflicting_candidate` | 3 | 0 | 21 | no | evidence_superseded |
| `archive/focusa-spec132` | `conflicting_candidate` | 3 | 0 | 6 | no | evidence_superseded |
| `archive/focusa-spec133` | `conflicting_candidate` | 2 | 0 | 9 | no | evidence_superseded |

## Classification policy

- `obsolete_integrated_ancestor`: ref tip is an ancestor of the locked release head.
- `patch_equivalent`: all non-ancestor ref commits have patch-equivalent commits in the locked release.
- `unique_candidate`: unique patch identity remains and requires explicit integration or supersession evidence.
- `conflicting_candidate`: trial merge reports conflicts and requires explicit integration or supersession evidence.
- Security-sensitive is a review tag, never an automatic integration authorization.
- A conflicting or unique ref is settled only by an explicit integration or evidence-supersession record with stable proof refs.

## Supersession evidence

### `agents/canary-a42`

The locked release implements the rollover materialization and recovery contract under the newer Spec130A lifecycle.

- `apps/pi-extension/tests/spec130-rollover-command-lifecycle.test.mjs`
- `tests/spec130a_proactive_compaction_runtime_test.sh`
- `tests/spec130a_release_stress_runtime_test.mts`

### `agents/focusa-bootstrap-w2`

Project Bootstrap discipline, broad-root rejection, provider isolation, and post-bind verification are covered by the locked Spec143 implementation.

- `tests/spec143_project_bootstrap_release_gate_test.py`
- `tests/spec96_broad_root_scope_isolation_static_test.sh`

### `agents/focusa-genesis-w2`

Atomic Genesis activation, first-Workpoint creation, ambient bootstrap, and warm resume are covered by the locked Spec143 implementation.

- `tests/spec143_project_genesis_release_gate_test.py`
- `tests/spec143_project_bootstrap_release_gate_test.py`

### `agents/spark-h2-runtime-matrix`

The strict native updater contract matrix is covered by the locked Spec132 runtime and portable-binary gates.

- `tests/spec132_portable_binary_selection_test.sh`
- `tests/spec132_pty_lifecycle_runtime_test.sh`
- `tests/spec132_public_uninstall_preservation_test.sh`

### `agents/worker-env-inventory`

The locked installer publishes and tests the complete OS, architecture, shell, package-manager, privilege, PATH, install, license, and policy inventory.

- `tests/spec128_installer_preflight_static_test.sh`
- `crates/focusa-cli/src/commands/install.rs`

### `archive/focusa-api-scope`

The locked release closes API singleton scope across typed project/workstream identity with hostile-scope and restart coverage.

- `tests/spec104_api_scope_singleton_closure_static_test.py`
- `tests/security_api_route_scope_dynamic_test.sh`
- `tests/spec96_broad_root_scope_isolation_static_test.sh`

### `archive/focusa-pi-scope`

The locked release replaces Pi singleton shadows with typed attachment and project/workstream scope stores plus lifecycle isolation tests.

- `apps/pi-extension/tests/spec104-attachment-runtime-isolation.test.mjs`
- `tests/spec104_pi_runtime_scope_integrity_test.sh`
- `tests/spec98_pi_scope_cache_switch_handling_runtime_test.mts`

### `archive/focusa-spec132`

The archived Spec132 draft and fixture proofs are superseded by the final locked runtime, ownership, portability, and uninstall-preservation gates.

- `tests/spec132_pi_extension_ownership_test.sh`
- `tests/spec132_portable_binary_selection_test.sh`
- `tests/spec132_pty_lifecycle_runtime_test.sh`
- `tests/spec132_public_uninstall_preservation_test.sh`

### `archive/focusa-spec133`

The archived phase-zero and Pi scope work is superseded by the complete locked Spec133 supervision, isolation, evidence, and operator gates.

- `tests/spec133_phase4_runtime_gate.sh`
- `tests/spec133_phase5_isolation_gate.sh`
- `tests/spec133_phase6_evidence_gate.sh`
- `tests/spec133_phase7_operator_gate.sh`
