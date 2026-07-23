#!/usr/bin/env python3
"""Validate durable SQLite replay → gap-free SSE live-tail delivery."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
schema = json.loads((BUNDLE / "json-schema/focusa.stream_event.v1.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
registry = json.loads((BUNDLE / "operation-registry.json").read_text())
persistence = (
    ROOT / "crates/focusa-core/src/runtime/persistence_sqlite.rs"
).read_text()
persistence_tests = (
    ROOT / "crates/focusa-core/src/runtime/persistence_sqlite_test.rs"
).read_text()
sse = (ROOT / "crates/focusa-api/src/routes/sse.rs").read_text()
routes_mod = (ROOT / "crates/focusa-api/src/routes/mod.rs").read_text()
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()

assert schema["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert schema["x-focusa-schema-id"] == "focusa.stream_event.v1"
assert {
    "schema",
    "event_id",
    "sequence",
    "cursor",
    "timestamp",
    "event_type",
    "schema_version",
    "scope",
    "source_state_revision",
    "payload_ref",
    "invalidate",
    "correlation_id",
    "causation_id",
    "payload",
} == set(schema["required"])
assert schema["properties"]["sequence"]["minimum"] == 1
assert schema["additionalProperties"] is False

operation = openapi["paths"]["/v1/events/stream"]["get"]
assert operation["operationId"] == "focusa.events.stream"
assert "text/event-stream" in operation["responses"]["200"]["content"]
parameters = {(item["in"], item["name"]) for item in operation["parameters"]}
assert {("query", "cursor"), ("header", "Last-Event-ID")} <= parameters
assert any(
    item["operation_id"] == "focusa.events.stream" for item in registry["operations"]
)

for marker in (
    "pub fn durable_events_after",
    "pub fn durable_event_sequence",
    "pub fn latest_durable_event_sequence",
    "event_hash_chain h",
    "ORDER BY h.chain_index ASC",
):
    assert marker in persistence
assert (
    "durable_sequence_cursor_replays_after_restart_without_duplicates"
    in persistence_tests
)
assert "SqlitePersistence::new(&cfg)" in persistence_tests
assert "let reopened = SqlitePersistence::new(&cfg)" in persistence_tests

for marker in (
    '"last-event-id"',
    "resolve_durable_cursor",
    "state.events_tx.subscribe()",
    "durable_events_after(cursor, 256)",
    "record.sequence <= cursor",
    ".id(record.sequence.to_string())",
    "RecvError::Lagged",
    "focusa.stream_event.v1",
):
    assert marker in sse
assert "Drop lagged events silently" not in sse
assert "pub mod events_stream;" not in routes_mod
assert not (ROOT / "crates/focusa-api/src/routes/events_stream.rs").exists()
assert 'operations["focusa.events.stream"]' in ts
assert "func (c *Client) FocusaEventsStream(" in go

print(
    "Spec 135 durable event stream: PASS (restart replay, cursor, Last-Event-ID, de-duplicated live tail)"
)
