# Operator Preview Release Checkpoint — 2026-05-26

## Current head

- Head: `915d3c9` — `test: guard operator preview public docs`
- Branch: `main`
- Posture: source-available commercial Operator Preview, Workpoint-first and proof-first.

## Shipped Operator Preview increments

- `391ffac` — `feat: add operator preview onboarding`
- `ccd4449` — `feat: add golden workpoint demo`
- `704b7fb` — `feat: add operator session card`
- `9ba6d04` — `feat: add workpoint copy prompt`
- `6191746` — `docs: sync operator preview public docs`
- `915d3c9` — `test: guard operator preview public docs`

## Buyer-visible command surface

```bash
focusa onboard --agent manual
focusa status --operator
focusa workpoint resume --mode compact_prompt
focusa workpoint resume --copy-prompt
scripts/demo-workpoint-happy-path.sh
```

## Public documentation updated

- `README.md`
- `docs/README.md`
- `docs/current/CLI_REFERENCE_CURRENT.md`
- `docs/current/CURRENT_RUNTIME_STATUS.md`
- `docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md`
- `docs/current/VALIDATION_AND_RELEASE_PROOF.md`
- `docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md`
- `docs/current/AGENT_COMMAND_COOKBOOK.md`
- `docs/current/NON_PI_AGENT_FOCUSA_USAGE.md`
- `docs/current/WORKPOINT_LIFECYCLE_GUIDE.md`
- `docs/evidence/PUBLIC_DOCS_OPERATOR_PREVIEW_SYNC_2026-05-26.md`

## Local verification completed

```bash
tests/spec96_public_docs_current_features_static_test.sh
rg -n "status --operator|copy-prompt|demo-workpoint|focusa onboard" README.md docs/current docs/README.md templates
bash -n scripts/demo-workpoint-happy-path.sh
focusa status --operator
focusa workpoint resume --copy-prompt
git diff --check
```

## CI gate status at checkpoint creation

- `915d3c9`: GitHub Actions run `26435946721` in progress.
- `6191746`: GitHub Actions run `26435814500` in progress.
- Prior Operator Preview commits `9ba6d04`, `704b7fb`, `ccd4449`, `391ffac`, `ea520a4`, and `178a3f3` were green at checkpoint creation.

## Release readiness rule

Do not tag/publish the Operator Preview candidate until the latest head run for `915d3c9` is green or an equivalent rerun is green.

## Known limits to preserve in release language

- Pi remains the best-supported deep harness path.
- Manual mode is supported through CLI/API continuation cards, not every agent-specific adapter.
- Menubar GUI is not the primary Operator Preview surface.
- Work-loop, metacognition, ontology, and governance surfaces are advanced preview areas unless current docs mark a specific command implemented.
- Team/multi-user, cloud sync, marketplace, and enterprise RBAC remain future surfaces.
