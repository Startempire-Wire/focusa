# 85 — Relation-Type Parity Closure Spec

**Date:** 2026-04-21  
**Status:** active (spec-first)  
**Epic:** `focusa-wkw5`

## Problem
Relation/link type vocabulary can drift between constants, projections, and CLI/test surfaces, causing ontology inconsistency.

## Scope
- `focusa-wkw5.1` audit matrix
- `focusa-wkw5.2` implementation parity
- `focusa-wkw5.3` tests + evidence

## Requirements
1. Canonical relation-type list and aliases must be explicit.
2. API world/primes + CLI must project consistent normalized relation names.
3. Compatibility aliases allowed only when documented.
4. Tests enforce exact parity (plus explicit alias exceptions).

## Execution Order (strict)
1. Build matrix (`.1`).
2. Implement parity (`.2`).
3. Add tests/evidence (`.3`).

## Validation Gates
- Compiles pass.
- Relation parity tests pass.
- Evidence markdown produced under `docs/evidence/`.

## Success Criteria
- Relation vocabulary consistent and auditable across runtime surfaces.
