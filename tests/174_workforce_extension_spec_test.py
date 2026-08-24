#!/usr/bin/env python3
"""Static authority gate for Spec 174 and its weak-model execution graph."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "docs/174-focusa-agent-workforce-browser-extension-mvp-slice-spec.md"
GRAPH = ROOT / "docs/178-focusa-agent-workforce-browser-extension-mvp-execution-taskgraph.md"

spec = SPEC.read_text()
graph = GRAPH.read_text()

for path, text in ((SPEC, spec), (GRAPH, graph)):
    lines = text.splitlines()
    assert len(lines) < 500, f"{path} exceeds 499 lines: {len(lines)}"
    assert "<<<<<<<" not in text and "TODO" not in text and "TBD" not in text

required_spec = [
    "## 1. Operator outcome",
    "### 2.1 Observation",
    "### 2.2 Orientation",
    "### 2.3 Creation",
    "### 2.4 Orchestration",
    "focusa.workforce_connection.v1",
    "focusa.browser_observation.v1",
    "focusa.browser_orientation.v1",
    "focusa.silent_session_approval_request.v1",
    "focusa.silent_session_approval_response.v1",
    "POST /v1/silent-sessions/{session_id}/approvals",
    "No browser-owned scheduler",
    "no content scripts",
    "maximum 12 turns",
    "maximum 30 minutes",
]
for marker in required_spec:
    assert marker in spec, f"missing Spec 174 authority marker: {marker}"

required_graph = [
    "## 7. Executable task graph",
    "## 8. Weak-model execution packet",
    "## 9. Bead-level done conditions",
    "174-00",
    "174-17",
    "closure_supported=true",
    "evidence_sufficiency=sufficient",
    "critical_objections=[]",
]
for marker in required_graph:
    assert marker in graph, f"missing execution marker: {marker}"

# Every node is defined exactly once as a done-condition heading.
for number in range(18):
    node = f"174-{number:02d}"
    count = len(re.findall(rf"^### {re.escape(node)} —", graph, flags=re.MULTILINE))
    assert count == 1, f"{node} done condition count={count}"

# Parse A -> B lines from the fenced DAG and prove acyclic reachability.
edges: dict[str, set[str]] = {f"174-{i:02d}": set() for i in range(18)}
for left, right in re.findall(r"(174-\d{2})\s*->\s*(174-\d{2})", graph):
    edges[left].add(right)
assert sum(map(len, edges.values())) >= 17, "dependency DAG is unexpectedly sparse"

visiting: set[str] = set()
visited: set[str] = set()
def visit(node: str) -> None:
    if node in visiting:
        raise AssertionError(f"task graph cycle at {node}")
    if node in visited:
        return
    visiting.add(node)
    for child in edges[node]:
        visit(child)
    visiting.remove(node)
    visited.add(node)
for node in edges:
    visit(node)

print("PASS: Spec 174 contracts and 18-node weak-model task graph are frozen")
