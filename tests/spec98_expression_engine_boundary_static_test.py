#!/usr/bin/env python3
"""Spec98 focusa-877z.5: Expression Engine deterministic-renderer boundary guard."""

from pathlib import Path
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = (
    ROOT / "docs/worksheets/focusa-877z.5-expression-engine-boundary-contract.yaml"
)
EXPR_DIR = ROOT / "crates/focusa-core/src/expression"
EXPR_MOD = EXPR_DIR / "mod.rs"
ENGINE = EXPR_DIR / "engine.rs"
OPENAI = ROOT / "crates/focusa-core/src/adapters/openai.rs"
PROXY = ROOT / "crates/focusa-api/src/routes/proxy.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.expression_engine_boundary_contract.v1":
        fail("unexpected .5 contract schema")
    if data.get("status") != "expression_engine_deterministic_renderer_only":
        fail("unexpected .5 contract status")

    mod = EXPR_MOD.read_text()
    for invariant in [
        "INVARIANT: No retrieval, memory maintenance, network/process I/O, or adaptive planning.",
        "INVARIANT: Callers gather prepared inputs; Expression Engine only renders and degrades explicitly.",
    ]:
        if invariant not in mod:
            fail(f"expression module missing invariant: {invariant}")

    engine = ENGINE.read_text()
    if (
        "pub struct AssemblyInput" not in engine
        or "pub fn assemble_from(input: AssemblyInput<'_>)" not in engine
    ):
        fail(
            "Expression Engine must expose prepared-input AssemblyInput + assemble_from API"
        )
    if "All inputs needed for prompt assembly, gathered by the caller." not in engine:
        fail("AssemblyInput must document caller-prepared inputs")

    forbidden_needles = [
        "crate::memory::",
        "focusa_core::memory",
        "resolve_contradictions",
        "UpsertSemantic",
        "ResolveSemanticContradictions",
        "mem0",
        "wiki",
        "reqwest",
        "tokio::process",
        "Command::new",
        "forward_request",
        "process_request(",
        "build_operator_first_slice",
        "WorkpointCheckpoint",
        "TrajectoryGoalDefined",
        "RequestNextContinuousTurn",
    ]
    for path in EXPR_DIR.glob("*.rs"):
        text = path.read_text()
        for needle in forbidden_needles:
            if needle in text:
                fail(
                    f"Expression Engine file {path.name} contains forbidden side-effect/adaptive surface: {needle}"
                )

    openai = OPENAI.read_text()
    if "Adapter-side adaptive slice planner." not in openai:
        fail(
            "build_operator_first_slice must be labeled adapter-side adaptive planning"
        )
    if "it is not the\n/// Expression Engine" not in openai:
        fail("adapter adaptive planner must explicitly not be the Expression Engine")

    proxy = PROXY.read_text()
    for stage in ["PRE-TURN ENRICHMENT", "Resolve contradictions", "PROMPT ASSEMBLY"]:
        if stage not in proxy:
            fail(f"proxy orchestration must keep explicit stage label: {stage}")
    if "Action::ResolveSemanticContradictions" not in proxy:
        fail(
            "proxy memory maintenance must remain reducer/action-routed outside Expression Engine"
        )

    print(
        "✓ PASS: Expression Engine is deterministic prepared-input renderer; retrieval/memory/adaptive planning stay outside"
    )


if __name__ == "__main__":
    main()
