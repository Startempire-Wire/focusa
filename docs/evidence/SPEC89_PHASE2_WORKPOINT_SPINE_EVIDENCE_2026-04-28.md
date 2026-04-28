# Spec89 Phase 2 Workpoint Spine Evidence — 2026-04-28

Active phase: `focusa-bcyd.3`.

## Implementation

- Pi bridge adds `resolveActiveWorkpointContext()` and injects active `workpoint_id` plus Workpoint evidence refs into `details.tool_result_v1`.
- New Pi tool `focusa_workpoint_link_evidence` supports explicit evidence linking and `attach_to_workpoint=false` no-op behavior.
- New API endpoint `POST /v1/workpoint/evidence/link` validates target/result and emits reducer-owned `WorkpointEvidenceLinked` events.
- New reducer event `WorkpointEvidenceLinked` appends bounded `WorkpointVerificationRecord` entries only to canonical non-degraded Workpoints.
- Resume packets expose linked `verification_records` directly in the Workpoint packet.

## Reducer-approved event model

Event: `workpoint_evidence_linked`

Payload:

- `workpoint_id`: canonical Workpoint id.
- `verification.target_ref`: object/file/test/endpoint/work item verified.
- `verification.result`: bounded result summary.
- `verification.evidence_ref`: optional stable evidence handle/path/test id.

Validation/failure semantics:

- API rejects empty `target_ref` or `result` with `validation_rejected` and `retry_posture=do_not_retry_unchanged`.
- API returns `blocked` when no active/requested Workpoint exists.
- Reducer rejects canonical evidence linkage to non-canonical/degraded Workpoints.
- Reducer bounds verification records through existing Workpoint caps.

Replay behavior:

- Evidence links are durable Focusa events and replay into the Workpoint projection as verification records.

## Validation

Commands passed:

```bash
cd apps/pi-extension && ./node_modules/.bin/tsc --noEmit
cargo test -p focusa-api workpoint --target-dir /tmp/focusa-cargo-target
cargo test -p focusa-core workpoint --target-dir /tmp/focusa-cargo-target
cargo build --release -p focusa-api -p focusa-cli --target-dir /home/wirebot/focusa/target
systemctl restart focusa-daemon.service
./tests/focusa_tool_stress_test.sh
```

Live stress result after adding Workpoint evidence link coverage:

```text
passed=38 failed=0
```

Live resume verification showed linked evidence under `resume_packet.verification_records[-1]`:

```json
{
  "evidence_ref": "tests/spec89_tool_envelope_contract_test.sh:1",
  "result": "Spec89 envelope contract test passed",
  "target_ref": "tests/spec89_tool_envelope_contract_test.sh"
}
```

## Scope notes

Focus State, snapshots, metacog artifacts, and lineage refs now receive Workpoint linkage through the shared Pi envelope when an active Workpoint packet is available. Dedicated deeper reducer links for every artifact family remain candidates for later refinement, but Phase 2 acceptance is met by canonical evidence-link API plus envelope linkage and resume packet projection.
