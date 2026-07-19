#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SPEC="$ROOT_DIR/docs/135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md"
DIRECTIVE="$ROOT_DIR/docs/agent/spec135-implementation-acceleration-directive.md"
DELIVERY="$ROOT_DIR/docs/135-series-current-manifest.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$SPEC" "$DIRECTIVE" "$DELIVERY"; do
  [[ -f "$file" ]] || fail "missing required Spec 135 acceleration file: $file"
done

for needle in \
  'focusa.interview.strategy.grill-with-docs.v1' \
  'Fact before question' \
  'Every operator-decision question' \
  'One question' \
  'Discovery Grill' \
  'Failure Grill' \
  'Spec-Readiness Grill' \
  'focusa.domain_term_candidate.v1' \
  'focusa.architecture_decision_candidate.v1'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing Interview decision: $needle"
done
pass "Grill-with-Docs Interview strategy is guarded"

for needle in \
  'F0 — freeze Spec 135 at 135K' \
  'F12 — complete one real Context action' \
  'Alpha 1 — Real Context' \
  'Alpha 2 — Real Role and Grill Interview' \
  'Alpha 3 — Real Spec and task' \
  'Alpha 4 — Workpoint and proof' \
  'Alpha 5 — UIAI artifact and live refresh' \
  'Alpha 6 — Multiplexing and isolation' \
  'Alpha 7 — Vertical projection' \
  'Alpha 8 — Permanent dogfood traversal'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing Foundation/Alpha marker: $needle"
done
pass "Foundation Train and Cross-Functional Alpha are mandatory"

for needle in \
  'OpenAPI 3.0.3' \
  'JSON Schema 2020-12' \
  'oapi-codegen v2.7.x' \
  'Docling Serve v1' \
  'Docling HybridChunker' \
  'TanStack Query' \
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
  'Syft SBOM'; do
  rg -n -F "$needle" "$SPEC" "$DIRECTIVE" >/dev/null || fail "missing decided accelerator: $needle"
done
pass "decided acceleration stack is explicit"

for needle in \
  'reuse_assessment:' \
  'primitive_submission:' \
  'Adopt' \
  'Wrap' \
  'Configure' \
  'Extend' \
  'Custom only after a failing conformance fixture' \
  'UIAI Engine Eval for all browser proof' \
  'Do not add Playwright' \
  'Vercel WorkflowAgent' \
  'machine-readable closure DAG' \
  'Permanent integration gate' \
  'No-deferral rule'; do
  rg -n -F "$needle" "$SPEC" "$DIRECTIVE" "$DELIVERY" >/dev/null || fail "missing speed/ownership instruction: $needle"
done
pass "decomposing agents receive fixed speed and primitive-submission rules"

if rg -n 'OpenAPI 3\.1|playwright_flow_ref|Focusa Svelte mappings on web_core for full production rendering' "$SPEC" "$DIRECTIVE"; then
  fail "stale acceleration decision remains"
fi

for needle in \
  'third-party' \
  'license' \
  'container' \
  'SBOM' \
  'MIT notice'; do
  rg -ni -F "$needle" "$SPEC" >/dev/null || fail "Spec 135H missing supply-chain requirement: $needle"
done
pass "license, notice, model, container, and SBOM governance is explicit"

echo "Spec 135H implementation acceleration static test: PASS"
