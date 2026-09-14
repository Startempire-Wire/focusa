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

## Surface conformance matrix (every surface, consistent)

| Surface | Contract | Producer tests | Consumer tests | Interop | Live proof |
| --- | --- | --- | --- | --- | --- |
| focusa-core | schema-typed modules | cargo suite per module | callers' tests | legacy snapshot parse (waypoint compat) | workspace gate |
| focusa-api (daemon) | route schemas + error envelope | route tests + clippy | SSE/envelope contract tests | old-version state load (fail-closed + compat) | e2e matrix (live daemon) |
| focusa-cli | clap schemas | cli tests | daemon routes it consumes | old-daemon responses tolerated | e2e matrix + bg self-test |
| focusa-terminal-ui | typed frames | tui tests | daemon SSE consumer | replay cursor compat | manual + matrix |
| focusa-license | facade contract | 9/9 suite | core engine (single source) | legacy tier parsing | entitlement live check |
| apps/pi-extension | tool schemas + toolResult | 19 test files | envelope contract tests | legacy envelope shape test | e2e matrix (Pi side) |
| apps/menubar | svelte props | tauri tests | daemon routes | version surfaces (stamp scripts) | parity gate |
| .pi/skills | frontmatter + manifest | skill-ownership audit | agent consumption (progressive disclosure) | packaged copies parity | release gate |
| docs/ | schemas + digests | doc-coverage audit | llms.txt + agent card consumers | version stamps | release gate |
| scripts/CI | gate exit codes | workspace gate | release gate chain | pipefail-safe (no false greens) | CI runs |
| Infra (build host, deploy) | musl static target | remote build | deployed binary self-check | glibc-2.28 compat | deploy smoke checks |

Every surface's row must be green before a release. A surface with a
missing proof is a release blocker, not a follow-up.
