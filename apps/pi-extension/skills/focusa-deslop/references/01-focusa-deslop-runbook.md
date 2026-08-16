# Deslop runbook — Focusa (2026-08-16)

## The slop taxonomy (7 Deadly Sins + Focusa extensions)

1. **Context stuffing** — whole files pasted where a function ref
   would do; preloads unbounded.
2. **Code bloat / overengineering** — defensive checks on
   trusted/validated paths, unnecessary wrappers, speculative
   generality.
3. **Silent rewrites** — changing behavior while "cleaning".
4. **Context smashing** — compacted context treated as authority
   (Focusa: the scoped Workpoint packet outranks transcript tail).
5. **Debugging by diffusion** — random diffs without evidence;
   (Focusa: diagnostics before patch).
6. **Stale docs** — comments/docs contradicting code.
7. **Opaque decisions** — magic values, unexplained invariants.

Focusa extensions:
8. **Envelope drift** — error/tool envelopes re-typed instead of the
   canonical constructors (audit: `scripts/audit-error-envelope-parity.mjs`).
9. **Route wrapper duplication** — the `Ok(Err(...)) → Json(...)`
   match arms (converge through error_envelope::internal_error).
10. **Type escapes** — `as any` in TS / `unwrap_or_default()` noise
    where a typed path exists.
11. **False greens** — gate chains where the pipe's exit masks the
    command's (`cmd | tail && echo GREEN` banned; pipefail + explicit
    EXIT markers required).

## The diff-scrub recipe (from the Cursor deslop command, adapted)

Against `origin/main` (or the branch base), remove:
- comments a human wouldn't add or inconsistent with the file;
- defensive try/catch + guards on trusted/validated codepaths;
- `as any` type escapes;
- style inconsistent with the file.

Rules: never change behavior; verify with the gate after scrubbing;
report a 1-3 sentence summary of what changed.

## The three-lens review (multi-agent protocol)

Run the review as THREE lenses — one per surface family — never one
blob review:
- **core lens** — crates/focusa-core (+focusa-license): types,
  reducers, stores, invariants, replay determinism.
- **api lens** — crates/focusa-api: routes, envelopes, permissions,
  the commit boundaries, ledger guards.
- **cli/extension lens** — focusa-cli, focusa-terminal-ui,
  apps/pi-extension: schemas, strict params, envelope shapes, parity.

Each lens reports: slop findings (with file:line), severity, and the
exact fix. Findings go to the same output shape so the main agent can
close the loop.

## The checks model (ampcode-style invariants)

Codebase invariants live as named checks the review MUST verify —
currently:
- `scripts/audit-error-envelope-parity.mjs` (0 legacy envelopes);
- `scripts/audit-skill-ownership.mjs` (triples verified);
- `scripts/audit-tool-taxonomy.mjs` (zero real dups);
- `scripts/audit-distribution-parity.mjs` (source == installed);
- `scripts/audit-doc-coverage.sh` (rustdoc baseline);
- `scripts/e2e-live-route-matrix.mjs` (21/21 live);
- `scripts/e2e-workspace-gate.sh` (workspace tests, pipefail-safe);
- deslop itself (`.deslop.toml` ceiling).

Each check = one agent pass in review; no check may be skipped.

## The close-the-loop procedure

1. `deslop .` (or CI artifact) → worst offenders first.
2. Three-lens review of the diff.
3. Fix findings (behavior-preserving only).
4. Re-run every check + the e2e matrix.
5. Summarize: what changed, why, evidence refs.
