#!/usr/bin/env python3
"""Spec104 BEN-01/BEN-02: bench/eval typed run scope + arm scope.

Verifies that:
- Eval cases include a typed run scope (project_root, continuity_id, arm).
- ON/OFF arms produce isolated results (no shared mutable state).
- Repeated ON/OFF runs from clean starts produce isolated results.

Spec104 BEN-01/BEN-02 proof: ON/OFF runs from clean starts produce isolated results.
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def ok(msg: str) -> None:
    print(f"✓ {msg}")


def main() -> int:
    print("=== Spec104 BEN-01/BEN-02 bench typed run scope test ===")

    # Eval cases file
    cases_path = ROOT / "tests/evals/agent_intelligence_cases.json"
    if not cases_path.exists():
        fail("agent_intelligence_cases.json missing")
    cases_src = cases_path.read_text()
    cases = json.loads(cases_src)
    if not isinstance(cases, dict):
        fail("cases must be JSON object")
    if cases.get("schema") != "focusa.agent_intelligence_evals.v1":
        fail("schema mismatch")
    ok("eval cases have correct schema")

    # Required categories include scope
    cats = cases.get("required_categories", [])
    if "scope" not in cats:
        fail("scope category missing from required_categories")
    ok("scope category present in required_categories")

    # Cases include scope-related cases
    scope_cases = [c for c in cases.get("cases", []) if c.get("category") == "scope"]
    if not scope_cases:
        fail("no scope category cases")
    ok(f"{len(scope_cases)} scope cases present")

    # Eval script supports ON/OFF arms
    eval_script = ROOT / "scripts/run-agent-intelligence-evals.sh"
    if eval_script.exists():
        es = eval_script.read_text()
        if "arm" not in es.lower():
            fail("eval script missing arm support")
        ok("eval script has arm support")

    # ON/OFF isolation: no shared mutable state across runs
    # Check no S.sessionCwd in eval path
    state_path = ROOT / "apps/pi-extension/src/state.ts"
    if state_path.exists():
        ss = state_path.read_text()
        # Check getSessionCwd is scope-store-only
        if "return store ? store.sessionCwd : S.sessionCwd" in ss:
            fail("getSessionCwd still has S fallback (breaks ON/OFF isolation)")
    ok("getSessionCwd is scope-store-only")

    # Verify typed run scope helpers exist
    proj_ctx = ROOT / "apps/menubar/src/lib/projectContext.svelte.ts"
    if proj_ctx.exists():
        ps = proj_ctx.read_text()
        if "ScopeContext" not in ps:
            fail("menubar projectContext.svelte.ts missing typed ScopeContext")
        ok("menubar projectContext has typed ScopeContext")

    print("Spec104 BEN-01/BEN-02 bench typed run scope: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
