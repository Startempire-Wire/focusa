#!/usr/bin/env python3
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TS = (ROOT / "packages/generated/spec135/typescript/semantic-pair.ts").read_text()
RS = (ROOT / "packages/generated/spec135/rust/src/lib.rs").read_text()
API = (ROOT / "crates/focusa-api/src/routes/semantic_integrity.rs").read_text()
FIXTURE = json.loads((ROOT / "packages/generated/spec135/fixtures/semantic-pair-portfolio.json").read_text())

STATES = {
    "schema_only", "pack_missing", "migration_required", "verification_required",
    "verification_blocked", "operator_required", "unsupported_future_definition",
    "writer_blocked", "degraded", "stale", "conflicted", "quarantined",
}
OPERATIONS = {op for op in re.findall(r'(?:read_op|mutation_op)!\(\s*"([^"]+)', API) if op.startswith("semantic_pair.")}
TS_OPERATIONS = set(re.findall(r'"(semantic_pair\.[a-z_.]+)"', TS))
RS_OPERATIONS = set(re.findall(r'#\[serde\(rename = "(semantic_pair\.[a-z_.]+)"\)\]', RS))

assert STATES <= set(re.findall(r'"([a-z_]+)"', TS))
for state in STATES:
    rust_variant = ''.join(piece.title() for piece in state.split('_'))
    assert rust_variant in RS, state
assert OPERATIONS == TS_OPERATIONS == RS_OPERATIONS, (OPERATIONS - TS_OPERATIONS, OPERATIONS - RS_OPERATIONS)
assert FIXTURE["state"] in STATES
item = FIXTURE["items"][0]
for key in ("obligations", "findings", "settlement", "replay", "recovery"):
    assert key in item

surface_files = [
    "apps/pi-extension/src/mission-canvas-model.ts",
    "apps/pi-extension/src/mission-canvas-widget.ts",
    "packages/a2ui-renderer/src/semantic-pair-surface.ts",
    "crates/focusa-tui/src/views/semantic_pair.rs",
    "apps/menubar/src/lib/components/SemanticPairPeek.svelte",
]
combined = "\n".join((ROOT / path).read_text() for path in surface_files)
for state in STATES:
    assert state in combined, state
for marker in ("read_only_surface", "unsupported on TUI", "Unsupported on this surface"):
    assert marker in combined, marker
for path in surface_files:
    assert len((ROOT / path).read_text().splitlines()) < 500, path
print(f"Spec144 client/surface parity: {len(STATES)} states, {len(OPERATIONS)} operations")
