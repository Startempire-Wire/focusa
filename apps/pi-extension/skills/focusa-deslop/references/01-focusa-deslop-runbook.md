# Deslop runbook — Focusa

## Taxonomy (7 sins, compressed)

1. Context stuffing — paste files where a ref would do.
2. Bloat — defensive checks on trusted paths, speculative wrappers.
3. Silent rewrites — behavior changes while "cleaning".
4. Context smashing — compacted context as authority (packet outranks
   transcript tail).
5. Diffusion debugging — diffs without evidence (diagnostics first).
6. Stale docs — comments contradicting code.
7. Opaque decisions — magic values, unexplained invariants.

Focusa additions: envelope drift (re-typed error/tool envelopes),
false greens (pipes masking exit codes — pipefail or explicit EXIT).

## Diff-scrub recipe

Against the branch base, remove: comments a human wouldn't add,
defensive guards on trusted paths, `as any` escapes, style drift.
Never change behavior. Re-run the gates. Summarize in 1-3 sentences.

## Three-lens review

- core (types/reducers/stores/replay) · api (routes/envelopes/boundaries)
  · cli/extension (schemas/parity). One agent per lens, one output shape.

## Checks (all must pass)

envelope-parity · skill-ownership · tool-taxonomy · distribution-parity
· doc-coverage · e2e-matrix (21/21) · workspace-gate · deslop ceiling.

## Close the loop

deslop → three-lens → fix (behavior-preserving) → all checks →
summary with evidence refs.
