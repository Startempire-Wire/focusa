#!/usr/bin/env python3
"""Spec104 remaining singleton closure static proof (non-building)."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
api = ROOT / "crates/focusa-api/src/routes"

bounded = (api / "bounded.rs").read_text()
metacog = (api / "metacognition.rs").read_text()
menubar = (ROOT / "apps/menubar/src-tauri/src/main.rs").read_text()

# Bounded resource/pressure state remains mutable runtime state, but every
# mutable map is keyed by a typed Host ScopeRef storage key.
assert "fn host_runtime_scope_key() -> String" in bounded
assert "ScopeRef::host(" in bounded
for symbol in [
    "RUNTIME_RESOURCE_MODE_OVERRIDE",
    "RESOURCE_MODE_LAST_OBSERVED",
    "RESOURCE_MODE_TRANSITIONS",
    "RESOURCE_MODE_TRANSITION_OMITTED",
    "RESOURCE_MODE_HYSTERESIS_STATE",
    "PRESSURE_TRANSITION",
    "PRESSURE_LAST_ACTIVE",
    "RESPONSE_SIZE_SAMPLES",
]:
    idx = bounded.index(f"static {symbol}")
    decl = bounded[idx : idx + 180]
    assert "BTreeMap<String" in decl
assert bounded.count("host_runtime_scope_key()") >= 10

# Metacognition has no singleton store and no global runtime/metacognition dir.
assert "static METACOG_STORE" not in metacog
assert "OnceLock" not in metacog
assert "fn store()" not in metacog
assert '.join("metacognition")' not in metacog
assert '.join("scoped-metacog")' in metacog
assert "metacog_by_scope" in metacog
assert "require_workstream_key()" in metacog

# Menubar bridge state is Tauri-managed and attachment-keyed, not static.
assert "static BRIDGE_COMPLETIONS" not in menubar
assert "static BRIDGE_LISTENERS" not in menubar
assert "OnceLock" not in menubar
assert "BridgeAttachmentKey" in menubar
assert "BridgeRuntimeState" in menubar
assert ".manage(Arc::new(BridgeRuntimeState::default()))" in menubar
assert "completions_by_attachment" in menubar
assert "listeners_by_attachment" in menubar

print("spec104 remaining singleton closure static proof: ok")
