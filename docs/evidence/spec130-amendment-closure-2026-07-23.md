# Spec 130 amendment closure proof — A4/A5

Date: 2026-07-23
Scope: Spec 130 §§38–54 amendment only
Beads: `focusa-w26jj.3.11`, `focusa-w26jj.3.12`

## Outcome

The rotating-continuity, real-OOM recovery, bounded persistence, crash recovery, capability-truth, and soak gates in §§52–53 pass. This closes the A4/A5 amendment work; it does not by itself claim the broader release/293-MUST gate or installed-provider cost telemetry.

Implementation/proof commits:

- `70f5e8b4` — four-hop cross-agent target-attachment runtime;
- `c5ad209e` — durable migration boundaries and idempotent target/receipt replay;
- `6635f23d` — million-event, 10,000-cycle, ten-segment soak;
- `4d5286d5` — bounded CI API-probe startup diagnostics.

## Exact proof commands

```bash
bash tests/spec130_compaction_mission_packet_static_test.sh
bash tests/spec130_bounded_persistence_test.sh
bash tests/spec130_native_session_pressure_test.sh
bash tests/spec130_adapter_capability_contract_static_test.sh
bash tests/spec130_rotating_continuity_transfer_static_test.sh
DAEMON_BIN="$PWD/target/debug/focusa-daemon" bash tests/spec130_cross_agent_handoff_runtime_test.sh
bash tests/spec130_million_event_soak_test.sh
bash tests/spec130_ci_api_probe_startup_budget_static_test.sh
bash tests/spec130_auto_compaction_test.sh
bash tests/spec130a_proactive_compaction_runtime_test.sh
npm --prefix apps/pi-extension run check
cargo clippy -p focusa-api --bin focusa-daemon -- -D warnings
```

All commands pass on the implementation worktree. The API binary used by live fixtures was built from the same source before execution.

## §52.1 static mapping

| Requirement | Proof |
|---|---|
| no volatile fields in semantic digest | `spec130_bounded_persistence_test.sh` |
| bounded anchor schema and hard cap | bounded-persistence static/runtime proof |
| no repeated full-state native write | bounded-persistence static guard |
| semantic project-switch dedupe | bounded-persistence runtime proof |
| dynamic continuity; no static continuity assumption | rotating-continuity static proof |
| adapter capability contract | adapter-capability static proof |
| Pi command-context boundary | typed `RolloverNewSession`/`withSession` static proof |
| startup preflight and streaming migration | native-session-pressure static/runtime proof |
| release closure remains evidence-gated | this matrix plus Beads closure state |

## §52.2 runtime mapping

1. Repeated unchanged persistence produces zero duplicate native appends.
2. Timestamp/telemetry-only changes retain the same semantic digest.
3. Workpoint revisions produce one bounded changed-state anchor.
4. WBM reuses the existing sidecar reference.
5. One million observations coalesce to 10,000 changed-state anchors.
6. Every soak anchor remains below the 8 KiB hard cap.
7. Normal/soft/hard/oversized/emergency pressure transitions pass.
8. Oversized startup refuses full loading and uses streaming migration.
9. Nine deterministic migration boundaries preserve the source and retry idempotently.
10. Every target continuity differs from its source and materializes canonically.
11. Pi → Claude → Codex → OpenCode → Pi preserves mission, lineage, blocker/evidence refs, and receipts.
12. Daemon-unavailable persistence remains bounded in the private sidecar while canonical claims stay blocked.

## §52.3 measured results

### Real OOM artifact

The immutable 1,366,418,944-byte artifact proof remains recorded in `docs/evidence/130-a5-real-oom-artifact-migration-proof-20260713.md`:

- source/archive SHA-256: `ba3bd327f7b04c1e6b47d86ab258db9d469282aee86917278bce9b58a16adf0c`;
- migration peak RSS: 174,208 KiB;
- bounded replacement: 8,357,644 bytes;
- replacement native load/export peak RSS: 278,448 KiB;
- canonical rotated target Workpoint and `target_resume_verified` receipt: passed.

### Million-event soak

```text
semantic_events=1,000,000
cycles=10,000
physical_segments=10
changed_state_native_appends=10,000
unchanged_state_suppressions=990,000
max_anchor_bytes=376
replay_slope_bytes_per_segment=0.1090909091
replay_rss_range_bytes=3,448,832
required_ref_loss=0
```

The replay working-set slope is statistically flat relative to total processed history.

### Crash/retry matrix

Injected boundaries:

```text
after_prepare
after_archive_write
after_archive_checksum
after_archive_seal
after_recovery_write
after_recovery_checksum
after_source_verify
after_manifest_write
after_manifest_commit
```

At every boundary: the source checksum is unchanged, partial committed and temporary files are removed, and an unchanged retry completes with archive/source checksum equality and bounded recovery.

Target materialization replay returns the original canonical target Workpoint. Repeating `verify_target` returns the original transition receipt with `idempotent_replay=true`; no duplicate receipt is appended.

### Cross-agent handoff

```text
hops=4
adapters=Pi,Claude,Codex,OpenCode,Pi
unique_target_workpoints=4
unique_transition_receipts=4
final_target_resume=canonical
capability_truth_preserved=true
```

Pi remains measured Tier B. Claude, Codex, and OpenCode remain conservative Tier D where dedicated native rollover capability is absent; Focusa target attachment does not overclaim native adapter support.

## Security and privacy

- The real source path remains omitted from repository evidence.
- Source files are never mutated; immutable archives use mode `0400`.
- Migration writes temp files, fsyncs file and parent directory, atomically links the committed name, then fsyncs directory cleanup.
- Recovery records and transfer receipts carry bounded refs/digests, not raw credentials or full private payloads.
- Runtime fixtures use temporary project/data roots and remove them on exit.

## §53 acceptance disposition

Criteria 26–45 pass through the bounded-persistence, pressure/migration, cross-agent, crash-retry, real-artifact, capability, CI-startup, and soak proofs above. Rollback always returns to the immutable source and never re-enables unbounded full-state native writes.

## Remaining broader gate

The A4/A5 amendment is complete. Broader Spec 130/release closure still requires any separately tracked installed-provider token/cache-cost and post-compaction RSS verdict that is not established by this amendment matrix; this document does not convert that external gate into completion.
