# Spec 133 Phase 7 gate — operator surfaces and rehydration

Date: 2026-07-23
Bead: `focusa-a6yq6.8.6`
Scope: Spec 133 §22

## Phase implementation commits

- `82350b0a` — persistent daemon SQLite dashboard;
- `8677b36c` — cursor-safe live views and complete guarded controls;
- `4fcd3776` — deduplicated multi-channel notifications;
- `d2d576d5` — exact creation wizard and context bounds;
- `98ff625d` — daemon-only Pi awareness and menubar card.

## Aggregate operator gate

`tests/spec133_phase7_operator_gate.sh` is wired into strict CI after Phase 6 and requires all five leaf evidence artifacts. It runs focused core notification/wizard, API Silent Session, CLI silent command, menubar check/test, and Pi extension typecheck/test suites.

Required server E2E behavior:

- dashboard survives Pi/plugin restart because SQLite/daemon is source;
- Pi status and menubar independently rehydrate from daemon APIs;
- all live modes preserve cursors across filtered events;
- every operator control uses exact run/generation, approval, lease, event, and receipt guards;
- handoff requires transfer/writer refs;
- all notification triggers dedupe across configured channels;
- exact thirteen-step wizard keeps mutation disabled through preview;
- summaries remain bounded and full output stays behind cursor/artifact handles;
- no projection mints authority or requires foreground Pi.

## Local non-building proof

Per operator policy, the server gate was not executed locally.

```bash
bash -n tests/spec133_phase7_operator_gate.sh
python3 - <<'PY'
from pathlib import Path
import yaml
yaml.safe_load(Path('.github/workflows/ci.yml').read_text())
PY
git diff --check
```

Result: passed.

## Required server proof

```bash
bash tests/spec133_phase7_operator_gate.sh
cargo test -p focusa-core
cargo test -p focusa-api
cargo test -p focusa-cli
cargo clippy -p focusa-core -p focusa-api -p focusa-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Browser/desktop proof must additionally cover cross-Pi and plugin restart, waiting-input notification/action UX, dashboard attention state, bounded context, and daemon-only operation.

## Gate disposition

Phase 7 implementation and local static review are complete and CI-gated. The phase remains unproven until the server executes the gate and E2E matrix successfully.
