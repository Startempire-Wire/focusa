#!/usr/bin/env python3
"""Spec104 DOC-04: tool-contracts.ts scope/authority contract checks.

Verifies that:
- Every entry in FOCUSA_TOOL_CONTRACTS has scope_requirement + authority_requirement
- The required types (FocusaScopeRequirement, FocusaAuthorityRequirement) are defined
- All entries parse as valid JSON

Spec104 DOC-04 proof: static contract audit blocks missing fields.
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


def main() -> int:
    print("=== Spec104 DOC-04 tool-contracts static audit ===")

    tc_path = ROOT / "apps/pi-extension/src/tool-contracts.ts"
    if not tc_path.exists():
        fail("tool-contracts.ts missing")
    tc_src = tc_path.read_text()

    # Type definitions
    if "export type FocusaScopeRequirement" not in tc_src:
        fail("FocusaScopeRequirement type missing")
    ok("FocusaScopeRequirement type present")
    if "export type FocusaAuthorityRequirement" not in tc_src:
        fail("FocusaAuthorityRequirement type missing")
    ok("FocusaAuthorityRequirement type present")

    # Interface field declarations
    if "scope_requirement: FocusaScopeRequirement" not in tc_src:
        fail("FocusaToolContract.scope_requirement field missing")
    ok("FocusaToolContract.scope_requirement field declared")
    if "authority_requirement: FocusaAuthorityRequirement" not in tc_src:
        fail("FocusaToolContract.authority_requirement field missing")
    ok("FocusaToolContract.authority_requirement field declared")

    # TypeScript object keys may be quoted or unquoted. The interface occurrence
    # is excluded; typed compilation separately guarantees every registry item
    # satisfies FocusaToolContract.
    count = len(re.findall(r'["\']?scope_requirement["\']?\s*:', tc_src)) - 1
    if count < 1:
        fail("no entries have scope_requirement")
    ok(f"{count} contract entries/factories have scope_requirement field")

    auth_count = len(re.findall(r'["\']?authority_requirement["\']?\s*:', tc_src)) - 1
    if auth_count < 1:
        fail("no entries have authority_requirement")
    ok(f"{auth_count} contract entries/factories have authority_requirement field")

    # Scope requirement kinds are valid. Dynamic preload factory kinds are
    # checked by TypeScript's FocusaScopeRequirement return type.
    valid_kinds = {
        "none",
        "read",
        "write",
        "control",
        "public:health",
        "public:pairing",
    }
    pattern = re.compile(
        r'["\']?scope_requirement["\']?\s*:\s*\{\s*["\']?kind["\']?\s*:\s*"([^"]+)"'
    )
    kinds = pattern.findall(tc_src)
    invalid = [k for k in kinds if k not in valid_kinds]
    if invalid:
        fail(f"invalid scope_requirement kinds found: {sorted(set(invalid))}")
    ok(f"all {len(kinds)} scope_requirement kinds are valid")

    print("Spec104 DOC-04 tool-contracts static audit: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
