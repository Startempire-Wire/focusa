# 84 — Action-Type Parity Closure Spec

**Date:** 2026-04-21  
**Status:** active (spec-first)  
**Epic:** `focusa-lpim`

## Problem
Action vocabulary drift risk exists across docs, ontology constants, API projections, CLI surfaces, and tests.

## Scope
- `focusa-lpim.1` audit matrix
- `focusa-lpim.2` implementation parity
- `focusa-lpim.3` tests + evidence

## Requirements
1. Canonical action-type source must be explicit and referenced.
2. API world/primes and CLI surfaces must expose aligned action vocab.
3. Any aliasing must be documented and test-allowed only where intentional.
4. Contract tests must fail on missing/extra action types.

## Execution Order (strict)
1. Build matrix (`.1`).
2. Implement parity (`.2`).
3. Add tests/evidence (`.3`).

## Validation Gates
- Relevant Rust/TS compiles pass.
- Parity tests pass.
- Evidence file generated under `docs/evidence/`.

## Success Criteria
- No unintended action-type drift across canonical surfaces.
- Reproducible proof artifact available.
