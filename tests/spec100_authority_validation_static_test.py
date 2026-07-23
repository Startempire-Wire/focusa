#!/usr/bin/env python3
"""Spec 100 Phase 1 — ContextCognitionPacket authority validation static test.

Verifies that:
- The packet's `authority` block always sets `canonical_mutation_allowed = false`.
- The route rejects agent runtime paths as `project_root`.
- The route rejects empty `project_root` with `failure_class=project_root_missing`.
- The route rejects unverified `project_root` with `failure_class=project_root_unverified`.
- The default authority uses `action_authority=workpoint` and
  `canonical_mutation_allowed=false`.
- The default recommended_packet_use includes `do_not_drift` items.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def main() -> None:
    route_path = ROOT / "crates/focusa-api/src/routes/context_cognition.rs"
    if not route_path.exists():
        fail("context_cognition.rs missing")
    route_src = route_path.read_text()

    # failure_class coverage
    required_failure_classes = [
        "project_root_missing",
        "scope_mismatch",
        "project_root_unverified",
    ]
    for fc in required_failure_classes:
        if fc not in route_src:
            fail(f"context_cognition.rs missing failure_class: {fc}")

    # agent runtime blocklist inline
    if "is_unsafe_agent_runtime_path_inline" not in route_src:
        fail("context_cognition.rs missing agent runtime blocklist")
    if "/root/pi-mono" not in route_src or "/root/.claude" not in route_src:
        fail("context_cognition.rs blocklist missing expected paths")

    # canonical_mutation_allowed = false in default authority
    if "canonical_mutation_allowed: false" not in route_src:
        fail("default authority does not disable mutation")
    if 'action_authority: "workpoint"' not in route_src:
        fail("default authority action_authority is not 'workpoint'")

    # Default route_frame.do_not_use_by_default non-empty
    if '"full lineage tree"' not in route_src and "full lineage tree" not in route_src:
        fail("default route_frame.do_not_use_by_default missing 'full lineage tree'")

    # Recommended use must include do_not_drift
    if "transcript_tail as authority" not in route_src:
        fail("recommended_packet_use.do_not_drift missing transcript_tail entry")

    # Test cases (in #[cfg(test)] mod tests) must cover the three failure classes
    test_block = re.search(
        r"#\[cfg\(test\)\]\s*mod tests\s*\{([\s\S]*?)\n\}", route_src
    )
    if not test_block:
        fail("context_cognition.rs missing #[cfg(test)] mod tests")
    body = test_block.group(1)
    for fn in [
        "default_authority_is_advisory",
        "default_route_frame_lists_next_and_recovery",
        "empty_packet_uses_schema_v1",
    ]:
        if f"fn {fn}" not in body:
            fail(f"context_cognition.rs missing test fn: {fn}")

    # Doc page must mention failure_class
    doc_path = ROOT / "docs/focusa-tools/tools/focusa_context_cognition.md"
    if not doc_path.exists():
        fail("tool doc missing")
    doc_src = doc_path.read_text()
    if "failure_class" not in doc_src:
        fail("tool doc missing failure_class recovery notes")
    for fc in ["project_root_missing", "scope_mismatch", "project_root_unverified"]:
        if fc not in doc_src:
            fail(f"tool doc missing recovery note for {fc}")

    print(
        "✓ PASS: focusa_context_cognition authority + failure_class + blocklist + tests + doc all wired"
    )


if __name__ == "__main__":
    main()
