# Production consistency policy — DEFAULT for every Focusa feature

Operator directive (2026-08-16): every Focusa feature must work on
every supported version and across multiple environments, consistently,
in production. This is the DEFAULT; a feature is not "done" until all
five proofs exist.

## The five mandatory proofs

1. **Versioned contract** — every wire payload carries a schema
   identifier; additive changes keep old consumers working
   (`#[serde(default)]` on every new field).
2. **Producer-side tests** — unit + integration tests for the emitting
   side (crates).
3. **Consumer-side tests** — the RECEIVING side is tested against the
   exact envelope (extension contract tests). Producer-green is not
   delivery-green (see the SSE drop root cause).
4. **Cross-version interop** — old-shape and new-shape payloads both
   parse on the current consumer; current producers emit shapes old
   consumers tolerate.
5. **Live e2e proof** — the e2e matrix exercises the feature against a
   running daemon across the supported environments (loopback dev,
   production host, remote build host); the matrix runs in the
   release gate.

## Application to the bg notification feature

- Contract: focusa.stream_event.v1 + event_type
  background_job_completion + output_tail (serde-defaulted).
- Producer tests: background_jobs suite (envelope fields) ✓.
- Consumer tests: bgCompletionEnvelopeContract (all fields present,
  output_tail bounded) ✓ — plus the legacy-shape test (no output_tail)
  added 2026-08-16.
- Interop: old envelopes without output_tail must still notify; new
  envelopes on old consumers degrade to the banner-only path.
- Live proof: SSE capture → bg run → completion event with tail
  observed at a real consumer (the SSE live-tail fix, deployed).
