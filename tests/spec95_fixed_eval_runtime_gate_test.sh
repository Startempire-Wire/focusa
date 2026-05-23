#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/focusa-target}"
PORT="${FOCUSA_SPEC95_EVAL_PORT:-18798}"
BASE="http://127.0.0.1:${PORT}/v1"
TMP_DIR="$(mktemp -d /tmp/focusa-spec95-eval-XXXXXX)"
PID=""
cleanup(){ [[ -n "${PID:-}" ]] && kill "$PID" 2>/dev/null || true; [[ -n "${PID:-}" ]] && wait "$PID" 2>/dev/null || true; rm -rf "$TMP_DIR"; }
trap cleanup EXIT
cd "$ROOT_DIR"
"${CARGO_BIN:-cargo}" build -p focusa-api >/dev/null
FOCUSA_BIND="127.0.0.1:${PORT}" FOCUSA_DATA_DIR="$TMP_DIR/data" "$TARGET_DIR/debug/focusa-daemon" >"$TMP_DIR/daemon.log" 2>&1 &
PID=$!
for _ in {1..100}; do curl -fsS "$BASE/health" >/dev/null 2>&1 && break; sleep .1; done
python3 - "$BASE" <<'PY'
import json, sys, urllib.request
base=sys.argv[1]
def req(path, body=None):
    if body is None:
        r=urllib.request.Request(base+path)
    else:
        r=urllib.request.Request(base+path, data=json.dumps(body).encode(), headers={'Content-Type':'application/json'}, method='POST')
    with urllib.request.urlopen(r, timeout=20) as resp: return json.loads(resp.read())
def assert_true(cond,msg):
    if not cond: raise SystemExit(msg)
results=[]
ctx=req('/ontology/context', {'current_ask':'implement fixed eval','target_refs':[],'budget_tokens':800})
results.append('ontology_context_selection')
assert_true(ctx.get('active_object_set'), 'context missing active objects')
assert_true(ctx.get('valid_next_actions'), 'context missing valid actions')
assert_true(ctx.get('canonical_truth_mutation') is False, 'context canonical mutation leak')
for obj in ctx['active_object_set']:
    assert_true('id' in obj and 'object_type' in obj and 'uncertainty' in obj, f'active object missing typed identity/uncertainty: {obj}')
ws=req('/ontology/working-set?include_reasons=true&limit=5')
results.append('working_set_relation_reasons')
assert_true(ws.get('members'), 'working set missing members')
for member in ws['members']:
    for key in ['id','object_type','score','link_path_reason','provenance_handles','verification_status','confidence','freshness','action_affordance_ids','uncertainty','rehydrate']:
        assert_true(key in member, f'working-set member missing {key}: {member}')
aff=req('/ontology/affordances')
results.append('action_affordance_selection')
assert_true('feasible_actions' in aff and 'blocked_actions' in aff, 'affordance lists missing')
for action in aff.get('feasible_actions',[]) + aff.get('blocked_actions',[]):
    for key in ['preconditions','permission_boundary','authority_boundary','estimated_latency','estimated_cost','cost','reliability','reversibility','rehydrate']:
        assert_true(key in action, f'affordance missing {key}: {action}')
rg=req('/ontology/retrieval-governor', {'current_ask':'implement fixed eval','target_refs':[],'budget_tokens':1000})
results.append('hybrid_retrieval')
assert_true(rg.get('retrieval_results'), 'retrieval results missing')
assert_true(rg.get('hybrid_ranker',{}).get('secondary_model_reranking') is not None, 'secondary reranking boundary missing')
critic=req('/ontology/execution-critic', {'intended_action':'verify','target_refs':['goal:active_mission'],'verification_hooks':['spec95 eval'],'tool_result':{'tool_name':'bash','ok':False,'status':'failed','target_refs':['goal:active_mission'],'evidence_refs':['eval:spec95'],'error':'fixture failure'}})
results.append('secondary_critic_recovery')
assert_true(critic.get('candidate_ontology_deltas'), 'critic missing candidate deltas')
assert_true(critic.get('canonical_truth_mutation') is False, 'critic canonical mutation leak')
proposal=req('/ontology/tool-result-proposals', {'tool_name':'bash','ok':False,'status':'failed','target_refs':['goal:active_mission'],'evidence_refs':['eval:spec95'],'error':'fixture failure','emit_proposals':False})
results.append('proposal_governance')
assert_true(proposal.get('reducer_promotion_records'), 'proposal lifecycle metadata missing')
pipe=req('/ontology/memory-pipeline', {'episodic_events':[{'event':'eval'}],'evidence_refs':['eval:spec95'],'synthesis_artifacts':[{'kind':'lesson'}],'eval_results':[{'score':0.9,'promote_learning':True}],'repeated_validation_count':2})
results.append('metacog_reuse_pipeline')
assert_true(pipe.get('pipeline_state')=='procedural_candidate_ready', pipe)
dash=req('/ontology/intelligence-dashboard')
results.append('dashboard_fixed_evals')
assert_true(dash.get('fixed_eval_suites',{}).get('fixture_count') >= 8, 'dashboard fixed eval suites missing')
assert_true(dash.get('deterministic_extractors'), 'deterministic extractor coverage missing')
adj=req('/ontology/adjacency')
results.append('no_hallucinated_canonical_links')
assert_true(adj.get('canonical_truth_mutation') is False, 'adjacency canonical mutation leak')
for node in adj.get('nodes',[])[:5]:
    for key in ['provenance_refs','verification_refs','working_set_memberships','action_affordance_ids','related_evidence_handles','related_workpoints']:
        assert_true(key in node, f'adjacency node missing {key}: {node}')
print(json.dumps({'fixed_eval_results':results,'passed':len(results)}))
PY
echo "SPEC95 fixed eval runtime gate: PASS"


ONTOLOGY_RS="$ROOT_DIR/crates/focusa-api/src/routes/ontology.rs"
if rg -n 'ontology_failure|ontology_validation_rejected|ontology_dispatch_failed|recovery_hint|misuse_hint|tool_result_v1' "$ONTOLOGY_RS" >/dev/null; then
  echo "✓ PASS: Ontology failures expose no-guess recovery contract"
else
  echo "✗ FAIL: Ontology failures lack no-guess recovery contract" >&2
  exit 1
fi
