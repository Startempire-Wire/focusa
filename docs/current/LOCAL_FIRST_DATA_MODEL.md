# Local-First Data Model

Focusa stores cognitive runtime state locally by default and exposes it through local daemon/CLI/Pi-tool surfaces.

## Local storage classes

- Focus State and Workpoints
- Trajectory and HLT ledger
- Evidence refs and proof handles
- Device pairing ledger
- Context/eval/optimizer ledgers
- Prediction and metacognition records
- Generated current docs/proof summaries

## Principles

- Local daemon is the authority surface; adapters are thin clients.
- Append-only ledgers preserve auditability.
- Generated docs derive from source registries/scripts rather than manual counts.
- Public sharing requires redaction policy gates.
- Cross-project reads require verified project identity and scoped continuity.

## Data movement

- Pi extension caches are local shadows, not authority.
- Session transfer/checkpoint tools move bounded packets, not raw transcript dumps.
- Public stream cards use redacted scope ids and default to `publish_allowed=false`.
