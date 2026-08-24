#!/usr/bin/env python3
import json
from pathlib import Path
R=Path(__file__).resolve().parents[1]
issues={item['id']:item for item in (json.loads(line) for line in (R/'.beads/issues.jsonl').read_text().splitlines() if line.strip())}
expected={f'focusa-q54m3.1.{i}' for i in range(1,6)} | {f'focusa-q54m3.2.{i}' for i in range(1,7)} | {f'focusa-q54m3.3.{i}' for i in range(1,6)} | {f'focusa-q54m3.4.{i}' for i in range(1,6)} | {f'focusa-q54m3.5.{i}' for i in range(1,8)} | {f'focusa-q54m3.6.{i}' for i in range(1,8)} | {f'focusa-a89or.{i}' for i in range(1,7)}
actual={iid for iid,item in issues.items() if item.get('issue_type')=='task' and 'full-implementation' in item.get('labels',[]) and 'workforce-full' in item.get('labels',[])}
assert len(expected)==41, len(expected)
assert actual==expected, f'implementation inventory drift: missing={sorted(expected-actual)} extra={sorted(actual-expected)}'
for iid in sorted(expected):
 item=issues[iid]; body=' '.join([item.get('title',''),item.get('description',''),item.get('acceptance_criteria','')])
 assert item.get('priority')==0 and 'release-blocker' in item.get('labels',[]), iid
 for marker in ['Purpose:','Dependencies:','Allowed implementation files:','Forbidden scope:','Input/output contracts:','Mechanical steps:','Tests:','Acceptance:','Evidence','Failure stop:','Handoff:']:
  assert marker in item.get('description',''), f'{iid} missing {marker}'
 body_without_policy=body.replace('no TODO/TBD','')
 assert 'TODO' not in body_without_policy and 'TBD' not in body_without_policy, iid
for external in ['focusa-no4ks','focusa-eaaf8.1','focusa-mc-full-b3','focusa-d4gp4','focusa-a89or']:
 assert external in issues, external
# Prove block graph acyclic directly from durable issue authority.
edges={iid:[dep['depends_on_id'] for dep in item.get('dependencies',[]) if dep.get('type')=='blocks'] for iid,item in issues.items()}
visiting=set(); visited=set()
def visit(node):
 if node in visiting: raise AssertionError(f'dependency cycle at {node}')
 if node in visited: return
 visiting.add(node)
 for parent in edges.get(node,[]): visit(parent)
 visiting.remove(node); visited.add(node)
for node in edges: visit(node)
# Required package-crossing edges and held-release edge remain explicit.
required_edges={
 'focusa-q54m3.3.1':'focusa-q54m3.6.1', 'focusa-q54m3.2.3':'focusa-a89or.3',
 'focusa-q54m3.3.3':'focusa-mc-full-b3', 'focusa-q54m3.3.5':'focusa-no4ks.9',
 'focusa-q54m3.6.2':'focusa-no4ks.13', 'focusa-d4gp4':'focusa-q54m3.6.7',
}
for child,parent in required_edges.items(): assert parent in edges.get(child,[]), f'missing edge {child}->{parent}'
# Source-backed readiness baseline: these implementations exist but are not live routers yet.
mod=(R/'crates/focusa-api/src/routes/mod.rs').read_text(); server=(R/'crates/focusa-api/src/server.rs').read_text()
for name in ['worksets','callgraph','session_fanout']:
 assert (R/f'crates/focusa-api/src/routes/{name}.rs').is_file(), name
 assert f'mod {name}' not in mod and f'{name}::router' not in server, f'{name} baseline unexpectedly changed; update authority gate'
# Active authority has corrected the stale manual count and intentionally uses beads, not rejected draft docs.
for iid in ['focusa-q54m3','focusa-q54m3.6.1']:
 body=issues[iid].get('description','')+' '+issues[iid].get('acceptance_criteria','')
 assert '41' in body and '38' not in body, iid
assert not list((R/'docs').glob('180-focusa-workforce-full-functionality*'))
assert not list((R/'docs').glob('181-focusa-workforce-full-functionality*'))
print('PASS: Workforce Full 41-node authority, dependency graph, and source readiness baseline')
