# Detour Scope Assurance + Implementation Status (2026-05-29)

## Purpose
Define the detour scope as acceptance-grade requirements (tiered assurances + confidence levels), decompose into beads, and verify whether implementation is already complete.

## Canonical Detour Need
Model-switch or auto-bootstrap must never misbind project scope. Focusa must:
1. detect and quarantine unsafe broad roots,
2. infer/verify project identity with evidence,
3. preserve already-verified in-session identity against cross-project overwrite,
4. enforce `project_root + continuity_id` as the Workpoint authority gate,
5. expose confidence/canonical/advisory clarity in operator-facing outputs,
6. provide deterministic recovery sequence when scope is stale/mismatched.

## Tiered Assurances + Confidence Boundaries

### Tier 0 — Quarantine (hard stop)
- Input condition: broad/unverified root (example: `/root`).
- Requirement: block durable project-aware writes.
- Confidence state: low/unverified.

### Tier 1 — Identity acquisition (evidence-backed)
- Requirement: `focusa_project_identity` resolves with local+remote evidence support.
- Confidence state: may rise from low -> medium/high only with corroborating signals.

### Tier 2 — Verified identity preservation (cross-project contamination guard)
- Requirement: if current in-session identity is verified (medium/high), a different incoming identity cannot overwrite it during model switch/bootstrap.
- Confidence state: preserve verified authority; reject divergent incoming scope.

### Tier 3 — Workpoint authority gate
- Requirement: canonical continuation/mutation allowed only when `project_root + continuity_id` match.
- Mismatch behavior: non-canonical/rejected continuity with explicit recovery instructions.

### Tier 4 — Transparency contract
- Requirement: tool/card outputs expose confidence tier, source, canonical vs advisory status, and best recovery tools.

### Tier 5 — Deterministic recovery route
- Required route: `project_identity -> project_verify -> trajectory_view (advisory) -> workpoint_resume/checkpoint`.

## Bead Decomposition (created + closed)

### Parent
- `focusa-khm6` — **Detour scope assurance contract verification and closure** (closed)

### Children
1. `focusa-khm6.1` — Tier 0/1 verification: broad-root quarantine + identity quorum (closed)
2. `focusa-khm6.2` — Tier 2 verification: preserve verified in-session identity (closed)
3. `focusa-khm6.3` — Tier 3 verification: project_root+continuity authority gate (closed)
4. `focusa-khm6.4` — Tier 4 verification: confidence + canonical/advisory transparency (closed)
5. `focusa-khm6.5` — Tier 5 verification: deterministic scope recovery route (closed)

## Implementation Verification Run (2026-05-29)

### Evidence: code lineage
- `b27ccde` — `fix: support remote project identity evidence`
- `2b5a0f0` — `fix: preserve verified in-session project identity on model switch`

### Evidence: static gates executed
All PASS:
- `tests/spec96_broad_root_scope_isolation_static_test.sh`
- `tests/spec96_project_identity_quorum_static_test.sh`
- `tests/spec96_utility_card_session_isolation_static_test.sh`
- `tests/spec96_stale_focus_frame_validation_static_test.sh`
- `tests/spec96_workpoint_resume_packet_v2_static_test.sh`
- `tests/spec96_scope_recovery_feedback_static_test.sh`
- `tests/spec96_trajectory_clarity_gate_static_test.sh`

### Evidence: runtime tool checks
- `focusa_project_identity(project_root=/home/wirebot/focusa)` -> `status=verified`, `confidence=high`
- `focusa_project_verify(...)` -> `verified=true`, `confidence=high`
- continuity mismatch path observed as non-canonical with scope-recovery guidance

## Status Decision
**Implemented:** YES for the original detour identity/workpoint authority tiers only.

**Not complete for anti-forgetting:** this report does not cover forced model attention, report replay after tool-output flood, or a general `AttentionRecallVerdict`. See `PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md` for the broader model-forgetting guard.

## If Future Regression Appears
If any tier fails, reopen `focusa-khm6` and implement at failing tier boundary only; preserve authority rule (`project_root + continuity_id`) and canonical/advisory separation.
