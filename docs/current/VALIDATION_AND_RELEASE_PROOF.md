# Validation and Release Proof

Current build validation should distinguish script checks from real runtime proof.

## Code checks

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

## Spec96 continuity/portability checks

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
tests/spec96_model_tool_instruction_static_test.sh
tests/spec96_workpoint_post_compaction_resume_static_test.sh
tests/spec96_silent_sessions_tool_static_test.sh
tests/spec96_focusa_aware_context_pressure_static_test.sh
tests/spec96_portable_identity_paths_static_test.sh
```

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
node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures
focusa workpoint current
focusa workpoint resume
```

## Mac app proof

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}/apps/menubar
bun install
bun run check
bun run build
```

## GitHub release proof

```bash
cd ${FOCUSA_PROJECT_ROOT:-<focusa-repo>}
gh run list --limit 6 --json databaseId,status,conclusion,workflowName,headBranch,displayTitle | jq -r '.[] | [.databaseId,.workflowName,.headBranch,.status,(.conclusion//""),.displayTitle] | @tsv'
gh release view v0.9.12-dev --json name,tagName,isDraft,isPrerelease,url,assets | jq '{tagName,name,isDraft,isPrerelease,url,assets:[.assets[].name]}'
```

For current real proof see:

- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md`
- `docs/evidence/SPEC91_LIVE_TOOL_CONTRACT_PROOF_2026-04-28.md`
- `docs/evidence/PRODUCTION_RELEASE_MAC_APP_GITHUB_FIX_2026-04-28.md`
- `docs/evidence/SPEC92_FULL_ROLLOUT_PROOF_2026-04-28.md`
- `docs/current/PRODUCTION_RELEASE_COMMANDS.md`


## Spec96 trajectory agent eval

Run `tests/spec96_trajectory_agent_golden_eval_test.sh` to verify Pi, CLI/API, and generic-agent trajectory prompts outperform without-trajectory baselines on mismatch, compaction, degraded, drift, assistance-reduction, and evidence-based DoD scenarios.
