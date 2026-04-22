# SPEC84 Runtime Evidence — Action-Type Parity

**Date:** 2026-04-21  
**Spec:** `docs/84-action-type-parity-spec.md`  
**Epic:** `focusa-lpim`

## Implemented parity surface
- Added CLI ontology command domain:
  - `focusa ontology primitives`
  - `focusa ontology world`
  - `focusa ontology contracts`

## Validation commands
```bash
timeout 180 cargo check -p focusa-cli
timeout 20 cargo run -q -p focusa-cli -- ontology primitives
timeout 200 bash tests/spec84_action_type_parity_runtime_test.sh
```

## Results
- CLI compile: pass
- `focusa ontology primitives`: returned action/link counts
- parity test: `PASS expected=91 api=91 cli=91`

## Notes
- Canonical source list currently includes one duplicate literal (`evaluate_retention`) in the constant array; parity test normalizes to unique sets for drift detection.

## Conclusion
Action-type parity surface now exists on CLI and runtime parity checks are enforced by exact-set comparison across canonical constant, API, and CLI JSON output.
