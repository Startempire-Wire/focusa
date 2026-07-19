#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md"
DIRECTIVE="$ROOT_DIR/docs/agent/spec135-implementation-acceleration-directive.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -f "$SPEC" ]] || fail "Spec 135H is missing"
[[ -f "$DIRECTIVE" ]] || fail "Spec 135 agent acceleration directive is missing"

for needle in \
  'focusa.interview.strategy.grill-with-docs.v1' \
  'Fact-versus-decision law' \
  'Every operator decision question includes one recommended answer' \
  'One-question law' \
  'Discovery Grill' \
  'Failure Grill' \
  'Spec-Readiness Grill' \
  'focusa.domain_term_candidate.v1' \
  'focusa.architecture_decision_candidate.v1'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing Interview acceleration decision: $needle"
done
pass "Grill-with-Docs Interview strategy is fully specified"

for needle in \
  'Full Completion DAG' \
  'Cross-Functional Alpha' \
  'Alpha 0 — Generated contract spine' \
  'Alpha 1 — Real Context' \
  'Alpha 2 — Real Role and Grill Interview' \
  'Alpha 3 — Real Spec and Task' \
  'Alpha 4 — Workpoint, proof, closure, and Receipt' \
  'Alpha 5 — UIAI rich artifact and live refresh' \
  'Alpha 6 — Multiplexing and isolation' \
  'Alpha 7 — Vertical projection' \
  'Alpha 8 — Spec 135 dogfood loop'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing Cross-Functional Alpha marker: $needle"
done
pass "Cross-Functional Alpha end-to-end path is mandatory"

for needle in \
  'Schemars' \
  'Utoipa / utoipa-axum' \
  'openapi-typescript' \
  'Docling Serve v1 API' \
  'Docling HybridChunker' \
  'TanStack Query for Svelte' \
  'TanStack Table' \
  'TanStack Virtual' \
  'Svelte Flow' \
  'petgraph' \
  'Tree-sitter' \
  'ast-grep' \
  'PDF.js' \
  'CodeMirror Merge' \
  'Apache ECharts' \
  'cargo-nextest' \
  'proptest' \
  'cargo-deny' \
  'Syft SBOM generation'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing decided accelerator: $needle"
done
pass "decided open-source acceleration stack is explicit"

for needle in \
  'reuse_assessment:' \
  'Adopt' \
  'Wrap' \
  'Configure' \
  'Extend' \
  'Custom only when the conformance fixture proves' \
  'Do not present framework, sequence, or product-option menus' \
  'Use vertical tracer-bullet tickets' \
  'Keep the Spec 135 dogfood path continuously green' \
  'Use expand-contract migration'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing decomposer speed instruction: $needle"
done
pass "decomposing agents receive decision-only speed instructions"

for needle in \
  'third-party notices' \
  'model-license' \
  'container-image provenance' \
  'CycloneDX or SPDX-compatible SBOM' \
  'Dependency replacement law' \
  'MIT notice'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing license/provenance requirement: $needle"
done
pass "license, notice, SBOM, and replacement governance is explicit"

for needle in \
  'Do not present option menus' \
  'Cross-Functional Alpha' \
  'focusa.interview.strategy.grill-with-docs.v1' \
  'Decided stack' \
  'reuse_assessment:' \
  'Use vertical tracer-bullet tickets' \
  'Permanent integration gate' \
  'No-deferral rule'; do
  rg -n -F "$needle" "$DIRECTIVE" >/dev/null || fail "agent acceleration directive missing: $needle"
done
pass "agent-facing decomposition directive preserves decision-only acceleration rules"

echo "Spec 135H implementation acceleration static test: PASS"
