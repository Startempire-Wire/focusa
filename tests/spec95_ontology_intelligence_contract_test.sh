#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ONT="$ROOT_DIR/crates/focusa-api/src/routes/ontology.rs"
PI="$ROOT_DIR/apps/pi-extension/src/turns.ts"
fail(){ echo "✗ FAIL: $1"; exit 1; }
pass(){ echo "✓ PASS: $1"; }

rg -n 'ONTOLOGY_READ_INDEX|OntologyReadIndex|incoming_by_type|outgoing_by_type|source_reducer_version|last_reducer_event_id|canonical_truth_mutation' "$ONT" >/dev/null || fail "adjacency read index metadata missing"
pass "adjacency read index metadata present"

rg -n 'ontology_working_set_projection|score|link_strength_score|link_path_reason|provenance_handles|verification_handles|confidence|freshness|action_affordance_ids|rehydrate' "$ONT" >/dev/null || fail "working-set scoring/reasons/provenance metadata missing"
pass "working-set scoring/reasons/provenance metadata present"

rg -n 'ontology_prompt_safe_context|active_object_set|relevant_link_paths|valid_next_actions|blocked_affordances|evidence_handles|uncertainty_flags' "$ONT" >/dev/null || fail "prompt-safe ontology context fields missing"
pass "prompt-safe ontology context fields present"

rg -n 'retrieval_results|semantic_memory|ecs_evidence|score|reasons|secondary_model_reranking|reranked_by|substrate":"none"' "$ONT" >/dev/null || fail "hybrid retrieval/reranking results missing"
pass "hybrid retrieval/reranking results present"

rg -n 'ontology_execution_critic|candidate_ontology_deltas|bounded_failure_proposal|recovery_suggestion|reducer_promotion_records' "$ONT" >/dev/null || fail "execution critic/proposal lifecycle path missing"
pass "execution critic proposal path present"

rg -n 'ontology_memory_promotion_pipeline|semantic_metacog_learning|procedural_playbook_hint|promotion_gates|deterministic_extractors|contradictory|rehydrate_needed|permission_boundary|estimated_cost' "$ONT" >/dev/null || fail "pipeline/extractor/uncertainty/affordance metadata missing"
pass "memory promotion pipeline present"

rg -n '/ontology/context|ACTIVE_OBJECT_SET|RELEVANT_LINK_PATHS|VALID_NEXT_ACTIONS|BLOCKED_AFFORDANCES|EVIDENCE_HANDLES|UNCERTAINTY_FLAGS' "$PI" >/dev/null || fail "Pi ontology pre-prompt sections missing"
pass "Pi ontology pre-prompt sections present"

echo "SPEC95 ontology intelligence contract: PASS"
