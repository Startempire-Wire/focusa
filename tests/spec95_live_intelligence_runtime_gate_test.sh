#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-target}"
PORT="${FOCUSA_SPEC95_PORT:-18795}"
BASE="http://127.0.0.1:${PORT}/v1"
TMP_DIR="$(mktemp -d /tmp/focusa-spec95-live-XXXXXX)"
DAEMON_LOG="$TMP_DIR/daemon.log"
PID=""
cleanup(){ [[ -n "${PID:-}" ]] && kill "$PID" 2>/dev/null || true; [[ -n "${PID:-}" ]] && wait "$PID" 2>/dev/null || true; rm -rf "$TMP_DIR"; }
trap cleanup EXIT
fail(){ echo "✗ FAIL: $1" >&2; [[ -f "$DAEMON_LOG" ]] && tail -60 "$DAEMON_LOG" >&2 || true; exit 1; }
pass(){ echo "✓ PASS: $1"; }
cd "$ROOT_DIR"
"${CARGO_BIN:-cargo}" build -p focusa-api >/dev/null
FOCUSA_BIND="127.0.0.1:${PORT}" FOCUSA_DATA_DIR="$TMP_DIR/data" "$TARGET_DIR/debug/focusa-daemon" >"$DAEMON_LOG" 2>&1 &
PID=$!
for _ in {1..100}; do curl -fsS "$BASE/health" >/dev/null 2>&1 && break; sleep 0.1; done
curl -fsS "$BASE/health" >/dev/null 2>&1 || fail "daemon health did not become ready"
pass "isolated daemon ready"
python3 - "$BASE" <<'PY'
import json, sys, time, urllib.request
base=sys.argv[1]
def req(path, body=None):
    if body is None:
        r=urllib.request.Request(base+path)
    else:
        r=urllib.request.Request(base+path, data=json.dumps(body).encode(), headers={'Content-Type':'application/json'}, method='POST')
    with urllib.request.urlopen(r, timeout=20) as resp:
        raw=resp.read(); return json.loads(raw), len(raw)
def require(cond,msg):
    if not cond: raise SystemExit(msg)
# Warm caches.
for _ in range(3):
    for path,body in [
      ('/ontology/adjacency',None),('/ontology/working-set?include_reasons=true',None),('/ontology/slices',None),('/ontology/affordances',None),
      ('/ontology/context',{'current_ask':'implement spec95','budget_tokens':500,'target_refs':['goal:active_mission']}),
      ('/ontology/retrieval-governor',{'current_ask':'implement spec95','budget_tokens':800,'target_refs':['goal:active_mission']}),
    ]: req(path,body)
checks=[]
def timed(path, body=None, budget=50):
    vals=[]; sizes=[]; payload=None
    for _ in range(20):
        t=time.perf_counter(); payload,size=req(path,body); vals.append((time.perf_counter()-t)*1000); sizes.append(size)
    s=sorted(vals); p95=s[int(len(s)*0.95)-1]
    require(p95 <= budget, f'{path} p95 {p95:.2f}ms > {budget}ms')
    checks.append({'route':path,'p50_ms':round(s[len(s)//2],2),'p95_ms':round(p95,2),'max_bytes':max(sizes)})
    return payload
adj=timed('/ontology/adjacency', budget=50)
require(adj.get('source')=='ontology_adjacency_read_index','adjacency source mismatch')
require(adj.get('index',{}).get('canonical_truth_mutation') is False,'adjacency claims canonical mutation')
require('object_type_counts' in adj and 'link_type_counts' in adj and 'last_reducer_event_id' in adj.get('index',{}),'adjacency missing parity/count metadata')
ws=timed('/ontology/working-set?include_reasons=true', budget=50)
require(ws.get('source')=='ontology_working_set_projection','working-set source mismatch')
require('members' in ws and 'next_cursor' in ws and 'rehydrate' in json.dumps(ws),'working-set missing cursor/rehydrate')
for m in ws.get('members',[]):
    require('id' in m and 'object_type' in m and 'score' in m and 'link_path_reason' in m and 'uncertainty' in m,'working-set member missing typed fields/reasons')
ctx=timed('/ontology/context', {'current_ask':'implement spec95 fully','budget_tokens':500,'target_refs':['goal:active_mission']}, 50)
for key in ['active_object_set','relevant_link_paths','valid_next_actions','blocked_affordances','evidence_handles','uncertainty_flags','canonical_truth_mutation','rehydrate']:
    require(key in ctx, f'context missing {key}')
require(ctx.get('canonical_truth_mutation') is False,'context mutates canonical truth')
aff=timed('/ontology/affordances', budget=75)
for key in ['feasible_actions','blocked_actions','valid_next_actions','verification_hooks_required','canonical_truth_mutation']:
    require(key in aff, f'affordances missing {key}')
require(aff.get('canonical_truth_mutation') is False,'affordances mutates canonical truth')
sl=timed('/ontology/slices', budget=50)
require(sl.get('projection_profile',{}).get('canonical_truth_mutation') is False,'slices missing projection boundary')
rg=timed('/ontology/retrieval-governor', {'current_ask':'implement spec95 fully','budget_tokens':800,'target_refs':['goal:active_mission']}, 50)
require(rg.get('source')=='ontology_retrieval_governor','retrieval governor source mismatch')
require('retrieval_plan' in rg and 'retrieval_results' in rg and 'hybrid_ranker' in rg,'retrieval governor missing plan/results/ranker')
empty,_=req('/ontology/retrieval-governor', {'current_ask':'','budget_tokens':800,'target_refs':[]})
require(empty.get('retrieval_plan',[{}])[0].get('substrate')=='none','retrieval governor did not allow no-retrieval path')
critic,_=req('/ontology/execution-critic', {'intended_action':'run tests','target_refs':['file:src/lib.rs'],'verification_hooks':['cargo test'],'tool_result':{'tool_name':'bash','ok':False,'status':'failed','target_refs':['file:src/lib.rs'],'evidence_refs':['test:unit'],'error':'compile failed'}})
require(critic.get('source')=='ontology_execution_critic' and critic.get('canonical_truth_mutation') is False and critic.get('candidate_ontology_deltas'), 'critic missing proposal-only deltas')
pipe,_=req('/ontology/memory-pipeline', {'episodic_events':[{'event':'tool_result'}],'evidence_refs':['test:unit'],'synthesis_artifacts':[{'kind':'lesson'}],'eval_results':[{'score':0.9,'promote_learning':True}],'repeated_validation_count':2})
stages={stage.get('stage'):stage for stage in pipe.get('stages',[])}
require(pipe.get('source')=='ontology_memory_promotion_pipeline' and stages.get('semantic_metacog_learning',{}).get('status')=='proposed' and stages.get('procedural_playbook_hint',{}).get('status')=='proposed', 'memory pipeline missing promotion artifacts')
dash,_=req('/ontology/intelligence-dashboard')
require(dash.get('source')=='ontology_intelligence_dashboard' and 'usefulness_metrics' in dash and 'fixed_eval_suites' in dash and 'latency_rss_overhead' in dash, 'dashboard missing usefulness/eval/latency metrics')
print(json.dumps({'latency_checks':checks}, indent=2))
PY
pass "ontology intelligence routes, proposal boundaries, and latency budgets verified"
ps -o pid,rss,vsz,pcpu,comm -p "$PID"
echo "SPEC95 live intelligence runtime gate: PASS"
