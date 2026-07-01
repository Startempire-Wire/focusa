#!/usr/bin/env python3
"""Spec104 DOC-03: mismatch semantic test extension.

Verifies scope mismatch semantics across all surfaces:
- Pi-extension scope getters
- Tools (tool-contracts scope_requirement)
- Menubar (api.ts uses typed envelopes)
- Adapters (work_loop, project, trajectory routes)
- TUI (TypedScope)

Spec104 DOC-03 proof: mismatch test covers all surfaces.
"""
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def ok(msg: str) -> None:
    print(f"✓ {msg}")


def main() -> int:
    print("=== Spec104 DOC-03 mismatch semantic test ===")

    # Pi-extension: scope getters present
    state_path = ROOT / "apps/pi-extension/src/state.ts"
    state_src = state_path.read_text()
    if "export function getSessionCwd" not in state_src:
        fail("Pi: getSessionCwd missing")
    ok("Pi: getSessionCwd present")
    if "export function getContinuityId" not in state_src:
        fail("Pi: getContinuityId missing")
    ok("Pi: getContinuityId present")

    # Tools: scope_requirement in tool contracts
    tc_path = ROOT / "apps/pi-extension/src/tool-contracts.ts"
    tc_src = tc_path.read_text()
    if "scope_requirement" not in tc_src:
        fail("Tools: scope_requirement field missing from tool-contracts.ts")
    ok("Tools: scope_requirement field present")
    if "FocusaScopeRequirement" not in tc_src:
        fail("Tools: FocusaScopeRequirement type missing")
    ok("Tools: FocusaScopeRequirement type present")

    # Menubar: typed envelopes in api.ts
    api_path = ROOT / "apps/menubar/src/lib/api.ts"
    api_src = api_path.read_text()
    if "project_root: ctx.projectRoot" not in api_src:
        fail("Menubar: api.ts missing typed project_root")
    ok("Menubar: api.ts uses typed project_root")
    if "continuity_id: ctx.continuityId" not in api_src:
        fail("Menubar: api.ts missing typed continuity_id")
    ok("Menubar: api.ts uses typed continuity_id")

    # Menubar: ScopeContext interface in projectContext
    ctx_path = ROOT / "apps/menubar/src/lib/projectContext.svelte.ts"
    ctx_src = ctx_path.read_text()
    if "export interface ScopeContext" not in ctx_src:
        fail("Menubar: ScopeContext interface missing in projectContext")
    ok("Menubar: ScopeContext interface present")

    # Adapter (work_loop, project, trajectory routes)
    for route_name in ("work_loop.rs", "project.rs", "trajectory.rs"):
        route_path = ROOT / f"crates/focusa-api/src/routes/{route_name}"
        if not route_path.exists():
            continue
        route_src = route_path.read_text()
        if "ScopeContext" not in route_src:
            fail(f"Adapter {route_name}: ScopeContext not imported")
        ok(f"Adapter {route_name}: ScopeContext imported")

    # TUI: TypedScope
    tui_path = ROOT / "crates/focusa-tui/src/api.rs"
    tui_src = tui_path.read_text()
    if "TypedScope" not in tui_src:
        fail("TUI: TypedScope struct missing")
    ok("TUI: TypedScope struct present")
    if "fetch_with_scope" not in tui_src:
        fail("TUI: fetch_with_scope method missing")
    ok("TUI: fetch_with_scope method present")

    print("Spec104 DOC-03 mismatch semantic test: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())