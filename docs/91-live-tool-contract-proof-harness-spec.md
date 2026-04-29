# Spec91 — Live Tool Contract Proof Harness

## 1. Purpose

Build a live proof harness that verifies the current Focusa runtime is serving the same tool contracts that the repository defines, and that the daemon, ontology API, docs, static registry, and validation scripts agree before release evidence is recorded.

Spec90 created the canonical tool contract registry and ontology projection. Spec91 proves the installed/running system is actually aligned with that registry.

## 2. Problem statement

Static parity can pass while the running daemon is stale, unreachable, or serving an older ontology projection. Focusa needs a repeatable live release proof that catches those mismatches without requiring unsafe mutations or real user data.

## 3. Non-goals

- Do not mutate Focus State, Workpoint state, Work-loop state, or metacognition state during default proof.
- Do not call mutating Pi tools with synthetic data unless an explicit safe fixture mode is added later.
- Do not expose secrets in proof logs.
- Do not replace Spec90 contract validation; Spec91 layers runtime proof on top of it.
- Do not require remote network access; proof targets the local daemon.

## 4. Current baseline

Current implemented surfaces from Spec90:

- `apps/pi-extension/src/tool-contracts.ts`
- `docs/current/focusa-tool-contracts.json`
- `GET /v1/ontology/tool-contracts`
- `scripts/validate-focusa-tool-contracts.mjs`
- `focusa_tool_doctor` contract coverage output

## 5. Proof harness requirements

The harness MUST verify:

1. Repository static validation passes.
2. Local daemon `/v1/health` is reachable and healthy.
3. Live `/v1/ontology/tool-contracts` returns `version=spec90.tool_contracts.v1`.
4. Live contract count equals static registry count.
5. Live contract names exactly match static registry names.
6. Live contract payload exactly matches `docs/current/focusa-tool-contracts.json` for deterministic fields.
7. API route inventory includes `/v1/ontology/tool-contracts`.
8. Required docs exist and link the proof harness/spec.
9. Output includes redacted, bounded evidence only.

## 6. Result format

The script MUST support:

```bash
node scripts/prove-focusa-tool-contracts-live.mjs
node scripts/prove-focusa-tool-contracts-live.mjs --json
```

Human output MUST include:

- `status`
- daemon health status
- static contract count
- live contract count
- mismatch count
- checked endpoint list

JSON output MUST include:

- `status`
- `health`
- `static_version`
- `live_version`
- `static_count`
- `live_count`
- `missing_live_contracts`
- `extra_live_contracts`
- `payload_equal`
- `checked_endpoints`
- `failures`

## 7. Safety/privacy requirements

- Do not print raw secrets.
- Do not include env vars in evidence output.
- Do not dump full contract JSON in default human output.
- Keep proof logs bounded.
- Use `guardian scan` before release evidence commit.

## 8. Bead decomposition requirements

Minimum beads:

1. Author Spec91 live proof harness spec.
2. Implement live proof script.
3. Add current documentation for live proof usage.
4. Wire README/docs index/changelog links.
5. Record proof evidence and secret scan.
6. Optional future: safe fixture mode for non-destructive live Pi tool class probes.

## 9. Acceptance criteria

Initial implementation is complete when:

- Spec91 exists and is linked.
- A live proof script exists and passes against the current daemon.
- Script validates static registry and live ontology projection match 43/43 contracts.
- Docs explain how to run the proof.
- Evidence records command outputs without secrets.
- Beads are created and closed/left-open according to current completion.
- Changes are committed, tagged, and pushed.

Full implementation is complete when:

- Safe fixture mode can exercise representative read-only and non-destructive tool classes.
- Release automation gates on both Spec90 static validation and Spec91 live proof.
- Doctor output can cite the latest live proof result/ref.

## 10. Implementation sequence

1. Create this spec.
2. Create bead epic/subtasks.
3. Implement `scripts/prove-focusa-tool-contracts-live.mjs`.
4. Add `docs/current/LIVE_TOOL_CONTRACT_PROOF.md`.
5. Update README, docs index, changelog.
6. Run static validation, live proof, TypeScript compile, Rust check, Guardian scan.
7. Record evidence.
8. Commit/tag/push.
