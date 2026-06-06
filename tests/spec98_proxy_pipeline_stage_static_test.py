#!/usr/bin/env python3
"""Spec98 focusa-877z.11: transparent proxy pipeline stage separation."""
from pathlib import Path
import re
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.11-proxy-pipeline-stage-contract.yaml"
TYPES = ROOT / "crates/focusa-core/src/types.rs"
OPENAI = ROOT / "crates/focusa-core/src/adapters/openai.rs"
ANTHROPIC = ROOT / "crates/focusa-core/src/adapters/anthropic.rs"
EXPR = ROOT / "crates/focusa-core/src/expression/engine.rs"

STAGES = [
    "RequestIntake",
    "UserInputExtraction",
    "RetrievalEnrichmentPlanning",
    "DeterministicExpressionRender",
    "ProviderRequestInjection",
    "ProviderCompatibilityShim",
    "UpstreamProviderForward",
    "RuntimeTelemetryCapture",
    "EvalRegeneration",
]


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def fn_body(source: str, name: str) -> str:
    marker = f"fn {name}"
    start = source.find(marker)
    if start == -1:
        marker = f"pub fn {name}"
        start = source.find(marker)
    if start == -1:
        marker = f"pub async fn {name}"
        start = source.find(marker)
    if start == -1:
        fail(f"missing function {name}")
    brace = source.find("{", start)
    depth = 0
    for i in range(brace, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start:i+1]
    fail(f"unterminated function {name}")


def main() -> None:
    contract = yaml.safe_load(CONTRACT.read_text())
    if contract.get("schema_version") != "focusa.proxy_pipeline_stage_contract.v1":
        fail("unexpected .11 contract schema")
    rule = contract.get("normative_rule", "")
    for phrase in ["deterministic expression rendering", "retrieval/enrichment", "provider compatibility shims", "runtime telemetry"]:
        if phrase not in rule:
            fail(f"contract normative rule missing {phrase}")

    types = TYPES.read_text()
    enum_start = types.find("pub enum ProxyPipelineStage")
    const_start = types.find("pub const PROXY_PIPELINE_STAGE_CONTRACT")
    if enum_start == -1 or const_start == -1:
        fail("types.rs must define ProxyPipelineStage and PROXY_PIPELINE_STAGE_CONTRACT")
    enum_body = types[enum_start:const_start]
    const_body = types[const_start:types.find("/// Separate systems participating", const_start)]
    for stage in STAGES:
        if stage not in enum_body:
            fail(f"ProxyPipelineStage missing {stage}")
        if f"ProxyPipelineStage::{stage}" not in const_body:
            fail(f"PROXY_PIPELINE_STAGE_CONTRACT missing {stage}")
    deterministic_true = re.findall(r"stage: ProxyPipelineStage::(\w+), deterministic_render: true", const_body)
    if deterministic_true != ["DeterministicExpressionRender"]:
        fail(f"only DeterministicExpressionRender may set deterministic_render=true, got {deterministic_true}")
    for stage, flag in [
        ("RetrievalEnrichmentPlanning", "retrieval_or_enrichment: true"),
        ("ProviderRequestInjection", "provider_request_mutation: true"),
        ("ProviderCompatibilityShim", "provider_request_mutation: true"),
        ("UpstreamProviderForward", "provider_io: true"),
        ("RuntimeTelemetryCapture", "telemetry_only: true"),
        ("EvalRegeneration", "eval_or_regeneration: true"),
    ]:
        row = re.search(rf"ProxyPipelineStageContract \{{ stage: ProxyPipelineStage::{stage},[^\n]+", const_body)
        if not row or flag not in row.group(0):
            fail(f"{stage} row missing side-effect flag {flag}")

    openai = OPENAI.read_text()
    anthropic = ANTHROPIC.read_text()
    for source_name, source, required in [
        ("openai", openai, ["Transparent pipeline", "RetrievalEnrichmentPlanning", "ProviderRequestInjection", "UpstreamProviderForward", "RuntimeTelemetryCapture"]),
        ("anthropic", anthropic, ["Transparent pipeline", "ProviderCompatibilityShim", "ProviderRequestInjection", "UpstreamProviderForward", "RuntimeTelemetryCapture"]),
    ]:
        for phrase in required:
            if phrase not in source:
                fail(f"{source_name} adapter missing stage label {phrase}")

    openai_planner = fn_body(openai, "build_operator_first_slice")
    if "RetrievalEnrichmentPlanning stage" not in openai:
        fail("OpenAI planner must be labeled as RetrievalEnrichmentPlanning")
    if "forward_request(" in openai_planner or "sanitize_for_compat" in openai_planner:
        fail("Retrieval/enrichment planner must not perform provider IO or compatibility sanitation")

    anthropic_sanitize = fn_body(anthropic, "sanitize_for_compat")
    if "ProviderCompatibilityShim stage" not in anthropic:
        fail("Anthropic sanitizer must be labeled ProviderCompatibilityShim")
    if "build_operator_first_slice" in anthropic_sanitize or "inject_system_prompt" in anthropic_sanitize:
        fail("Provider compatibility shim must not perform retrieval planning or prompt injection")

    expr = EXPR.read_text()
    expr_header = expr[:expr.find("use crate::")]
    for phrase in ["No reasoning, planning, or implicit summarization", "Deterministic output"]:
        if phrase not in expr_header:
            fail(f"Expression Engine header missing deterministic boundary phrase {phrase}")
    for forbidden in ["forward_request", "sanitize_for_compat", "retrieval_governor", "eval_regeneration"]:
        if forbidden in expr:
            fail(f"Expression Engine must not contain proxy/eval stage {forbidden}")

    print("✓ PASS: proxy pipeline stages are explicit and deterministic expression render is isolated")


if __name__ == "__main__":
    main()
