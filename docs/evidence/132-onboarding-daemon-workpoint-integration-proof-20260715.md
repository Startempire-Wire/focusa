# Spec 132 Onboarding Daemon and First Workpoint Integration Proof

Date: 2026-07-15

Bead: `focusa-w26jj.9.3.3`

Platform proven: Linux x86_64, live systemd daemon

## Scope

Prove a fresh project onboarding run reaches a real canonical Trajectory and first Workpoint while the daemon, service, CLI, TUI, login-shell PATH, and Pi extension are ready. Demo-only Workpoints are explicitly rejected by the runtime test.

## Defect closed

`WorkpointCheckpointReason` supported `session_start`, but the API JSON guard rejected that value before the checkpoint handler. Onboarding therefore created a canonical Trajectory but could not create or resume its first Workpoint.

The guard now accepts `session_start`, with a focused unit regression test. Onboarding now:

1. defines a canonical project Trajectory through `/v1/trajectory/define-goal`;
2. checkpoints `focusa-onboard-first-mission` through `/v1/workpoint/checkpoint`;
3. resumes the same scoped canonical Workpoint through `/v1/workpoint/resume`;
4. reports additive `trajectory`, `workpoint`, and `resume` JSON fields without daemon-global project selection.

## Live proof

Installed runtime:

- `focusa 0.9.94-dev`, SHA-256 `1f954990a2481bcf933b9da289159dd0bd7e02b4e50cf7bdac60fef85725c58d`
- `focusa-daemon 0.9.94-dev`, SHA-256 `89de6722f5cfe7490045cc9df73702b43fa6d47f56f11859ab692e25bb24eda9`
- `focusa-tui 0.9.94-dev`, SHA-256 `2ba22e6c9d7c836d3ee0bdfbcdc17962732c593da4233bed170a578d7d2a353e`
- `focusa-daemon.service`: `ActiveState=active`, `SubState=running`, PID `2295847` during proof

Authoritative gates:

```text
focusa-api scope_guard_accepts_session_start_checkpoint_reason: PASS
cargo clippy -p focusa-api --all-targets -- -D warnings: PASS
cargo clippy -p focusa-cli --all-targets -- -D warnings: PASS
tests/spec_focusa_112_onboard_scoped_static_test.sh: PASS
tests/onboard_json_quiet_runtime_test.sh: PASS
tests/onboard_clean_scope_runtime_test.sh: PASS
tests/onboard_runtime_integration_test.sh: PASS
```

The end-to-end runtime test verifies:

- active systemd service and healthy daemon API;
- `focusa`, `focusa-daemon`, `focusa-tui`, and `pi` on current and login-shell PATH;
- Pi extension loading via the registered `--no-focusa` option;
- remote-derived project marker and canonical project identity;
- canonical Trajectory creation and scoped retrieval;
- canonical first Workpoint checkpoint, resume, and scoped retrieval;
- absence of `demo workpoint` and `focusa-onboard-demo` substitutions.

## Truth boundary

This is Linux runtime proof only. It does not claim macOS or Windows installer proof. `tests/spec_focusa_117_newbie_onboarding_qa_static_test.sh` still reports its pre-existing README-link baseline gap; that unrelated documentation gate was not changed or closed here.
