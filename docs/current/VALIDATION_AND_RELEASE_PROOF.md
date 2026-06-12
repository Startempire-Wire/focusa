# Validation and Release Proof

Current build validation should distinguish script checks from real runtime proof.

## Code checks

## Context Authority checks

Run these whenever changing action authority, Phone Bridge pairing, runtime/daemon repair, HLT/TL projection, or binary install behavior.

```bash
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_preflight_golden_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_environment_contract_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_runtime_inventory_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_binary_preflight_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority CARGO=/root/.cargo/bin/cargo tests/spec_context_authority_intent_mode_test.sh
tests/spec_context_authority_hlt_ladder_static_test.sh
tests/spec_context_authority_pair_preflight_static_test.sh
CARGO_TARGET_DIR=/tmp/focusa-target-context-authority /root/.cargo/bin/cargo check -q -p focusa-api -p focusa-cli --locked
```


```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
cargo test --workspace
cargo clippy --workspace -- -D warnings
./scripts/ci/run-spec-gates.sh
node scripts/validate-focusa-tool-contracts.mjs
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
```

## Skill checks

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
node scripts/validate-skill-hygiene.mjs
```

## Spec96 continuity/portability/resource checks

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
tests/spec96_model_tool_instruction_static_test.sh
tests/spec96_workpoint_post_compaction_resume_static_test.sh
tests/spec96_silent_sessions_tool_static_test.sh
tests/spec96_focusa_aware_context_pressure_static_test.sh
tests/spec96_portable_identity_paths_static_test.sh
tests/spec96_focus_trajectory_short_term_sync_static_test.sh
tests/spec82_low_resource_efficiency_static_test.sh
```

## Spec97 Reflex Primitive checks

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
tests/spec97_api_native_reflex_and_ontology_static_test.sh
tests/spec97_reflex_direct_route_static_test.sh
tests/spec97_reflex_runtime_dogfood_test.sh
tests/spec97_reflex_envelope_metadata_static_test.sh
tests/spec97_reflex_traverse_routing_static_test.sh
tests/spec97_reflex_primitive_registry_static_test.sh
tests/spec97_reflex_golden_scenarios_static_test.sh
tests/spec97_reflex_utility_card_static_test.sh
```

Note: `tests/spec97_reflex_runtime_dogfood_test.sh` temporarily activates LowMem through `/v1/resource/mode` to prove degraded reflex suggestions, then restores the runtime override on exit.

## Runtime proof

A real release proof should verify the installed daemon/CLI, not only shell scripts:

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
cargo build --release --bins
systemctl restart focusa-daemon
sleep 2
systemctl status focusa-daemon --no-pager -l
readlink -f /proc/$(systemctl show -p MainPID --value focusa-daemon)/exe
curl -sS --max-time 5 http://127.0.0.1:8787/v1/health | jq .
curl -sS --max-time 5 http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'
curl -sS --max-time 5 'http://127.0.0.1:8787/v1/reflex/primitives?family=recovery&limit=2' | jq '.status, .read_only, .advisory_only, (.items|length)'
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
tests/spec97_reflex_runtime_dogfood_test.sh
focusa workpoint current
focusa workpoint resume
focusa status --operator
focusa workpoint resume --copy-prompt
scripts/demo-workpoint-happy-path.sh
```

## Mac app proof

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/menubar
bun install
bun run check
bun run build
bun run tauri -- build
```

Native Tauri bundle builds are host-dependent. On Linux, the builder needs a GTK/GLib stack new enough for the resolved Tauri dependencies; the current AlmaLinux server exposes `glib-2.0` 2.56.4 and is suitable for web build proof, not native Tauri bundle proof.

## UIAI browser evidence proof

```bash
cd /home/wpuiai/uiai-engine
FOCUSA_WORKPOINT_ID=<workpoint-id> \
FOCUSA_CONTINUITY_ID=<continuity-id> \
FOCUSA_PROJECT_ROOT=${FOCUSA_PROJECT_ROOT:-<focusa-repo>} \
FOCUSA_EVIDENCE_REF="uiai-browser-diagnostics-stress:/tmp/uiai-focusa-scope-verify.json" \
SESSIONS=1 ROUNDS=1 OUT=/tmp/uiai-focusa-scope-verify.json make browser-stress
jq '.ok, .focusa_evidence' /tmp/uiai-focusa-scope-verify.json
```

Expected result: `ok=true` and `.focusa_evidence.focusa_scope` includes `workpoint_id`, `continuity_id`, `project_root`, and `evidence_ref`.

## GitHub release proof

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
gh run list --limit 6 --json databaseId,status,conclusion,workflowName,headBranch,displayTitle | jq -r '.[] | [.databaseId,.workflowName,.headBranch,.status,(.conclusion//""),.displayTitle] | @tsv'
gh release view v0.9.25-dev --json name,tagName,isDraft,isPrerelease,url,assets | jq '{tagName,name,isDraft,isPrerelease,url,assets:[.assets[].name]}'
```

For Operator Preview proof, also verify first-run and manual-continuation surfaces:

```bash
focusa onboard --agent manual
focusa status --operator
focusa workpoint resume --copy-prompt
scripts/demo-workpoint-happy-path.sh
```

For current real proof see:

- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`
- `docs/evidence/SPEC91_LIVE_TOOL_CONTRACT_PROOF_2026-04-28.md`
- `docs/evidence/PRODUCTION_RELEASE_MAC_APP_GITHUB_FIX_2026-04-28.md`
- `docs/evidence/SPEC92_FULL_ROLLOUT_PROOF_2026-04-28.md`
- `docs/current/PRODUCTION_RELEASE_COMMANDS.md`
- `docs/evidence/SPEC97_REFLEX_DIRECT_API_LIVE_PROOF_2026-05-25.md`
- `docs/evidence/PUBLIC_DOCS_RELEASE_SYNC_2026-05-26.md`


## Spec96 trajectory agent eval

Run `tests/spec96_trajectory_agent_golden_eval_test.sh` to verify Pi, CLI/API, and generic-agent trajectory prompts outperform without-trajectory baselines on mismatch, compaction, degraded, drift, assistance-reduction, and evidence-based DoD scenarios.
