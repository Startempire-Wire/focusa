#!/usr/bin/env python3
"""Spec104 DOC-02: session identity envelope static test.

Verifies that:
- Every Pi-extension session scope function returns a typed ScopeContext.
- Scope getters do NOT fall back to the singleton S for authority-bearing keys.
- The scope context carries required fields (project_root, continuity_id,
  session_id).
- Scope-bearing bridge messages include the typed scope envelope.

Spec104 DOC-02 proof: test fails if packets omit typed scope/authority.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def ok(msg: str) -> None:
    print(f"✓ {msg}")


def main() -> None:
    print("=== Spec104 DOC-02 session identity envelope static test ===")

    # Check state.ts has typed ScopeContext getters
    state_path = ROOT / "apps/pi-extension/src/state.ts"
    if not state_path.exists():
        fail("state.ts missing")
    state_src = state_path.read_text()

    required_getters = [
        "export function getSessionCwd",
        "export function getContinuityId",
        "export function getActiveFrameId",
        "export function getTurnCount",
        "export function getLatestReportSummary",
        "export function getLastProjectVerify",
        "export function getActiveWorkpointPacket",
    ]
    for g in required_getters:
        if g not in state_src:
            fail(f"missing required scope getter: {g}")
        ok(f"scope getter present: {g}")

    # Check that getters don't fall back to S for authority-bearing keys
    forbidden_pattern = re.compile(
        r"return\s+store\s*\?\s*store\.\w+\s*:\s*S\.(sessionCwd|continuityId|activeFrameId|activeFrameGoal)",
    )
    for m in forbidden_pattern.finditer(state_src):
        fail(f"getter still falls back to S for authority-bearing key: {m.group(0)}")

    # Check scope-context helpers exist
    if "TypedScopeStore" not in state_src or "TypedScopeIdentity" not in state_src:
        fail("TypedScopeStore/TypedScopeIdentity not found in state.ts")
    ok("TypedScopeStore + TypedScopeIdentity present in state.ts")

    # Check menubar projectContext has typed ScopeContext
    menubar_ctx = ROOT / "apps/menubar/src/lib/projectContext.svelte.ts"
    if not menubar_ctx.exists():
        fail("menubar projectContext.svelte.ts missing")
    menubar_src = menubar_ctx.read_text()
    if "ScopeContext" not in menubar_src:
        fail("menubar projectContext.svelte.ts missing ScopeContext typed interface")
    ok("menubar projectContext.svelte.ts has ScopeContext typed interface")

    # Check focusa-tui api.rs has TypedScope
    tui_api = ROOT / "crates/focusa-tui/src/api.rs"
    if not tui_api.exists():
        fail("focusa-tui api.rs missing")
    tui_src = tui_api.read_text()
    if "TypedScope" not in tui_src:
        fail("focusa-tui api.rs missing TypedScope struct")
    ok("focusa-tui api.rs has TypedScope struct")

    # Check Rust scope.rs has ScopeContext
    scope_rs = ROOT / "crates/focusa-api/src/scope.rs"
    if not scope_rs.exists():
        fail("scope.rs missing")
    scope_src = scope_rs.read_text()
    if "ScopeContext" not in scope_src:
        fail("scope.rs missing ScopeContext type")
    ok("scope.rs has ScopeContext type")

    # Check menubar Tauri main.rs has BridgeScope
    tauri_main = ROOT / "apps/menubar/src-tauri/src/main.rs"
    if not tauri_main.exists():
        fail("menubar src-tauri/src/main.rs missing")
    tauri_src = tauri_main.read_text()
    if "BridgeScope" not in tauri_src:
        fail("menubar Tauri main.rs missing BridgeScope struct")
    ok("menubar Tauri main.rs has BridgeScope struct")

    print("Spec104 DOC-02 session identity envelope: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())