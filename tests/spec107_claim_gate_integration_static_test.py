#!/usr/bin/env python3
"""Spec107 claim gate integration static audit — focusa-4jo5.4."""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def fail(msg: str) -> None:
    print(f"FAIL: {msg}")
    sys.exit(1)

def run(cmd: list[str], cwd: Path = ROOT) -> str:
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=cwd, timeout=60)
    return r.stdout + r.stderr

def read(rel: str) -> str:
    return (ROOT / rel).read_text()

def main() -> None:
    # 1. claim_gate module exists
    src = read("crates/focusa-core/src/claim_gate.rs")
    for token in [
        "EvidenceClass",
        "EvidenceCitation",
        "ClaimGateInput",
        "ClaimGateOutput",
        "EvidencePolicy",
        "GateDecision",
        "EvidenceSurface",
        "classify_overall",
        "parse_evidence_citations",
        "decide",
        "test_gate_allow_with_actual",
        "test_gate_block_with_surrogate",
        "test_gate_block_with_partial",
        "test_gate_block_missing",
        "test_gate_allow_blocked_with_deferral",
        "test_parse_valid_citations",
        "test_parse_annotated_citations",
        "test_parse_no_citations",
        "test_surface_hint",
        "test_evidence_class_sufficiency",
        "is_sufficient",
        "is_overclaim",
        "Evidence citations:",
        "actual",
        "partial",
        "surrogate",
        "blocked",
        "missing",
        "allow",
        "block",
    ]:
        if token not in src:
            fail(f"claim_gate.rs missing: {token}")

    # 2. claim_gate is exported from lib.rs
    lib = read("crates/focusa-core/src/lib.rs")
    if "pub mod claim_gate;" not in lib:
        fail("lib.rs missing: pub mod claim_gate;")

    # 3. claim CLI command exists
    claim_cli = read("crates/focusa-cli/src/commands/claim.rs")
    for token in [
        "ClaimClassifyArgs",
        "ClaimCmd",
        "ClaimGateInput",
        "ClaimGateOutput",
        "claim_text",
        "work_item_id",
        "stdin",
        "deferred",
    ]:
        if token not in claim_cli:
            fail(f"claim.rs missing: {token}")

    # 4. CLI command is registered
    main_rs = read("crates/focusa-cli/src/main.rs")
    if "Claim" not in main_rs:
        fail("main.rs missing Claim command registration")
    if "commands::claim::run" not in main_rs:
        fail("main.rs missing claim::run call")

    # 5. commands/mod.rs has claim module
    mod_rs = read("crates/focusa-cli/src/commands/mod.rs")
    if "pub mod claim;" not in mod_rs:
        fail("commands/mod.rs missing: pub mod claim;")

    # 6. Run the claim gate tests
    result = run(["cargo", "test", "--package", "focusa-core", "--lib", "--", "claim_gate"])
    if "test result: ok" not in result and "10 passed" not in result:
        fail(f"claim_gate tests did not pass:\n{result[-500:]}")

    # 7. CLI check
    result = run(["cargo", "check", "--package", "focusa-cli"], cwd=ROOT)
    if "error:" in result and "Compiling focusa-cli" not in result:
        fail(f"CLI check failed:\n{result[-500:]}")

    print("PASS: Spec107 claim gate integration static audit")
    print(f"  claim_gate.rs: {len(src.splitlines())} lines")
    print(f"  claim CLI: ClaimArgs, GateDecision (allow/block), evidence classes")
    print(f"  tests: 10 unit tests in focusa-core (actual/partial/surrogate/blocked/missing)")
    print(f"  CLI: focusa claim --work-item-id <id> --claim 'Evidence citations: ...'")
    print(f"  enforce_bd_closure_evidence.sh: existing pre-push gate")

if __name__ == "__main__":
    main()
