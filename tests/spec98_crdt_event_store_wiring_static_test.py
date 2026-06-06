#!/usr/bin/env python3
"""Spec98 Phase 5: CRDT/event-store production wiring contract guard."""
from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.24-crdt-event-store-wiring.yaml"
CRDT = ROOT / "crates/focusa-core/src/sync/crdt.rs"
PERSIST = ROOT / "crates/focusa-core/src/runtime/persistence_sqlite.rs"
TYPES = ROOT / "crates/focusa-core/src/types.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.crdt_event_store_wiring_contract.v1":
        fail("unexpected contract schema_version")
    if data.get("status") != "crdt_event_store_gap_mapped":
        fail("contract must explicitly map current CRDT/event-store gap")

    crdt = CRDT.read_text()
    for token in ["pub struct VectorClock", "pub struct CrdtEvent", "pub struct CrdtLog", "pub fn merge_remote", "pub struct ConflictResolver", "lamport_ts"]:
        if token not in crdt:
            fail(f"CRDT primitive missing {token}")
    if "self.events.sort_by" not in crdt or "vector_clock.compare" not in crdt:
        fail("CRDT log must sort by causal vector clock")

    persist = PERSIST.read_text()
    for token in ["CREATE TABLE IF NOT EXISTS events", "CREATE TABLE IF NOT EXISTS event_hash_chain", "pub fn append_event", "payload_sha256", "event_chain_hash"]:
        if token not in persist:
            fail(f"SQLite event-store primitive missing {token}")
    for idx in ["idx_events_machine", "idx_events_session", "idx_events_thread"]:
        if idx not in persist:
            fail(f"event index missing {idx}")

    types = TYPES.read_text()
    event_struct_start = types.find("pub struct EventLogEntry")
    if event_struct_start < 0:
        fail("EventLogEntry missing")
    event_struct = types[event_struct_start:types.find("// ─── Workers", event_struct_start)]
    for field in ["machine_id", "instance_id", "session_id", "thread_id", "is_observation"]:
        if field not in event_struct:
            fail(f"EventLogEntry missing sync field {field}")

    required = data.get("required_production_wiring") or {}
    columns = set(required.get("event_store_schema", {}).get("required_future_columns") or [])
    for col in ["project_root_key", "workstream_key", "vector_clock_json", "lamport_ts", "mutation_class", "causal_parent_ids_json"]:
        if col not in columns:
            fail(f"contract missing future event-store column {col}")
    proofs = set(data.get("proof_requirements") or [])
    for proof in ["static CRDT primitives present", "static SQLite event hash chain present", "static production gap is explicit"]:
        if proof not in proofs:
            fail(f"contract missing proof requirement: {proof}")
    print("✓ PASS: CRDT/event-store wiring foundation and explicit production gap are mapped")


if __name__ == "__main__":
    main()
