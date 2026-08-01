#!/usr/bin/env python3
"""Static release regression for bounded Pi semantic event integration."""
from pathlib import Path

root = Path(__file__).resolve().parents[1]
wrap = (root / "crates/focusa-cli/src/commands/wrap.rs").read_text()
polish = (root / "apps/pi-extension/src/polish.ts").read_text()
telemetry = (root / "crates/focusa-api/src/routes/telemetry.rs").read_text()
index = (root / "apps/pi-extension/src/index.ts").read_text()

assert 'is_tui && harness_name == "pi"' in wrap
assert "run_interactive(harness_path" in wrap
assert "MAX_RECORDING_BYTES" in wrap
assert "String::from_utf8_lossy(&bytes[..limit])" in wrap
assert 'schema: "focusa.pi_semantic_event.v1"' in polish
assert "MAX_OFFLINE_SPOOL = 64" in polish
assert "pi-semantic-spool.json" in polish
assert 'focusaFetch("/ecs/store"' in polish
assert "artifact_handle" in polish
assert 'get("semantic_event")' in telemetry
assert '"status": "duplicate"' in telemetry
assert 'scope_kind: "host"' in index
assert "verifiedScopeRefForRoot(projectRoot)" in index
assert "if (!verifiedScopeRefForRoot(projectRoot)) return extensionKey" in index
print("Pi semantic event bridge and nonblocking bootstrap static gate: PASS")
