# Spec 133 Phase 6 gate — reconstruction and outcome receipts

Date: 2026-07-23
Bead: `focusa-a6yq6.7.7`
Scope: Spec 133 §19–§20

## Phase implementation commits

- `9b637086` — exact Ask and Focusa authority integration;
- `de1bf619` — Context Cognition, ontology, and action-specific Context Authority;
- `266a4750` — runtime and semantic checkpoint policy;
- `61f6b42b` — completion evidence and fail-closed decision;
- `7cbf45e2` — Spec119 receipt projections and governed closure;
- `1a8413a7` — Session Transfer, prediction, and metacog bindings.

## Transcript-free reconstruction

`TranscriptFreeReconstructionBundle` reconstructs a session without transcript content from:

- exact original Ask;
- effective config ref/hash;
- ProjectIdentity, Continuity, workspace;
- requested/effective/observed model;
- Workpoint checkpoint history;
- output cursor and event refs;
- starting/ending git status, bounded/full diff, files changed;
- evidence, receipts, and Session Transfer refs;
- final completion evaluation ref.

`reconstruct_without_transcript` requires a matching `work_session` receipt for the same session/run/project/continuity/latest Workpoint/final evaluation. It returns `transcript_required = false` only after every durable ref validates. Missing refs or mismatched scope fail closed.

## No exit-equals-done

`project_work_session_outcome_receipt` emits:

- `completed` only for a completed evaluation, completed lifecycle, and committed receipt;
- `blocked` only for blocked evaluation/lifecycle;
- `failed` only for failed evaluation/lifecycle;
- no final receipt for `incomplete`/`completing` process-exit-only state.

Every payload explicitly includes `process_exit_is_completion = false`.

## Server CI gate

`tests/spec133_phase6_evidence_gate.sh` is wired into strict CI after Phase 5. It requires all six leaf evidence artifacts and runs focused authority, authorization, bootstrap, checkpoint, completion, receipts, continuation, reconstruction, runner mutation, and API Silent Session tests.

Per operator policy, the gate was **not executed locally**.

## Local non-building proof

```bash
bash -n tests/spec133_phase6_evidence_gate.sh
python3 - <<'PY'
from pathlib import Path
import yaml
yaml.safe_load(Path('.github/workflows/ci.yml').read_text())
PY
rustfmt --edition 2024 --check <changed Rust files>
git diff --check
```

Result: passed.

## Required server proof

```bash
bash tests/spec133_phase6_evidence_gate.sh
cargo test -p focusa-core
cargo test -p focusa-session-runner
cargo test -p focusa-api
cargo clippy -p focusa-core -p focusa-session-runner -p focusa-api --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Server proof must include completed/blocked/failed outcome receipts, incomplete process-exit rejection, transcript-free reconstruction, missing-ref and scope-mismatch rejection, and every Phase 6 leaf test.

## Gate disposition

Phase 6 implementation and local static review are complete and CI-gated. The phase remains unproven until the server executes the gate successfully; no local completion claim substitutes for that evidence.
