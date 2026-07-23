#!/usr/bin/env python3
"""Spec98 focusa-877z.2: memory maintenance and prompt-state routing guard."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.2-memory-prompt-routing-contract.yaml"
TYPES = ROOT / "crates/focusa-core/src/types.rs"
REDUCER = ROOT / "crates/focusa-core/src/reducer.rs"
DAEMON = ROOT / "crates/focusa-core/src/runtime/daemon.rs"
TURN = ROOT / "crates/focusa-api/src/routes/turn.rs"
PROXY = ROOT / "crates/focusa-api/src/routes/proxy.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.memory_prompt_routing_contract.v1":
        fail("unexpected .2 contract schema")
    if (
        data.get("status")
        != "semantic_cleanup_event_routed_and_prompt_buffers_runtime_action_routed"
    ):
        fail("unexpected .2 contract status")

    types = TYPES.read_text()
    daemon = DAEMON.read_text()
    reducer = REDUCER.read_text()
    proxy = PROXY.read_text()
    turn = TURN.read_text()

    for needle in [
        "ResolveSemanticContradictions",
        "SemanticMemoryContradictionsResolved",
        "UpdateActiveTurnRuntime",
    ]:
        if needle not in types:
            fail(f"types.rs missing {needle}")
    if "Action::ResolveSemanticContradictions" not in daemon:
        fail("daemon must translate Action::ResolveSemanticContradictions")
    if "FocusaEvent::SemanticMemoryContradictionsResolved" not in daemon:
        fail("daemon must emit SemanticMemoryContradictionsResolved")
    if "FocusaEvent::SemanticMemoryContradictionsResolved" not in reducer:
        fail("reducer must handle SemanticMemoryContradictionsResolved")
    if "resolve_contradictions(&mut state.memory)" not in reducer:
        fail("semantic contradiction cleanup must be reducer-backed")
    if (
        "semantic::resolve_contradictions" in proxy
        or "resolve_contradictions(&mut focusa.memory)" in proxy
    ):
        fail("proxy.rs must not directly mutate semantic memory contradiction cleanup")
    if "Action::ResolveSemanticContradictions" not in proxy:
        fail(
            "proxy.rs must dispatch semantic cleanup through Action::ResolveSemanticContradictions"
        )

    for route_name, text in [("turn.rs", turn), ("proxy.rs", proxy)]:
        if "Action::UpdateActiveTurnRuntime" not in text:
            fail(f"{route_name} must keep prompt buffers action-routed")
        for forbidden in ["turn.raw_user_input =", "turn.assembled_prompt ="]:
            if forbidden in text:
                fail(f"{route_name} directly mutates prompt buffer: {forbidden}")

    print(
        "✓ PASS: memory maintenance is reducer-routed and prompt buffers remain runtime action-routed"
    )


if __name__ == "__main__":
    main()
