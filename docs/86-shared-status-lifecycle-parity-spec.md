# 86 — Shared-Status Lifecycle Parity Closure Spec

**Date:** 2026-04-21  
**Status:** active (spec-first)  
**Epic:** `focusa-9uow`

## Problem
Shared status vocabulary/lifecycle transitions may not be uniformly represented across reducer state, API payloads, CLI output, and tests.

## Scope
- `focusa-9uow.1` lifecycle matrix
- `focusa-9uow.2` mapping implementation
- `focusa-9uow.3` tests + evidence

## Requirements
1. Status values and transition rules must be documented in one lifecycle matrix.
2. Reducer->API->CLI mappings must be explicit and stable.
3. Backward compatibility aliases must be explicit and minimal.
4. Tests validate both vocabulary parity and transition validity.

## Execution Order (strict)
1. Build matrix (`.1`).
2. Implement mappings (`.2`).
3. Add tests/evidence (`.3`).

## Validation Gates
- Compiles pass.
- Lifecycle parity tests pass.
- Evidence markdown generated under `docs/evidence/`.

## Success Criteria
- Shared status lifecycle is consistent, test-enforced, and operationally observable.
